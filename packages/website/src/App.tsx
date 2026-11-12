const GithubIcon = () => (
    <svg aria-hidden="true" viewBox="0 0 24 24">
        <path d="M12 2a10 10 0 0 0-3.16 19.49c.5.09.68-.22.68-.48v-1.87c-2.78.6-3.37-1.18-3.37-1.18-.45-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.61.07-.61 1 .07 1.53 1.03 1.53 1.03.9 1.53 2.34 1.09 2.91.83.09-.65.35-1.09.64-1.34-2.22-.25-4.56-1.11-4.56-4.94 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.64 0 0 .84-.27 2.75 1.02A9.57 9.57 0 0 1 12 6.82c.85 0 1.7.11 2.5.34 1.9-1.29 2.74-1.02 2.74-1.02.55 1.37.2 2.39.1 2.64.64.7 1.03 1.59 1.03 2.68 0 3.84-2.34 4.68-4.57 4.93.36.31.68.92.68 1.86v2.76c0 .27.18.58.69.48A10 10 0 0 0 12 2Z" />
    </svg>
);

const ArrowIcon = () => (
    <svg aria-hidden="true" viewBox="0 0 20 20"><path d="m7 4 6 6-6 6M4 10h9" /></svg>
);

const DatabaseMark = () => (
    <svg aria-hidden="true" className="brand-mark" viewBox="0 0 48 48">
        <ellipse cx="24" cy="13" rx="14" ry="7" />
        <path d="M10 13v11c0 4 6 7 14 7s14-3 14-7V13M10 24v11c0 4 6 7 14 7s14-3 14-7V24" />
    </svg>
);

const principles = [
    ['01', 'One core, two transports', 'Desktop and Web call the same Rust application services. Product behavior and policy stay in one place; only the delivery channel changes.'],
    ['02', 'Capabilities, not assumptions', 'The interface asks what a driver or runtime can do, then exposes only the operations that are actually supported.'],
    ['03', 'Local-first by default', 'The desktop app keeps daily database work close to your machine while Web mode supports controlled shared access.'],
    ['04', 'Open and inspectable', 'The interface, application layer, transports, and plugin contracts are open source—so teams can review how access really works.'],
] as const;

const securityLayers = [
    ['01', 'WHO', 'Identity & session', 'Named local accounts or trusted reverse-proxy identity establish who makes each Web request. Passwords are stored as Argon2id hashes.'],
    ['02', 'WHAT', 'Authorization', 'Commands carry an authorization level, from session access through database and sensitive operations to local administration.'],
    ['03', 'HOW', 'Request boundary', 'Same-origin sessions, CSRF tokens, capability negotiation, and server-side checks protect the browser-to-backend boundary.'],
    ['04', 'WHERE', 'Secrets & network', 'Connection and plugin secrets stay behind backend APIs. SSH and Kubernetes tunnels provide controlled routes to private databases.'],
    ['05', 'PROOF', 'Audit trail', 'Backend-generated events attribute auditable operations to an application principal without recording passwords, tokens, or result sets.'],
] as const;

