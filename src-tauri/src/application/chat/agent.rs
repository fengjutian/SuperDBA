//! The bounded turn loop.
//!
//! A turn alternates model calls and tool executions until the model answers
//! without tools or [`MAX_MODEL_CALLS_PER_TURN`] is reached. Long turns are
//! checkpointed in batches so database-wide analysis continues automatically
//! without losing completed work.

use super::provider::{self, ProviderToolCall};
use super::store;
use super::tools;
use super::{
    clear_cancelled, emit_chat_event, is_cancelled, ChatContext, ChatMessage, ChatRole,
    ChatSendRequest, ChatSession, ChatToolCall, ChatToolStatus,
};
use crate::ai_activity::AiActivityEvent;
use crate::application::ai::resolve_model;
use crate::config::{
    DEFAULT_AI_AUDIT_ENABLED, DEFAULT_AI_AUDIT_MAX_ENTRIES, DEFAULT_AI_CHAT_CALLS_PER_BATCH,
    DEFAULT_AI_CHAT_MAX_BATCHES, MAX_AI_CHAT_BATCHES, MAX_AI_CHAT_CALLS_PER_BATCH,
};
use crate::runtime::{state::ApplicationState, RuntimeContext};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

const MAX_TOOL_RESULT_BYTES: usize = 8 * 1024;
const MAX_SELECTED_SQL_CHARS: usize = 2000;
const MAX_RESULT_PREVIEW_CHARS: usize = 2000;
const MAX_PLAN_CHARS: usize = 3000;

