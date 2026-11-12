//! Conversational DBA agent.
//!
//! A conversation is a durable [`ChatSession`] with a bounded, tool-using turn
//! loop. Tool execution reuses the same application services as the editor, so
//! authorization, cancellation, and dialect handling do not fork. M1 exposes
//! read-only tools only: a write statement is rejected by the tool layer, not
//! by the prompt.

mod agent;
mod plan_facts;
#[cfg(test)]
#[path = "chat/plan_facts_tests.rs"]
mod plan_facts_tests;
mod provider;
mod starters;
mod store;
mod stream;
mod tools;

#[cfg(test)]
mod tests;

use crate::runtime::{state::ApplicationState, RuntimeContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const MODEL_CALLS_PER_BATCH: usize = crate::config::DEFAULT_AI_CHAT_CALLS_PER_BATCH as usize;
pub const MAX_MODEL_CALL_BATCHES: usize = crate::config::DEFAULT_AI_CHAT_MAX_BATCHES as usize;
pub const MAX_MODEL_CALLS_PER_TURN: usize = MODEL_CALLS_PER_BATCH * MAX_MODEL_CALL_BATCHES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatToolStatus {
    Pending,
    Ok,
    Error,
    Denied,
    LimitReached,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub status: ChatToolStatus,
    pub duration_ms: u64,
    pub summary: String,
    /// Bounded structured result. Kept so the conversation can be replayed to
    /// the model on the next turn without re-running the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    /// Reasoning returned by providers with interleaved thinking (MiniMax).
    /// Replayed to the provider on later turns but never rendered as an answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ChatToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Human-readable degradation note (provider without tool support, loop
    /// limit reached, truncated tool output).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<ChatContinuation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatContinuation {
    pub completed_calls: usize,
    pub total_calls: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSessionSummary {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatContext {
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub connection_name: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub selected_sql: Option<String>,
    #[serde(default)]
    pub last_result: Option<String>,
    #[serde(default)]
    pub last_plan: Option<String>,
    /// Enforces read-only tool execution, normally for production connections.
    #[serde(default)]
    pub strict_mode: bool,
    /// Masks sensitive query-result columns before tool output reaches the model.
    #[serde(default)]
    pub mask_sensitive_results: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendRequest {
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Client-generated id used to correlate progress events and cancellation.
    #[serde(default)]
    pub request_id: Option<String>,
    pub message: String,
    #[serde(default)]
    pub context: ChatContext,
    #[serde(default)]
    pub auto_continue: bool,
    /// Produces a conclusion from persisted evidence without exposing database tools.
    #[serde(default)]
    pub summarize_only: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    /// Authenticated identity injected by the server transport. This is never
    /// accepted from a serialized client request.
    #[serde(skip)]
    pub(crate) security_identity: Option<ChatSecurityIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChatSecurityIdentity {
    pub subject: String,
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
}

/// Asks the configured model for the opening questions of an empty conversation.
///
/// The frontend renders deterministic questions immediately and swaps in these
/// only when the call succeeds, so a slow or failing provider costs nothing but
/// nicer wording.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSuggestStartersRequest {
    pub provider: String,
    #[serde(default)]
    pub model: String,
    /// BCP-47 tag of the language the questions must be written in (`zh`, `en`).
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub context: ChatContext,
    /// Table names of the active scope; the questions have to name real tables.
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug)]
pub enum ChatCommand {
    ListSessions,
    GetSession { session_id: String },
    DeleteSession { session_id: String },
    Send(ChatSendRequest),
    Cancel { request_id: String },
    SuggestStarters(ChatSuggestStartersRequest),
}

/// Request ids that the user asked to stop. The key includes the transport
/// session, so a browser session cannot stop another session's turn even if it
/// learns a request id. The flag is checked between stream chunks, before every
/// tool call, and before every model call, so a cancelled turn persists its
/// partial transcript instead of disappearing.
static CANCELLED_REQUESTS: once_cell::sync::Lazy<
    std::sync::Mutex<std::collections::HashSet<String>>,
> = once_cell::sync::Lazy::new(Default::default);

/// Desktop callers share the `desktop` scope; web callers are scoped to their
/// authenticated session id.
pub(crate) fn cancellation_key(web_session_id: Option<Uuid>, request_id: &str) -> String {
    match web_session_id {
        Some(session_id) => format!("{session_id}:{request_id}"),
        None => format!("desktop:{request_id}"),
    }
}

pub(crate) fn is_cancelled(web_session_id: Option<Uuid>, request_id: &str) -> bool {
    if request_id.is_empty() {
        return false;
    }
    CANCELLED_REQUESTS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .contains(&cancellation_key(web_session_id, request_id))
}

pub(crate) fn clear_cancelled(web_session_id: Option<Uuid>, request_id: &str) {
    if request_id.is_empty() {
        return;
    }
    CANCELLED_REQUESTS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&cancellation_key(web_session_id, request_id));
}

pub(crate) fn emit_chat_event(
    runtime: &RuntimeContext,
    web_session_id: Option<Uuid>,
    payload: serde_json::Value,
) {
    let result = match web_session_id {
        Some(session_id) => runtime
            .events
            .emit_to(session_id, "ai://chat-event", payload),
        None => runtime.events.emit("ai://chat-event", payload),
    };
    if let Err(error) = result {
        log::debug!("Failed to emit chat event: {error}");
    }
}

pub async fn execute(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    web_session_id: Option<Uuid>,
    command: ChatCommand,
) -> Result<Value, String> {
    let config_dir = runtime.paths.config_dir().to_path_buf();
    match command {
        ChatCommand::ListSessions => {
            let sessions = tokio::task::spawn_blocking(move || store::list(&config_dir))
                .await
                .map_err(|error| error.to_string())??;
            serde_json::to_value(sessions).map_err(|error| error.to_string())
        }
        ChatCommand::GetSession { session_id } => {
            let session =
                tokio::task::spawn_blocking(move || store::load(&config_dir, &session_id))
                    .await
                    .map_err(|error| error.to_string())??;
            serde_json::to_value(session).map_err(|error| error.to_string())
        }
        ChatCommand::DeleteSession { session_id } => {
            tokio::task::spawn_blocking(move || store::delete(&config_dir, &session_id))
                .await
                .map_err(|error| error.to_string())??;
            Ok(Value::Null)
        }
        ChatCommand::Send(request) => {
            let session = agent::run_turn(runtime, state, web_session_id, request).await?;
            serde_json::to_value(session).map_err(|error| error.to_string())
        }
        ChatCommand::Cancel { request_id } => {
            CANCELLED_REQUESTS
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(cancellation_key(web_session_id, &request_id));
            Ok(Value::Null)
        }
        ChatCommand::SuggestStarters(request) => {
            let starters = starters::suggest(runtime, state, &request).await?;
            serde_json::to_value(starters).map_err(|error| error.to_string())
        }
    }
}

/// Builds a fresh conversation. The first user message becomes the title.
pub(crate) fn new_session(provider: &str, model: &str, context: &ChatContext) -> ChatSession {
    let now = crate::ai_activity::now_iso8601();
    ChatSession {
        id: Uuid::new_v4().to_string(),
        title: "New conversation".to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        connection_id: context.connection_id.clone(),
        created_at: now.clone(),
        updated_at: now,
        messages: Vec::new(),
        continuation: None,
    }
}
