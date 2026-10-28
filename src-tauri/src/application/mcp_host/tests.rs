use super::*;

#[test]
fn lists_codex_as_command_client() {
    let clients = get_all_clients();
    let codex = clients
        .iter()
        .find(|client| client.id == "codex")
        .expect("Codex should be listed as an MCP client");

    assert_eq!(codex.name, "Codex");
    assert_eq!(codex.client_type, "command");
    assert!(
        codex
            .config_path
            .as_ref()
            .is_some_and(|path| path.ends_with(".codex/config.toml")),
        "Codex config path should point to ~/.codex/config.toml"
    );
}

#[test]
fn builds_codex_manual_command() {
    assert_eq!(
        build_manual_command("codex", "/Applications/SuperDBA.app/superdba").as_deref(),
        Some("codex mcp add superdba -- /Applications/SuperDBA.app/superdba --mcp")
    );
}

#[test]
fn builds_claude_code_manual_command() {
    assert_eq!(
        build_manual_command("claude_code", "/Applications/SuperDBA.app/superdba").as_deref(),
        Some(
            "claude mcp add --scope user superdba /Applications/SuperDBA.app/superdba -- --mcp"
        )
    );
}

#[test]
fn detects_codex_toml_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[mcp_servers.superdba]
command = "/Applications/SuperDBA.app/superdba"
args = ["--mcp"]
"#,
    )
    .unwrap();

    assert!(is_command_client_installed(&config_path));
}

#[test]
fn lists_minimax_code_as_file_client() {
    let clients = get_all_clients();
    let minimax = clients
        .iter()
        .find(|client| client.id == "minimax")
        .expect("MiniMax Code should be listed as an MCP client");

    assert_eq!(minimax.name, "MiniMax Code");
    assert_eq!(minimax.client_type, "file");
    assert!(
        minimax
            .config_path
            .as_ref()
            .is_some_and(|path| path.ends_with(".minimax/mcp.json")),
        "MiniMax Code config path should point to ~/.minimax/mcp.json"
    );
}

#[test]
fn minimax_entry_declares_stdio_transport() {
    let entry = build_stdio_server_entry("minimax", "/Applications/SuperDBA.app/superdba");

    assert_eq!(entry["command"], "/Applications/SuperDBA.app/superdba");
    assert_eq!(entry["args"], json!(["--mcp"]));
    assert_eq!(entry["type"], "stdio");
    assert_eq!(entry["enabled"], true);
}

#[test]
fn other_file_clients_keep_a_minimal_entry() {
    let entry = build_stdio_server_entry("cursor", "/Applications/SuperDBA.app/superdba");

    assert_eq!(entry["command"], "/Applications/SuperDBA.app/superdba");
    assert!(entry.get("type").is_none(), "unexpected transport metadata");
    assert!(entry.get("enabled").is_none(), "unexpected enabled flag");
}

#[test]
fn rejects_unknown_host_client_without_writing_configuration() {
    let error = install_config_blocking("unknown-client")
        .expect_err("unknown MCP clients must be rejected");

    assert_eq!(error, "Unknown client: unknown-client");
}

#[test]
fn claude_desktop_config_is_directly_under_the_os_claude_directory() {
    let clients = get_all_clients();
    let claude = clients.iter().find(|client| client.id == "claude").unwrap();
    assert_eq!(
        claude.config_path,
        Some(
            BaseDirs::new()
                .unwrap()
                .config_dir()
                .join("Claude")
                .join("claude_desktop_config.json")
        )
    );
}
