<h1 align="center">SuperDBA</h1>

<p align="center"><strong>An open-source database workspace for people and AI agents.</strong></p>

<p align="center">
  Explore schemas, write and run SQL, edit data, build visual queries, inspect execution plans, and work with SQL notebooks—from the desktop app or a browser.
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

## Why SuperDBA?

SuperDBA keeps the work around a database in one place. The schema explorer, SQL editor, notebooks, visual tools, data grid, query history, and AI integrations share the same connections and permissions.

- **One workspace, two runtimes.** Use the native Tauri desktop app or run the same React interface with `superdba web`.
- **SQL work without context switching.** Browse objects, edit SQL in Monaco, inspect multiple result sets, modify rows, and export data.
- **Visual tools when text is not enough.** Build queries with a graph, explore ER diagrams, and read interactive `EXPLAIN` plans.
- **Notebooks for repeatable analysis.** Combine SQL, Markdown, parameters, charts, and cross-cell results.
- **Extensible database support.** Built-in SQL drivers are complemented by installable drivers and UI extensions from the plugin registry.
- **AI on your terms.** Use optional text-to-SQL features, local models through Ollama, or the built-in MCP server from compatible AI clients.

## Highlights

### Database workspace

- Saved connections, secure credential storage, SSH tunnels, and per-connection appearance
- Searchable database explorer for tables, columns, keys, indexes, views, and routines
- Multi-database and multi-schema workflows where supported by the driver
- ER diagrams, schema actions, SQL import, and SQL export

### SQL editor and results

- Monaco-powered editor with dialect-aware highlighting, completion, formatting, and query splitting
- Tabbed sessions, SQL file editing, saved queries, query history, and customizable shortcuts
- Selected-statement or multi-statement execution with independent result sets
- Virtualized data grid with filtering, sorting, pagination, inline edits, batch changes, and export
- Visual query builder and interactive execution-plan viewer

### Notebooks, plugins, and AI

- SQL and Markdown cells with parameters, charts, cross-cell variables, and HTML/CSV/JSON export
- JSON-RPC driver plugins that can be implemented in any language
- Optional plugin-provided UI extensions
- MCP server for schema discovery and controlled query execution
- Optional AI assistance with cloud providers or local Ollama models

## Database support

SuperDBA includes native support for MySQL/MariaDB, PostgreSQL, and SQLite. The plugin registry adds databases and services such as DuckDB, ClickHouse, SQL Server, Oracle, Redis, MongoDB, BigQuery, DynamoDB, Firestore, Trino, and others.

Availability and maturity change independently of the application release. See the live [driver and plugin registry](https://superdba.dev/plugins) for the current list and installation status.

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

Linux users can also install the Flatpak or AUR package. AppImage, `.deb`, `.rpm`, Windows, and macOS installers are available from the [download page](https://superdba.dev/download) and [GitHub Releases](https://github.com/SuperDBA/SuperDBA/releases).

> [!IMPORTANT]
> Snap users must connect the password manager interface before saving credentials:
>
> ```bash
> sudo snap connect superdba:password-manager-service
> ```

## Desktop and Web modes

The desktop and browser interfaces use the same frontend and the same Rust application services. Only the transport changes.

| Mode | Start command | Best for |
| --- | --- | --- |
| Desktop | `superdba` | Native dialogs, local files, OS integration, and everyday desktop use |
| Web | `superdba web` | Browser access, containers, remote hosts, and managed deployments |
| MCP | `superdba --mcp` | Connecting compatible AI clients to SuperDBA capabilities |

`superdba web` binds to loopback by default and uses an authenticated browser session. Read the [Web mode operations guide](./docs/web/WEB_MODE_OPERATIONS.md) before exposing it beyond the local machine, and follow the [remote security guide](./docs/web/WEB_REMOTE_SECURITY.md) for reverse-proxy deployments.

Container and Helm instructions are in [Web mode containers](./docs/web/WEB_MODE_CONTAINERS.md).

## Architecture

```text
packages/web-ui/                  React 19 + TypeScript frontend
        │
        ├── Tauri IPC ──────────> src-tauri/src/transport/desktop/
        └── HTTP / WebSocket ───> src-tauri/src/transport/web/
                                      │
                                      ▼
                             src-tauri/src/application/
                                      │
                         drivers, plugins, storage, security
```

Desktop commands and Web RPC handlers are thin transport adapters. Shared behavior belongs in `src-tauri/src/application/`, which keeps both runtimes aligned.

The main technologies are React 19, TypeScript, Tailwind CSS v4, Rust, Tauri v2, Axum, and SQLx. Architecture decisions are recorded in [docs/architecture/decisions](./docs/architecture/decisions/README.md).

## Development

### Prerequisites

- Node.js 24 (the repository version is in [`.node-version`](./.node-version))
- pnpm 9 or newer
- Stable Rust toolchain
- Tauri's [platform-specific system dependencies](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
git clone https://github.com/SuperDBA/SuperDBA.git
cd SuperDBA
corepack enable
pnpm install
```

Run the frontend with fast refresh:

```bash
pnpm dev
```

Run the full desktop application:

```bash
pnpm tauri dev
```

Build and run the browser transport:

```bash
pnpm build
pnpm dev:web
```

### Quality checks

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm test:rust
```

Useful focused checks:

```bash
pnpm lint:theme
pnpm test:contrast
pnpm web:boundaries
pnpm test:web-contract
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for platform setup, test conventions, pull-request expectations, and plugin development. Repository-specific design rules live in [docs/DESIGN.md](./docs/DESIGN.md).

## Plugin development

Driver plugins communicate with SuperDBA over JSON-RPC on standard input/output, so they are not tied to a particular programming language.

```bash
pnpm create @superdba/plugin@latest my-driver --db-type=network
```

Start with the [Plugin Guide](./plugins/PLUGIN_GUIDE.md) and the [`@superdba/plugin-api` compatibility guide](./packages/plugin-api/README.md).

## Contributing

Bug reports, focused feature proposals, documentation improvements, translations, drivers, and code contributions are welcome.

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md) and the repository rules in [AGENTS.md](./AGENTS.md).
2. Search [existing issues](https://github.com/SuperDBA/SuperDBA/issues) before opening a new one.
3. Keep changes focused and include tests appropriate to the affected behavior.
4. Join [Discord](https://discord.com/invite/K2hmhfHRSt) when you want to discuss an implementation direction.

The [changelog](./CHANGELOG.md) records shipped releases. Planned work is tracked in [GitHub Issues](https://github.com/SuperDBA/SuperDBA/issues).

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