pub async fn run_turn(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    web_session_id: Option<Uuid>,
    request: ChatSendRequest,
) -> Result<ChatSession, String> {
    let message = request.message.trim();
    if message.is_empty() {
        return Err("The chat message must not be empty".to_string());
    }
    if !provider::supports_tool_calling(&request.provider) {
        return Err(format!(
            "The {} provider does not support chat tools yet. Pick an OpenAI, Anthropic, DeepSeek, MiniMax, OpenRouter, Requesty, custom OpenAI-compatible, or Ollama model in Settings 鈫?AI.",
            request.provider
        ));
    }

    let config = crate::application::persistence::load_config(runtime);
    let limits = batch_limits(&config);
    let ollama_port = config.ai_ollama_port.unwrap_or(11434);
    let model = resolve_model(&request.provider, &request.model, &config, ollama_port).await?;

    let config_dir = runtime.paths.config_dir().to_path_buf();
    let mut session = match request.session_id.as_deref() {
        Some(session_id) => {
            let session_id = session_id.to_string();
            tokio::task::spawn_blocking({
                let config_dir = config_dir.clone();
                move || store::load(&config_dir, &session_id)
            })
            .await
            .map_err(|error| error.to_string())??
        }
        None => super::new_session(&request.provider, &model, &request.context),
    };
    session.provider = request.provider.clone();
    session.model = model.clone();

    let context = merged_context(&request.context, &session);
    if context.connection_id.is_some() {
        session.connection_id = context.connection_id.clone();
    }
    if session.messages.is_empty() {
        session.title = store::title_from_message(message);
    }
    store::append_message(&mut session, ChatRole::User, message.to_string());

    let system_prompt = system_prompt(&context);
    let tool_schemas = tools::definitions();
    let request_id = request.request_id.clone().unwrap_or_default();
    clear_cancelled(web_session_id, &request_id);

    emit_chat_event(
        runtime,
        web_session_id,
        json!({
            "type": "started",
            "requestId": request_id,
            "sessionId": session.id,
        }),
    );
    let mut tool_cache = HashMap::new();

    for round in 0..limits.total_calls {
        if is_cancelled(web_session_id, &request_id) {
            let index = push_assistant(&mut session, String::new(), None);
            session.messages[index].notice = Some("Stopped by the user.".to_string());
            persist(runtime, &session).await?;
            emit_cancelled(runtime, web_session_id, &request_id, &session.id);
            clear_cancelled(web_session_id, &request_id);
            return Ok(session);
        }

        let final_report = round + 1 == limits.total_calls;
        let request_prompt = if final_report {
            format!(
                "{system_prompt}\nFinal batch instruction:\n- Do not call tools. Summarize completed coverage, evidence-backed findings, unfinished scope, and the exact next step for a follow-up."
            )
        } else {
            system_prompt.clone()
        };
        let model_messages = messages_for_model(&session.messages, limits.calls_per_batch);
        let request_tools = if final_report {
            &[]
        } else {
            tool_schemas.as_slice()
        };
        let reply = provider::complete_streaming(
            runtime,
            &state.credential_cache,
            &config,
            &request.provider,
            &model,
            &request_prompt,
            &model_messages,
            request_tools,
            &mut |delta| {
                emit_chat_event(
                    runtime,
                    web_session_id,
                    json!({
                        "type": "delta",
                        "requestId": request_id,
                        "sessionId": session.id,
                        "text": delta,
                    }),
                );
            },
            &|| is_cancelled(web_session_id, &request_id),
        )
        .await;

        let reply = match reply {
            Ok(provider::StreamOutcome::Reply(reply)) => reply,
            Ok(provider::StreamOutcome::Cancelled(reply)) => {
                let index = push_assistant(&mut session, reply.content, reply.reasoning);
                session.messages[index].notice = Some("Stopped by the user.".to_string());
                persist(runtime, &session).await?;
                emit_cancelled(runtime, web_session_id, &request_id, &session.id);
                clear_cancelled(web_session_id, &request_id);
                return Ok(session);
            }
            Err(error) => {
                let index = push_assistant(&mut session, String::new(), None);
                session.messages[index].notice = Some(error.clone());
                session.messages[index].content =
                    format!("The assistant could not answer: {error}");
                persist(runtime, &session).await?;
                emit_chat_event(
                    runtime,
                    web_session_id,
                    json!({
                        "type": "failed",
                        "requestId": request_id,
                        "sessionId": session.id,
                        "message": error,
                    }),
                );
                clear_cancelled(web_session_id, &request_id);
                return Ok(session);
            }
        };

        let index = push_assistant(&mut session, reply.content, reply.reasoning);
        if reply.tool_calls.is_empty() {
            if final_report {
                emit_batch_progress(
                    runtime,
                    web_session_id,
                    &request_id,
                    &session.id,
                    limits,
                    limits.total_calls,
                );
            }
            persist(runtime, &session).await?;
            emit_finished(runtime, web_session_id, &request_id, &session.id);
            clear_cancelled(web_session_id, &request_id);
            return Ok(session);
        }

        let calls = execute_calls(
            runtime,
            state,
            web_session_id,
            &context,
            &request_id,
            &session.id,
            &reply.tool_calls,
            &mut tool_cache,
        )
        .await;
        session.messages[index].tool_calls = calls;

        let completed_calls = round + 1;
        if completed_calls == limits.total_calls {
            session.messages[index].notice = Some(format!(
                "Stopped after {} model calls across automatic batches. Ask a follow-up to continue from the saved analysis.", limits.total_calls
            ));
        } else if is_batch_boundary(completed_calls, limits.calls_per_batch) {
            persist(runtime, &session).await?;
            emit_batch_progress(
                runtime,
                web_session_id,
                &request_id,
                &session.id,
                limits,
                completed_calls,
            );
            log::debug!(
                "AI chat session {} completed a {}-call batch; continuing automatically ({}/{})",
                session.id,
                limits.calls_per_batch,
                completed_calls,
                limits.total_calls,
            );
        }
        session.updated_at = crate::ai_activity::now_iso8601();
    }

    persist(runtime, &session).await?;
    emit_finished(runtime, web_session_id, &request_id, &session.id);
    clear_cancelled(web_session_id, &request_id);
    Ok(session)
}

