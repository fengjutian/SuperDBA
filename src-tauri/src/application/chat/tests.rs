use super::agent::{
    batch_limits, bound_result, build_audit_event, is_batch_boundary, messages_for_model,
    reserve_database_query_budget, system_prompt, tool_cache_key,
};
use super::provider::{self, ProviderToolCall};
use super::store;
use super::tools::{self, classify_statement, is_single_read_only_statement, StatementClass};
use super::{
    cancellation_key, ChatContext, ChatMessage, ChatRole, ChatSession, ChatToolCall, ChatToolStatus,
};
use serde_json::json;

#[tokio::test]
async fn chat_queries_are_serialized_per_connection() {
    let connection_id = format!("chat-serialization-{}", uuid::Uuid::new_v4());
    let first = tools::query_semaphore(&connection_id)
        .acquire_owned()
        .await
        .expect("first permit");
    let second = tools::query_semaphore(&connection_id);

    assert!(second.clone().try_acquire_owned().is_err());
    drop(first);
    assert!(second.try_acquire_owned().is_ok());
}

#[tokio::test]
async fn chat_queries_on_different_connections_do_not_block_each_other() {
    let suffix = uuid::Uuid::new_v4();
    let first = tools::query_semaphore(&format!("chat-a-{suffix}"))
        .acquire_owned()
        .await
        .expect("first connection permit");
    let second = tools::query_semaphore(&format!("chat-b-{suffix}"));

    assert!(second.try_acquire_owned().is_ok());
    drop(first);
}

#[test]
fn chat_turn_query_budget_stops_repeated_database_scans() {
    let mut used = 0;
    for _ in 0..12 {
        assert!(reserve_database_query_budget("run_query", &mut used));
    }
    assert!(!reserve_database_query_budget("explain_query", &mut used));
    assert!(reserve_database_query_budget("describe_table", &mut used));
    assert_eq!(used, 12);
}

fn message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage {
        id: "message-1".to_string(),
        role,
        content: content.to_string(),
        reasoning: None,
        created_at: "2026-11-03T00:00:00Z".to_string(),
        tool_calls: Vec::new(),
        tool_call_id: None,
        notice: None,
    }
}

fn session() -> ChatSession {
    ChatSession {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Test".to_string(),
        provider: "openai".to_string(),
        model: "gpt-test".to_string(),
        connection_id: Some("connection-1".to_string()),
        created_at: "2026-11-03T00:00:00Z".to_string(),
        updated_at: "2026-11-03T00:00:00Z".to_string(),
        messages: Vec::new(),
    }
}

#[test]
fn title_is_collapsed_and_bounded() {
    assert_eq!(
        store::title_from_message("  select   *\n from users "),
        "select * from users"
    );
    assert_eq!(store::title_from_message("   "), "New conversation");
    let long = "a".repeat(120);
    let title = store::title_from_message(&long);
    assert_eq!(title.chars().count(), 61);
    assert!(title.ends_with('…'));
}

#[test]
fn session_ids_reject_path_traversal() {
    assert!(store::is_valid_session_id(
        "0d6f6f7e-1a2b-4c3d-8e9f-0123456789ab"
    ));
    assert!(!store::is_valid_session_id("../escape"));
    assert!(!store::is_valid_session_id("a/b"));
    assert!(!store::is_valid_session_id(""));
}

#[test]
fn sessions_round_trip_and_delete() {
    let directory = tempfile::tempdir().unwrap();
    let mut session = session();
    store::append_message(&mut session, ChatRole::User, "hello".to_string());
    store::save(directory.path(), &session).unwrap();

    let summaries = store::list(directory.path()).unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].message_count, 1);

    let loaded = store::load(directory.path(), &session.id).unwrap();
    assert_eq!(loaded.messages.len(), 1);
    assert_eq!(loaded.messages[0].content, "hello");

    store::delete(directory.path(), &session.id).unwrap();
    assert!(store::load(directory.path(), &session.id).is_err());
    assert!(store::list(directory.path()).unwrap().is_empty());
}

