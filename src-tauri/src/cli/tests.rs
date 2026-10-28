use super::{try_parse_from, Args, Command, WebArgs, WebAuthMode};
use clap::{error::ErrorKind, Parser};
use std::path::PathBuf;

fn web_args(args: &Args) -> &WebArgs {
    match args.command.as_ref() {
        Some(Command::Web(web_args)) => web_args,
        None => panic!("expected the web subcommand"),
    }
}

#[test]
fn parses_desktop_defaults() {
    let args = Args::try_parse_from(["superdba"]).expect("defaults should parse");

    assert!(!args.mcp);
    assert!(!args.debug);
    assert!(args.explain.is_none());
    assert!(args.command.is_none());
}

#[test]
fn parses_all_web_options() {
    let args = Args::try_parse_from([
        "superdba",
        "web",
        "--host",
        "localhost",
        "--port",
        "9090",
        "--no-open",
        "--web-root",
        "/tmp/superdba-web",
        "--auth",
        "password",
        "--public-url",
        "https://superdba.example.com",
        "--allowed-origin",
        "https://superdba.example.com",
        "--allowed-origin",
        "https://admin.example.com",
        "--allow-high-risk",
        "--server-file-browser-root",
        "/srv/databases",
        "--server-file-browser-root",
        "/var/backups",
        "--debug",
    ])
    .expect("web options should parse");

    let web_args = web_args(&args);
    assert_eq!(web_args.host, "localhost");
    assert_eq!(web_args.port, 9090);
    assert!(web_args.no_open);
    assert_eq!(web_args.web_root, Some(PathBuf::from("/tmp/superdba-web")));
    assert_eq!(web_args.auth, Some(WebAuthMode::Password));
    assert_eq!(
        web_args.public_url.as_deref(),
        Some("https://superdba.example.com")
    );
    assert_eq!(
        web_args.allowed_origins,
        [
            "https://superdba.example.com".to_string(),
            "https://admin.example.com".to_string()
        ]
    );
    assert!(web_args.allow_high_risk);
    assert_eq!(
        web_args.server_file_browser_roots,
        [
            PathBuf::from("/srv/databases"),
            PathBuf::from("/var/backups")
        ]
    );
    assert!(args.debug);
}

#[test]
fn web_conflicts_with_mcp_mode() {
    let error = Args::try_parse_from(["superdba", "--mcp", "web"])
        .expect_err("web and MCP modes must conflict");

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}

#[test]
fn web_conflicts_with_explain_mode() {
    let error = Args::try_parse_from(["superdba", "--explain", "plan.json", "web"])
        .expect_err("web and explain modes must conflict");

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}

#[test]
fn web_options_require_web_subcommand() {
    for option in [
        "--host",
        "--port",
        "--web-root",
        "--auth",
        "--public-url",
        "--allowed-origin",
        "--server-file-browser-root",
    ] {
        let value = match option {
            "--host" => "localhost",
            "--port" => "9090",
            "--web-root" => "/tmp/superdba-web",
            "--auth" => "proxy",
            "--public-url" | "--allowed-origin" => "https://superdba.example.com",
            "--server-file-browser-root" => "/srv/databases",
            _ => unreachable!(),
        };
        let error = Args::try_parse_from(["superdba", option, value])
            .expect_err("web options must require the web subcommand");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    for option in ["--no-open", "--allow-high-risk"] {
        let error = Args::try_parse_from(["superdba", option])
            .expect_err("the Web option must require the web subcommand");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }
}

#[test]
fn rejects_legacy_web_flag() {
    let error = Args::try_parse_from(["superdba", "--web"])
        .expect_err("web must be a subcommand, not a flag");

    assert_eq!(error.kind(), ErrorKind::UnknownArgument);
}

#[test]
fn help_lists_web_subcommand_and_options() {
    let error = Args::try_parse_from(["superdba", "--help"])
        .expect_err("help should be returned as a clap display error");

    assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    assert!(error.to_string().contains("web"));

    let error = Args::try_parse_from(["superdba", "web", "--help"])
        .expect_err("web help should be returned as a clap display error");
    assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    let help = error.to_string();
    for option in [
        "--host <HOST>",
        "--port <PORT>",
        "--no-open",
        "--web-root <PATH>",
        "--auth <MODE>",
        "--public-url <URL>",
        "--allowed-origin <ORIGIN>",
        "--allow-high-risk",
        "--server-file-browser-root <PATH>",
    ] {
        assert!(
            help.contains(option),
            "help did not contain {option}: {help}"
        );
    }
    assert!(help.contains("127.0.0.1"));
    assert!(help.contains("8080"));
}

#[test]
fn platform_launch_arguments_fall_back_to_desktop_defaults() {
    for platform_argument in ["-psn_0_12345", "superdba://plugins/example"] {
        let args = try_parse_from(["superdba", platform_argument])
            .expect("platform launch metadata should use desktop defaults");

        assert!(args.command.is_none());
        assert!(!args.mcp);
        assert!(!args.debug);
        assert!(args.explain.is_none());
    }
}

#[test]
fn user_parse_errors_do_not_fall_back_to_desktop_mode() {
    let error = try_parse_from(["superdba", "--mcp", "web"])
        .expect_err("a mode conflict must remain visible to the user");

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}
