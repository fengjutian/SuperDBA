<br />
<p align="center">
  <img src=".github/assets/banner-light.png#gh-light-mode-only" alt="SuperDBA" width="100%">
  <img src=".github/assets/banner-dark.png#gh-dark-mode-only" alt="SuperDBA" width="100%">
</p>

# SuperDBA

SuperDBA is an open-source desktop SQL workspace with 3 built-in database drivers and 26 shipped driver plugins, including DuckDB, ClickHouse, Redis and Firestore. Its built-in MCP server lets Claude, Cursor and Devin (formerly Windsurf) read your schema and run queries in the same app you already use.

<p align="center">
  <b><a href="https://superdba.dev">Website</a></b> ·
  <b><a href="https://superdba.dev/wiki">Docs</a></b> ·
  <b><a href="https://superdba.dev/download">Download</a></b> ·
  <b><a href="./CHANGELOG.md">Changelog</a></b>
</p>

<br />

<p align="center">
  <img src="https://img.shields.io/github/release/SuperDBA/SuperDBA.svg?style=flat" alt="Release" />
  <img src="https://img.shields.io/github/stars/SuperDBA/SuperDBA?style=flat" alt="Stars" />
  <img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2FSuperDBA%2FSuperDBA%2Fdownload-stats%2Fdownloads.json&amp;style=flat" alt="Downloads" />
  <img src="https://github.com/SuperDBA/SuperDBA/workflows/Release/badge.svg" alt="Build & Release" />
  <a href="https://discord.com/invite/K2hmhfHRSt"><img src="https://img.shields.io/discord/1502944695808950282?color=5865F2&logo=discord&logoColor=white" alt="Discord" /></a>
  <a href="https://gitster.dev/repo/SuperDBA/SuperDBA"><img src="https://gitster.dev/api/repositories/badge/cmlko1jr60005ne4yh7i7oy3e" alt="Gitster" /></a>
  <br />
  <a href="https://snapcraft.io/superdba"><img src="https://img.shields.io/badge/snap-superdba-blue?logo=snapcraft" alt="Snap Store" /></a>
  <a href="https://flatpark.org/apps/dev.superdba.SuperDBA/"><img src="https://img.shields.io/badge/flatpak-superdba-4A90D9?logo=flatpak&logoColor=white" alt="Flatpak (Flatpark)" /></a>
  <a href="https://aur.archlinux.org/packages/superdba-bin"><img src="https://img.shields.io/badge/AUR-superdba--bin-1793D1?logo=archlinux&logoColor=white" alt="AUR" /></a>
  <a href="https://winstall.app/apps/Debba.SuperDBA"><img src="https://img.shields.io/winget/v/Debba.SuperDBA?label=WinGet&logo=windows&color=0078D4" alt="WinGet" /></a>
</p>

<p align="center">
  <a href="https://vercel.com/open-source-program"><img alt="Vercel OSS Program" src="https://vercel.com/oss/program-badge-2026.svg" /></a>
</p>

<p align="center">
  <sub>
    <a href="./README.md">English</a> ·
    <a href="./docs/translations/README.it.md">Italiano</a> ·
    <a href="./docs/translations/README.es.md">Español</a> ·
    <a href="./docs/translations/README.zh-CN.md">中文</a> ·
    <a href="./docs/translations/README.fr.md">Français</a> ·
    <a href="./docs/translations/README.de.md">Deutsch</a> ·
    <a href="./docs/translations/README.ja.md">日本語</a> ·
    <a href="./docs/translations/README.ru.md">Русский</a> ·
    <a href="./docs/translations/README.tl.md">Tagalog</a> ·
    <a href="./docs/translations/README.ko.md">한국어</a> ·
    <a href="./docs/translations/README.pt-BR.md">Português (Brasil)</a>
  </sub>
</p>

<br />

<div align="center">
  <img src="https://raw.githubusercontent.com/SuperDBA/website/main/public/img/overview.gif" alt="SuperDBA, a desktop SQL workspace, showing its query editor and data grid" />
</div>

## Download

```bash
winget install Debba.SuperDBA    # Windows
brew install --cask SuperDBA     # macOS
sudo snap install SuperDBA       # Linux
```

Or grab an installer directly:

