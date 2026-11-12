<h1 align="center">SuperDBA</h1>

<p align="center"><strong>面向开发者与 AI 智能体的开源数据库工作空间。</strong></p>

<p align="center">
  浏览数据库结构、编写和审查 SQL、编辑数据、分析执行计划、创建 Notebook，并连接 AI 工具——支持桌面端与浏览器。
</p>

<p align="center">
  <a href="https://superdba.dev">官网</a> ·
  <a href="https://superdba.dev/wiki">文档</a> ·
  <a href="https://superdba.dev/download">下载</a> ·
  <a href="https://superdba.dev/plugins">插件</a> ·
  <a href="https://discord.com/invite/K2hmhfHRSt">Discord</a>
</p>

<p align="center">
  <a href="https://github.com/SuperDBA/SuperDBA/releases"><img src="https://img.shields.io/github/v/release/SuperDBA/SuperDBA?label=release" alt="最新版本"></a>
  <a href="https://github.com/SuperDBA/SuperDBA/stargazers"><img src="https://img.shields.io/github/stars/SuperDBA/SuperDBA?style=flat" alt="GitHub Stars"></a>
  <a href="https://github.com/SuperDBA/SuperDBA/actions"><img src="https://github.com/SuperDBA/SuperDBA/workflows/Release/badge.svg" alt="构建状态"></a>
  <a href="https://discord.com/invite/K2hmhfHRSt"><img src="https://img.shields.io/discord/1502944695808950282?color=5865F2&logo=discord&logoColor=white" alt="Discord"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="Apache 2.0 许可证"></a>
</p>

<p align="center">
  <a href="./README.md">English</a> ·
  <a href="./README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/SuperDBA/website/main/public/img/overview.gif" alt="SuperDBA SQL 编辑器和数据表格" width="100%">
</p>

## 一个工作空间，覆盖完整数据库流程

SuperDBA 将数据库探索、查询、数据编辑、可视化、自动化和 AI 辅助整合到一致的界面中。连接配置、权限、结构上下文、查询历史、Notebook、插件和安全策略集中管理，不再散落于多个工具。

- **查询前先理解数据库。** 搜索表、列、主外键、索引、视图和存储过程，并通过 ER 图查看对象关系。
- **结合上下文编写 SQL。** 使用基于 Monaco 的方言感知编辑器，支持自动补全、格式化、语句选择、已保存查询、历史记录和多结果集。
- **直接处理查询结果。** 在虚拟化数据表格中筛选、排序、分页、导出和编辑数据，并在提交前审查批量修改。
- **理解 SQL 的执行过程。** 通过可视化方式构建查询，并将受支持的 `EXPLAIN` 输出转换成交互式执行计划。
- **让分析可重复。** 在 Notebook 中组合 SQL、Markdown、参数、图表和跨单元格结果，并导出为 HTML、CSV 或 JSON。
- **通过插件扩展能力。** 独立安装数据库驱动和 UI 扩展，无需等待核心应用发布新版本。
- **接入自己的 AI。** 使用受支持的云端提供商或本地 Ollama 模型完成 Text-to-SQL 和对话，也可通过内置 MCP 服务器向兼容的 AI 客户端开放受控能力。

## 选择适合你的运行方式

桌面应用和 Web 模式使用同一套 React 界面与 Rust 应用服务。两者只在传输层不同，核心行为保持一致。

| 模式 | 命令 | 适用场景 |
| --- | --- | --- |
| 桌面端 | `superdba` | 原生对话框、本地文件、系统凭据存储和日常桌面使用 |
| Web | `superdba web` | 浏览器访问、容器、远程主机和托管部署 |
| MCP | `superdba --mcp` | 向兼容的 AI 客户端提供受控的 SuperDBA 工具 |

Web 模式默认仅监听本机回环地址，并使用经过身份验证的浏览器会话。将其开放给其他设备或网络前，请先阅读 Web 运维与远程安全文档。

## 数据库与插件支持

SuperDBA 原生支持 MySQL/MariaDB、PostgreSQL 和 SQLite 工作流。插件系统可进一步接入 SQL、NoSQL、向量数据库及服务，包括 DuckDB、ClickHouse、SQL Server、Oracle、Redis、MongoDB、BigQuery、DynamoDB、Firestore 和 Trino 等。

