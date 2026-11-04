//! Database tools exposed to the agent.
//!
//! Execution reuses the application metadata and query services, so a tool call
//! is subject to the same connection resolution, dialect handling, and
//! cancellation as the editor. Read-only statements run directly; a write or
//! DDL statement is held behind the same file-queue approval gate the MCP
//! server uses, and only runs after the user approves it.

use super::{emit_chat_event, is_cancelled, ChatContext, ChatToolStatus};
use crate::ai_approval::{self, PendingApproval};
use crate::application::ai_query_guard;
use crate::application::ai_sql_safety;
use crate::application::metadata;
use crate::application::queries::{self, QueryRequestScope, QueryResponsePolicy};
use crate::runtime::{state::ApplicationState, RuntimeContext};
use serde::Deserialize;
use serde_json::{json, Value};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

pub const MAX_TABLES_PER_PAGE: usize = 50;
pub const MAX_TABLES_PER_BATCH: usize = 5;
pub const MAX_ROWS: u32 = 50;
pub const DEFAULT_ROWS: u32 = 20;
/// Keep automated diagnostics bounded even when the database has no statement timeout.
pub const QUERY_TIMEOUT: Duration = ai_query_guard::QUERY_TIMEOUT;
/// How long a chat tool waits for the user to approve a write.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(180);
const APPROVAL_POLL_INTERVAL: Duration = Duration::from_millis(200);
const CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Everything a tool needs: the runtime, the live turn identity, and the
/// connection context the user is looking at.
pub struct ToolCallContext<'a> {
    pub runtime: &'a RuntimeContext,
    pub state: &'a ApplicationState,
    pub web_session_id: Option<Uuid>,
    pub chat_session_id: &'a str,
    pub request_id: &'a str,
    pub chat: &'a ChatContext,
}

impl ToolCallContext<'_> {
    fn cancelled(&self) -> bool {
        is_cancelled(self.web_session_id, self.request_id)
    }
}

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
    use crate::application::tool_registry::{for_surface, ToolSurface};
    for_surface(ToolSurface::Chat)
        .map(|spec| ToolSchema {
            name: spec.name,
            description: spec.chat_description,
            input_schema: chat_input_schema(spec.name),
        })
        .collect()
}

/// Chat wire schema for one registry tool: the connection is implicit (the
/// active one), so only the table-specific arguments remain.
fn chat_input_schema(name: &str) -> Value {
    match name {
        "list_tables" => json!({
            "type": "object",
            "properties": {
                "schema": {"type": "string", "description": "Schema to inspect. Defaults to the active schema."},
                "offset": {"type": "integer", "minimum": 0, "description": "Zero-based table offset. Defaults to 0."},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50, "description": "Tables to return in this page. Defaults to 50."}
            },
            "additionalProperties": false
        }),
        "describe_table" => json!({
            "type": "object",
            "properties": {
                "table": {"type": "string", "description": "Table name."},
                "schema": {"type": "string", "description": "Schema that owns the table. Defaults to the active schema."}
            },
            "required": ["table"],
            "additionalProperties": false
        }),
        "describe_tables" => json!({
            "type": "object",
            "properties": {
                "tables": {
                    "type": "array",
                    "items": {"type": "string"},
                    "minItems": 1,
                    "maxItems": 5,
                    "description": "Table names to inspect together."
                },
                "schema": {"type": "string", "description": "Schema that owns the tables. Defaults to the active schema."}
            },
            "required": ["tables"],
            "additionalProperties": false
        }),
        "run_query" => json!({
            "type": "object",
            "properties": {
                "sql": {"type": "string", "description": "One SQL statement. Writes require approval."},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50, "description": "Maximum rows to return. Defaults to 20."}
            },
            "required": ["sql"],
            "additionalProperties": false
        }),
        "explain_query" => json!({
            "type": "object",
            "properties": {
                "sql": {"type": "string", "description": "Statement to explain without executing it."}
            },
            "required": ["sql"],
            "additionalProperties": false
        }),
        "diagnose_query" => json!({
            "type": "object",
            "properties": {
                "sql": {"type": "string", "description": "Statement to diagnose."},
                "analyze": {"type": "boolean", "description": "Run a read-only statement to collect runtime statistics. Defaults to false."}
            },
            "required": ["sql"],
            "additionalProperties": false
        }),
        other => json!({
            "type": "object",
            "properties": {},
            "_tool": other,
        }),
    }
}

