//! Canonical identity of the database tools.
//!
//! The MCP server and the in-app chat agent expose overlapping tools with
//! different parameter shapes (MCP clients pass `connection_id`; the chat agent
//! binds the active connection). This module owns the single source of truth for
//! a tool's *identity* — name, surface membership, read-only policy, and the
//! per-surface description — so a tool cannot be added to one surface and
//! forgotten on the other. Each surface still owns its wire schema.

/// Where a tool is exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolSurface {
    Mcp,
    Chat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: &'static str,
    /// Description shown to MCP clients. Empty for chat-only tools.
    pub mcp_description: &'static str,
    /// Description shown to the in-app model. Empty for MCP-only tools.
    pub chat_description: &'static str,
    /// True when the tool never mutates database state.
    pub read_only: bool,
    pub mcp: bool,
    pub chat: bool,
}

pub const TOOLS: &[ToolSpec] = &[
    ToolSpec {
        name: "list_connections",
        mcp_description: "List all saved database connections",
        chat_description: "",
        read_only: true,
        mcp: true,
        chat: false,
    },
    ToolSpec {
        name: "list_databases",
        mcp_description: "List all databases available for a connection",
        chat_description: "",
        read_only: true,
        mcp: true,
        chat: false,
    },
    ToolSpec {
        name: "list_tables",
        mcp_description: "List all tables in a database connection",
        chat_description:
            "List tables in the active connection, optionally limited to one schema.",
        read_only: true,
        mcp: true,
        chat: true,
    },
    ToolSpec {
        name: "describe_table",
        mcp_description: "Get the full schema of a table: columns, indexes, and foreign keys",
        chat_description: "Describe one table: columns, indexes, and foreign keys.",
        read_only: true,
        mcp: true,
        chat: true,
    },
    ToolSpec {
        name: "describe_tables",
        mcp_description: "",
        chat_description:
            "Describe a batch of up to five tables for cross-table schema and index analysis.",
        read_only: true,
        mcp: false,
        chat: true,
    },
    ToolSpec {
        name: "run_query",
        mcp_description: "Execute a SQL query on a specific connection. If the query already contains a LIMIT clause, it will be respected.",
        chat_description: "Run one SQL statement and return at most 50 rows. A write or DDL statement is queued for the user's approval before it executes.",
        read_only: false,
        mcp: true,
        chat: true,
    },
    ToolSpec {
        name: "explain_query",
        mcp_description: "",
        chat_description: "Capture the database execution plan for a SQL statement without running it.",
        read_only: true,
        mcp: false,
        chat: true,
    },
    ToolSpec {
        name: "diagnose_query",
        mcp_description: "",
        chat_description: "Diagnose one SQL statement: capture its plan, summarize scans, relations, and index usage, and suggest the next checks.",
        read_only: true,
        mcp: false,
        chat: true,
    },
];

pub fn spec(name: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|tool| tool.name == name)
}

pub fn for_surface(surface: ToolSurface) -> impl Iterator<Item = &'static ToolSpec> {
    TOOLS.iter().filter(move |tool| match surface {
        ToolSurface::Mcp => tool.mcp,
        ToolSurface::Chat => tool.chat,
    })
}

/// Whether a tool may change database state. Unknown tools are treated as
/// mutating so a caller fails closed.
pub fn is_read_only(name: &str) -> bool {
    spec(name).map(|tool| tool.read_only).unwrap_or(false)
}