#[test]
fn rejects_multi_statement_and_writes() {
    assert!(is_single_read_only_statement("SELECT 1").is_ok());
    assert!(is_single_read_only_statement("select * from users;").is_ok());
    assert!(is_single_read_only_statement("WITH a AS (SELECT 1) SELECT * FROM a").is_ok());
    assert!(is_single_read_only_statement("SELECT 'semi;colon' AS value").is_ok());
    assert!(is_single_read_only_statement("SELECT 1 -- ; DROP TABLE users").is_ok());

    assert!(is_single_read_only_statement("SELECT 1; DROP TABLE users").is_err());
    assert!(is_single_read_only_statement("DELETE FROM users").is_err());
    assert!(is_single_read_only_statement("WITH a AS (SELECT 1) DELETE FROM users").is_err());
    assert!(is_single_read_only_statement("").is_err());
}

#[test]
fn classifies_writes_for_the_approval_gate() {
    assert!(matches!(
        classify_statement("SELECT 1"),
        StatementClass::ReadOnly
    ));
    assert!(matches!(
        classify_statement("DELETE FROM users"),
        StatementClass::Write("DELETE")
    ));
    assert!(matches!(
        classify_statement("CREATE TABLE t (id int)"),
        StatementClass::Write("DDL")
    ));
    assert!(matches!(
        classify_statement("WITH a AS (SELECT 1) DELETE FROM users"),
        StatementClass::Write(_)
    ));
    assert!(matches!(
        classify_statement("SHOW TABLES"),
        StatementClass::Rejected(_)
    ));
    assert!(matches!(
        classify_statement("SELECT 1; SELECT 2"),
        StatementClass::Rejected(_)
    ));
}

#[test]
fn cancellation_keys_are_scoped_to_the_transport_session() {
    let first = uuid::Uuid::new_v4();
    let second = uuid::Uuid::new_v4();
    assert_ne!(
        cancellation_key(Some(first), "request"),
        cancellation_key(Some(second), "request")
    );
    assert_ne!(
        cancellation_key(Some(first), "request"),
        cancellation_key(None, "request")
    );
    assert_eq!(
        cancellation_key(None, "request"),
        "desktop:request".to_string()
    );
}

#[test]
fn chat_tool_audit_event_captures_sql_context_and_result() {
    let context = ChatContext {
        connection_id: Some("connection-1".to_string()),
        connection_name: Some("Production".to_string()),
        database: Some("app".to_string()),
        ..ChatContext::default()
    };
    let call = ProviderToolCall {
        id: "call-1".to_string(),
        name: "run_query".to_string(),
        arguments: json!({"sql": "SELECT * FROM users", "limit": 20}),
    };
    let outcome = tools::ToolOutcome {
        status: ChatToolStatus::Ok,
        summary: "2 rows returned in 4 ms".to_string(),
        result: json!({"rowCount": 2}),
    };

    let event = build_audit_event("chat-session", &context, &call, &outcome, 4);

    assert_eq!(event.session_id, "chat-session");
    assert_eq!(event.tool, "run_query");
    assert_eq!(event.connection_id.as_deref(), Some("connection-1"));
    assert_eq!(event.connection_name.as_deref(), Some("Production"));
    assert_eq!(event.database.as_deref(), Some("app"));
    assert_eq!(event.query.as_deref(), Some("SELECT * FROM users"));
    assert_eq!(event.query_kind.as_deref(), Some("select"));
    assert_eq!(event.status, "success");
    assert_eq!(event.rows, Some(2));
    assert_eq!(event.error, None);
    assert_eq!(event.client_hint.as_deref(), Some("SuperDBA chat"));
}

#[test]
fn chat_tool_audit_event_records_denials_as_errors() {
    let call = ProviderToolCall {
        id: "call-2".to_string(),
        name: "run_query".to_string(),
        arguments: json!({"sql": "DROP TABLE users"}),
    };
    let outcome = tools::ToolOutcome {
        status: ChatToolStatus::Denied,
        summary: "The statement was denied".to_string(),
        result: json!({"denied": true}),
    };

    let event = build_audit_event("chat-session", &ChatContext::default(), &call, &outcome, 1);

    assert_eq!(event.query_kind.as_deref(), Some("ddl"));
    assert_eq!(event.status, "denied");
    assert_eq!(event.error.as_deref(), Some("The statement was denied"));
}

