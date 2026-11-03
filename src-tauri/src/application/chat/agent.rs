//! The bounded turn loop.
//!
//! A turn alternates model calls and tool executions until the model answers
//! without tools or [`MAX_MODEL_CALLS_PER_TURN`] is reached. The conversation is
//! persisted once per turn, including every tool invocation, so the record the
//! user reads is the record the model saw.

use super::provider::{self, ProviderToolCall};
use super::store;
use super::tools;
use super::{
    ChatContext, ChatMessage, ChatRole, ChatSendRequest, ChatSession, ChatToolCall,
    MAX_MODEL_CALLS_PER_TURN,
};
use crate::application::ai::resolve_model;
use crate::runtime::{state::ApplicationState, RuntimeContext};
use serde_json::{json, Value};
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
            "The {} provider does not support chat tools yet. Pick an OpenAI, Anthropic, DeepSeek, MiniMax, OpenRouter, Requesty, custom OpenAI-compatible, or Ollama model in Settings → AI.",
            request.provider
        ));
    }

    let config = crate::application::persistence::load_config(runtime);
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

    for round in 0..MAX_MODEL_CALLS_PER_TURN {
        let reply = provider::complete(
            runtime,
            &state.credential_cache,
            &config,
            &request.provider,
            &model,
            &system_prompt,
            &session.messages,
            &tool_schemas,
        )
        .await;

        let reply = match reply {
            Ok(reply) => reply,
            Err(error) => {
                let index = push_assistant(&mut session, String::new(), None);
                session.messages[index].notice = Some(error.clone());
                session.messages[index].content =
                    format!("The assistant could not answer: {error}");
                persist(runtime, &session).await?;
                return Ok(session);
            }
        };

        let index = push_assistant(&mut session, reply.content.clone(), reply.reasoning);
        if reply.tool_calls.is_empty() {
            persist(runtime, &session).await?;
            return Ok(session);
        }

        let calls =
            execute_calls(runtime, state, web_session_id, &context, &reply.tool_calls).await;
        session.messages[index].tool_calls = calls;

        if round + 1 == MAX_MODEL_CALLS_PER_TURN {
            session.messages[index].notice = Some(format!(
                "Stopped after {MAX_MODEL_CALLS_PER_TURN} model calls. Ask a narrower follow-up to continue."
            ));
        }
        session.updated_at = crate::ai_activity::now_iso8601();
    }

    persist(runtime, &session).await?;
    Ok(session)
}

async fn execute_calls(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    web_session_id: Option<Uuid>,
    context: &ChatContext,
    calls: &[ProviderToolCall],
) -> Vec<ChatToolCall> {
    let mut executed = Vec::with_capacity(calls.len());
    for call in calls {
        let started = std::time::Instant::now();
        let outcome = tools::execute(
            runtime,
            state,
            web_session_id,
            context,
            &call.name,
            &call.arguments,
        )
        .await;
        let (result, truncated) = bound_result(outcome.result, MAX_TOOL_RESULT_BYTES);
        let mut summary = outcome.summary;
        if truncated {
            summary.push_str(" (output truncated)");
        }
        log::debug!(
            "AI chat tool {} on {:?} finished in {} ms with {:?}",
            call.name,
            context.connection_id,
            started.elapsed().as_millis(),
            outcome.status
        );
        executed.push(ChatToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments: call.arguments.clone(),
            status: outcome.status,
            duration_ms: started.elapsed().as_millis() as u64,
            summary,
            result: Some(result),
        });
    }
    executed
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
         - Use `list_tables` and `describe_table` to inspect structure, `run_query` for read-only SELECTs, and `explain_query` for plans.\n\
         - Only read-only statements can run through the tools. If the user asks for a write, DDL, or administrative command, explain the change and offer the SQL for the editor instead.\n\
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
    value.push_str("\n… (truncated)");
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