#[derive(Deserialize)]
struct ListTablesArgs {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct DescribeTableArgs {
    table: String,
    #[serde(default)]
    schema: Option<String>,
}

#[derive(Deserialize)]
struct DescribeTablesArgs {
    tables: Vec<String>,
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

#[derive(Deserialize)]
struct DiagnoseQueryArgs {
    sql: String,
    #[serde(default)]
    analyze: bool,
}

pub async fn execute(tool: &ToolCallContext<'_>, name: &str, arguments: &Value) -> ToolOutcome {
    let Some(connection_id) = tool.chat.connection_id.as_deref() else {
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
            let schema = args.schema.or_else(|| tool.chat.schema.clone());
            list_tables(
                tool,
                connection_id,
                schema,
                tool.chat.database.clone(),
                args.offset,
                args.limit.unwrap_or(MAX_TABLES_PER_PAGE),
            )
            .await
        }
        "describe_table" => {
            let args: DescribeTableArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            let schema = args.schema.or_else(|| tool.chat.schema.clone());
            if args.table.trim().is_empty() {
                return ToolOutcome::error("`table` must not be empty.");
            }
            describe_table(
                tool,
                connection_id,
                args.table,
                schema,
                tool.chat.database.clone(),
            )
            .await
        }
        "describe_tables" => {
            let args: DescribeTablesArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            let schema = args.schema.or_else(|| tool.chat.schema.clone());
            describe_tables(
                tool,
                connection_id,
                args.tables,
                schema,
                tool.chat.database.clone(),
            )
            .await
        }
        "run_query" => {
            let args: RunQueryArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            run_query(tool, connection_id, args).await
        }
        "explain_query" => {
            let args: ExplainQueryArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => return ToolOutcome::error(format!("Invalid arguments: {error}")),
            };
            explain_query(tool, connection_id, args).await
        }
        other => ToolOutcome::error(format!("Unknown tool: {other}")),
    }
}

async fn list_tables(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    schema: Option<String>,
    database: Option<String>,
    offset: usize,
    limit: usize,
) -> ToolOutcome {
    let tables = match guarded_database_call(
        tool,
        connection_id,
        "table listing",
        ai_query_guard::METADATA_TIMEOUT,
        metadata::get_tables(
            tool.runtime,
            tool.web_session_id,
            connection_id,
            schema,
            database,
        ),
    )
    .await
    {
        Ok(tables) => tables,
        Err(outcome) => return outcome,
    };
    match tables {
        Ok(tables) => {
            let total = tables.len();
            let limit = limit.clamp(1, MAX_TABLES_PER_PAGE);
            let names = tables
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|table| table.name)
                .collect::<Vec<_>>();
            let page_end = offset.saturating_add(names.len());
            let next_offset = (page_end < total).then_some(page_end);
            let summary = if let Some(next) = next_offset {
                format!("{} of {total} tables (next offset: {next})", names.len())
            } else {
                format!("{} of {total} tables", names.len())
            };
            ToolOutcome::ok(
                summary,
                json!({
                    "tables": names,
                    "total": total,
                    "offset": offset,
                    "limit": limit,
                    "nextOffset": next_offset,
                    "hasMore": next_offset.is_some(),
                }),
            )
        }
        Err(error) => ToolOutcome::error(format!("Failed to list tables: {error}")),
    }
}