#[test]
fn bounded_results_drop_rows_before_previews() {
    let rows = (0..10)
        .map(|_| json!(["aaaaaaaaaaaaaaaaaaaaaaaaaaaa"]))
        .collect::<Vec<_>>();
    let value = json!({
        "columns": ["id"],
        "rows": rows,
        "rowCount": 10,
    });
    let (bounded, truncated) = bound_result(value, 90);
    assert!(truncated);
    assert_eq!(bounded["truncated"], json!(true));
    let rows = bounded["rows"].as_array().expect("rows stay structured");
    assert!(rows.len() < 10);

    let (small, truncated) = bound_result(json!({"ok": true}), 1024);
    assert!(!truncated);
    assert_eq!(small, json!({"ok": true}));
}

#[test]
fn system_prompt_carries_connection_context() {
    let context = ChatContext {
        connection_id: Some("connection-1".to_string()),
        connection_name: Some("Analytics".to_string()),
        schema: Some("public".to_string()),
        database: Some("warehouse".to_string()),
        selected_sql: Some("SELECT * FROM orders".to_string()),
        last_result: None,
        last_plan: None,
    };
    let prompt = system_prompt(&context);
    assert!(prompt.contains("Connection: Analytics"));
    assert!(prompt.contains("Schema: public"));
    assert!(prompt.contains("SELECT * FROM orders"));
    assert!(prompt.contains("read-only"));
    assert!(prompt.contains("describe_tables"));
    assert!(prompt.contains("every page and batch"));
    assert!(batch_limits(&crate::config::AppConfig::default()).total_calls > 16);
}

#[test]
fn long_turns_checkpoint_at_batch_boundaries() {
    assert!(!is_batch_boundary(0, 16));
    assert!(!is_batch_boundary(15, 16));
    assert!(is_batch_boundary(16, 16));
    assert!(is_batch_boundary(32, 16));
}

#[test]
fn chat_batch_limits_use_defaults_and_clamp_saved_values() {
    let defaults = batch_limits(&crate::config::AppConfig::default());
    assert_eq!((defaults.calls_per_batch, defaults.max_batches), (16, 4));

    let config = crate::config::AppConfig {
        ai_chat_calls_per_batch: Some(100),
        ai_chat_max_batches: Some(0),
        ..Default::default()
    };
    let clamped = batch_limits(&config);
    assert_eq!((clamped.calls_per_batch, clamped.max_batches), (32, 1));
    assert_eq!(clamped.total_calls, 32);
}

#[test]
fn tool_cache_only_accepts_read_only_calls() {
    let read = ProviderToolCall {
        id: "read".to_string(),
        name: "run_query".to_string(),
        arguments: json!({"sql": "SELECT 1", "limit": 20}),
    };
    let write = ProviderToolCall {
        id: "write".to_string(),
        name: "run_query".to_string(),
        arguments: json!({"sql": "DELETE FROM users"}),
    };
    let describe = ProviderToolCall {
        id: "describe".to_string(),
        name: "describe_table".to_string(),
        arguments: json!({"table": "users"}),
    };

    assert_eq!(tool_cache_key(&read), tool_cache_key(&read));
    assert!(tool_cache_key(&read).is_some());
    assert!(tool_cache_key(&describe).is_some());
    assert!(tool_cache_key(&write).is_none());
}

#[test]
fn old_tool_results_are_compacted_only_for_model_replay() {
    let mut messages = Vec::new();
    for index in 0..=16 {
        let mut assistant = message(ChatRole::Assistant, "Checking.");
        assistant.id = format!("message-{index}");
        assistant.tool_calls.push(ChatToolCall {
            id: format!("call-{index}"),
            name: "run_query".to_string(),
            arguments: json!({"sql": format!("SELECT {index}")}),
            status: ChatToolStatus::Ok,
            duration_ms: 1,
            summary: format!("result {index}"),
            result: Some(json!({"rows": [[index]]})),
        });
        messages.push(assistant);
    }

    let replay = messages_for_model(&messages, 16);
    assert_eq!(
        messages[0].tool_calls[0].result,
        Some(json!({"rows": [[0]]}))
    );
    assert_eq!(
        replay[0].tool_calls[0].result.as_ref().unwrap()["compacted"],
        json!(true)
    );
    assert_eq!(replay[1].tool_calls[0].result, Some(json!({"rows": [[1]]})));
}

