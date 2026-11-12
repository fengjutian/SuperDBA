<h1 align="center">SuperDBA</h1>

<p align="center"><strong>An open-source database workspace for people and AI agents.</strong></p>

<p align="center">
  Explore schemas, write and review SQL, edit data, inspect execution plans, build notebooks, and connect AI tools—on the desktop or in a browser.
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
  <a href="./README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/SuperDBA/website/main/public/img/overview.gif" alt="SuperDBA query editor and data grid" width="100%">
</p>

## One workspace for the complete database workflow

SuperDBA brings database exploration, querying, data editing, visualization, automation, and AI assistance into one consistent interface. Connections, permissions, schema context, query history, notebooks, plugins, and security policies stay together instead of being scattered across separate tools.

- **Understand a database before querying it.** Search tables, columns, keys, indexes, views, and routines, then inspect relationships in an ER diagram.
- **Write SQL with context.** Use a Monaco-powered, dialect-aware editor with completion, formatting, statement selection, saved queries, history, and multiple result sets.
- **Work directly with results.** Filter, sort, paginate, export, and edit rows in a virtualized grid with reviewable batch changes.
- **See what a query is doing.** Build SQL visually and turn supported `EXPLAIN` output into an interactive execution-plan view.
- **Make analysis repeatable.** Combine SQL, Markdown, parameters, charts, and cross-cell results in notebooks that can be exported to HTML, CSV, or JSON.
- **Extend the workspace.** Install database drivers and UI contributions as plugins without waiting for a core application release.
- **Bring your own AI.** Use optional text-to-SQL and chat features with supported providers or local Ollama models, and expose controlled database capabilities through the built-in MCP server.

## Run it your way

The desktop app and Web mode use the same React interface and Rust application services. Only the transport changes, so core behavior remains aligned.

| Mode | Command | Best for |
| --- | --- | --- |
| Desktop | `superdba` | Native dialogs, local files, OS credential storage, and everyday desktop use |
| Web | `superdba web` | Browser access, containers, remote hosts, and managed deployments |
| MCP | `superdba --mcp` | Giving compatible AI clients controlled access to SuperDBA tools |

Web mode listens on loopback by default and uses an authenticated browser session. Review the Web operations and remote-security documentation before exposing it to another machine or network.

## Database and plugin support

SuperDBA includes native workflows for MySQL/MariaDB, PostgreSQL, and SQLite. The plugin system adds SQL, NoSQL, vector, and service integrations such as DuckDB, ClickHouse, SQL Server, Oracle, Redis, MongoDB, BigQuery, DynamoDB, Firestore, and Trino.

Driver availability and maturity evolve independently from the core application. Check the [plugin registry](https://superdba.dev/plugins) for the current list and platform support.

## Install

Download installers and portable packages for Windows, macOS, and Linux from the [download page](https://superdba.dev/download) or [GitHub Releases](https://github.com/SuperDBA/SuperDBA/releases).

```bash
# Windows
winget install Debba.SuperDBA

# macOS
brew install --cask SuperDBA

# Linux (Snap)
sudo snap install SuperDBA
```

Linux packages are also available for Flatpak and AUR, together with AppImage, `.deb`, and `.rpm` builds.

> [!IMPORTANT]
> Snap users must connect the password-manager interface before saving credentials:
>
> ```bash
> sudo snap connect superdba:password-manager-service
> ```

## Architecture

```text
packages/web-ui/                    React + TypeScript interface
       │
       ├── Tauri IPC ────────────── src-tauri/src/transport/desktop/
       └── HTTP / WebSocket ─────── src-tauri/src/transport/web/
                                             │
                                             ▼
                              src-tauri/src/application/
                              shared application services
                                             │
                                             ▼
                               drivers · plugins · storage
                                    security · MCP · AI
```

Desktop commands and Web RPC handlers are thin transport adapters over `src-tauri/src/application/`. This boundary keeps connection, query, persistence, plugin, and security behavior shared across both runtimes. The main stack is React 19, TypeScript, Tailwind CSS v4, Rust, Tauri v2, Axum, and SQLx.

## Develop locally

### Prerequisites

- Node.js 24 (see [`.node-version`](./.node-version))
- pnpm 10 through Corepack (see [`package.json`](./package.json))
- A stable Rust toolchain
- [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/)

### Set up and run

```bash
git clone https://github.com/SuperDBA/SuperDBA.git
cd SuperDBA
corepack enable
pnpm install

# Frontend development server
pnpm dev

# Complete desktop application
pnpm tauri dev

# Build the frontend and start Web mode
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

Changes involving themes or Web mode may also require `pnpm lint:theme`, `pnpm test:contrast`, `pnpm web:boundaries`, and `pnpm test:web-contract`. See [CONTRIBUTING.md](./CONTRIBUTING.md) for platform setup, repository conventions, and pull-request guidance.

## Build a plugin

Driver plugins are standalone processes that communicate with SuperDBA over JSON-RPC through standard input and output. They can be written in any language that implements the protocol.

```bash
pnpm create @superdba/plugin@latest my-driver --db-type=network
```

Continue with the [Plugin Guide](./plugins/PLUGIN_GUIDE.md) and the [`@superdba/plugin-api` compatibility guide](./packages/plugin-api/README.md).

## Contribute

Bug reports, focused feature proposals, documentation improvements, plugins, and code contributions are welcome.

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md) and [AGENTS.md](./AGENTS.md).
2. Search [existing issues](https://github.com/SuperDBA/SuperDBA/issues) before opening a new one.
3. Keep changes focused and include tests appropriate to the affected behavior.
4. Join [Discord](https://discord.com/invite/K2hmhfHRSt) to discuss implementation ideas with the community.

See the [changelog](./CHANGELOG.md) for released work and [GitHub Issues](https://github.com/SuperDBA/SuperDBA/issues) for planned work.

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
