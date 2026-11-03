//! Local conversation storage.
//!
//! One JSON document per conversation under `<config>/chat/sessions/`. Writes
//! go through a temporary file and a rename so a crash cannot truncate an
//! existing conversation.

use super::{ChatMessage, ChatRole, ChatSession, ChatSessionSummary};
use std::fs;
use std::path::{Path, PathBuf};

const SESSIONS_DIR: &str = "chat";
const SESSIONS_SUBDIR: &str = "sessions";
const MAX_TITLE_CHARS: usize = 60;

fn sessions_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(SESSIONS_DIR).join(SESSIONS_SUBDIR)
}

fn session_path(config_dir: &Path, session_id: &str) -> Result<PathBuf, String> {
    if !is_valid_session_id(session_id) {
        return Err("Invalid conversation id".to_string());
    }
    Ok(sessions_dir(config_dir).join(format!("{session_id}.json")))
}

/// Session ids are UUIDs; restricting the alphabet also blocks path traversal.
pub(crate) fn is_valid_session_id(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id.len() <= 64
        && session_id
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

/// Turns the first user message into a bounded, single-line title.
pub(crate) fn title_from_message(message: &str) -> String {
    let collapsed = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return "New conversation".to_string();
    }
    if collapsed.chars().count() <= MAX_TITLE_CHARS {
        return collapsed;
    }
    let mut title: String = collapsed.chars().take(MAX_TITLE_CHARS).collect();
    title.push('…');
    title
}

pub(crate) fn summary_of(session: &ChatSession) -> ChatSessionSummary {
    ChatSessionSummary {
        id: session.id.clone(),
        title: session.title.clone(),
        provider: session.provider.clone(),
        model: session.model.clone(),
        connection_id: session.connection_id.clone(),
        created_at: session.created_at.clone(),
        updated_at: session.updated_at.clone(),
        message_count: session.messages.len(),
    }
}

pub(crate) fn list(config_dir: &Path) -> Result<Vec<ChatSessionSummary>, String> {
    let directory = sessions_dir(config_dir);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    let entries = fs::read_dir(&directory).map_err(|error| error.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(session) = serde_json::from_str::<ChatSession>(&contents) {
            sessions.push(summary_of(&session));
        }
    }
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(sessions)
}

pub(crate) fn load(config_dir: &Path, session_id: &str) -> Result<ChatSession, String> {
    let path = session_path(config_dir, session_id)?;
    let contents =
        fs::read_to_string(&path).map_err(|_| "The conversation no longer exists".to_string())?;
    serde_json::from_str(&contents).map_err(|error| format!("Corrupt conversation: {error}"))
}

pub(crate) fn save(config_dir: &Path, session: &ChatSession) -> Result<(), String> {
    let path = session_path(config_dir, &session.id)?;
    let directory = sessions_dir(config_dir);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let payload = serde_json::to_vec_pretty(session).map_err(|error| error.to_string())?;
    let temporary = directory.join(format!(".{}.json.tmp", session.id));
    fs::write(&temporary, payload).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete(config_dir: &Path, session_id: &str) -> Result<(), String> {
    let path = session_path(config_dir, session_id)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Appends a message and refreshes the timestamps. Kept here so the agent loop
/// and future importers share one consistent mutation path.
pub(crate) fn append_message(session: &mut ChatSession, role: ChatRole, content: String) -> String {
    let now = crate::ai_activity::now_iso8601();
    let message = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role,
        content,
        created_at: now.clone(),
        tool_calls: Vec::new(),
        tool_call_id: None,
        notice: None,
    };
    let id = message.id.clone();
    session.messages.push(message);
    session.updated_at = now;
    id
}
