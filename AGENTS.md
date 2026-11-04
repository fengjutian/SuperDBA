# AGENTS.md

> [!IMPORTANT]
> **The backend layout changed (October 2026).** All Tauri code now lives in `src-tauri/src/transport/desktop/`, and desktop commands are thin wrappers over `src-tauri/src/application/`. Read [Backend layout](#backend-layout-desktop-and-web-transports) before you start, rebase, or review a PR that touches `src-tauri/`.

## Directives
Adhere to the rules defined in the [rules directory](./.rules/):
- [General Rules](./.rules/general.md) (Logging & Language)
- [Rust Rules](./.rules/rust.md) (Backend module structure and Rust testing)
- [TypeScript Rules](./.rules/typescript.md)
- [React Rules](./.rules/react.md)
- [Modal Styling Rules](./.rules/modals.md) (Modal component structure and styling)
- [Testing Conventions](./.rules/testing.md) (Test file organization and structure)
- [Design Token Rules](./.rules/design.md) (Theme tokens for colors, radii and fonts; see also [DESIGN.md](./docs/DESIGN.md))
- [Accessibility Rules](./.rules/accessibility.md) (jsx-a11y lint, keyboard access, dialogs, focus)
- [Plugin Driver Rules](./.rules/plugins.md) (capabilities over driver ids, dialect-aware SQL, plugin contract)

When reviewing pull requests, follow [REVIEW.md](./REVIEW.md).

## Creating feature issues

- Write issues in English and follow [the Feature Proposal template](./.github/ISSUE_TEMPLATE/feature_request.yml), including when creating issues through an API or CLI.
- Use the `[Feat]: ` title prefix and the existing `feature request` label. Check existing issues for duplicates and inspect available labels before adding other labels.
- Include Prerequisites, Feature Proposal, Motivation, Example, Acceptance criteria, and the Contributor notes footer. Add Where to start when concrete implementation pointers have been verified; do not invent file paths or claim unverified bugs.
- Keep each issue focused on one capability, with explicit scope and testable acceptance criteria. Link related issues when useful.
- Always retain the Contributor notes footer with links to the repository rules and [Discord](https://discord.com/invite/K2hmhfHRSt). Adapt implementation and validation notes to the issue and current repository layout.
- Assign `good first issue` only to bounded work with a clear implementation path that does not require architectural decisions or security-sensitive authorization changes. Do not apply it just because a UI change looks small.

## Backend layout: desktop and web transports

The Rust backend serves two transports over the same React frontend:

- **Tauri desktop app:** `src-tauri/src/transport/desktop/`
- **`superdba web` server:** `src-tauri/src/transport/web/`

Shared logic lives in `src-tauri/src/application/` (and plain modules), so the app and the web behave the same.

- **Tauri code goes in `transport/desktop/` only.** That covers `#[tauri::command]`, `tauri_plugin_*`, and anything taking an `AppHandle`, `Window` or Tauri `State`. It is the only module behind the `desktop` cargo feature; `cargo build --no-default-features` builds the server without Tauri.
- **A desktop command is a thin wrapper over `application/`.** Implement the behaviour there, call it from the desktop command, and give the web RPC (`transport/web/rpc.rs`) a route to the same function.
- **The desktop command registry is in `transport/desktop/mod.rs`** (`run()` and `generate_handler!`), not in `lib.rs`.
- **CI enforces the boundary:** `cargo check --no-default-features` must pass, and a grep fails the build when `tauri::` or `tauri_plugin` appears outside `transport/desktop/`. See rule 7 in [Rust Rules](./.rules/rust.md).

**Branches and PRs started before this layout:**
- Rebase onto it.
- Move new or changed Tauri commands from `commands.rs`, `config.rs` and the like into the mirrored file under `transport/desktop/`.
- Drop `#[cfg(feature = "desktop")]` and `#[cfg_attr(feature = "desktop", tauri::command)]` from shared modules.
- Register commands in `transport/desktop/mod.rs`.
- Check for these commands, which had no frontend caller and were removed: `list_all_preferences`, `get_theme`, `save_blob_to_file`, `preview_superdba_import`, `apply_superdba_import`, `export_connections_payload`, `import_connections_payload`, `encrypt_export_payload`, `decrypt_export_payload`, `open_visual_explain_window`, `open_json_viewer_window`, `get_json_viewer_session`, `complete_json_viewer_session`, `open_results_window` and `close_results_window`.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **SuperDBA** (37564 symbols, 83670 relationships, 621 execution flows).

> Index stale? Run `node .gitnexus/run.cjs analyze --index-only` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? Bootstrap with `npx`, `bunx`, or `pnpm dlx` — e.g. `bunx gitnexus@latest analyze` (npm 11 npx crash; #1939).

## Always Do

- **MUST run impact before editing.** Use `impact({target: "symbolName", direction: "upstream"})` or `node .gitnexus/run.cjs impact "symbolName" --direction upstream --repo .`; report callers, processes, and risk. Never substitute grep for graph analysis.
- **MUST analyze graph changes before committing.** Use `detect_changes({scope: "all"})` (MCP) or `node .gitnexus/run.cjs detect-changes --scope all --repo .` (CLI fallback). `partial: true` or `truncated: true` is not a clean check — a zero means unseen, not unaffected; re-run it. For regression review: `detect_changes({scope: "compare", base_ref: "main"})` or `node .gitnexus/run.cjs detect-changes --scope compare --base-ref "main" --repo .`.
- MUST warn on HIGH/CRITICAL `risk` pre-edit; never use `riskSharedAxes` to waive a HIGH/CRITICAL `risk` warning. Compare File/symbol: MCP File omits axes; Graph-RAG expands File.
- **MUST treat `risk: UNKNOWN` as unresolved, not as low.** An empty caller set is not evidence the symbol is unused — it can also mean the callers are not resolvable by the index (plain-object property access, dynamic dispatch, cross-language calls). `impact` pairs `UNKNOWN` with a `riskNote` saying so. Confirm with a text search before treating the symbol as safe to change or delete; do not proceed on the strength of a zero.
- **MUST use `query({search_query: "concept"})` for concepts/flows, `context({name: "symbolName"})` for a named symbol, or `impact` for blast radius, on read-only callers, dependencies, imports, or execution flow.** Graph first; text search only for empty/`UNKNOWN`/literals.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method before MCP/CLI impact analysis.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis, and never read `UNKNOWN` as an all-clear — it means the walk could not answer, which is the one verdict that requires confirming by other means.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit before MCP/CLI graph change analysis.

## Resources

| Resource | Use for |
| --- | --- |
| `gitnexus://repo/SuperDBA/context` | Codebase overview, check index freshness |
| `gitnexus://repo/SuperDBA/clusters` | All functional areas |
| `gitnexus://repo/SuperDBA/processes` | All execution flows |
| `gitnexus://repo/SuperDBA/process/{name}` | Step-by-step execution trace |

<!-- gitnexus:end -->