fn emit_batch_progress(
    runtime: &RuntimeContext,
    web_session_id: Option<Uuid>,
    request_id: &str,
    session_id: &str,
    limits: BatchLimits,
    completed_calls: usize,
) {
    emit_chat_event(
        runtime,
        web_session_id,
        json!({
            "type": "batchProgress",
            "requestId": request_id,
            "sessionId": session_id,
            "completedBatch": completed_calls / limits.calls_per_batch,
            "totalBatches": limits.max_batches,
            "completedCalls": completed_calls,
            "totalCalls": limits.total_calls,
        }),
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BatchLimits {
    pub calls_per_batch: usize,
    pub max_batches: usize,
    pub total_calls: usize,
}

pub(crate) fn batch_limits(config: &crate::config::AppConfig) -> BatchLimits {
    let calls_per_batch = config
        .ai_chat_calls_per_batch
        .unwrap_or(DEFAULT_AI_CHAT_CALLS_PER_BATCH)
        .clamp(1, MAX_AI_CHAT_CALLS_PER_BATCH) as usize;
    let max_batches = config
        .ai_chat_max_batches
        .unwrap_or(DEFAULT_AI_CHAT_MAX_BATCHES)
        .clamp(1, MAX_AI_CHAT_BATCHES) as usize;
    BatchLimits {
        calls_per_batch,
        max_batches,
        total_calls: calls_per_batch * max_batches,
    }
}

pub(crate) fn is_batch_boundary(completed_calls: usize, calls_per_batch: usize) -> bool {
    calls_per_batch > 0 && completed_calls > 0 && completed_calls % calls_per_batch == 0
}

pub(crate) fn messages_for_model(
    messages: &[ChatMessage],
    calls_per_batch: usize,
) -> Vec<ChatMessage> {
    let tool_turns = messages
        .iter()
        .filter(|message| !message.tool_calls.is_empty())
        .count();
    let compact_before = tool_turns.saturating_sub(calls_per_batch.max(1));
    let mut seen_tool_turns = 0;
    messages
        .iter()
        .cloned()
        .map(|mut message| {
            if !message.tool_calls.is_empty() {
                if seen_tool_turns < compact_before {
                    for call in &mut message.tool_calls {
                        call.result = Some(json!({
                            "compacted": true,
                            "status": call.status,
                            "summary": call.summary,
                        }));
                    }
                }
                seen_tool_turns += 1;
            }
            message
        })
        .collect()
}

fn emit_finished(
    runtime: &RuntimeContext,
    web_session_id: Option<Uuid>,
    request_id: &str,
    session_id: &str,
) {
    emit_chat_event(
        runtime,
        web_session_id,
        json!({
            "type": "finished",
            "requestId": request_id,
            "sessionId": session_id,
        }),
    );
}

fn emit_cancelled(
    runtime: &RuntimeContext,
    web_session_id: Option<Uuid>,
    request_id: &str,
    session_id: &str,
) {
    emit_chat_event(
        runtime,
        web_session_id,
        json!({
            "type": "cancelled",
            "requestId": request_id,
            "sessionId": session_id,
        }),
    );
}

async fn execute_calls(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    web_session_id: Option<Uuid>,
    context: &ChatContext,
    request_id: &str,
    chat_session_id: &str,
    calls: &[ProviderToolCall],
    cache: &mut HashMap<String, ChatToolCall>,
) -> Vec<ChatToolCall> {
    let mut executed = Vec::with_capacity(calls.len());
    let config = crate::application::persistence::load_config(runtime);
    let audit_enabled = config.ai_audit_enabled.unwrap_or(DEFAULT_AI_AUDIT_ENABLED);
    let audit_max_entries = config
        .ai_audit_max_entries
        .unwrap_or(DEFAULT_AI_AUDIT_MAX_ENTRIES) as usize;
    for call in calls {
        if is_cancelled(web_session_id, request_id) {
            break;
        }
        emit_chat_event(
            runtime,
            web_session_id,
            json!({
                "type": "toolStarted",
                "requestId": request_id,
                "callId": call.id,
                "name": call.name,
                "arguments": call.arguments,
            }),
        );
        let cache_key = tool_cache_key(call);
        if let Some(cached) = cache_key.as_ref().and_then(|key| cache.get(key)) {
            let mut reused = cached.clone();
            reused.id = call.id.clone();
            reused.duration_ms = 0;
            reused.summary = format!("Reused cached result: {}", cached.summary);
            emit_chat_event(
                runtime,
                web_session_id,
                json!({
                    "type": "toolFinished",
                    "requestId": request_id,
                    "callId": call.id,
                    "name": call.name,
                    "status": reused.status,
                    "durationMs": 0,
                    "summary": reused.summary,
                }),
            );
            executed.push(reused);
            continue;
        }
        let started = std::time::Instant::now();
        let tool = tools::ToolCallContext {
            runtime,
            state,
            web_session_id,
            chat_session_id,
            request_id,
            chat: context,
        };
        let outcome = tools::execute(&tool, &call.name, &call.arguments).await;
        let duration_ms = started.elapsed().as_millis() as u64;
        if audit_enabled {
            let event = build_audit_event(chat_session_id, context, call, &outcome, duration_ms);
            if let Err(error) = crate::ai_activity::append_and_rotate_in(
                runtime.paths.config_dir(),
                &event,
                audit_max_entries,
            ) {
                log::warn!("Failed to write the AI chat audit event: {error}");
            }
        }
        let (result, truncated) = bound_result(outcome.result, MAX_TOOL_RESULT_BYTES);
        let mut summary = outcome.summary;
        if truncated {
            summary.push_str(" (output truncated)");
        }
        log::debug!(
            "AI chat tool {} on {:?} finished in {} ms with {:?}",
            call.name,
            context.connection_id,
            duration_ms,
            outcome.status
        );
        emit_chat_event(
            runtime,
            web_session_id,
            json!({
                "type": "toolFinished",
                "requestId": request_id,
                "callId": call.id,
                "name": call.name,
                "status": outcome.status,
                "durationMs": duration_ms,
                "summary": summary,
            }),
        );
        let executed_call = ChatToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments: call.arguments.clone(),
            status: outcome.status,
            duration_ms,
            summary,
            result: Some(result),
        };
        if executed_call.status == ChatToolStatus::Ok {
            if let Some(key) = cache_key {
                cache.insert(key, executed_call.clone());
            }
        }
        executed.push(executed_call);
    }
    executed
}

pub(crate) fn tool_cache_key(call: &ProviderToolCall) -> Option<String> {
    let cacheable = if call.name == "run_query" {
        call.arguments
            .get("sql")
            .and_then(Value::as_str)
            .is_some_and(|sql| {
                matches!(
                    tools::classify_statement(sql),
                    tools::StatementClass::ReadOnly
                )
            })
    } else {
        crate::application::tool_registry::is_read_only(&call.name)
    };
    cacheable.then(|| {
        format!(
            "{}:{}",
            call.name,
            serde_json::to_string(&call.arguments).unwrap_or_default()
        )
    })
}

pub(super) fn build_audit_event(
    chat_session_id: &str,
    context: &ChatContext,
    call: &ProviderToolCall,
    outcome: &tools::ToolOutcome,
    duration_ms: u64,
) -> AiActivityEvent {
    let query = call
        .arguments
        .get("sql")
        .and_then(Value::as_str)
        .map(str::to_string);
    let query_kind = query
        .as_deref()
        .map(crate::ai_activity::classify_query_kind)
        .map(str::to_string);
    let rows = outcome
        .result
        .get("rowCount")
        .or_else(|| outcome.result.get("affectedRows"))
        .or_else(|| outcome.result.get("total"))
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok());
    let status = match outcome.status {
        ChatToolStatus::Ok => "success",
        ChatToolStatus::Error => "error",
        ChatToolStatus::Denied => "denied",
        ChatToolStatus::Pending => "pending",
    };

    AiActivityEvent {
        id: crate::ai_activity::new_uuid(),
        session_id: chat_session_id.to_string(),
        timestamp: crate::ai_activity::now_iso8601(),
        tool: call.name.clone(),
        connection_id: context.connection_id.clone(),
        connection_name: context.connection_name.clone(),
        database: context.database.clone(),
        query,
        query_kind,
        duration_ms,
        status: status.to_string(),
        rows,
        error: (outcome.status != ChatToolStatus::Ok).then(|| outcome.summary.clone()),
        client_hint: Some("SuperDBA chat".to_string()),
        approval_id: None,
    }
}

