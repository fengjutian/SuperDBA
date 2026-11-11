<h1 align="center">SuperDBA</h1>

<p align="center"><strong>The open-source database workspace for people and AI agents.</strong></p>

<p align="center">
  Explore schemas, write SQL, edit data, build visual queries, inspect execution plans, and create notebooks from a native desktop app or your browser.
</p>

<p align="center">
  <a href="https://superdba.dev">Website</a> ·
  <a href="https://superdba.dev/wiki">Documentation</a> ·
  <a href="https://superdba.dev/download">Download</a> ·
  <a href="https://superdba.dev/plugins">Plugins</a> ·
  <a href="https://discord.com/invite/K2hmhfHRSt">Discord</a>
</p>

<p align="center">
  <a href="https://github.com/SuperDBA/SuperDBA/releases"><img src="https://img.shields.io/github/v/release/SuperDBA/SuperDBA?label=release" alt="Latest release"></a>
  <a href="https://github.com/SuperDBA/SuperDBA/stargazers"><img src="https://img.shields.io/github/stars/SuperDBA/SuperDBA?style=flat" alt="GitHub stars"></a>
  <a href="https://github.com/SuperDBA/SuperDBA/actions"><img src="https://github.com/SuperDBA/SuperDBA/workflows/Release/badge.svg" alt="Build status"></a>
  <a href="https://discord.com/invite/K2hmhfHRSt"><img src="https://img.shields.io/discord/1502944695808950282?color=5865F2&logo=discord&logoColor=white" alt="Discord"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="Apache 2.0 license"></a>
</p>

<p align="center">
  <a href="./README.md">English</a> ·
  <a href="./docs/translations/README.zh-CN.md">简体中文</a> ·
  <a href="./docs/translations/README.es.md">Español</a> ·
  <a href="./docs/translations/README.fr.md">Français</a> ·
  <a href="./docs/translations/README.de.md">Deutsch</a> ·
  <a href="./docs/translations/README.it.md">Italiano</a> ·
  <a href="./docs/translations/README.ja.md">日本語</a> ·
  <a href="./docs/translations/README.ko.md">한국어</a> ·
  <a href="./docs/translations/README.pt-BR.md">Português</a> ·
  <a href="./docs/translations/README.ru.md">Русский</a> ·
  <a href="./docs/translations/README.tl.md">Tagalog</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/SuperDBA/website/main/public/img/overview.gif" alt="SuperDBA query editor and data grid" width="100%">
</p>

## One workspace for the whole database workflow

SuperDBA brings the tools around a database into one consistent workspace. Connections, permissions, schema context, query history, notebooks, visual tools, plugins, and optional AI assistance work together instead of living in separate applications.

- **Explore before you query.** Search tables, columns, keys, indexes, views, and routines; inspect relationships in an ER diagram.
- **Write and run SQL with context.** Use a Monaco-powered, dialect-aware editor with completion, formatting, statement selection, multiple result sets, saved queries, and history.
- **Work directly with results.** Filter, sort, paginate, export, and edit rows in a virtualized data grid with reviewable batch changes.
- **See what SQL is doing.** Build queries visually and turn supported `EXPLAIN` output into an interactive execution-plan view.
- **Make analysis repeatable.** Combine SQL, Markdown, parameters, charts, and cross-cell results in notebooks that export to HTML, CSV, or JSON.
- **Extend the workspace.** Add database drivers and UI contributions through installable plugins rather than waiting for a core release.
- **Bring your own AI.** Use optional text-to-SQL and chat features with supported providers or local Ollama models, or connect compatible clients through the built-in MCP server.

## Run it your way

The desktop app and Web mode share the same React interface and Rust application services. The transport changes; the product behavior stays aligned.

| Mode | Command | Use it for |
| --- | --- | --- |
| Desktop | `superdba` | Native dialogs, local files, OS credential storage, and daily desktop work |
| Web | `superdba web` | Browser access, containers, remote hosts, and managed deployments |
| MCP | `superdba --mcp` | Exposing controlled SuperDBA capabilities to compatible AI clients |

Web mode listens on loopback by default and uses an authenticated browser session. Before making it remotely reachable, read the [operations guide](./docs/web/WEB_MODE_OPERATIONS.md) and [remote security guide](./docs/web/WEB_REMOTE_SECURITY.md). Container and Helm deployment instructions are in [Web mode containers](./docs/web/WEB_MODE_CONTAINERS.md).

## Database support

