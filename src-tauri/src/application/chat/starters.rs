//! Opening questions for an empty conversation.
//!
//! The chat panel renders deterministic questions from the local table list
//! immediately; this module only supplies the nicer, model-written version. It
//! never builds a session, never runs a tool, and never streams: one plain
//! completion, then a defensive parse. Any failure is reported to the frontend,
//! which keeps its local questions.

use super::provider;
use super::{ChatContext, ChatMessage, ChatRole, ChatSuggestStartersRequest};
use crate::application::ai::resolve_model;
use crate::runtime::{state::ApplicationState, RuntimeContext};

/// Table names sent to the model. Long schemas keep their first names instead
/// of paying for a prompt the empty state does not need.
const MAX_PROMPT_TABLES: usize = 40;
const MAX_TABLE_NAME_CHARS: usize = 64;
/// The panel shows three questions; a model that returns more is trimmed.
pub(crate) const MAX_STARTERS: usize = 3;
/// Keeps a question usable as a one-line button.
pub(crate) const MAX_STARTER_CHARS: usize = 90;

pub(crate) async fn suggest(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    request: &ChatSuggestStartersRequest,
) -> Result<Vec<String>, String> {
    if !provider::supports_tool_calling(&request.provider) {
        return Err(format!(
            "The {} provider does not support chat yet. Pick an OpenAI, Anthropic, DeepSeek, MiniMax, OpenRouter, Requesty, custom OpenAI-compatible, or Ollama model in Settings → AI.",
            request.provider
        ));
    }

    let config = crate::application::persistence::load_config(runtime);
    let ollama_port = config.ai_ollama_port.unwrap_or(11434);
    let model = resolve_model(&request.provider, &request.model, &config, ollama_port).await?;
    let language = request.language.as_deref().unwrap_or("en");

    let reply = provider::complete(
        runtime,
        &state.credential_cache,
        &config,
        &request.provider,
        &model,
        &prompt(&request.context, &request.tables, language),
        &[ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: ChatRole::User,
            content: "Suggest the questions.".to_string(),
            reasoning: None,
            created_at: crate::ai_activity::now_iso8601(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            notice: None,
        }],
        &[],
    )
    .await?;

    let starters = parse(&reply.content);
    if starters.is_empty() {
        return Err("The model did not return usable starter questions.".to_string());
    }
    Ok(starters)
}

/// Instruction text for the completion. Kept separate so tests can assert on
/// the context it carries without calling a provider.
pub(crate) fn prompt(context: &ChatContext, tables: &[String], language: &str) -> String {
    let mut prompt = String::from(
        "You write the three opening questions shown above the input of a database assistant.\n\
         \n\
         Rules:\n\
         - Return JSON only: an array of exactly three short question strings.\n\
         - Every question must be answerable by inspecting the schema, and must name real tables from the list when it refers to one.\n\
         - Vary the angle: where to start, what a table holds, what is missing or slow.\n\
         - No numbering, no markdown, no answers, no commentary.\n",
    );

    if let Some(database) = context
        .database
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        prompt.push_str(&format!("\nDatabase: {database}"));
    }
    if let Some(schema) = context.schema.as_deref().filter(|value| !value.is_empty()) {
        prompt.push_str(&format!("\nSchema: {schema}"));
    }

    let names = bounded_tables(tables);
    if names.is_empty() {
        prompt.push_str("\n\nThe user has not loaded any tables yet. Ask what is in the database.");
    } else {
        prompt.push_str(&format!(
            "\n\nTables ({} of the active scope):\n- {}",
            names.len(),
            names.join("\n- ")
        ));
    }

    prompt.push_str("\n\nWrite every question in the language tagged `");
    prompt.push_str(language);
    prompt.push_str("`.\n");
    prompt
}

fn bounded_tables(tables: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    tables
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .filter(|name| seen.insert(name.to_ascii_lowercase()))
        .take(MAX_PROMPT_TABLES)
        .map(|name| {
            if name.chars().count() > MAX_TABLE_NAME_CHARS {
                name.chars().take(MAX_TABLE_NAME_CHARS).collect()
            } else {
                name.to_string()
            }
        })
        .collect()
}

