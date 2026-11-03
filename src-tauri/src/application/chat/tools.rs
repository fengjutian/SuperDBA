//! Read-only database tools exposed to the agent.
//!
//! Execution reuses the application metadata and query services, so a tool call
//! is subject to the same connection resolution, dialect handling, and
//! cancellation as the editor. The read-only policy is enforced here rather
//! than in the prompt: a write statement becomes a `denied` tool result.

use super::{ChatContext, ChatToolStatus};
use crate::application::metadata;
use crate::application::queries::{self, QueryRequestScope, QueryResponsePolicy};
use crate::runtime::{state::ApplicationState, RuntimeContext};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub const MAX_TABLES: usize = 200;
pub const MAX_ROWS: u32 = 50;
pub const DEFAULT_ROWS: u32 = 20;

#[derive(Clone, Debug)]
pub struct ToolSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

#[derive(Clone, Debug)]
pub struct ToolOutcome {
    pub status: ChatToolStatus,
    pub summary: String,
    pub result: Value,
}

impl ToolOutcome {
    fn ok(summary: impl Into<String>, result: Value) -> Self {
        Self {
            status: ChatToolStatus::Ok,
            summary: summary.into(),
            result,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status: ChatToolStatus::Error,
            summary: message.clone(),
            result: json!({ "error": message }),
        }
    }

    fn denied(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status: ChatToolStatus::Denied,
            summary: message.clone(),
            result: json!({ "error": message, "denied": true }),
        }
    }
}

pub fn definitions() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            name: "list_tables",
            description: "List tables in the active connection, optionally limited to one schema.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "schema": {"type": "string", "description": "Schema to inspect. Defaults to the active schema."}
                },
                "additionalProperties": false
            }),
        },
        ToolSchema {
            name: "describe_table",
            description: "Describe one table: columns, indexes, and foreign keys.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "table": {"type": "string", "description": "Table name."},
                    "schema": {"type": "string", "description": "Schema that owns the table. Defaults to the active schema."}
                },
                "required": ["table"],
                "additionalProperties": false
            }),
        },
        ToolSchema {
            name: "run_query",
            description: "Run a single read-only SELECT statement and return at most 50 rows. Writes and DDL are refused.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sql": {"type": "string", "description": "A single SELECT or WITH statement."},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 50, "description": "Maximum rows to return. Defaults to 20."}
                },
                "required": ["sql"],
                "additionalProperties": false
            }),
        },
        ToolSchema {
            name: "explain_query",
            description: "Capture the database execution plan for a SQL statement without running it.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sql": {"type": "string", "description": "Statement to explain."},
                    "analyze": {"type": "boolean", "description": "Run the statement to collect runtime statistics. Defaults to false."}
                },
                "required": ["sql"],
                "additionalProperties": false
            }),
        },
    ]
}

#[derive(Deserialize)]
struct ListTablesArgs {
    #[serde(default)]
    schema: Option<String>,
}

#[derive(Deserialize)]
struct DescribeTableArgs {
    table: String,
    #[serde(default)]
    schema: Option<String>,
}