fn push_assistant(session: &mut ChatSession, content: String, reasoning: Option<String>) -> usize {
    let now = crate::ai_activity::now_iso8601();
    session.messages.push(ChatMessage {
        id: Uuid::new_v4().to_string(),
        role: ChatRole::Assistant,
        content,
        reasoning,
        created_at: now.clone(),
        tool_calls: Vec::new(),
        tool_call_id: None,
        notice: None,
    });
    session.updated_at = now;
    session.messages.len() - 1
}

async fn persist(runtime: &RuntimeContext, session: &ChatSession) -> Result<(), String> {
    let config_dir = runtime.paths.config_dir().to_path_buf();
    let session = session.clone();
    tokio::task::spawn_blocking(move || store::save(&config_dir, &session))
        .await
        .map_err(|error| error.to_string())?
}

fn merged_context(request: &ChatContext, session: &ChatSession) -> ChatContext {
    ChatContext {
        connection_id: request
            .connection_id
            .clone()
            .or_else(|| session.connection_id.clone()),
        connection_name: request.connection_name.clone(),
        schema: request.schema.clone(),
        database: request.database.clone(),
        selected_sql: request.selected_sql.clone(),
        last_result: request.last_result.clone(),
        last_plan: request.last_plan.clone(),
    }
}

/// Instruction text shared by every provider.
pub(crate) fn system_prompt(context: &ChatContext) -> String {
    let mut prompt = String::from(
        "You are SuperDBA, a database assistant embedded in a SQL workspace.\n\
         \n\
         Rules:\n\
         - Gather evidence with the tools before answering schema or data questions. Never invent table or column names.\n\
         - Use paginated `list_tables` and `describe_tables` batches to inspect large schemas; use `describe_table` for one table, `run_query` for read-only SELECTs, and `explain_query` for plans.\n\
         - For database-wide requests, work through every page and batch before answering. Keep a compact running synthesis, avoid repeating completed batches, and combine the evidence into one final analysis.\n\
         - Read-only statements run immediately. A single write or DDL statement is sent to the user for approval before it runs: explain the impact in one sentence first, and never try to batch several statements.\n\
         - Keep answers short and concrete. Put SQL in fenced ```sql blocks.\n\
         - Cite what you observed (row counts, columns, plan nodes) instead of guessing.\n\
         - When a tool fails, report the error and continue with what you know.\n",
    );
    let mut context_lines = Vec::new();
    if let Some(name) = context
        .connection_name
        .as_deref()
        .filter(|name| !name.is_empty())
    {
        context_lines.push(format!("Connection: {name}"));
    } else if let Some(id) = context.connection_id.as_deref() {
        context_lines.push(format!("Connection id: {id}"));
    }
    if let Some(database) = context.database.as_deref() {
        context_lines.push(format!("Database: {database}"));
    }
    if let Some(schema) = context.schema.as_deref() {
        context_lines.push(format!("Schema: {schema}"));
    }
    if !context_lines.is_empty() {
        prompt.push_str("\nActive context:\n");
        for line in context_lines {
            prompt.push_str("- ");
            prompt.push_str(&line);
            prompt.push('\n');
        }
    }
    if let Some(sql) = context
        .selected_sql
        .as_deref()
        .filter(|sql| !sql.trim().is_empty())
    {
        prompt.push_str("\nSelected SQL in the editor:\n```sql\n");
        prompt.push_str(&bounded(sql, MAX_SELECTED_SQL_CHARS));
        prompt.push_str("\n```\n");
    }
    if let Some(result) = context
        .last_result
        .as_deref()
        .filter(|result| !result.trim().is_empty())
    {
        prompt.push_str("\nLast result set summary:\n");
        prompt.push_str(&bounded(result, MAX_RESULT_PREVIEW_CHARS));
        prompt.push('\n');
    }
    if let Some(plan) = context
        .last_plan
        .as_deref()
        .filter(|plan| !plan.trim().is_empty())
    {
        prompt.push_str("\nLast execution plan:\n");
        prompt.push_str(&bounded(plan, MAX_PLAN_CHARS));
        prompt.push('\n');
    }
    prompt
}

fn bounded(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut value: String = text.chars().take(limit).collect();
    value.push_str("\n鈥?(truncated)");
    value
}

/// Keeps a tool result within the context budget, preferring to drop trailing
/// rows before falling back to a text preview.
pub(crate) fn bound_result(value: Value, max_bytes: usize) -> (Value, bool) {
    let mut value = value;
    let fits = |value: &Value| {
        serde_json::to_string(value)
            .map(|text| text.len() <= max_bytes)
            .unwrap_or(false)
    };
    if fits(&value) {
        return (value, false);
    }
    if let Value::Object(map) = &mut value {
        map.insert("truncated".to_string(), json!(true));
    }
    loop {
        if fits(&value) {
            return (value, true);
        }
        let dropped = match &mut value {
            Value::Object(map) => match map.get_mut("rows") {
                Some(Value::Array(rows)) => rows.pop().is_some(),
                _ => false,
            },
            _ => false,
        };
        if !dropped {
            break;
        }
    }
    let preview = serde_json::to_string(&value).unwrap_or_default();
    (
        json!({
            "truncated": true,
            "preview": bounded(&preview, max_bytes.min(4096)),
        }),
        true,
    )
}
