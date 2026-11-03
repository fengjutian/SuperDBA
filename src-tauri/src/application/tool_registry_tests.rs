use super::tool_registry::{for_surface, is_read_only, spec, ToolSurface, TOOLS};
use std::collections::HashSet;

#[test]
fn tool_names_are_unique() {
    let names = TOOLS.iter().map(|tool| tool.name).collect::<HashSet<_>>();
    assert_eq!(names.len(), TOOLS.len());
}

#[test]
fn surfaces_expose_the_expected_tools() {
    let chat = for_surface(ToolSurface::Chat)
        .map(|tool| tool.name)
        .collect::<HashSet<_>>();
    assert_eq!(
        chat,
        HashSet::from([
            "list_tables",
            "describe_table",
            "describe_tables",
            "run_query",
            "explain_query",
            "diagnose_query"
        ])
    );

    let mcp = for_surface(ToolSurface::Mcp)
        .map(|tool| tool.name)
        .collect::<HashSet<_>>();
    assert_eq!(
        mcp,
        HashSet::from([
            "list_connections",
            "list_databases",
            "list_tables",
            "describe_table",
            "run_query"
        ])
    );
}

#[test]
fn every_surface_tool_has_a_description() {
    for tool in TOOLS {
        if tool.mcp {
            assert!(!tool.mcp_description.is_empty(), "{}", tool.name);
        }
        if tool.chat {
            assert!(!tool.chat_description.is_empty(), "{}", tool.name);
        }
    }
}

#[test]
fn read_only_policy_is_centralised() {
    assert!(is_read_only("list_tables"));
    assert!(is_read_only("describe_table"));
    assert!(is_read_only("describe_tables"));
    assert!(is_read_only("explain_query"));
    assert!(!is_read_only("run_query"));
    // Unknown tools fail closed.
    assert!(!is_read_only("drop_everything"));
}

#[test]
fn lookup_is_exact() {
    assert_eq!(spec("run_query").map(|tool| tool.chat), Some(true));
    assert!(spec("Run_Query").is_none());
    assert!(spec("").is_none());
}