驱动的可用性和成熟度独立于核心应用演进。请访问[插件市场](https://superdba.dev/plugins)查看最新列表和平台支持情况。

## 安装

你可以从[下载页面](https://superdba.dev/download)或 [GitHub Releases](https://github.com/SuperDBA/SuperDBA/releases) 获取 Windows、macOS 和 Linux 安装包及便携版本。

```bash
# Windows
winget install Debba.SuperDBA

# macOS
brew install --cask SuperDBA

# Linux（Snap）
sudo snap install SuperDBA
```

Linux 还提供 Flatpak、AUR、AppImage、`.deb` 和 `.rpm` 软件包。

> [!IMPORTANT]
> Snap 用户在保存凭据前，必须连接密码管理器接口：
>
> ```bash
> sudo snap connect superdba:password-manager-service
> ```

## 架构

```text
packages/web-ui/                    React + TypeScript 界面
       │
       ├── Tauri IPC ────────────── src-tauri/src/transport/desktop/
       └── HTTP / WebSocket ─────── src-tauri/src/transport/web/
                                             │
                                             ▼
                              src-tauri/src/application/
                                    共享应用服务
                                             │
                                             ▼
                              驱动 · 插件 · 存储 · 安全 · MCP · AI
```

桌面命令和 Web RPC 处理器都是 `src-tauri/src/application/` 之上的轻量传输适配器。该边界让桌面端和 Web 端共享连接、查询、持久化、插件与安全逻辑。主要技术栈包括 React 19、TypeScript、Tailwind CSS v4、Rust、Tauri v2、Axum 和 SQLx。

## 本地开发

### 环境要求

- Node.js 24（参见 [`.node-version`](./.node-version)）
- 通过 Corepack 使用 pnpm 10（参见 [`package.json`](./package.json)）
- Stable Rust 工具链
- [Tauri 平台依赖](https://v2.tauri.app/start/prerequisites/)

### 初始化并运行

```bash
git clone https://github.com/SuperDBA/SuperDBA.git
cd SuperDBA
corepack enable
pnpm install

# 前端开发服务器
pnpm dev

# 完整桌面应用
pnpm tauri dev

# 构建前端并启动 Web 模式
pnpm build
pnpm dev:web
```

### 验证修改

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm test:rust
```

涉及主题或 Web 模式的修改可能还需要运行 `pnpm lint:theme`、`pnpm test:contrast`、`pnpm web:boundaries` 和 `pnpm test:web-contract`。平台配置、仓库约定和 Pull Request 指南请参阅 [CONTRIBUTING.md](./CONTRIBUTING.md)。

## 开发插件

驱动插件是通过标准输入和输出与 SuperDBA 进行 JSON-RPC 通信的独立进程，可使用任何能够实现该协议的语言编写。

```bash
pnpm create @superdba/plugin@latest my-driver --db-type=network
```

后续开发请参阅[插件开发指南](./plugins/PLUGIN_GUIDE.md)和 [`@superdba/plugin-api` 兼容性指南](./packages/plugin-api/README.md)。

## 参与贡献

欢迎提交 Bug 报告、聚焦的功能建议、文档改进、插件和代码贡献。

1. 阅读 [CONTRIBUTING.md](./CONTRIBUTING.md) 和 [AGENTS.md](./AGENTS.md)。
2. 创建 Issue 前先搜索[已有问题](https://github.com/SuperDBA/SuperDBA/issues)。
3. 保持修改范围清晰，并为受影响的行为添加适当测试。
4. 加入 [Discord](https://discord.com/invite/K2hmhfHRSt)，与社区讨论实现思路。

已发布内容请查看[更新日志](./CHANGELOG.md)，规划中的工作请查看 [GitHub Issues](https://github.com/SuperDBA/SuperDBA/issues)。

<!-- SPONSORS:START -->

## 赞助者与支持者

SuperDBA 的持续发展离不开赞助者和社区贡献者。当前名单与赞助方式请访问 [superdba.dev/sponsors](https://superdba.dev/sponsors)。

<!-- SPONSORS:END -->

## 许可证

SuperDBA 基于 [Apache License 2.0](./LICENSE) 发布。

---

<p align="center">
  如果 SuperDBA 对你有帮助，欢迎为<a href="https://github.com/SuperDBA/SuperDBA">项目点亮 Star</a>，或向社区分享你的使用反馈。
</p>