async fn describe_tables(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    tables: Vec<String>,
    schema: Option<String>,
    database: Option<String>,
) -> ToolOutcome {
    if tables.is_empty() || tables.len() > MAX_TABLES_PER_BATCH {
        return ToolOutcome::error(format!(
            "`tables` must contain between 1 and {MAX_TABLES_PER_BATCH} names."
        ));
    }
    if tables.iter().any(|table| table.trim().is_empty()) {
        return ToolOutcome::error("Table names must not be empty.");
    }

    let mut results = Vec::with_capacity(tables.len());
    let mut failures = 0;
    for table in tables {
        let outcome = describe_table(
            tool,
            connection_id,
            table.clone(),
            schema.clone(),
            database.clone(),
        )
        .await;
        if outcome.status == ChatToolStatus::Ok {
            results.push(outcome.result);
        } else {
            failures += 1;
            results.push(json!({"table": table, "error": outcome.summary}));
        }
    }

    ToolOutcome::ok(
        format!(
            "{} tables described{}",
            results.len(),
            if failures == 0 {
                String::new()
            } else {
                format!(", {failures} failed")
            }
        ),
        json!({"tables": results, "count": results.len(), "failures": failures}),
    )
}

async fn describe_table(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    table: String,
    schema: Option<String>,
    database: Option<String>,
) -> ToolOutcome {
    let description = guarded_database_call(
        tool,
        connection_id,
        "table description",
        ai_query_guard::METADATA_TIMEOUT,
        async {
            let columns = metadata::get_columns(
                tool.runtime,
                tool.web_session_id,
                connection_id,
                table.clone(),
                schema.clone(),
                database.clone(),
            )
            .await;
            let indexes = metadata::get_indexes(
                tool.runtime,
                tool.web_session_id,
                connection_id,
                table.clone(),
                schema.clone(),
                database.clone(),
            )
            .await;
            let foreign_keys = metadata::get_foreign_keys(
                tool.runtime,
                tool.web_session_id,
                connection_id,
                table.clone(),
                schema,
                database,
            )
            .await;
            (columns, indexes, foreign_keys)
        },
    )
    .await;
    let (columns, indexes, foreign_keys) = match description {
        Ok(description) => description,
        Err(outcome) => return outcome,
    };

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
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    args: RunQueryArgs,
) -> ToolOutcome {
    let limit = args.limit.unwrap_or(DEFAULT_ROWS).clamp(1, MAX_ROWS);
    if let Err(reason) = ai_sql_safety::validate_ai_sql(&args.sql) {
        return ToolOutcome::denied(reason);
    }
    let (sql, approved) = match classify_statement(&args.sql) {
        StatementClass::Rejected(reason) => return ToolOutcome::denied(reason),
        StatementClass::ReadOnly => (args.sql, false),
        StatementClass::Write(kind) => match request_approval(tool, &args.sql, kind).await {
            ApprovalOutcome::Approved(sql) => {
                if let Err(reason) = ai_sql_safety::validate_ai_sql(&sql) {
                    return ToolOutcome::denied(reason);
                }
                match classify_statement(&sql) {
                    StatementClass::Rejected(reason) => return ToolOutcome::denied(reason),
                    _ => (sql, true),
                }
            }
            ApprovalOutcome::Denied(reason) => return ToolOutcome::denied(reason),
            ApprovalOutcome::Cancelled => return ToolOutcome::denied(
                "The user stopped the turn before approving this statement. Nothing was executed.",
            ),
            ApprovalOutcome::TimedOut => {
                return ToolOutcome::error("Timed out waiting for approval. Nothing was executed.")
            }
        },
    };
    execute_statement(tool, connection_id, sql, limit, approved).await
}