/// Turns a model reply into at most [`MAX_STARTERS`] one-line questions.
///
/// Providers wrap JSON in fences, prepend prose, or ignore the format request
/// entirely, so this accepts a JSON array of strings anywhere in the text and
/// otherwise falls back to the non-empty lines.
pub(crate) fn parse(raw: &str) -> Vec<String> {
    let mut questions: Vec<String> = Vec::new();
    if let Some(values) = json_array(raw) {
        questions.extend(values);
    }
    if questions.is_empty() {
        questions.extend(raw.lines().map(|line| line.to_string()));
    }

    let mut seen = std::collections::HashSet::new();
    questions
        .into_iter()
        .map(|question| clean(&question))
        .filter(|question| !question.is_empty())
        .filter(|question| seen.insert(question.to_lowercase()))
        .take(MAX_STARTERS)
        .collect()
}

fn json_array(raw: &str) -> Option<Vec<String>> {
    let start = raw.find('[')?;
    let end = raw.rfind(']')?;
    if end <= start {
        return None;
    }
    let values: Vec<serde_json::Value> = serde_json::from_str(&raw[start..=end]).ok()?;
    let strings: Vec<String> = values
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();
    if strings.is_empty() {
        None
    } else {
        Some(strings)
    }
}

fn clean(question: &str) -> String {
    let mut text = question.trim();
    loop {
        let previous = text;
        text = text
            .trim_start_matches(|c: char| matches!(c, '-' | '*' | '•' | '>' | '#' | ' '))
            .trim_start();
        text = strip_ordered_marker(text).unwrap_or(text);
        if text == previous {
            break;
        }
    }
    let text = text
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '「' | '」'))
        .trim();
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > MAX_STARTER_CHARS {
        let mut short: String = collapsed.chars().take(MAX_STARTER_CHARS - 1).collect();
        short.push('…');
        short
    } else {
        collapsed
    }
}

fn strip_ordered_marker(text: &str) -> Option<&str> {
    let digits = text.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let rest = &text[digits..];
    [".", ")", "、", ":"]
        .iter()
        .find_map(|marker| rest.strip_prefix(marker).map(str::trim_start))
}

#[cfg(test)]
mod tests {
    use super::{clean, parse, prompt, MAX_STARTER_CHARS};
    use crate::application::chat::ChatContext;

    #[test]
    fn parses_a_fenced_json_array_and_trims_to_three() {
        let raw = "```json\n[\"Which tables should I start with?\", \"What does orders hold?\", \"Which indexes are missing?\", \"extra\"]\n```";
        assert_eq!(
            parse(raw),
            vec![
                "Which tables should I start with?",
                "What does orders hold?",
                "Which indexes are missing?",
            ]
        );
    }

    #[test]
    fn falls_back_to_lines_and_strips_list_markers() {
        let raw = "1. Why is orders slow?\n\n- Which indexes are missing?\n• What columns does users have?\n";
        assert_eq!(
            parse(raw),
            vec![
                "Why is orders slow?",
                "Which indexes are missing?",
                "What columns does users have?",
            ]
        );
    }

    #[test]
    fn dedupes_and_bounds_long_questions() {
        let long = "a".repeat(MAX_STARTER_CHARS * 2);
        let parsed = parse(&format!("[\"{long}\", \"{long}\", \"   \"]"));
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].chars().count(), MAX_STARTER_CHARS);
        assert!(parsed[0].ends_with('…'));
    }

    #[test]
    fn drops_quotes_and_collapses_whitespace() {
        assert_eq!(clean("\"  What   is   in users? \" "), "What is in users?");
        assert_eq!(clean(""), "");
    }

    #[test]
    fn prompt_names_the_tables_and_language() {
        let context = ChatContext {
            schema: Some("public".to_string()),
            database: Some("app".to_string()),
            ..Default::default()
        };
        let text = prompt(
            &context,
            &[
                "users".to_string(),
                "orders".to_string(),
                "users".to_string(),
            ],
            "zh",
        );
        assert!(text.contains("- users"));
        assert!(text.contains("- orders"));
        assert!(text.contains("Database: app"));
        assert!(text.contains("Schema: public"));
        assert!(text.contains("`zh`"));
        assert_eq!(text.matches("- users").count(), 1);
    }

    #[test]
    fn prompt_without_tables_asks_for_the_contents() {
        let text = prompt(&ChatContext::default(), &[], "en");
        assert!(text.contains("has not loaded any tables"));
    }
}