SuperDBA ships with core SQL drivers and can add SQL, NoSQL, and service integrations through the plugin registry. MySQL/MariaDB, PostgreSQL, and SQLite workflows are supported; additional drivers include options for DuckDB, ClickHouse, SQL Server, Oracle, Redis, MongoDB, BigQuery, DynamoDB, Firestore, Trino, and more.

Drivers evolve independently from the application, and built-in drivers may move to plugins over time. Check the live [plugin registry](https://superdba.dev/plugins) for current availability, platform support, and maturity.

## Install

### Package managers

```bash
# Windows
winget install Debba.SuperDBA

# macOS
brew install --cask SuperDBA

# Linux (Snap)
sudo snap install SuperDBA
```

Linux packages are also available for Flatpak and AUR. AppImage, `.deb`, `.rpm`, Windows, and macOS installers are available from the [download page](https://superdba.dev/download) and [GitHub Releases](https://github.com/SuperDBA/SuperDBA/releases).

> [!IMPORTANT]
> Snap users must connect the password-manager interface before saving credentials:
>
> ```bash
> sudo snap connect superdba:password-manager-service
> ```

## Architecture

```text
packages/web-ui/                         React + TypeScript
       │
       ├── Tauri IPC ────────────────► src-tauri/src/transport/desktop/
       │
       └── HTTP / WebSocket ─────────► src-tauri/src/transport/web/
                                                │
                                                ▼
                                  src-tauri/src/application/
                                  shared application services
                                                │
                                                ▼
                              drivers · plugins · storage · security
```

Desktop commands and Web RPC handlers are thin transport adapters over `src-tauri/src/application/`. Keeping behavior in the shared application layer gives both runtimes the same connection, query, persistence, plugin, and security semantics.

The main stack is React 19, TypeScript, Tailwind CSS v4, Rust, Tauri v2, Axum, and SQLx. Significant design choices are recorded in the [architecture decision log](./docs/architecture/decisions/README.md).

## Develop locally

### Prerequisites

- Node.js 24 (the exact version is in [`.node-version`](./.node-version))
- pnpm 10 through Corepack (the exact version is pinned in [`package.json`](./package.json))
- Stable Rust toolchain
- Tauri's [platform-specific system dependencies](https://v2.tauri.app/start/prerequisites/)

### Set up the repository

```bash
git clone https://github.com/SuperDBA/SuperDBA.git
cd SuperDBA
corepack enable
pnpm install
```

Start the frontend development server:

```bash
pnpm dev
```

Run the complete desktop application:

```bash
pnpm tauri dev
```

Build the frontend and start Web mode:

```bash
pnpm build
pnpm dev:web
```

### Validate a change

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm test:rust
```

Changes that affect themes or the Web transport may also need:

```bash
pnpm lint:theme
pnpm test:contrast
pnpm web:boundaries
pnpm test:web-contract
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for platform setup, testing conventions, pull-request expectations, and repository workflows. UI and theme rules live in [docs/DESIGN.md](./docs/DESIGN.md).

## Build a plugin

Driver plugins are standalone processes that communicate with SuperDBA over JSON-RPC on standard input and output. They can be written in any language that can implement the protocol.

Create a starter project:

```bash
pnpm create @superdba/plugin@latest my-driver --db-type=network
```

Continue with the [Plugin Guide](./plugins/PLUGIN_GUIDE.md) and the [`@superdba/plugin-api` compatibility guide](./packages/plugin-api/README.md).

## Contribute

Bug reports, focused feature proposals, documentation improvements, translations, plugins, and code contributions are welcome.

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md) and the repository instructions in [AGENTS.md](./AGENTS.md).
2. Search [existing issues](https://github.com/SuperDBA/SuperDBA/issues) before opening a new one.
3. Keep changes focused and include tests appropriate to the affected behavior.
4. Join [Discord](https://discord.com/invite/K2hmhfHRSt) to discuss an implementation direction with the community.

See the [changelog](./CHANGELOG.md) for shipped releases and [GitHub Issues](https://github.com/SuperDBA/SuperDBA/issues) for planned work.

<!-- SPONSORS:START -->

## Sponsors and supporters

SuperDBA is made possible by its sponsors and community contributors. See the current list and sponsorship options at [superdba.dev/sponsors](https://superdba.dev/sponsors).

<!-- SPONSORS:END -->

## License

SuperDBA is licensed under the [Apache License 2.0](./LICENSE).

---

<p align="center">
  If SuperDBA helps you, consider <a href="https://github.com/SuperDBA/SuperDBA">starring the repository</a> or sharing feedback with the community.
</p>