async fn execute_statement(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    sql: String,
    limit: u32,
    approved: bool,
) -> ToolOutcome {
    let _permit = match acquire_query_permit(tool, connection_id).await {
        Ok(permit) => permit,
        Err(outcome) => return outcome,
    };
    let scope = QueryRequestScope {
        session_id: tool.web_session_id,
        request_id: Some(tool.request_id),
    };
    let started = Instant::now();
    let query = queries::execute_query(
        tool.runtime,
        &tool.state.query_cancellation,
        scope,
        QueryResponsePolicy::Unbounded,
        connection_id.to_string(),
        sql,
        Some(limit),
        Some(1),
        tool.chat.schema.clone(),
        None,
        tool.chat.database.clone(),
    );
    tokio::pin!(query);
    let result = tokio::select! {
        result = &mut query => result,
        _ = tokio::time::sleep(QUERY_TIMEOUT) => {
            return ToolOutcome::error(format!(
                "Query timed out after {} seconds and was cancelled to protect database stability.",
                QUERY_TIMEOUT.as_secs()
            ));
        }
        _ = wait_for_cancellation(tool) => {
            return ToolOutcome::denied(
                "The user stopped the turn. The running query was cancelled."
            );
        }
    };
    match result {
        Ok(result) => {
            let row_count = result.rows.len();
            let truncated = result.truncated || row_count > limit as usize;
            let is_result_set = !result.columns.is_empty();
            let affected_rows = result.affected_rows;
            let rows = result
                .rows
                .into_iter()
                .take(limit as usize)
                .collect::<Vec<_>>();
            let prefix = if approved { "Approved. " } else { "" };
            let elapsed_ms = started.elapsed().as_millis();
            let summary = if is_result_set {
                format!("{prefix}{row_count} rows returned in {elapsed_ms} ms")
            } else {
                format!("{prefix}{affected_rows} rows affected in {elapsed_ms} ms")
            };
            ToolOutcome::ok(
                summary,
                json!({
                    "columns": result.columns,
                    "rows": rows,
                    "rowCount": row_count,
                    "affectedRows": affected_rows,
                    "truncated": truncated,
                    "approved": approved,
                }),
            )
        }
        Err(error) => ToolOutcome::error(format!("Query failed: {error}")),
    }
}

pub(super) fn query_semaphore(connection_id: &str) -> Arc<Semaphore> {
    ai_query_guard::semaphore(connection_id)
}

async fn acquire_query_permit(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
) -> Result<OwnedSemaphorePermit, ToolOutcome> {
    tokio::select! {
        permit = ai_query_guard::acquire(connection_id) => permit.map_err(ToolOutcome::error),
        _ = wait_for_cancellation(tool) => Err(ToolOutcome::denied(
            "The user stopped the turn before the query started."
        )),
    }
}

async fn guarded_database_call<T, F>(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    label: &str,
    timeout: Duration,
    future: F,
) -> Result<T, ToolOutcome>
where
    F: Future<Output = T>,
{
    let _permit = acquire_query_permit(tool, connection_id).await?;
    tokio::pin!(future);
    tokio::select! {
        result = &mut future => Ok(result),
        _ = tokio::time::sleep(timeout) => Err(ToolOutcome::error(format!(
            "{label} timed out after {} seconds and was cancelled to protect database stability.",
            timeout.as_secs()
        ))),
        _ = wait_for_cancellation(tool) => Err(ToolOutcome::denied(format!(
            "The user stopped the turn. The running {label} was cancelled."
        ))),
    }
}

async fn wait_for_cancellation(tool: &ToolCallContext<'_>) {
    while !tool.cancelled() {
        tokio::time::sleep(CANCELLATION_POLL_INTERVAL).await;
    }
}

