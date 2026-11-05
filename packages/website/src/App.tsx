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

const FeatureIcon = ({name}: {name: 'editor' | 'visual' | 'notebook' | 'agent'}) => {
    const paths = {
        editor: <path d="M5 6h14M5 12h9M5 18h6M17 15l2 2-4 4-3 1 1-3 4-4Z" />,
        visual: <path d="M6 6h4v4H6V6Zm8 8h4v4h-4v-4ZM8 10v6h6M16 14V8h-6" />,
        notebook: <path d="M7 3h10a2 2 0 0 1 2 2v16H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Zm2 5h6M9 12h6M9 16h4" />,
        agent: <path d="M12 3v3M6 8h12a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2Zm3 5v2M9 13v2M8 20v2m8-2v2" />,
    };
    return <svg aria-hidden="true" className="feature-icon" viewBox="0 0 24 24">{paths[name]}</svg>;
};

export function App() {
    return (
        <div className="site-shell">
            <header className="topbar">
                <a className="brand" href="#top" aria-label="SuperDBA home">
                    <DatabaseMark />
                    <span>SuperDBA</span>
                </a>
                <nav aria-label="Primary navigation">
                    <a href="#workspace">Workspace</a>
                    <a href="#features">Features</a>
                    <a href="#modes">Desktop &amp; Web</a>
                    <a href="https://superdba.dev/wiki">Docs</a>
                </nav>
                <a className="nav-action" href="https://github.com/SuperDBA/SuperDBA">
                    <GithubIcon />
                    <span>GitHub</span>
                </a>
            </header>

            <main id="top">
                <section className="hero" aria-labelledby="hero-title">
                    <div className="hero-copy">
                        <p className="eyebrow"><span /> Open source database workspace</p>
                        <h1 id="hero-title">Your database work,<br /><em>finally in one place.</em></h1>
                        <p className="hero-lede">
                            Explore schemas, write and run SQL, edit data, build visual queries, inspect plans, and collaborate with AI — from desktop or browser.
                        </p>
                        <div className="hero-actions">
                            <a className="button button-primary" href="https://superdba.dev/download">Download SuperDBA <ArrowIcon /></a>
                            <a className="button button-secondary" href="https://github.com/SuperDBA/SuperDBA"><GithubIcon /> View source</a>
                        </div>
                        <p className="hero-note">Free and open source · Windows, macOS, Linux &amp; Web</p>
                    </div>

                    <div className="product-window" aria-label="Illustration of the SuperDBA SQL workspace">
                        <div className="window-bar">
                            <div className="window-dots"><span /><span /><span /></div>
                            <div className="window-title">analytics · SuperDBA</div>
                            <div className="window-command">⌘ K</div>
                        </div>
                        <div className="workspace-grid">
                            <aside className="schema-panel">
                                <div className="panel-heading"><span>Explorer</span><b>+</b></div>
                                <div className="connection"><i /> production</div>
                                <div className="tree-row tree-muted">⌄ public</div>
                                <div className="tree-row active">▦ customers <small>24k</small></div>
                                <div className="tree-row">▦ orders <small>86k</small></div>
                                <div className="tree-row">▦ products <small>1.2k</small></div>
                                <div className="tree-row tree-muted">› analytics</div>
                            </aside>
                            <div className="editor-panel">
                                <div className="tab-row"><span className="tab-active">Customer growth.sql</span><span>Untitled 2</span><button type="button" aria-label="Create a new query tab">+</button></div>
                                <div className="query-area">
                                    <div className="line-numbers">1<br />2<br />3<br />4<br />5<br />6</div>
                                    <pre><code><span className="sql-keyword">SELECT</span>{'\n'}  date_trunc(<span className="sql-string">'month'</span>, created_at) <span className="sql-keyword">AS</span> month,{'\n'}  count(*) <span className="sql-keyword">AS</span> new_customers{'\n'}<span className="sql-keyword">FROM</span> customers{'\n'}<span className="sql-keyword">GROUP BY</span> month{'\n'}<span className="sql-keyword">ORDER BY</span> month;</code></pre>
                                    <button className="run-button" type="button" aria-label="Run query illustration">▶ Run</button>
                                </div>
                                <div className="results-area">
                                    <div className="result-tabs"><span>Results</span><span>Chart</span><small>12 rows · 38 ms</small></div>
                                    <div className="chart" aria-hidden="true">
                                        {[38, 48, 43, 61, 57, 72, 68, 84, 79, 92, 88, 100].map((height, index) => <i key={index} style={{'--bar-height': `${height}%`} as React.CSSProperties} />)}
                                    </div>
                                    <div className="chart-labels"><span>Jan</span><span>Apr</span><span>Jul</span><span>Oct</span><span>Dec</span></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <section className="database-strip" aria-label="Supported databases">
                    <p>One workspace. Your databases.</p>
                    <div><span>PostgreSQL</span><span>MySQL</span><span>SQLite</span><span>DuckDB</span><span>ClickHouse</span><span>SQL Server</span><span className="more-databases">+ more via plugins</span></div>
                </section>

                <section className="workspace-section" id="workspace">
                    <div className="section-heading">
                        <p className="eyebrow"><span /> Built for real database work</p>
                        <h2>Less context switching.<br /><em>More useful answers.</em></h2>
                        <p>Connections, schemas, queries, results, history, and AI tools share one secure workspace — so your flow stays intact.</p>
                    </div>
                    <div className="workflow" aria-label="SuperDBA workflow">
                        <div className="workflow-item"><b>01</b><span>Connect</span><p>Secure credentials, SSH tunnels, and connection-aware workspaces.</p></div>
                        <div className="workflow-line" />
                        <div className="workflow-item"><b>02</b><span>Understand</span><p>Search schemas, inspect relationships, and explore execution plans.</p></div>
                        <div className="workflow-line" />
                        <div className="workflow-item"><b>03</b><span>Build</span><p>Write SQL, edit rows, create notebooks, and export what matters.</p></div>
                    </div>
                </section>

                <section className="features-section" id="features">
                    <article className="feature-card feature-wide">
                        <div><FeatureIcon name="editor" /><p className="card-kicker">SQL workspace</p><h3>From idea to result, without leaving the editor.</h3><p>Dialect-aware Monaco editing, saved queries, history, multiple result sets, inline data editing, filtering, pagination, and export.</p></div>
                        <div className="mini-editor" aria-hidden="true"><div className="mini-tabs">query.sql <span>38 ms</span></div><pre><span>SELECT</span> region, sum(revenue){'\n'}<span>FROM</span> sales <span>GROUP BY</span> region;</pre><div className="mini-table"><b>region</b><b>revenue</b><span>APAC</span><span>$842,930</span><span>EMEA</span><span>$731,240</span><span>AMER</span><span>$1,024,510</span></div></div>
                    </article>
                    <article className="feature-card"><FeatureIcon name="visual" /><p className="card-kicker">Visual tools</p><h3>See structure, not just rows.</h3><p>Build queries visually, explore ER diagrams, and turn complex execution plans into something you can reason about.</p><div className="node-map" aria-hidden="true"><i className="node-a">customers</i><i className="node-b">orders</i><i className="node-c">products</i><span /></div></article>
                    <article className="feature-card"><FeatureIcon name="notebook" /><p className="card-kicker">SQL notebooks</p><h3>Analysis that explains itself.</h3><p>Combine SQL, Markdown, parameters, charts, and cross-cell results in repeatable notebooks built to share.</p><div className="notebook-stack" aria-hidden="true"><span>Markdown</span><span>SQL</span><span>Chart</span></div></article>
                    <article className="feature-card feature-agent"><FeatureIcon name="agent" /><p className="card-kicker">AI on your terms</p><h3>Your schema. Your model. Your control.</h3><p>Use cloud providers, local models through Ollama, or connect compatible AI clients through the built-in MCP server.</p><div className="agent-prompt" aria-hidden="true"><span>Ask your database…</span><b>↗</b></div></article>
                </section>

                <section className="modes-section" id="modes">
                    <div className="modes-copy"><p className="eyebrow"><span /> One interface, two runtimes</p><h2>Native when you want it.<br /><em>In the browser when you need it.</em></h2><p>The same React interface and Rust application services power desktop and Web modes. Only the transport changes.</p><a className="text-link" href="https://superdba.dev/wiki">Explore the architecture <ArrowIcon /></a></div>
                    <div className="mode-cards">
                        <div className="mode-card"><span className="mode-number">01</span><div><h3>Desktop</h3><p>Native dialogs, local files, OS integration, and everyday database work.</p><code>superdba</code></div><b>Windows · macOS · Linux</b></div>
                        <div className="mode-card mode-card-accent"><span className="mode-number">02</span><div><h3>Web</h3><p>Browser access for containers, remote hosts, and managed deployments.</p><code>superdba web</code></div><b>Docker · Compose · Helm</b></div>
                    </div>
                </section>

                <section className="cta-section">
                    <div><p className="eyebrow"><span /> Open source, ready to explore</p><h2>Bring every database<br /><em>into one workspace.</em></h2></div>
                    <div><p>Start with the desktop app, run it in your browser, or dig into the source. SuperDBA is free and Apache-2.0 licensed.</p><div className="hero-actions"><a className="button button-primary" href="https://superdba.dev/download">Download now <ArrowIcon /></a><a className="button button-secondary" href="https://superdba.dev/wiki">Read the docs</a></div></div>
                </section>
            </main>

            <footer>
                <a className="brand" href="#top"><DatabaseMark /><span>SuperDBA</span></a>
                <p>Open-source database workspace for people and AI agents.</p>
                <div><a href="https://github.com/SuperDBA/SuperDBA">GitHub</a><a href="https://superdba.dev/wiki">Documentation</a><a href="https://superdba.dev/plugins">Plugins</a><a href="https://discord.com/invite/K2hmhfHRSt">Discord</a></div>
            </footer>
        </div>
    );
}