#[test]
fn chat_tools_expose_pagination_and_batch_schema_inspection() {
    let definitions = tools::definitions();
    let list_tables = definitions
        .iter()
        .find(|tool| tool.name == "list_tables")
        .expect("list_tables is registered");
    assert_eq!(
        list_tables.input_schema["properties"]["offset"]["minimum"],
        json!(0)
    );
    assert_eq!(
        list_tables.input_schema["properties"]["limit"]["maximum"],
        json!(50)
    );

    let describe_tables = definitions
        .iter()
        .find(|tool| tool.name == "describe_tables")
        .expect("describe_tables is registered");
    assert_eq!(
        describe_tables.input_schema["properties"]["tables"]["maxItems"],
        json!(5)
    );
}

#[test]
fn openai_body_encodes_tool_calls_and_results() {
    let mut assistant = message(ChatRole::Assistant, "");
    assistant.tool_calls.push(ChatToolCall {
        id: "call-1".to_string(),
        name: "list_tables".to_string(),
        arguments: json!({"schema": "public"}),
        status: ChatToolStatus::Ok,
        duration_ms: 5,
        summary: "3 tables".to_string(),
        result: Some(json!({"tables": ["users"]})),
    });

    let body = provider::openai_body(
        "gpt-test",
        "system",
        &[message(ChatRole::User, "what tables?"), assistant],
        &super::tools::definitions(),
    );
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages[0]["role"], json!("system"));
    assert_eq!(messages[1]["role"], json!("user"));
    assert_eq!(messages[2]["role"], json!("assistant"));
    assert_eq!(messages[2]["tool_calls"][0]["id"], json!("call-1"));
    assert_eq!(messages[3]["role"], json!("tool"));
    assert_eq!(messages[3]["tool_call_id"], json!("call-1"));
    assert!(body["tools"].as_array().unwrap().len() >= 4);
}

#[test]
fn anthropic_body_batches_tool_results_into_one_user_turn() {
    let mut assistant = message(ChatRole::Assistant, "Checking.");
    for id in ["call-1", "call-2"] {
        assistant.tool_calls.push(ChatToolCall {
            id: id.to_string(),
            name: "list_tables".to_string(),
            arguments: json!({}),
            status: ChatToolStatus::Ok,
            duration_ms: 1,
            summary: "ok".to_string(),
            result: Some(json!({"tables": []})),
        });
    }
    let body = provider::anthropic_body(
        "claude-test",
        "system",
        &[message(ChatRole::User, "tables?"), assistant],
        &super::tools::definitions(),
    );
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages[0]["role"], json!("user"));
    assert_eq!(messages[1]["role"], json!("assistant"));
    assert_eq!(messages[1]["content"][1]["type"], json!("tool_use"));
    assert_eq!(messages[2]["role"], json!("user"));
    let results = messages[2]["content"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["type"], json!("tool_result"));
    assert_eq!(body["tools"][0]["input_schema"]["type"], json!("object"));
}

#[test]
fn unsupported_providers_are_explicitly_unavailable() {
    assert!(!provider::supports_tool_calling("chatgpt"));
    assert!(provider::supports_tool_calling("minimax"));
    assert!(provider::supports_tool_calling("anthropic"));
    assert!(provider::supports_tool_calling("ollama"));
}

#[test]
fn minimax_body_replays_reasoning_with_tool_turns() {
    let mut assistant = message(ChatRole::Assistant, "Checking.");
    assistant.reasoning = Some("I should list the tables first.".to_string());
    assistant.tool_calls.push(ChatToolCall {
        id: "call-1".to_string(),
        name: "list_tables".to_string(),
        arguments: json!({"schema": "public"}),
        status: ChatToolStatus::Ok,
        duration_ms: 5,
        summary: "3 tables".to_string(),
        result: Some(json!({"tables": ["users"]})),
    });

    let body = provider::minimax_body(
        "MiniMax-M3.1-Flash",
        "system",
        &[message(ChatRole::User, "what tables?"), assistant],
        &super::tools::definitions(),
    );
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(
        messages[2]["reasoning_content"],
        json!("I should list the tables first.")
    );
    assert_eq!(messages[2]["tool_calls"][0]["id"], json!("call-1"));
    assert_eq!(messages[3]["role"], json!("tool"));
    // The plain OpenAI wire format must not send the field back: DeepSeek and
    // friends reject unknown reasoning fields.
    let mut reasoning = message(ChatRole::Assistant, "");
    reasoning.reasoning = Some("private chain of thought".to_string());
    let plain = provider::openai_body("gpt-test", "system", &[reasoning], &[]);
    assert!(plain["messages"][1].get("reasoning_content").is_none());
}