async fn explain_query(
    tool: &ToolCallContext<'_>,
    connection_id: &str,
    args: ExplainQueryArgs,
) -> ToolOutcome {
    if args.analyze {
        return ToolOutcome::denied(
            "EXPLAIN ANALYZE is disabled in DBA chat because it executes the statement. Use plain EXPLAIN or run it explicitly in the editor.",
        );
    }
    let _permit = match acquire_query_permit(tool, connection_id).await {
        Ok(permit) => permit,
        Err(outcome) => return outcome,
    };
    let scope = QueryRequestScope {
        session_id: tool.web_session_id,
        request_id: Some(tool.request_id),
    };
    let query = queries::explain_query_plan(
        tool.runtime,
        &tool.state.query_cancellation,
        scope,
        connection_id.to_string(),
        args.sql,
        args.analyze,
        tool.chat.schema.clone(),
        tool.chat.database.clone(),
    );
    tokio::pin!(query);
    let result = tokio::select! {
        result = &mut query => result,
        _ = tokio::time::sleep(QUERY_TIMEOUT) => {
            return ToolOutcome::error(format!(
                "Explain timed out after {} seconds and was cancelled to protect database stability.",
                QUERY_TIMEOUT.as_secs()
            ));
        }
        _ = wait_for_cancellation(tool) => {
            return ToolOutcome::denied(
                "The user stopped the turn. The running explain was cancelled."
            );
        }
    };
    match result {
        Ok(plan) => match serde_json::to_value(&plan) {
            Ok(value) => ToolOutcome::ok("Execution plan captured", value),
            Err(error) => ToolOutcome::error(format!("Failed to serialize the plan: {error}")),
        },
        Err(error) => ToolOutcome::error(format!("Failed to explain the statement: {error}")),
    }
}

enum ApprovalOutcome {
    Approved(String),
    Denied(String),
    Cancelled,
    TimedOut,
}

/// Holds a write statement behind the same pending-approval queue the MCP
/// server uses, so the approval modal, audit trail, and edited-query override
/// apply unchanged.
async fn request_approval(tool: &ToolCallContext<'_>, sql: &str, kind: &str) -> ApprovalOutcome {
    let config_dir = tool.runtime.paths.config_dir().to_path_buf();
    let approval_id = ai_approval::new_approval_id();
    let pending = PendingApproval {
        id: approval_id.clone(),
        created_at: crate::ai_activity::now_iso8601(),
        session_id: tool.chat_session_id.to_string(),
        connection_id: tool.chat.connection_id.clone().unwrap_or_default(),
        connection_name: tool
            .chat
            .connection_name
            .clone()
            .or_else(|| tool.chat.connection_id.clone())
            .unwrap_or_default(),
        database: tool.chat.database.clone(),
        query: sql.to_string(),
        query_kind: kind.to_string(),
        client_hint: Some("SuperDBA chat".to_string()),
        explain_plan: None,
        explain_error: None,
    };
    if let Err(error) = ai_approval::write_pending_in(&config_dir, &pending) {
        return ApprovalOutcome::Denied(format!("Could not request approval: {error}"));
    }
    if let Ok(payload) = serde_json::to_value(&pending) {
        let emitted = match tool.web_session_id {
            Some(session_id) => {
                tool.runtime
                    .events
                    .emit_to(session_id, "ai://pending_approval", payload)
            }
            None => tool.runtime.events.emit("ai://pending_approval", payload),
        };
        if let Err(error) = emitted {
            log::warn!("Failed to emit the pending chat approval: {error}");
        }
    }
    emit_chat_event(
        tool.runtime,
        tool.web_session_id,
        json!({
            "type": "approvalRequired",
            "requestId": tool.request_id,
            "approvalId": approval_id,
            "query": sql,
            "queryKind": kind,
        }),
    );
    let started = Instant::now();
    loop {
        if tool.cancelled() {
            let _ = ai_approval::clear_in(&config_dir, &approval_id);
            emit_chat_event(
                tool.runtime,
                tool.web_session_id,
                json!({
                    "type": "approvalResolved",
                    "requestId": tool.request_id,
                    "approvalId": approval_id,
                    "decision": "cancelled",
                }),
            );
            return ApprovalOutcome::Cancelled;
        }
        match ai_approval::read_decision_in(&config_dir, &approval_id) {
            Ok(Some(decision)) => {
                let _ = ai_approval::clear_in(&config_dir, &approval_id);
                emit_chat_event(
                    tool.runtime,
                    tool.web_session_id,
                    json!({
                        "type": "approvalResolved",
                        "requestId": tool.request_id,
                        "approvalId": approval_id,
                        "decision": decision.decision,
                    }),
                );
                if decision.decision == "approve" {
                    return ApprovalOutcome::Approved(
                        decision.edited_query.unwrap_or_else(|| sql.to_string()),
                    );
                }
                return ApprovalOutcome::Denied(
                    decision
                        .reason
                        .unwrap_or_else(|| "The user denied this statement.".to_string()),
                );
            }
            Ok(None) => {}
            Err(error) => {
                let _ = ai_approval::clear_in(&config_dir, &approval_id);
                return ApprovalOutcome::Denied(format!("Approval failed: {error}"));
            }
        }
        if started.elapsed() >= APPROVAL_TIMEOUT {
            let _ = ai_approval::clear_in(&config_dir, &approval_id);
            emit_chat_event(
                tool.runtime,
                tool.web_session_id,
                json!({
                    "type": "approvalResolved",
                    "requestId": tool.request_id,
                    "approvalId": approval_id,
                    "decision": "timeout",
                }),
            );
            return ApprovalOutcome::TimedOut;
        }
        tokio::time::sleep(APPROVAL_POLL_INTERVAL).await;
    }
}