export function App() {
    return (
        <div className="site-shell">
            <header className="topbar">
                <a className="brand" href="#top" aria-label="SuperDBA home"><DatabaseMark /><span>SuperDBA</span></a>
                <nav aria-label="Primary navigation">
                    <a href="#principles">Principles</a><a href="#architecture">How it works</a><a href="#security">Security</a><a href="https://superdba.dev/wiki">Docs</a>
                </nav>
                <a className="nav-action" href="https://github.com/SuperDBA/SuperDBA"><GithubIcon /><span>GitHub</span></a>
            </header>

            <main id="top">
                <section className="hero" aria-labelledby="hero-title">
                    <div>
                        <p className="eyebrow"><span /> Open source database workspace</p>
                        <h1 id="hero-title">See the data.<br /><em>Keep the control.</em></h1>
                        <p className="hero-lede">SuperDBA brings schemas, SQL, results, visual tools, and AI into one workspace—built on a shared Rust core with explicit security boundaries.</p>
                        <div className="actions">
                            <a className="button primary" href="https://superdba.dev/download">Download SuperDBA <ArrowIcon /></a>
                            <a className="button secondary" href="https://github.com/SuperDBA/SuperDBA"><GithubIcon /> Read the source</a>
                        </div>
                        <p className="hero-note">Free and open source · Windows, macOS, Linux &amp; Web</p>
                    </div>
                    <div className="hero-system" aria-label="SuperDBA request flow illustration">
                        <div className="system-header"><span>REQUEST / QUERY</span><i>protected path</i></div>
                        <div className="system-flow">
                            {[
                                ['01', 'Workspace', 'SQL + intent'],
                                ['02', 'Transport', 'Desktop / Web'],
                                ['03', 'Policy gate', 'Identity + capability'],
                                ['04', 'Driver', 'Dialect-aware execution'],
                            ].map(([number, title, detail], index) => (
                                <div className="flow-wrap" key={number}>
                                    <div className={index === 2 ? 'flow-node secure' : 'flow-node'}><small>{number}</small><strong>{title}</strong><span>{detail}</span></div>
                                    {index < 3 && <b aria-hidden="true">↓</b>}
                                </div>
                            ))}
                        </div>
                        <div className="system-status"><span /> Application policy checked before execution</div>
                    </div>
                </section>

                <section className="database-strip" aria-label="Supported databases">
                    <p>One workspace. Your databases.</p>
                    <div><span>PostgreSQL</span><span>MySQL</span><span>SQLite</span><span>DuckDB</span><span>ClickHouse</span><span>SQL Server</span><span className="accent">+ plugins</span></div>
                </section>

                <section className="principles" id="principles">
                    <div className="section-intro">
                        <p className="eyebrow"><span /> Design principles</p>
                        <h2>Powerful tools.<br /><em>Understandable boundaries.</em></h2>
                        <p>SuperDBA is designed as a system you can reason about—not a black box between you and your data.</p>
                    </div>
                    <div className="principle-grid">
                        {principles.map(([number, title, copy]) => <article key={number}><span>{number}</span><h3>{title}</h3><p>{copy}</p></article>)}
                    </div>
                </section>

                <section className="architecture" id="architecture">
                    <div>
                        <p className="eyebrow"><span /> How it works</p>
                        <h2>The interface changes.<br /><em>The rules do not.</em></h2>
                        <p className="section-copy">The desktop app and <code>superdba web</code> use the same React interface and Rust application layer. Thin transports translate native commands or HTTP requests into the same application operations.</p>
                        <ol className="steps">
                            <li><b>1</b><span><strong>Describe the operation</strong>Typed commands define the request, response, and required authorization.</span></li>
                            <li><b>2</b><span><strong>Cross the transport</strong>Tauri handles native integration; Web RPC handles browser sessions and request security.</span></li>
                            <li><b>3</b><span><strong>Enforce shared policy</strong>The application layer owns behavior so transports cannot quietly diverge.</span></li>
                            <li><b>4</b><span><strong>Use a capable driver</strong>Database-specific features are exposed through explicit driver capabilities.</span></li>
                        </ol>
                    </div>
                    <div className="layers" aria-label="SuperDBA architecture diagram">
                        <div className="layer"><span>Interface</span><strong>One React workspace</strong><small>Editor · Explorer · Results · AI</small></div>
                        <div className="transport"><span>Desktop transport</span><i /><span>Web transport</span></div>
                        <div className="layer core"><span>Application</span><strong>One policy and service layer</strong><small>Queries · Connections · Audit · Plugins</small></div>
                        <div className="layer"><span>Drivers</span><strong>Capability-aware database access</strong><small>PostgreSQL · MySQL · SQLite · more</small></div>
                    </div>
                </section>

                <section className="security" id="security">
                    <div className="security-heading">
                        <div><p className="eyebrow"><span /> Defense in depth</p><h2>Security is a stack,<br /><em>not a switch.</em></h2></div>
                        <p>Each layer has a narrow job. Together they establish identity, limit authority, protect requests, isolate secrets, and leave evidence—while database-native grants remain the final authority over the data itself.</p>
                    </div>
                    <div className="security-stack">
                        {securityLayers.map(([number, tag, title, copy]) => (
                            <article className="security-layer" key={number}><span>{number}</span><div><small>{tag}</small><h3>{title}</h3></div><p>{copy}</p><i aria-hidden="true" /></article>
                        ))}
                    </div>
                    <aside className="security-note"><strong>Shared responsibility</strong><p>SuperDBA adds application controls; it does not replace TLS, network policy, individual database accounts, least-privilege grants, backups, or database-native auditing.</p></aside>
                </section>

                <section className="proof">
                    <p className="eyebrow"><span /> Inspect the implementation</p>
                    <h2>Trust is better<br /><em>when it is verifiable.</em></h2>
                    <div><p>Read the source, review the architecture, and deploy the runtime that fits your environment.</p><a className="button primary" href="https://github.com/SuperDBA/SuperDBA">Explore on GitHub <ArrowIcon /></a></div>
                </section>
            </main>

            <footer><a className="brand" href="#top"><DatabaseMark /><span>SuperDBA</span></a><p>Open source database workspace.</p><div><a href="https://superdba.dev/wiki">Docs</a><a href="https://github.com/SuperDBA/SuperDBA">Source</a><a href="https://discord.com/invite/K2hmhfHRSt">Discord</a></div></footer>
        </div>
    );
}