#[derive(Deserialize)]
struct RunQueryArgs {
    sql: String,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct ExplainQueryArgs {
    sql: String,
    #[serde(default)]
    analyze: bool,
}

pub async fn execute(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    session_id: Option<Uuid>,
    context: &ChatContext,
    name: &str,
    arguments: &Value,
) -> ToolOutcome {
    let Some(connection_id) = context.connection_id.as_deref() else {
        return ToolOutcome::error(
            "No database connection is selected. Ask the user to open a connection first.",
        );
    };
    match name {
        "list_tables" => {
            let args: ListTablesArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            let schema = args.schema.or_else(|| context.schema.clone());
            list_tables(
                runtime,
                session_id,
                connection_id,
                schema,
                context.database.clone(),
            )
            .await
        }
        "describe_table" => {
            let args: DescribeTableArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            let schema = args.schema.or_else(|| context.schema.clone());
            if args.table.trim().is_empty() {
                return ToolOutcome::error("`table` must not be empty.");
            }
            describe_table(
                runtime,
                session_id,
                connection_id,
                args.table,
                schema,
                context.database.clone(),
            )
            .await
        }
        "run_query" => {
            let args: RunQueryArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            run_query(runtime, state, session_id, context, connection_id, args).await
        }
        "explain_query" => {
            let args: ExplainQueryArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            explain_query(runtime, state, session_id, context, connection_id, args).await
        }
        other => ToolOutcome::error(format!("Unknown tool: {other}")),
    }
}

async fn list_tables(
    runtime: &RuntimeContext,
    session_id: Option<Uuid>,
    connection_id: &str,
    schema: Option<String>,
    database: Option<String>,
) -> ToolOutcome {
    match metadata::get_tables(runtime, session_id, connection_id, schema, database).await {
        Ok(tables) => {
            let total = tables.len();
            let names = tables
                .into_iter()
                .take(MAX_TABLES)
                .map(|table| table.name)
                .collect::<Vec<_>>();
            let truncated = total > names.len();
            let summary = if truncated {
                format!("{total} tables (showing the first {MAX_TABLES})")
            } else {
                format!("{total} tables")
            };
            ToolOutcome::ok(
                summary,
                json!({"tables": names, "total": total, "truncated": truncated}),
            )
        }
        Err(error) => ToolOutcome::error(format!("Failed to list tables: {error}")),
    }
}

async fn describe_table(
    runtime: &RuntimeContext,
    session_id: Option<Uuid>,
    connection_id: &str,
    table: String,
    schema: Option<String>,
    database: Option<String>,
) -> ToolOutcome {
    let columns = metadata::get_columns(
        runtime,
        session_id,
        connection_id,
        table.clone(),
        schema.clone(),
        database.clone(),
    )
    .await;
    let indexes = metadata::get_indexes(
        runtime,
        session_id,
        connection_id,
        table.clone(),
        schema.clone(),
        database.clone(),
    )
    .await;
    let foreign_keys = metadata::get_foreign_keys(
        runtime,
        session_id,
        connection_id,
        table.clone(),
        schema,
        database,
    )
    .await;

    let (columns, indexes, foreign_keys) = match (columns, indexes, foreign_keys) {
        (Ok(columns), Ok(indexes), Ok(foreign_keys)) => (columns, indexes, foreign_keys),
        (Err(error), _, _) => {
            return ToolOutcome::error(format!("Failed to describe `{table}`: {error}"))
        }
        (_, Err(error), _) => {
            return ToolOutcome::error(format!("Failed to read indexes for `{table}`: {error}"))
        }
        (_, _, Err(error)) => {
            return ToolOutcome::error(format!(
                "Failed to read foreign keys for `{table}`: {error}"
            ))
        }
    };

    let summary = format!(
        "{table}: {} columns, {} indexes, {} foreign keys",
        columns.len(),
        indexes.len(),
        foreign_keys.len()
    );
    ToolOutcome::ok(
        summary,
        json!({
            "table": table,
            "columns": columns,
            "indexes": indexes,
            "foreignKeys": foreign_keys,
        }),
    )
}

async fn run_query(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    session_id: Option<Uuid>,
    context: &ChatContext,
    connection_id: &str,
    args: RunQueryArgs,
) -> ToolOutcome {
    if let Err(reason) = is_single_read_only_statement(&args.sql) {
        return ToolOutcome::denied(reason);
    }
    let limit = args.limit.unwrap_or(DEFAULT_ROWS).clamp(1, MAX_ROWS);
    let scope = QueryRequestScope {
        session_id,
        request_id: Some("ai-chat"),
    };
    let started = std::time::Instant::now();
    let result = queries::execute_query(
        runtime,
        &state.query_cancellation,
        scope,
        QueryResponsePolicy::Unbounded,
        connection_id.to_string(),
        args.sql,
        Some(limit),
        Some(1),
        context.schema.clone(),
        None,
        context.database.clone(),
    )
    .await;
    match result {
        Ok(result) => {
            let row_count = result.rows.len();
            let truncated = result.truncated || row_count > limit as usize;
            let rows = result
                .rows
                .into_iter()
                .take(limit as usize)
                .collect::<Vec<_>>();
            let summary = format!(
                "{row_count} rows returned in {} ms",
                started.elapsed().as_millis()
            );
            ToolOutcome::ok(
                summary,
                json!({
                    "columns": result.columns,
                    "rows": rows,
                    "rowCount": row_count,
                    "truncated": truncated,
                }),
            )
        }
        Err(error) => ToolOutcome::error(format!("Query failed: {error}")),
    }
}

async fn explain_query(
    runtime: &RuntimeContext,
    state: &ApplicationState,
    session_id: Option<Uuid>,
    context: &ChatContext,
    connection_id: &str,
    args: ExplainQueryArgs,
) -> ToolOutcome {
    let scope = QueryRequestScope {
        session_id,
        request_id: Some("ai-chat"),
    };
    let result = queries::explain_query_plan(
        runtime,
        &state.query_cancellation,
        scope,
        connection_id.to_string(),
        args.sql,
        args.analyze,
        context.schema.clone(),
        context.database.clone(),
    )
    .await;
    match result {
        Ok(plan) => match serde_json::to_value(&plan) {
            Ok(value) => ToolOutcome::ok("Execution plan captured", value),
            Err(error) => ToolOutcome::error(format!("Failed to serialize the plan: {error}")),
        },
        Err(error) => ToolOutcome::error(format!("Failed to explain the statement: {error}")),
    }
}

/// Accepts exactly one read-only statement.
///
/// The check runs on a copy with string literals and comments removed, so a
/// semicolon inside a literal or a DML keyword inside a comment does not
/// produce a false deny.
pub(crate) fn is_single_read_only_statement(sql: &str) -> Result<(), String> {
    let stripped = strip_sql_noise(sql);
    let trimmed = stripped.trim();
    let without_trailing = trimmed.trim_end_matches(';').trim_end();
    if trimmed.is_empty() {
        return Err("The statement is empty.".to_string());
    }
    if without_trailing.contains(';') {
        return Err(
            "Only one statement can run through chat. Run multiple statements in the editor."
                .to_string(),
        );
    }
    let words = words_of(without_trailing);
    let Some(first) = words.first() else {
        return Err("The statement is empty.".to_string());
    };
    match first.as_str() {
        "SELECT" => Ok(()),
        "WITH" => {
            const WRITES: [&str; 11] = [
                "INSERT", "UPDATE", "DELETE", "MERGE", "CREATE", "DROP", "ALTER", "TRUNCATE",
                "GRANT", "REVOKE", "COPY",
            ];
            if words.iter().any(|word| WRITES.contains(&word.as_str())) {
                return Err(
                    "`WITH` is only allowed for read-only queries. Write operations must run in the editor."
                        .to_string(),
                );
            }
            Ok(())
        }
        _ => Err(format!(
            "Chat can only run read-only SELECT statements; `{first}` is not allowed. Use the editor for writes and DDL."
        )),
    }
}

/// Removes comments, string literals, and quoted identifiers from `sql`,
/// preserving length-independent tokens (whitespace is inserted).
fn strip_sql_noise(sql: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        Single,
        Double,
        Backtick,
        LineComment,
        BlockComment,
    }

