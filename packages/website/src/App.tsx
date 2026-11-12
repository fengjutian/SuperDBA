const GithubIcon = () => (
    <svg aria-hidden="true" viewBox="0 0 24 24">
        <path d="M12 2a10 10 0 0 0-3.16 19.49c.5.09.68-.22.68-.48v-1.87c-2.78.6-3.37-1.18-3.37-1.18-.45-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.61.07-.61 1 .07 1.53 1.03 1.53 1.03.9 1.53 2.34 1.09 2.91.83.09-.65.35-1.09.64-1.34-2.22-.25-4.56-1.11-4.56-4.94 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.64 0 0 .84-.27 2.75 1.02A9.57 9.57 0 0 1 12 6.82c.85 0 1.7.11 2.5.34 1.9-1.29 2.74-1.02 2.74-1.02.55 1.37.2 2.39.1 2.64.64.7 1.03 1.59 1.03 2.68 0 3.84-2.34 4.68-4.57 4.93.36.31.68.92.68 1.86v2.76c0 .27.18.58.69.48A10 10 0 0 0 12 2Z" />
    </svg>
);

const ArrowIcon = () => (
    <svg aria-hidden="true" viewBox="0 0 20 20">
        <path d="m7 4 6 6-6 6M4 10h9" />
    </svg>
);

const DatabaseMark = () => (
    <svg aria-hidden="true" className="brand-mark" viewBox="0 0 48 48">
        <ellipse cx="24" cy="13" rx="14" ry="7" />
        <path d="M10 13v11c0 4 6 7 14 7s14-3 14-7V13M10 24v11c0 4 6 7 14 7s14-3 14-7V24" />
    </svg>
);

const principles = [
    [
        '01',
        'One core, two transports',
        'Desktop and Web call the same Rust application services. Product behavior and policy stay in one place; only the delivery channel changes.',
        '一个核心，两种传输',
        '桌面端与 Web 调用相同的 Rust 应用服务。产品行为与策略只维护一份，变化的只是交付通道。',
    ],
    [
        '02',
        'Capabilities, not assumptions',
        'The interface asks what a driver or runtime can do, then exposes only the operations that are actually supported.',
        '以能力为准，而非假设',
        '界面先询问驱动或运行环境支持什么，再只呈现真正可用的操作。',
    ],
    [
        '03',
        'Local-first by default',
        'The desktop app keeps daily database work close to your machine while Web mode supports controlled shared access.',
        '默认本地优先',
        '桌面应用让日常数据库工作靠近本机；需要协作时，Web 模式提供可控的共享访问。',
    ],
    [
        '04',
        'Open and inspectable',
        'The interface, application layer, transports, and plugin contracts are open source—so teams can review how access really works.',
        '开放且可审查',
        '界面、应用层、传输层和插件契约全部开源，团队可以直接审查访问逻辑。',
    ],
] as const;

const securityLayers = [
    [
        '01',
        'WHO',
        'Identity & session',
        'Named local accounts or trusted reverse-proxy identity establish who makes each Web request. Passwords are stored as Argon2id hashes.',
        '身份与会话',
        '具名本地账户或可信反向代理身份用于确认每个 Web 请求的发起者；密码以 Argon2id 哈希保存。',
    ],
    [
        '02',
        'WHAT',
        'Authorization',
        'Commands carry an authorization level, from session access through database and sensitive operations to local administration.',
        '授权控制',
        '命令携带授权等级，覆盖会话访问、数据库操作、敏感操作到本地管理。',
    ],
    [
        '03',
        'HOW',
        'Request boundary',
        'Same-origin sessions, CSRF tokens, capability negotiation, and server-side checks protect the browser-to-backend boundary.',
        '请求边界',
        '同源会话、CSRF 令牌、能力协商与服务端检查共同保护浏览器到后端的边界。',
    ],
    [
        '04',
        'WHERE',
        'Secrets & network',
        'Connection and plugin secrets stay behind backend APIs. SSH and Kubernetes tunnels provide controlled routes to private databases.',
        '密钥与网络',
        '连接和插件密钥保留在后端 API 之后；SSH 与 Kubernetes 隧道为私有数据库提供受控路径。',
    ],
    [
        '05',
        'PROOF',
        'Audit trail',
        'Backend-generated events attribute auditable operations to an application principal without recording passwords, tokens, or result sets.',
        '审计轨迹',
        '后端生成的事件将可审计操作归属到应用主体，同时不记录密码、令牌或结果集。',
    ],
] as const;