#[test]
fn parses_openai_and_anthropic_tool_calls() {
    let openai = json!({
        "choices": [{"message": {"content": null, "reasoning_content": "I will run a query", "tool_calls": [
            {"id": "call-1", "function": {"name": "run_query", "arguments": "{\"sql\":\"SELECT 1\"}"}}
        ]}}]
    });
    let reply = provider::parse_openai(&openai).unwrap();
    assert!(reply.content.is_empty());
    assert_eq!(reply.reasoning.as_deref(), Some("I will run a query"));
    assert_eq!(reply.tool_calls[0].name, "run_query");
    assert_eq!(reply.tool_calls[0].arguments["sql"], json!("SELECT 1"));

    let anthropic = json!({
        "content": [
            {"type": "text", "text": "Let me check."},
            {"type": "tool_use", "id": "toolu_1", "name": "list_tables", "input": {"schema": "public"}}
        ]
    });
    let reply = provider::parse_anthropic(&anthropic).unwrap();
    assert_eq!(reply.content, "Let me check.");
    assert_eq!(reply.tool_calls[0].id, "toolu_1");
    assert_eq!(reply.tool_calls[0].arguments["schema"], json!("public"));
}

mod stream_parsing {
    use super::super::stream::{take_lines, AnthropicStream, OllamaStream, OpenAiStream};
    use serde_json::json;

    #[test]
    fn take_lines_keeps_partial_tail() {
        let mut buffer = String::from("data: one\ndata: tw");
        let lines = take_lines(&mut buffer);
        assert_eq!(lines, vec!["data: one".to_string()]);
        assert_eq!(buffer, "data: tw");
        buffer.push_str("o\n");
        assert_eq!(take_lines(&mut buffer), vec!["data: two".to_string()]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn openai_stream_assembles_text_and_fragmented_tools() {
        let mut state = OpenAiStream::default();
        assert_eq!(
            state.apply(&json!({"choices": [{"delta": {"content": "Hel"}}]})),
            Some("Hel".to_string())
        );
        state.apply(&json!({"choices": [{"delta": {"tool_calls": [{
            "index": 0, "id": "call_1", "function": {"name": "run_query", "arguments": "{\"sql\":"}
        }]}}]}));
        state.apply(&json!({"choices": [{"delta": {"tool_calls": [{
            "index": 0, "function": {"arguments": "\"SELECT 1\"}"}
        }]}}]}));
        let reply = state.finish();
        assert_eq!(reply.content, "Hel");
        assert_eq!(reply.tool_calls.len(), 1);
        assert_eq!(reply.tool_calls[0].name, "run_query");
        assert_eq!(reply.tool_calls[0].arguments["sql"], json!("SELECT 1"));
    }

    #[test]
    fn anthropic_stream_assembles_text_and_tool_input() {
        let mut state = AnthropicStream::default();
        state.apply(
            &json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text"}}),
        );
        assert_eq!(
            state.apply(&json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Checking"}})),
            Some("Checking".to_string())
        );
        state.apply(&json!({"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "toolu_1", "name": "list_tables"}}));
        state.apply(&json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"schema\":"}}));
        state.apply(&json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "\"public\"}"}}));
        state.apply(&json!({"type": "content_block_stop", "index": 1}));
        let reply = state.finish();
        assert_eq!(reply.content, "Checking");
        assert_eq!(reply.tool_calls[0].id, "toolu_1");
        assert_eq!(reply.tool_calls[0].arguments["schema"], json!("public"));
    }

    #[test]
    fn ollama_stream_reads_message_chunks() {
        let mut state = OllamaStream::default();
        assert_eq!(
            state.apply(&json!({"content": "Hi"})),
            Some("Hi".to_string())
        );
        state.apply(
            &json!({"tool_calls": [{"function": {"name": "list_tables", "arguments": {}}}]}),
        );
        let reply = state.finish();
        assert_eq!(reply.content, "Hi");
        assert_eq!(reply.tool_calls[0].name, "list_tables");
    }
}