/// How the chat tool will treat a statement.
pub(crate) enum StatementClass {
    ReadOnly,
    Write(&'static str),
    Rejected(String),
}

const WRITE_KEYWORDS: [&str; 14] = [
    "INSERT", "UPDATE", "DELETE", "REPLACE", "MERGE", "CREATE", "DROP", "ALTER", "TRUNCATE",
    "GRANT", "REVOKE", "COPY", "CALL", "VACUUM",
];

/// Classifies a statement for the chat tool. The check runs on a copy with
/// string literals and comments removed, so a semicolon inside a literal or a
/// DML keyword inside a comment does not change the verdict.
pub(crate) fn classify_statement(sql: &str) -> StatementClass {
    let stripped = strip_sql_noise(sql);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return StatementClass::Rejected("The statement is empty.".to_string());
    }
    let without_trailing = trimmed.trim_end_matches(';').trim_end();
    if without_trailing.contains(';') {
        return StatementClass::Rejected(
            "Only one statement can run through chat. Run multiple statements in the editor."
                .to_string(),
        );
    }
    let words = words_of(without_trailing);
    let Some(first) = words.first() else {
        return StatementClass::Rejected("The statement is empty.".to_string());
    };
    if first == "SELECT" {
        return StatementClass::ReadOnly;
    }
    if first == "WITH" {
        let writes = words
            .iter()
            .any(|word| WRITE_KEYWORDS.contains(&word.as_str()));
        return if writes {
            StatementClass::Write("WITH (data change)")
        } else {
            StatementClass::ReadOnly
        };
    }
    if WRITE_KEYWORDS.contains(&first.as_str()) {
        return StatementClass::Write(statement_kind(first));
    }
    StatementClass::Rejected(format!(
        "Chat cannot run `{first}`. Supported: a read-only SELECT, or one write statement that the user approves."
    ))
}

fn statement_kind(word: &str) -> &'static str {
    match word {
        "INSERT" => "INSERT",
        "UPDATE" => "UPDATE",
        "DELETE" => "DELETE",
        "REPLACE" => "REPLACE",
        "MERGE" => "MERGE",
        "CREATE" | "DROP" | "ALTER" | "TRUNCATE" => "DDL",
        "GRANT" | "REVOKE" => "PRIVILEGES",
        "COPY" | "CALL" | "VACUUM" => "ADMIN",
        _ => "WRITE",
    }
}

/// Kept for callers that only need the read-only verdict (tests, docs).
pub(crate) fn is_single_read_only_statement(sql: &str) -> Result<(), String> {
    match classify_statement(sql) {
        StatementClass::ReadOnly => Ok(()),
        StatementClass::Write(_) => {
            Err("Chat can only run read-only SELECT statements without approval.".to_string())
        }
        StatementClass::Rejected(reason) => Err(reason),
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