export function App() {
    const [theme, setTheme] = useState<'dark' | 'light'>(() => {
        const savedTheme = localStorage.getItem('superdba-website-theme');
        if (savedTheme === 'dark' || savedTheme === 'light') return savedTheme;
        return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    });

    useEffect(() => {
        document.documentElement.dataset.theme = theme;
        localStorage.setItem('superdba-website-theme', theme);
    }, [theme]);

    return (
        <div className="site-shell">
            <header className="topbar">
                <a className="brand" href="#top" aria-label="SuperDBA home">
                    <DatabaseMark />
                    <span>SuperDBA</span>
                </a>
                <nav aria-label="Primary navigation / 主导航">
                    <a href="#principles">
                        Principles <small>原则</small>
                    </a>
                    <a href="#architecture">
                        How it works <small>原理</small>
                    </a>
                    <a href="#security">
                        Security <small>安全</small>
                    </a>
                    <a href="https://superdba.dev/wiki">
                        Docs <small>文档</small>
                    </a>
                </nav>
                <div className="topbar-actions">
                    <span className="language-badge" aria-label="English and Chinese">
                        EN / 中文
                    </span>
                    <button
                        className="theme-toggle"
                        type="button"
                        onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                        aria-label={
                            theme === 'dark'
                                ? 'Use light theme / 使用浅色主题'
                                : 'Use dark theme / 使用深色主题'
                        }
                        aria-pressed={theme === 'light'}
                    >
                        <span aria-hidden="true">{theme === 'dark' ? '☼' : '◐'}</span>
                    </button>
                    <a className="nav-action" href="https://github.com/SuperDBA/SuperDBA">
                        <GithubIcon />
                        <span>GitHub</span>
                    </a>
                </div>
            </header>

            <main id="top">
                <section className="hero" aria-labelledby="hero-title">
                    <div>
                        <p className="eyebrow">
                            <span /> Open source database workspace
                            <b lang="zh-CN">开源数据库工作台</b>
                        </p>
                        <h1 id="hero-title">
                            See the data.
                            <br />
                            <em>Keep the control.</em>
                        </h1>
                        <p className="hero-title-zh" lang="zh-CN">
                            看清数据，掌控始终在手。
                        </p>
                        <p className="hero-lede">
                            SuperDBA brings schemas, SQL, results, visual tools, and AI into one
                            workspace—built on a shared Rust core with explicit security boundaries.
                            <span className="zh-copy" lang="zh-CN">
                                SuperDBA 将模式浏览、SQL、结果集、可视化工具与 AI
                                集成在同一工作台中，并以共享的 Rust 核心和清晰的安全边界为基础。
                            </span>
                        </p>
                        <div className="actions">
                            <a className="button primary" href="https://superdba.dev/download">
                                Download SuperDBA <small lang="zh-CN">下载</small> <ArrowIcon />
                            </a>
                            <a
                                className="button secondary"
                                href="https://github.com/SuperDBA/SuperDBA"
                            >
                                <GithubIcon /> Read the source <small lang="zh-CN">查看源码</small>
                            </a>
                        </div>
                        <p className="hero-note">
                            Free and open source · Windows, macOS, Linux &amp; Web
                            <span lang="zh-CN">免费开源 · 支持桌面端与 Web</span>
                        </p>
                    </div>
                    <div className="hero-system" aria-label="SuperDBA request flow illustration">
                        <div className="system-header">
                            <span>REQUEST / QUERY · 请求 / 查询</span>
                            <i>protected path · 受保护路径</i>
                        </div>
                        <div className="system-flow">
                            {[
                                ['01', 'Workspace', 'SQL + intent'],
                                ['02', 'Transport', 'Desktop / Web'],
                                ['03', 'Policy gate', 'Identity + capability'],
                                ['04', 'Driver', 'Dialect-aware execution'],
                            ].map(([number, title, detail], index) => (
                                <div className="flow-wrap" key={number}>
                                    <div className={index === 2 ? 'flow-node secure' : 'flow-node'}>
                                        <small>{number}</small>
                                        <strong>{title}</strong>
                                        <span>{detail}</span>
                                    </div>
                                    {index < 3 && <b aria-hidden="true">↓</b>}
                                </div>
                            ))}
                        </div>
                        <div className="system-status">
                            <span /> Application policy checked before execution ·
                            执行前统一策略检查
                        </div>
                    </div>
                </section>

                <section className="database-strip" aria-label="Supported databases">
                    <p>
                        One workspace. Your databases.{' '}
                        <b lang="zh-CN">一个工作台，连接你的数据库。</b>
                    </p>
                    <div>
                        <span>PostgreSQL</span>
                        <span>MySQL</span>
                        <span>SQLite</span>
                        <span>DuckDB</span>
                        <span>ClickHouse</span>
                        <span>SQL Server</span>
                        <span className="accent">+ plugins</span>
                    </div>
                </section>

                <section className="principles" id="principles">
                    <div className="section-intro">
                        <p className="eyebrow">
                            <span /> Design principles
                            <b lang="zh-CN">设计原则</b>
                        </p>
                        <h2>
                            Powerful tools.
                            <br />
                            <em>Understandable boundaries.</em>
                        </h2>
                        <p className="heading-zh" lang="zh-CN">
                            工具足够强大，边界依然清晰。
                        </p>
                        <p>
                            SuperDBA is designed as a system you can reason about—not a black box
                            between you and your data.
                        </p>
                    </div>
                    <div className="principle-grid">
                        {principles.map(([number, title, copy, zhTitle, zhCopy]) => (
                            <article key={number}>
                                <span>{number}</span>
                                <h3>{title}</h3>
                                <h4 lang="zh-CN">{zhTitle}</h4>
                                <p>{copy}</p>
                                <p className="zh-copy" lang="zh-CN">
                                    {zhCopy}
                                </p>
                            </article>
                        ))}
                    </div>
                </section>

                <section className="architecture" id="architecture">
                    <div>
                        <p className="eyebrow">
                            <span /> How it works
                            <b lang="zh-CN">工作原理</b>
                        </p>
                        <h2>
                            The interface changes.
                            <br />
                            <em>The rules do not.</em>
                        </h2>
                        <p className="heading-zh" lang="zh-CN">
                            界面可以变化，规则保持一致。
                        </p>
                        <p className="section-copy">
                            The desktop app and <code>superdba web</code> use the same React
                            interface and Rust application layer. Thin transports translate native
                            commands or HTTP requests into the same application operations.
                            <span className="zh-copy" lang="zh-CN">
                                桌面应用与 superdba web 使用同一套 React 界面和 Rust
                                应用层。轻量传输层把原生命令或 HTTP 请求转换为相同的应用操作。
                            </span>
                        </p>
                        <ol className="steps">
                            <li>
                                <b>1</b>
                                <span>
                                    <strong>Describe the operation</strong>Typed commands define the
                                    request, response, and required authorization.
                                    <small lang="zh-CN">
                                        描述操作：类型化命令声明请求、响应以及所需授权等级。
                                    </small>
                                </span>
                            </li>
                            <li>
                                <b>2</b>
                                <span>
                                    <strong>Cross the transport</strong>Tauri handles native
                                    integration; Web RPC handles browser sessions and request
                                    security.
                                    <small lang="zh-CN">
                                        穿过传输层：Tauri 负责原生集成；Web RPC
                                        负责浏览器会话与请求安全。
                                    </small>
                                </span>
                            </li>
                            <li>
                                <b>3</b>
                                <span>
                                    <strong>Enforce shared policy</strong>The application layer owns
                                    behavior so transports cannot quietly diverge.
                                    <small lang="zh-CN">
                                        执行共享策略：行为由应用层统一负责，避免规则悄然分叉。
                                    </small>
                                </span>
                            </li>
                            <li>
                                <b>4</b>
                                <span>
                                    <strong>Use a capable driver</strong>Database-specific features
                                    are exposed through explicit driver capabilities.
                                    <small lang="zh-CN">
                                        调用能力驱动：数据库特有功能通过明确的能力声明对外提供。
                                    </small>
                                </span>
                            </li>
                        </ol>
                    </div>
                    <div className="layers" aria-label="SuperDBA architecture diagram">
                        <div className="layer">
                            <span>Interface · 界面层</span>
                            <strong>One React workspace · 同一套 React 工作台</strong>
                            <small>Editor · Explorer · Results · AI</small>
                        </div>
                        <div className="transport">
                            <span>Desktop transport · 桌面传输</span>
                            <i />
                            <span>Web transport · Web 传输</span>
                        </div>
                        <div className="layer core">
                            <span>Application · 应用层</span>
                            <strong>One policy and service layer · 统一策略与服务层</strong>
                            <small>Queries · Connections · Audit · Plugins</small>
                        </div>
                        <div className="layer">
                            <span>Drivers · 驱动层</span>
                            <strong>Capability-aware database access · 能力感知的数据库访问</strong>
                            <small>PostgreSQL · MySQL · SQLite · more</small>
                        </div>
                    </div>
                </section>

                <section className="security" id="security">
                    <div className="security-heading">
                        <div>
                            <p className="eyebrow">
                                <span /> Defense in depth
                                <b lang="zh-CN">纵深防御</b>
                            </p>
                            <h2>
                                Security is a stack,
                                <br />
                                <em>not a switch.</em>
                            </h2>
                            <p className="heading-zh" lang="zh-CN">
                                安全是一套层次，不是一个开关。
                            </p>
                        </div>
                        <p>
                            Each layer has a narrow job. Together they establish identity, limit
                            authority, protect requests, isolate secrets, and leave evidence—while
                            database-native grants remain the final authority over the data itself.
                        </p>
                    </div>
                    <div className="security-stack">
                        {securityLayers.map(([number, tag, title, copy, zhTitle, zhCopy]) => (
                            <article className="security-layer" key={number}>
                                <span>{number}</span>
                                <div>
                                    <small>{tag}</small>
                                    <h3>{title}</h3>
                                    <h4 lang="zh-CN">{zhTitle}</h4>
                                </div>
                                <p>{copy}</p>
                                <p className="zh-copy" lang="zh-CN">
                                    {zhCopy}
                                </p>
                                <i aria-hidden="true" />
                            </article>
                        ))}
                    </div>
                    <aside className="security-note">
                        <strong>
                            Shared responsibility <span lang="zh-CN">共同责任</span>
                        </strong>
                        <p>
                            SuperDBA adds application controls; it does not replace TLS, network
                            policy, individual database accounts, least-privilege grants, backups,
                            or database-native auditing.
                            <span className="zh-copy" lang="zh-CN">
                                SuperDBA 提供应用层控制，但不能替代
                                TLS、网络策略、个人数据库账户、最小权限授权、备份或数据库原生审计。
                            </span>
                        </p>
                    </aside>
                </section>

                <section className="proof">
                    <p className="eyebrow">
                        <span /> Inspect the implementation
                        <b lang="zh-CN">审查实现</b>
                    </p>
                    <h2>
                        Trust is better
                        <br />
                        <em>when it is verifiable.</em>
                    </h2>
                    <p className="heading-zh" lang="zh-CN">
                        信任更可靠，当它可以被验证。
                    </p>
                    <div>
                        <p>
                            Read the source, review the architecture, and deploy the runtime that
                            fits your environment.
                            <span className="zh-copy" lang="zh-CN">
                                阅读源代码、审查架构，并选择适合你环境的运行方式。
                            </span>
                        </p>
                        <a className="button primary" href="https://github.com/SuperDBA/SuperDBA">
                            Explore on GitHub <ArrowIcon />
                        </a>
                    </div>
                </section>
            </main>

            <footer>
                <a className="brand" href="#top">
                    <DatabaseMark />
                    <span>SuperDBA</span>
                </a>
                <p>Open source database workspace.</p>
                <div>
                    <a href="https://superdba.dev/wiki">Docs</a>
                    <a href="https://github.com/SuperDBA/SuperDBA">Source</a>
                    <a href="https://discord.com/invite/K2hmhfHRSt">Discord</a>
                </div>
            </footer>
        </div>
    );
}
import {useEffect, useState} from 'react';