- **Windows:** [![Windows](https://img.shields.io/badge/Windows-Download-blue?logo=windows)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba_0.30.0_x64-setup.exe)

- **macOS:** [![macOS (Apple Silicon)](https://img.shields.io/badge/macOS-Apple%20Silicon-black?logo=apple)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba_0.30.0_aarch64.dmg) [![macOS (Intel)](https://img.shields.io/badge/macOS-Intel-black?logo=apple)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba_0.30.0_x64.dmg)

- **Linux:** [![Linux AppImage](https://img.shields.io/badge/Linux-AppImage-green?logo=linux)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba_0.30.0_amd64.AppImage) [![Linux .deb](https://img.shields.io/badge/Linux-.deb-orange?logo=debian)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba_0.30.0_amd64.deb) [![Linux .rpm](https://img.shields.io/badge/Linux-.rpm-red?logo=redhat)](https://github.com/SuperDBA/SuperDBA/releases/download/v0.30.0/superdba-0.30.0-1.x86_64.rpm)

The app UI is available in English, Italian, Spanish, Chinese (Simplified), French, German, Japanese, Russian, Korean, Tagalog and Portuguese (Brazilian).

> [!TIP]
> **Discord:** [Join our Discord server](https://discord.com/invite/K2hmhfHRSt) to talk with the maintainers, share feedback, and get help from the community.

## Table of Contents

- [Why SuperDBA?](#why-superdba)
  - [Database support](#database-support)
- [Installation](#installation)
  - [Windows](#windows)
  - [macOS](#macos)
  - [Linux (Snap)](#linux-snap)
  - [Linux (Flatpak)](#linux-flatpak)
  - [Linux (AppImage)](#linux-appimage)
  - [Arch Linux (AUR)](#arch-linux-aur)
- [Updates](#updates)
- [Discord](#discord)
- [Changelog](#changelog)
- [Features](#features)
  - [Connection Management](#connection-management)
  - [Database Explorer](#database-explorer)
  - [SQL Editor](#sql-editor)
  - [SQL Notebooks](#sql-notebooks)
  - [Keyboard Shortcuts](#keyboard-shortcuts)
  - [Visual Query Builder](#visual-query-builder)
  - [Visual EXPLAIN](#visual-explain)
  - [Data Grid](#data-grid)
  - [Plugin System](#plugin-system)
  - [Logging](#logging)
  - [Configuration Storage](#configuration-storage)
  - [AI Features (Optional)](#ai-features-optional)
  - [MCP Server: AI Agent Integration](#mcp-server-ai-agent-integration)
- [Tech Stack](#tech-stack)
- [Development](#development)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [Sponsors and supporters](#sponsors-and-supporters)
- [Origin Story](#origin-story)
- [License](#license)

## Why SuperDBA?

|                                                                    |          **SuperDBA**           |           DBeaver CE           |     TablePlus      |   Beekeeper Studio    |
| ------------------------------------------------------------------ | :------------------------------: | :----------------------------: | :----------------: | :-------------------: |
| License                                                            |         Apache 2.0, free         | Apache 2.0, free (Pro is paid) |     Commercial     | GPLv3 (paid editions) |
| SQL notebooks (SQL + Markdown cells, cross-cell variables, charts) |                ✅                |               ❌               |         ❌         |          ❌           |
| Built-in MCP server for AI agents                                  |                ✅                |               ❌               |         ❌         |          ❌           |
| Plugins in **any language** (JSON-RPC over stdio)                  |                ✅                |      Java/Eclipse plugins      | JavaScript plugins |          ❌           |
| AI text-to-SQL with **local models** (Ollama)                      |                ✅                |    Cloud-based AI assistant    |         ❌         |          ❌           |
| Visual EXPLAIN with interactive plan graphs                        |                ✅                |               ✅               |         ❌         |          ❌           |
| Databases out of the box                                           | 3 built-in + 26 driver plugins |              100+              |        20+         |          ~10          |

> [!NOTE]
> Comparison as of June 2026; features in other tools may have changed since. If you need dozens of drivers, use DBeaver. SuperDBA focuses on doing a few databases well.

### Database support

PostgreSQL, MySQL/MariaDB and SQLite ship built in. Everything else is a plugin. The built-in PostgreSQL driver is deprecated in favour of the [PostgreSQL plugin](https://github.com/SuperDBA/superdba-postgresql-plugin), which SuperDBA installs automatically. Current coverage, mirroring the [driver & plugin coverage](https://superdba.dev/#driver-coverage) on the website:

**Shipped**

| Database                 | Plugin                                                                                          | Database       | Plugin                                                                                          |
| ------------------------ | ----------------------------------------------------------------------------------------------- | -------------- | ----------------------------------------------------------------------------------------------- |
| Google BigQuery          | [superdba-bigquery-plugin](https://github.com/SuperDBA/superdba-bigquery-plugin)           | CockroachDB    | [superdba-cockroachdb-plugin](https://github.com/SuperDBA/superdba-cockroachdb-plugin)     |
| ClickHouse               | [superdba-clickhouse-plugin](https://github.com/SuperDBA/superdba-clickhouse-plugin)       | LibSQL / Turso | [superdba-libsql-plugin](https://github.com/SuperDBA/superdba-libsql-plugin)               |
| Cloudflare D1            | [superdba_cloudflare_d1_plugin](https://github.com/josejorge/superdba_cloudflare_d1_plugin)   | MongoDB        | [superdba-mongodb-plugin](https://github.com/danielnuld/superdba-mongodb-plugin)              |
| Cloudflare D1 (HTTP API) | [cloudflare-superdba](https://github.com/GabrielMalava/cloudflare-superdba)                   | MongoDB Atlas  | [superdba-mongodb-plugin](https://github.com/SuperDBA/superdba-mongodb-plugin)             |
| DM / Dameng              | [superdba-dameng-plugin](https://github.com/haos666/superdba-dameng-plugin)                   | Oracle         | [superdba-oracle-plugin](https://github.com/SuperDBA/superdba-oracle-plugin)               |
| DuckDB                   | [superdba-duckdb-plugin](https://github.com/SuperDBA/superdba-duckdb-plugin)               | Redis (Go)     | [superdba-redis-plugin-go](https://github.com/gzamboni/superdba-redis-plugin-go)              |
| DynamoDB                 | [superdba-dynamodb-plugin](https://github.com/SuperDBA/superdba-dynamodb-plugin)           | Redis (Rust)   | [superdba-redis-plugin](https://github.com/nicholas-papachriston/superdba-redis-plugin)       |
| Elasticsearch            | [superdba-elasticsearch-plugin](https://github.com/SuperDBA/superdba-elasticsearch-plugin) | SQL Server     | [superdba-sqlserver-plugin](https://github.com/SuperDBA/superdba-sqlserver-plugin)         |
| Firestore                | [firestore-superdba](https://codeberg.org/NewtTheWolf/firestore-superdba)                     | CSV Folder     | [superdba-csv-plugin](https://github.com/SuperDBA/superdba-csv-plugin)                     |
| IBM Db2                  | [superdba-db2-plugin](https://github.com/SuperDBA/superdba-db2-plugin)                     | Google Sheets  | [superdba-google-sheets-plugin](https://github.com/SuperDBA/superdba-google-sheets-plugin) |
| IBM Informix             | [superdba-informix-plugin](https://github.com/danielnuld/superdba-informix-plugin)            | HackerNews     | [superdba-hackernews-plugin](https://github.com/SuperDBA/superdba-hackernews-plugin)       |
| Cassandra / ScyllaDB     | [superdba-cassandra-plugin](https://github.com/SuperDBA/superdba-cassandra-plugin)         | Harper         | [superdba-harper-plugin](https://github.com/HarperFast/superdba-harper-plugin)                |
| PostgreSQL               | [superdba-postgresql-plugin](https://github.com/SuperDBA/superdba-postgresql-plugin)       | Trino          | [superdba-trino-plugin](https://github.com/SuperDBA/superdba-trino-plugin)                 |

**On the bounty board**

| Status      | Databases                                                                    |
| ----------- | ---------------------------------------------------------------------------- |
| In progress | Amazon Redshift, Etcd, Meilisearch, Snowflake, SQL Anywhere                                |
| Scoped      | TiDB                                                            |
| Open        | Firebird, SurrealDB                                      |

> [!NOTE]
> **Shipped** drivers are installable from the [plugin registry](https://superdba.dev/plugins). Everything else is on the [bounty board](https://superdba.dev/plugins/bounties): claim one, sponsor one, or [request a database](https://github.com/SuperDBA/SuperDBA/discussions).

## Installation

### Windows

**WinGet (Recommended)**

```bash
winget install Debba.SuperDBA
```

**Direct Download**

Download the installer from the [Releases page](https://github.com/SuperDBA/SuperDBA/releases) and run it:

```
superdba_x.x.x_x64-setup.exe
```

Follow the on-screen instructions to complete the installation.

### macOS

**Homebrew (Recommended)**

```bash
brew install --cask SuperDBA
```

[![Homebrew](https://img.shields.io/badge/Homebrew-Repository-orange?logo=homebrew)](https://github.com/SuperDBA/homebrew-superdba)

**Direct Download**

Builds from **v0.13.1** onward are signed and notarized by Apple, so they open without any extra steps.

<details>
<summary>Notes for releases before v0.13.1</summary>

<br />

The notes below only apply to **older releases (before v0.13.1)** downloaded directly:

- You need to allow accessibility access (Privacy & Security) to the SuperDBA app. If you are upgrading and already have SuperDBA on the allowed list, remove it manually before accessibility access can be granted to the new version.
- You may need to run `xattr -c /Applications/superdba.app` after copying the app to the Applications directory.

</details>

### Linux (Snap)

```bash
sudo snap install SuperDBA
sudo snap connect superdba:password-manager-service   # allow keychain access for saved credentials
```

> [!IMPORTANT]
> The `password-manager-service` interface is not auto-connected by the Snap Store yet. Without it, saving a connection fails with a `Platform secure storage failure` error.

[![Snap Store](https://img.shields.io/badge/snap-superdba-blue?logo=snapcraft)](https://snapcraft.io/superdba)

### Linux (Flatpak)

```bash
flatpak remote-add --if-not-exists flatpark https://dl.flatpark.org/flatpark.flatpakrepo
flatpak install flatpark dev.superdba.SuperDBA
```

[![Flatpak (Flatpark)](https://img.shields.io/badge/flatpak-superdba-4A90D9?logo=flatpak&logoColor=white)](https://flatpark.org/apps/dev.superdba.SuperDBA/)

### Linux (AppImage)

Download the `.AppImage` file from the [Releases page](https://github.com/SuperDBA/SuperDBA/releases), make it executable and run it:

```bash
chmod +x superdba_x.x.x_amd64.AppImage
./superdba_x.x.x_amd64.AppImage
```

### Arch Linux (AUR)

```bash
yay -S superdba-bin
```

## Updates

SuperDBA checks for updates automatically on startup and notifies you when a new version is available. You can also download the latest version directly from the [Releases page](https://github.com/SuperDBA/SuperDBA/releases).

## Discord

Join our [Discord server](https://discord.com/invite/K2hmhfHRSt) to talk with the maintainers, share feedback, suggest features, or get help from the community.

## Changelog

Every release is documented in [CHANGELOG.md](./CHANGELOG.md).

## Features

### Connection Management

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/connections)</sub>

- Support for **MySQL/MariaDB**, **PostgreSQL** (with multi-schema support) and **SQLite**, with multi-database selection per connection.
- Save, manage, and clone connection profiles, with optional secure password storage in the system **Keychain**.
- **SSH Tunneling** with automatic readiness detection.
- **Per-Connection Appearance:** override the icon ([Lucide](https://lucide.dev/icons/), emoji, or custom image) and accent color of each saved connection.

### Database Explorer

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/schema-management)</sub>

- **Tree View:** Browse tables, columns, keys, indexes, views, and stored routines, with inline editing from the sidebar.
- **ER Diagram:** Interactive Entity-Relationship visualization (pan, zoom, layout) with selective table diagram generation.
- **Context Actions:** Show data, count rows, modify schema, duplicate/delete tables.
- **SQL Dump & Import:** Export and restore databases with a single flow.

### SQL Editor

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/editor)</sub>

- **Monaco Editor** with syntax highlighting and auto-completion, in a tabbed interface with isolated connections per tab and resizable **split view**.
- **Multi-Statement Execution:** Run All, Run Selected, or pick individual queries. Results appear in separate tabs with independent pagination.
- **Smart Query Splitting:** Correctly handles stored procedures, functions, and `$$`-delimited blocks.
- **SQL Files:** Open, edit, and save `.sql`, `.psql`, and `.pgsql` files in editor tabs without executing them.
- **Saved Queries** and an **AI assist overlay** directly in the editor.

### SQL Notebooks

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/notebooks)</sub>

- **Multi-Cell Workspace:** Combine SQL and Markdown cells in a single document, with inline results and bar/line/pie charts.
- **Cross-Cell Variables:** Reference another cell's full result as a table with `{{cell_N}}` (expanded to a CTE at run time), plus global `@paramName` parameters.
- **Run All:** Sequential execution with stop-on-error option and completion summary.
- **Persistence & Export:** Auto-saved as `.superdba-notebook` files; export as HTML, CSV, or JSON.
- Outline panel, drag & drop cell reordering, and AI-generated cell names.

### Keyboard Shortcuts

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/keyboard-shortcuts)</sub>

- **Built-in shortcuts** for navigation, editor, and data grid actions, platform-aware (`Cmd` on macOS, `Ctrl` on Windows/Linux).
- **Fully customizable:** Remap any non-locked shortcut from **Settings → Keyboard Shortcuts**; overrides persist to `keybindings.json`.
- Hold `Ctrl+Shift` in the sidebar to reveal numbered badges (1–9) for instant connection switching.

### Visual Query Builder

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/visual-query-builder)</sub>

- **Drag-and-Drop:** Build queries visually with ReactFlow.
- **Visual JOINs:** Connect tables to create relationships.
- **Advanced Logic:** WHERE/HAVING filters, aggregates (COUNT, SUM, AVG), sorting, and limits.
- **Real-time SQL:** Instant code generation.

### Visual EXPLAIN

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/visual-explain)</sub>

- **Interactive Plan Graphs:** Inspect execution plans as navigable node graphs instead of raw text.
- **Table, Raw, and AI Views:** Switch between exact node metrics, original database output, and optional AI-assisted analysis.
- **Cross-Database Support:** Works with PostgreSQL, MySQL/MariaDB, and SQLite using the best available `EXPLAIN` format per driver.
- **Faster Optimization Loops:** Spot expensive scans, estimate gaps, join behavior, and optimizer choices without leaving the editor.

### Data Grid

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/data-grid)</sub>

- **Inline & Batch Editing:** Modify cells and commit multiple changes at once; create, delete, and multi-select rows.
- **Export:** Save results as CSV or JSON, or copy selected rows straight to the clipboard.
- **JSON & JSONB Cells:** Syntax-highlighted in the grid, with a dedicated editor window (Tree / Monaco / Raw modes).
- **Spatial Data:** Initial GEOMETRY support for MySQL.

### Plugin System

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/plugins)</sub>

SuperDBA is **hackable with an external plugin system**. Plugins are standalone executables that communicate with the app over **JSON-RPC 2.0 via stdin/stdout**, and can be written in any language.

- **Install Plugins:** Browse and install community drivers from **Settings → Available Plugins**, no restart required.
- **Manage Drivers:** View all registered drivers (built-in and plugins) in **Settings → Installed Drivers** and uninstall plugins with one click.
- **Any Database:** Add support for DuckDB, MongoDB, or any other database by writing or installing a plugin.
- **Plugin Registry:** Official plugins are listed in [`plugins/registry.json`](./plugins/registry.json).
- **Developer Guide:** See [`plugins/PLUGIN_GUIDE.md`](./plugins/PLUGIN_GUIDE.md) to build your own driver in any language.
- **Declarative themes (development):** **Settings → Appearance** supports local packages (**Import theme → Local package**), previews and personal/VS Code imports without executable plugin activation. See the [theme author guide](./packages/create-plugin/THEMES.md) for the separate `superdba-theme` CLI, packaging and release gates; public runtime/tooling rollout is not implied by this development feature.

### Logging

- Real-time log viewer in Settings, with level filtering and export to `.log` files.
- Automatically expand and inspect SQL queries in logs.
- **CLI Debug Mode:** Start with `superdba --debug` for verbose logging from launch.

### Configuration Storage

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/configuration)</sub>

Configuration is stored in `~/.config/superdba/` (Linux), `~/Library/Application Support/superdba/` (macOS), or `%APPDATA%\superdba\` (Windows): connection profiles, saved queries, app settings (`config.json`), custom themes, and per-connection editor preferences. You can move this folder from **Settings > Storage** (or with the `SUPERDBA_DATA_DIR` environment variable), for example to an iCloud Drive or Dropbox folder to sync connections across machines; installed plugins always stay local. Tabs and queries are restored when you reopen a connection. The wiki covers the full file layout and every `config.json` option, including custom AI model overrides.

<details>
<summary>How Follow System works on Linux</summary>

<br />

On Linux, **Follow System** reads the XDG desktop settings portal's `org.freedesktop.appearance/color-scheme` preference and follows its live updates, including GNOME's dark-mode toggle with the standard Adwaita GTK theme. The resolved light/dark theme is applied explicitly to GTK window decorations and the webview. A portal value of `0` (no preference) resolves to light, so switching back to the desktop default cannot reuse the app's previously forced dark theme. If the portal is unavailable or returns an unsupported value, SuperDBA falls back to the native window theme, then the browser media query. The default window capability grants `core:window:allow-set-theme` to allow native theme changes. macOS and Windows use native theme notifications. See [the portal specification](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html).

Portal command reads reuse one cached D-Bus connection, while the live watcher keeps its own connection. Failed reads discard the cached connection so the next request can reconnect; failed connection attempts are not cached. The existing two-second command timeout also bounds concurrent requests waiting for the cache.

</details>

### AI Features (Optional)

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/ai-assistant)</sub>

Optional Text-to-SQL and query explanation powered by **OpenAI**, **Anthropic**, **MiniMax**, **DeepSeek**, **OpenRouter**, **Requesty**, **Ollama** (local models, no API key, full privacy), and any **OpenAI-compatible API** (Groq, Perplexity, Azure OpenAI, LocalAI, ...). Model lists are fetched from your provider and cached locally; custom models can be configured per provider.

### MCP Server: AI Agent Integration

<sub>[Full reference on superdba.dev →](https://superdba.dev/wiki/mcp-server)</sub>

SuperDBA includes a built-in **MCP (Model Context Protocol) server** that lets AI agents read your database schema and execute queries directly from their chat interface.

```bash
superdba --mcp
```

**One-click setup** for Claude Desktop, Cursor, and Windsurf: open **Settings → MCP Server Integration**, click **Install Config** next to your client, and restart it. Manual configuration is covered in the wiki.

#### Available tools

Once connected, your AI agent can:

| Tool               | Description                                               |
| ------------------ | --------------------------------------------------------- |
| `list_connections` | List all saved database connections                       |
| `list_databases`   | List all databases available for a connection             |
| `list_tables`      | List tables in a connection (with optional schema filter) |
| `describe_table`   | Get full schema: columns, indexes, foreign keys           |
| `run_query`        | Execute any SQL query and return results                  |

Every tool accepts an optional `output_format` argument. JSON is the default, preserving the existing response format for all clients. You can choose JSON or [TOON](https://toonformat.dev/) as the default under **Settings → MCP Server Integration**; a tool call's `output_format` argument overrides that preference. TOON is especially compact for tabular query results passed to an LLM. MCP transport remains JSON-RPC in either mode; only the text inside the tool result changes.

#### Example prompts

> "Show me all tables in my production database and describe the `orders` table"

> "Write and run a query to find the top 10 customers by total order value this month"

> "Check if there are any missing indexes on the `users` table"

## Tech Stack

| Layer    | Stack                                 |
| -------- | ------------------------------------- |
| Frontend | React 19, TypeScript, Tailwind CSS v4 |
| Backend  | Rust, Tauri v2, SQLx                  |

## Development

**Setup**

```bash
pnpm install
pnpm tauri dev
```

**Build**

```bash
pnpm tauri build
```

## Roadmap

- [x] [Plugin registry platform — OAuth publishing, release sync, download analytics](https://github.com/SuperDBA/SuperDBA/issues/196)
- [x] [SQL Server driver — implementation roadmap & call for contributors](https://github.com/SuperDBA/SuperDBA/issues/150)
- [x] [[Feat]: Allow loading of multiple Databases per connection](https://github.com/SuperDBA/SuperDBA/issues/47)
- [x] [Command Palette](https://github.com/SuperDBA/SuperDBA/issues/25)
- [x] [JSON/JSONB Editor & Viewer](https://github.com/SuperDBA/SuperDBA/issues/24)
- [x] [SQL Formatting / Prettier](https://github.com/SuperDBA/SuperDBA/issues/23)
- [x] [Visual Explain Analyze](https://github.com/SuperDBA/SuperDBA/issues/22)
- [x] [Plugin System](https://github.com/SuperDBA/SuperDBA/issues/19)
- [x] [Query History](https://github.com/SuperDBA/SuperDBA/issues/18)
- [ ] [UI design system & visual identity — call for contributors](https://github.com/SuperDBA/SuperDBA/issues/195)
- [ ] [Feature: Remote Control](https://github.com/SuperDBA/SuperDBA/issues/46)
- [ ] [Data Compare / Diff Tool](https://github.com/SuperDBA/SuperDBA/issues/21)
- [ ] [Team Collaboration](https://github.com/SuperDBA/SuperDBA/issues/20)
- [ ] [Better SQLite Support](https://github.com/SuperDBA/SuperDBA/issues/17)
- [ ] [Better PostgreSQL Support](https://github.com/SuperDBA/SuperDBA/issues/16)

## Contributing

Contributions are welcome, see [CONTRIBUTING.md](./CONTRIBUTING.md). Good places to start:

- [UI design system & visual identity: call for contributors](https://github.com/SuperDBA/SuperDBA/issues/195)
- Write a driver plugin in any language with the [Plugin Guide](./plugins/PLUGIN_GUIDE.md)

<!-- SPONSORS:START -->

## Sponsors and supporters

- <a href="https://www.serversmtp.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/turbosmtp_compact.png" height="28" alt="turboSMTP" /></a> **[turboSMTP](https://www.serversmtp.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Professional SMTP relay — your emails delivered straight to the inbox, never to spam
- <a href="https://www.kilo.ai/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/kilocode_compact.png" height="28" alt="Kilo Code" /></a> **[Kilo Code](https://www.kilo.ai/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Open source AI coding agent — build, ship, and iterate faster with 500+ models
- <a href="https://openai.com/codex/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/openai_compact.png" height="28" alt="OpenAI" /></a> **[OpenAI](https://openai.com/codex/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Supporting SuperDBA through the Codex for Open Source program.
- <a href="https://m.do.co/c/f6ab3d158275?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/digitalocean_compact.png" height="28" alt="DigitalOcean" /></a> **[DigitalOcean](https://m.do.co/c/f6ab3d158275?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Simple, predictable cloud infrastructure for developers and growing teams.
- <a href="https://vercel.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/vercel_compact.svg" height="28" alt="Vercel" /></a> **[Vercel](https://vercel.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — The platform for the modern web — ship, preview, and scale frontend apps with zero config.
- <a href="https://usero.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/usero_compact.png" height="28" alt="Usero" /></a> **[Usero](https://usero.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Feedback becomes code. Automatically.
- <a href="https://devglobe.app/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/devglobe_compact.png" height="28" alt="DevGlobe" /></a> **[DevGlobe](https://devglobe.app/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Connect your IDE, show up on the globe, and showcase your projects to a community of builders.
- <a href="https://tolgee.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/tolgee_compact.svg" height="28" alt="Tolgee" /></a> **[Tolgee](https://tolgee.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Open-source localization platform — translate your app in context, without the spreadsheet chaos.
- <a href="https://1password.com/developers?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/1password_compact.png" height="28" alt="1Password" /></a> **[1Password](https://1password.com/developers?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — The password and secrets manager developers trust — free for open-source projects.
- <a href="https://www.jetbrains.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/jetbrains_compact.png" height="28" alt="JetBrains" /></a> **[JetBrains](https://www.jetbrains.com/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Professional developer tools — IntelliJ IDEA, WebStorm, DataGrip and the rest of the All Products Pack.
- <a href="https://signpath.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor" target="_blank"><img src="https://superdba.dev/img/logos/sponsors/signpath_compact.png" height="28" alt="SignPath" /></a> **[SignPath](https://signpath.io/?utm_source=SuperDBA&utm_medium=referral&utm_campaign=sponsor)** — Code signing for open source — signed Windows releases without the certificate bill.

_[Become a sponsor →](https://superdba.dev/sponsors)_

<!-- SPONSORS:END -->

## Origin Story

SuperDBA started as an experiment: how far could AI-assisted development get in building a working tool from scratch? Further than expected: it's now an actively maintained project with regular releases and a plugin ecosystem.

## License

[Apache License 2.0](./LICENSE)

---

<p align="center">
  Like SuperDBA? <a href="https://github.com/SuperDBA/SuperDBA">Star the repo</a> ⭐, it helps the project a lot.
</p>

<p align="center">
  <a href="https://repostars.dev/?repos=SuperDBA%2Fsuperdba&theme=dark">
    <img src="https://repostars.dev/api/embed?repo=SuperDBA%2Fsuperdba&theme=dark" alt="RepoStars" />
  </a>
</p>
x