    let characters: Vec<char> = sql.chars().collect();
    let mut output = String::with_capacity(sql.len());
    let mut state = State::Normal;
    let mut index = 0;
    while index < characters.len() {
        let current = characters[index];
        let next = characters.get(index + 1).copied();
        match state {
            State::Normal => match (current, next) {
                ('-', Some('-')) => {
                    state = State::LineComment;
                    index += 2;
                    continue;
                }
                ('#', _) => {
                    state = State::LineComment;
                    index += 1;
                    continue;
                }
                ('/', Some('*')) => {
                    state = State::BlockComment;
                    index += 2;
                    continue;
                }
                ('\'', _) => {
                    state = State::Single;
                    output.push(' ');
                }
                ('"', _) => {
                    state = State::Double;
                    output.push(' ');
                }
                ('`', _) => {
                    state = State::Backtick;
                    output.push(' ');
                }
                _ => output.push(current),
            },
            State::Single => {
                if current == '\'' {
                    if next == Some('\'') {
                        index += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::Double => {
                if current == '"' {
                    if next == Some('"') {
                        index += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::Backtick => {
                if current == '`' {
                    if next == Some('`') {
                        index += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::LineComment => {
                if current == '\n' {
                    state = State::Normal;
                    output.push('\n');
                }
            }
            State::BlockComment => {
                if current == '*' && next == Some('/') {
                    state = State::Normal;
                    index += 2;
                    continue;
                }
            }
        }
        index += 1;
    }
    output
}

fn words_of(sql: &str) -> Vec<String> {
    sql.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_uppercase())
        .collect()
}
