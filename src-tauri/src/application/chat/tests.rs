use super::agent::{bound_result, system_prompt};
use super::provider;
use super::store;
use super::tools::is_single_read_only_statement;
use super::{ChatContext, ChatMessage, ChatRole, ChatSession, ChatToolCall, ChatToolStatus};
use serde_json::json;

fn message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage {
        id: "message-1".to_string(),
        role,
        content: content.to_string(),
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
fn bounded_results_drop_rows_before_previews() {
    let value = json!({
        "columns": ["id"],
        "rows": [[1], [2], [3], [4]],
        "rowCount": 4,
    });
    let (bounded, truncated) = bound_result(value, 90);
    assert!(truncated);
    assert_eq!(bounded["truncated"], json!(true));
    assert!(bounded["rows"].as_array().unwrap().len() < 4);

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
    assert!(!provider::supports_tool_calling("minimax"));
    assert!(provider::supports_tool_calling("anthropic"));
    assert!(provider::supports_tool_calling("ollama"));
}

#[test]
fn parses_openai_and_anthropic_tool_calls() {
    let openai = json!({
        "choices": [{"message": {"content": null, "tool_calls": [
            {"id": "call-1", "function": {"name": "run_query", "arguments": "{\"sql\":\"SELECT 1\"}"}}
        ]}}]
    });
    let reply = provider::parse_openai(&openai).unwrap();
    assert!(reply.content.is_empty());
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
