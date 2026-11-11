import {useMemo, useRef, useState, useEffect, useCallback} from 'react';
import {useTranslation} from 'react-i18next';
import {
    ChevronDown,
    ChevronRight,
    MessageSquarePlus,
    Send,
    Trash2,
    X,
    Loader2,
    Sparkles,
    Square,
    Wrench,
    Copy,
    Check,
    AlertCircle,
    RotateCcw,
    Settings,
    Database,
    Code2,
    Table2,
    ArrowDown,
} from 'lucide-react';
import {useNavigate} from 'react-router-dom';
import {useChatAgent} from '@/hooks/useChat';
import {ChatMarkdown} from '@/components/chat/ChatMarkdown';
import {useDatabase} from '@/hooks/useDatabase';
import {useEditor} from '@/hooks/useEditor';
import {useRightSidebar} from '@/hooks/useRightSidebar';
import {useSettings} from '@/hooks/useSettings';
import {
    extractSqlBlock,
    formatQueryResultSummary,
    formatToolResult,
    shouldSubmitChat,
    classifyChatError,
    isQueryBudgetPause,
    summarizeChatTools,
} from '@/utils/chat';
import {
    buildLocalStarters,
    cachedStarters,
    rememberStarters,
    sanitizeGeneratedStarters,
    selectPromptTables,
    starterCacheKey,
} from '@/utils/chatStarters';
import type {ChatMessage, ChatToolCall} from '@/types/chat';
import {useCopyFeedback} from '@/hooks/useCopyFeedback';
import {isDbaChatProviderSupported} from '@/utils/dbaChat';

function ToolCallRow({call}: {call: ChatToolCall}) {
    const {t} = useTranslation();
    const [open, setOpen] = useState(false);
    const result = useMemo(() => formatToolResult(call.result), [call.result]);
    const budgetPaused = call.status === 'denied' && isQueryBudgetPause(call.summary);
    const statusClass =
        call.status === 'ok'
            ? 'text-accent-success'
            : call.status === 'denied' || budgetPaused
              ? 'text-accent-warning'
              : 'text-accent-error';
    const statusLabel = budgetPaused
        ? t('chat.toolStatus.paused')
        : t(`chat.toolStatus.${call.status}`, {defaultValue: call.status});

    return (
        <div className="border border-default rounded-md bg-surface-secondary/60">
            <button
                type="button"
                onClick={() => setOpen((value) => !value)}
                aria-expanded={open}
                className="w-full flex items-center gap-2 px-2 py-1.5 text-left text-xs hover:bg-surface-hover rounded-md"
            >
                {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                <Wrench size={12} className="text-muted" />
                <span className="font-mono text-primary">{call.name}</span>
                <span className={statusClass}>{statusLabel}</span>
                <span className="text-muted truncate flex-1">{call.summary}</span>
                <span className="text-muted tabular-nums">{call.durationMs} ms</span>
            </button>
            {open && (
                <div className="px-3 pb-2 space-y-2">
                    <div>
                        <div className="text-[10px] uppercase tracking-wide text-muted mb-1">
                            {t('chat.arguments', {defaultValue: 'Arguments'})}
                        </div>
                        <pre className="text-[11px] font-mono bg-base border border-default rounded p-2 overflow-x-auto max-h-40">
                            {JSON.stringify(call.arguments, null, 2)}
                        </pre>
                    </div>
                    {result && (
                        <div>
                            <div className="text-[10px] uppercase tracking-wide text-muted mb-1">
                                {t('chat.result', {defaultValue: 'Result'})}
                            </div>
                            <pre className="text-[11px] font-mono bg-base border border-default rounded p-2 overflow-auto max-h-52">
                                {result}
                            </pre>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

function ToolActivity({calls}: {calls: ChatToolCall[]}) {
    const {t} = useTranslation();
    const summary = useMemo(() => summarizeChatTools(calls), [calls]);
    const [open, setOpen] = useState(() =>
        calls.some(
            (call) =>
                call.status === 'pending' ||
                (call.status !== 'ok' && !isQueryBudgetPause(call.summary)),
        ),
    );

    return (
        <div className="rounded-md border border-default bg-surface-secondary/40">
            <button
                type="button"
                onClick={() => setOpen((value) => !value)}
                aria-expanded={open}
                className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-surface-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
            >
                {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                <Wrench size={12} className="text-muted" aria-hidden="true" />
                <span className="min-w-0 flex-1 truncate text-secondary">
                    {t('chat.toolActivity', {
                        count: summary.total,
                        completed: summary.completed,
                        defaultValue: '{{count}} tool calls · {{completed}} completed',
                    })}
                </span>
                {summary.pending > 0 && (
                    <span className="text-accent-info">{t('chat.toolStatus.pending')}</span>
                )}
                {summary.issues > 0 && (
                    <span className="text-accent-error">
                        {t('chat.toolIssues', {
                            count: summary.issues,
                            defaultValue: '{{count}} issues',
                        })}
                    </span>
                )}
                <span className="shrink-0 tabular-nums text-muted">{summary.durationMs} ms</span>
            </button>
            {open && (
                <div className="space-y-1 border-t border-default p-1.5">
                    {calls.map((call) => (
                        <ToolCallRow key={call.id} call={call} />
                    ))}
                </div>
            )}
        </div>
    );
}

function ChatErrorBanner({
    message,
    sending,
    onRetry,
    onSettings,
    onDismiss,
}: {
    message: string;
    sending: boolean;
    onRetry: () => void;
    onSettings: () => void;
    onDismiss: () => void;
}) {
    const {t} = useTranslation();
    const [showDetails, setShowDetails] = useState(false);
    const kind = classifyChatError(message);
    const summary = t(`chat.error.${kind}`, {
        defaultValue: t('chat.error.unknown'),
    });

    return (
        <div
            role="alert"
            className="mx-3 mb-2 rounded-md border border-accent-error/30 bg-accent-error/10 px-2.5 py-2 text-xs text-accent-error"
        >
            <div className="flex items-start gap-2">
                <AlertCircle size={14} className="mt-0.5 shrink-0" aria-hidden="true" />
                <div className="min-w-0 flex-1">
                    <div className="font-medium">{summary}</div>
                    <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px]">
                        <button
                            type="button"
                            onClick={onRetry}
                            disabled={sending}
                            className="inline-flex items-center gap-1 hover:underline disabled:opacity-50"
                        >
                            <RotateCcw size={11} aria-hidden="true" />
                            {t('common.retry')}
                        </button>
                        {kind === 'authentication' && (
                            <button type="button" onClick={onSettings} className="hover:underline">
                                {t('chat.error.openSettings')}
                            </button>
                        )}
                        <button
                            type="button"
                            onClick={() => setShowDetails((value) => !value)}
                            aria-expanded={showDetails}
                            className="hover:underline"
                        >
                            {showDetails ? t('common.hideDetails') : t('common.showDetails')}
                        </button>
                    </div>
                    {showDetails && (
                        <pre className="mt-2 max-h-32 overflow-auto whitespace-pre-wrap break-words rounded border border-accent-error/20 bg-base p-2 font-mono text-[10px] text-secondary">
                            {message}
                        </pre>
                    )}
                </div>
                <button
                    type="button"
                    onClick={onDismiss}
                    aria-label={t('common.dismiss', {defaultValue: 'Dismiss'})}
                    className="rounded p-0.5 hover:bg-accent-error/10 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                >
                    <X size={12} />
                </button>
            </div>
        </div>
    );
}

function MessageBubble({
    message,
    onInsert,
}: {
    message: ChatMessage;
    onInsert: (sql: string) => void;
}) {
    const {t} = useTranslation();
    const sql = message.role === 'assistant' ? extractSqlBlock(message.content) : null;
    const {copied: messageCopied, copy: copyMessage} = useCopyFeedback(1500);
    const {copied: sqlCopied, copy: copySqlText} = useCopyFeedback(1500);
    const [inserted, setInserted] = useState(false);

    const copySql = async () => {
        if (!sql) return;
        await copySqlText(sql);
    };

    if (message.role === 'user') {
        return (
            <div className="self-end max-w-[92%] rounded-lg bg-accent-primary/15 border border-accent-primary/30 px-3 py-2 text-sm text-primary whitespace-pre-wrap break-words">
                {message.content}
            </div>
        );
    }

    return (
        <div className="max-w-full space-y-2">
            <ChatMarkdown content={message.content} />
            {message.toolCalls && message.toolCalls.length > 0 && (
                <ToolActivity calls={message.toolCalls} />
            )}
            <div className="flex items-center gap-3">
                <button
                    type="button"
                    onClick={() => void copyMessage(message.content)}
                    aria-label={t('common.copy')}
                    className="inline-flex items-center gap-1 text-[11px] text-accent hover:underline"
                >
                    {messageCopied ? <Check size={12} /> : <Copy size={12} />}
                    {messageCopied ? t('chat.copied', {defaultValue: 'Copied'}) : t('common.copy')}
                </button>
                {sql && (
                    <>
                        <button
                            type="button"
                            onClick={() => void copySql()}
                            className="text-[11px] text-accent hover:underline"
                        >
                            {sqlCopied
                                ? t('chat.copied', {defaultValue: 'Copied'})
                                : t('chat.copySql', {defaultValue: 'Copy SQL'})}
                        </button>
                        <button
                            type="button"
                            onClick={() => {
                                onInsert(sql);
                                setInserted(true);
                                window.setTimeout(() => setInserted(false), 1500);
                            }}
                            className="text-[11px] text-accent hover:underline"
                        >
                            {inserted
                                ? t('chat.inserted', {defaultValue: 'Inserted'})
                                : t('chat.insertSql', {defaultValue: 'Insert into editor'})}
                        </button>
                    </>
                )}
            </div>
            {message.notice && (
                <div className="text-[11px] text-accent-warning border border-accent-warning/30 rounded px-2 py-1">
                    {message.notice}
                </div>
            )}
        </div>
    );
}

export function ChatPanel() {
    const {t, i18n} = useTranslation();
    const navigate = useNavigate();
    const {close} = useRightSidebar();
    const {activeTab, updateTab, addTab} = useEditor();
    const {activeConnectionId, activeSchema, activeDatabaseName, connections, tables} =
        useDatabase();
    const {settings} = useSettings();
    const {
        sessions,
        session,
        sending,
        stream,
        error,
        continuationRequired,
        send,
        suggestStarters,
        cancel,
        openSession,
        startNewSession,
        deleteSession,
        setError,
    } = useChatAgent();

    const [input, setInput] = useState('');
    const [showSessions, setShowSessions] = useState(false);
    const [showJumpToLatest, setShowJumpToLatest] = useState(false);
    const scrollRef = useRef<HTMLDivElement>(null);
    const followsLatest = useRef(true);

    /**
     * Appends the SQL to the active editor tab, or opens a new console tab when
     * the active tab is not a SQL editor (table/users/notebook).
     */
    const insertSql = useCallback(
        (sql: string) => {
            const editable =
                activeTab &&
                activeTab.type !== 'table' &&
                activeTab.type !== 'users' &&
                activeTab.type !== 'notebook';
            if (!activeTab || !editable) {
                addTab({query: sql});
                return;
            }
            const existing = activeTab.query?.trim() ?? '';
            if (!existing) {
                updateTab(activeTab.id, {query: sql});
                return;
            }
            const separator = existing.endsWith(';') ? '\n\n' : ';\n\n';
            updateTab(activeTab.id, {query: `${existing}${separator}${sql}`});
        },
        [activeTab, addTab, updateTab],
    );

    const messages = session?.messages ?? [];

    useEffect(() => {
        const node = scrollRef.current;
        if (node && followsLatest.current) node.scrollTop = node.scrollHeight;
    }, [messages.length, sending, stream?.text, stream?.tools.length]);

    const connectionName = connections.find((entry) => entry.id === activeConnectionId)?.name;
    const activeConnection = connections.find((entry) => entry.id === activeConnectionId);
    const strictMode = activeConnection?.environment === 'production';
    const chatProvider = settings.dbaChatProvider ?? settings.aiProvider;
    const chatModel = settings.dbaChatModel?.trim() || settings.aiModel || '';
    const callsPerBatch = settings.aiChatCallsPerBatch ?? 16;
    const maxBatches = settings.aiChatMaxBatches ?? 4;
    const totalBudget = callsPerBatch * maxBatches;
    const completedCalls =
        stream?.batchProgress?.completedCalls ?? continuationRequired?.completedCalls ?? 0;
    const effectiveTotalBudget =
        stream?.batchProgress?.totalCalls ?? continuationRequired?.totalCalls ?? totalBudget;
    const remainingBudget = Math.max(0, effectiveTotalBudget - completedCalls);
    const analysisMode =
        callsPerBatch === 8 && maxBatches === 2
            ? t('settings.dbaChat.presetQuick')
            : callsPerBatch === 16 && maxBatches === 4
              ? t('settings.dbaChat.presetStandard')
              : callsPerBatch === 24 && maxBatches === 6
                ? t('settings.dbaChat.presetDeep')
                : t('settings.dbaChat.presetCustom');

    const promptTables = useMemo(
        () => selectPromptTables(tables.map((table) => table.name)),
        [tables],
    );

    /** Instant, offline-safe questions derived only from the loaded table names. */
    const localStarters = useMemo(
        () =>
            buildLocalStarters(promptTables).map(({key, fallback, params}) =>
                t(key, {defaultValue: fallback, ...params}),
            ),
        [promptTables, t],
    );

    /**
     * Last model answer, tagged with the scope it was written for. Tagging
     * keeps a previous connection's questions off the screen while the new
     * scope loads, without writing state from the effect body.
     */
    const [generatedStarters, setGeneratedStarters] = useState<{
        key: string;
        values: string[];
    } | null>(null);
    const startersKey = useMemo(
        () =>
            starterCacheKey(
                settings.aiProvider ?? '',
                settings.aiModel ?? '',
                i18n.language,
                activeConnectionId,
                promptTables,
            ),
        [settings.aiProvider, settings.aiModel, i18n.language, activeConnectionId, promptTables],
    );
    const showEmptyState = messages.length === 0 && !stream;
    const customStarters = sanitizeGeneratedStarters(settings.dbaChatCustomStarters ?? []);

    // Model-written questions are a progressive enhancement: the empty state
    // paints the local set immediately and this effect only swaps it in later.
    useEffect(() => {
        if (!showEmptyState || settings.dbaChatShowSuggestions === false) return;
        if (customStarters.length > 0) return;
        if (cachedStarters(startersKey)) return;
        if (settings.dbaChatGenerateSuggestions === false) return;
        if (!settings.aiProvider || promptTables.length === 0) return;
        let cancelled = false;
        void suggestStarters(
            {
                ...(activeConnectionId ? {connectionId: activeConnectionId} : {}),
                ...(connectionName ? {connectionName} : {}),
                ...(activeSchema ? {schema: activeSchema} : {}),
                ...(activeDatabaseName ? {database: activeDatabaseName} : {}),
            },
            promptTables,
            i18n.language,
        ).then((values) => {
            if (cancelled || values.length === 0) return;
            rememberStarters(startersKey, values);
            setGeneratedStarters({key: startersKey, values});
        });
        return () => {
            cancelled = true;
        };
    }, [
        activeConnectionId,
        activeDatabaseName,
        activeSchema,
        connectionName,
        customStarters.length,
        i18n.language,
        promptTables,
        settings.aiProvider,
        settings.dbaChatGenerateSuggestions,
        settings.dbaChatShowSuggestions,
        showEmptyState,
        startersKey,
        suggestStarters,
    ]);

    const starterPrompts =
        settings.dbaChatShowSuggestions === false
            ? []
            : customStarters.length > 0
              ? customStarters
              : settings.dbaChatGenerateSuggestions === false
                ? localStarters
                : ((generatedStarters?.key === startersKey ? generatedStarters.values : null) ??
                  cachedStarters(startersKey) ??
                  localStarters);

    const buildContext = () => ({
        ...(activeConnectionId ? {connectionId: activeConnectionId} : {}),
        ...(connectionName ? {connectionName} : {}),
        ...(activeSchema ? {schema: activeSchema} : {}),
        ...(activeDatabaseName ? {database: activeDatabaseName} : {}),
        ...(settings.dbaChatIncludeCurrentSql !== false && activeTab?.selectedSql
            ? {selectedSql: activeTab.selectedSql}
            : {}),
        ...(settings.dbaChatIncludeLastResult === true && activeTab?.result
            ? {
                  lastResult: formatQueryResultSummary(activeTab.result, 20, 2000, {
                      enabled: settings.dbaChatMaskSensitiveResults !== false,
                      patterns: settings.columnMaskingPatterns,
                      overrides: settings.columnMaskingOverrides,
                      connectionId: activeConnectionId ?? undefined,
                      tableName: activeTab.activeTable ?? undefined,
                  }),
              }
            : {}),
        ...(settings.dbaChatIncludeLastPlan !== false && activeTab?.lastExplainPlan
            ? {lastPlan: activeTab.lastExplainPlan}
            : {}),
        ...(strictMode ? {strictMode: true} : {}),
        ...(settings.dbaChatMaskSensitiveResults !== false ? {maskSensitiveResults: true} : {}),
    });

    const submit = async () => {
        if (!input.trim() || sending) return;
        const message = input;
        const succeeded = await send(message, buildContext());
        if (succeeded) setInput('');
    };

    const providerReady = isDbaChatProviderSupported(chatProvider);
    const includesCurrentSql =
        settings.dbaChatIncludeCurrentSql !== false && Boolean(activeTab?.selectedSql);
    const includesLastResult =
        settings.dbaChatIncludeLastResult === true && Boolean(activeTab?.result);
    const includesLastPlan =
        settings.dbaChatIncludeLastPlan !== false && Boolean(activeTab?.lastExplainPlan);

    return (
        <div className="flex flex-col h-full min-h-0">
            <header className="flex items-center gap-2 px-3 py-2 border-b border-default">
                <Sparkles size={16} className="text-accent" />
                <h2 className="text-sm font-semibold text-primary flex-1">
                    {t('chat.title', {defaultValue: 'DBA chat'})}
                </h2>
                <button
                    type="button"
                    onClick={() => navigate('/settings?tab=dba-chat')}
                    aria-label={t('chat.openSettings')}
                    title={t('chat.openSettings')}
                    className="p-1 rounded text-muted hover:text-primary hover:bg-surface-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                >
                    <Settings size={14} aria-hidden="true" />
                </button>
                <button
                    type="button"
                    onClick={() => setShowSessions((value) => !value)}
                    aria-label={t('chat.sessions', {defaultValue: 'Conversations'})}
                    title={t('chat.sessions', {defaultValue: 'Conversations'})}
                    aria-expanded={showSessions}
                    className="p-1 rounded hover:bg-surface-hover text-muted hover:text-primary"
                >
                    <ChevronDown size={14} />
                </button>
                <button
                    type="button"
                    onClick={startNewSession}
                    aria-label={t('chat.newSession', {defaultValue: 'New conversation'})}
                    title={t('chat.newSession', {defaultValue: 'New conversation'})}
                    className="p-1 rounded hover:bg-surface-hover text-muted hover:text-primary"
                >
                    <MessageSquarePlus size={14} />
                </button>
                <button
                    type="button"
                    onClick={close}
                    aria-label={t('common.close', {defaultValue: 'Close'})}
                    className="p-1 rounded hover:bg-surface-hover text-muted hover:text-primary"
                >
                    <X size={14} />
                </button>
            </header>

            <div className="flex flex-wrap items-center gap-1.5 border-b border-default bg-surface-secondary/40 px-3 py-1.5 text-[10px] text-secondary">
                <span>{t('chat.statusMode', {mode: analysisMode})}</span>
                <span aria-hidden="true">·</span>
                <span title={`${chatProvider ?? ''} / ${chatModel}`}>
                    {t('chat.statusModel', {
                        model: [chatProvider, chatModel].filter(Boolean).join(' / ') || '-',
                    })}
                </span>
                <span aria-hidden="true">·</span>
                <span>{t('chat.statusBudget', {count: remainingBudget})}</span>
                {strictMode && (
                    <span className="rounded bg-accent-warning/15 px-1 py-0.5 text-accent-warning">
                        {t('chat.productionStrict')}
                    </span>
                )}
            </div>

            {showSessions && (
                <div className="border-b border-default max-h-56 overflow-auto">
                    {sessions.length === 0 ? (
                        <p className="px-3 py-3 text-xs text-muted">
                            {t('chat.noSessions', {defaultValue: 'No conversations yet.'})}
                        </p>
                    ) : (
                        sessions.map((entry) => (
                            <div
                                key={entry.id}
                                className="flex items-center gap-2 px-3 py-2 text-xs hover:bg-surface-hover"
                            >
                                <button
                                    type="button"
                                    onClick={() => {
                                        void openSession(entry.id);
                                        setShowSessions(false);
                                    }}
                                    className="flex-1 text-left min-w-0"
                                >
                                    <div className="truncate text-primary">{entry.title}</div>
                                    <div className="text-muted">
                                        {entry.messageCount}{' '}
                                        {t('chat.messages', {defaultValue: 'messages'})}
                                    </div>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => void deleteSession(entry.id)}
                                    aria-label={t('chat.deleteSession', {
                                        defaultValue: 'Delete conversation',
                                    })}
                                    className="p-1 rounded text-muted hover:text-accent-error hover:bg-surface-secondary"
                                >
                                    <Trash2 size={12} />
                                </button>
                            </div>
                        ))
                    )}
                </div>
            )}

            <div className="relative flex-1 min-h-0">
                <div
                    ref={scrollRef}
                    onScroll={(event) => {
                        const node = event.currentTarget;
                        const nearBottom =
                            node.scrollHeight - node.scrollTop - node.clientHeight < 48;
                        followsLatest.current = nearBottom;
                        setShowJumpToLatest(!nearBottom);
                    }}
                    className="h-full overflow-y-auto px-3 py-3"
                >
                    {messages.length === 0 && !stream ? (
                        <div className="h-full flex flex-col items-center justify-center text-center gap-3 px-2">
                            <Sparkles size={20} className="text-accent" />
                            <p className="text-sm text-primary">
                                {t('chat.emptyTitle', {defaultValue: 'Ask about this database'})}
                            </p>
                            <p className="text-xs text-muted max-w-[240px]">
                                {t('chat.emptyHint', {
                                    defaultValue:
                                        'The assistant inspects the schema with read-only tools and shows every query it runs.',
                                })}
                            </p>
                            {starterPrompts.map((prompt) => (
                                <button
                                    key={prompt}
                                    type="button"
                                    onClick={() => setInput(prompt)}
                                    className="text-xs text-left border border-default rounded px-2 py-1.5 hover:bg-surface-hover text-secondary w-full"
                                >
                                    {prompt}
                                </button>
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col gap-3">
                            {messages
                                .filter((message) => message.role !== 'system')
                                .map((message) => (
                                    <MessageBubble
                                        key={message.id}
                                        message={message}
                                        onInsert={insertSql}
                                    />
                                ))}
                            {stream && (
                                <div className="max-w-full space-y-2">
                                    {stream.text ? (
                                        <ChatMarkdown content={stream.text} />
                                    ) : stream.failed ? null : (
                                        <div className="flex items-center gap-2 text-xs text-muted">
                                            <Loader2 size={12} className="animate-spin" />
                                            {t('chat.working', {
                                                defaultValue: 'Gathering evidence…',
                                            })}
                                        </div>
                                    )}
                                    {stream.tools.length > 0 && (
                                        <ToolActivity calls={stream.tools} />
                                    )}
                                    {stream.batchProgress && (
                                        <div className="rounded border border-accent-info/40 bg-accent-info/10 px-2 py-1 text-[11px] text-accent-info">
                                            {t('chat.batchProgress', {
                                                defaultValue:
                                                    'Batch {{completedBatch}}/{{totalBatches}} complete - {{completedCalls}}/{{totalCalls}} model calls',
                                                ...stream.batchProgress,
                                            })}
                                        </div>
                                    )}
                                    {stream.awaitingApproval && (
                                        <div className="border border-accent-warning/40 bg-accent-warning/10 rounded px-2 py-1.5 space-y-1">
                                            <div className="text-[11px] text-accent-warning">
                                                {t('chat.waitingApproval', {
                                                    defaultValue: 'Waiting for approval: {{kind}}',
                                                    kind: stream.awaitingApproval.queryKind,
                                                })}
                                            </div>
                                            <pre className="text-[11px] font-mono bg-base border border-default rounded p-1.5 overflow-x-auto max-h-24">
                                                {stream.awaitingApproval.query}
                                            </pre>
                                            <div className="text-[10px] text-muted">
                                                {t('chat.approvalHint', {
                                                    defaultValue:
                                                        'Approve or deny it in the approval dialog. The statement runs only after you approve.',
                                                })}
                                            </div>
                                        </div>
                                    )}
                                    {stream.cancelled && (
                                        <div className="text-[11px] text-accent-warning">
                                            {t('chat.cancelled', {defaultValue: 'Stopped.'})}
                                        </div>
                                    )}
                                </div>
                            )}
                            {continuationRequired && !sending && (
                                <button
                                    type="button"
                                    onClick={() =>
                                        void send(t('chat.continuePrompt'), buildContext())
                                    }
                                    className="self-start rounded-md bg-accent-primary px-3 py-1.5 text-xs font-medium text-inverse hover:opacity-90 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                                >
                                    {t('chat.continueInvestigation')}
                                </button>
                            )}
                        </div>
                    )}
                </div>
                {showJumpToLatest && (
                    <button
                        type="button"
                        onClick={() => {
                            const node = scrollRef.current;
                            if (!node) return;
                            followsLatest.current = true;
                            setShowJumpToLatest(false);
                            node.scrollTo({top: node.scrollHeight, behavior: 'smooth'});
                        }}
                        className="absolute bottom-2 left-1/2 inline-flex -translate-x-1/2 items-center gap-1 rounded-full border border-default bg-elevated px-2.5 py-1 text-[11px] text-secondary shadow-md hover:bg-surface-hover hover:text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                    >
                        <ArrowDown size={12} aria-hidden="true" />
                        {t('chat.jumpToLatest')}
                    </button>
                )}
            </div>

            {error && (
                <ChatErrorBanner
                    message={error}
                    sending={sending}
                    onRetry={() => void submit()}
                    onSettings={() => navigate('/settings?tab=dba-chat')}
                    onDismiss={() => setError(null)}
                />
            )}

            {!providerReady && (
                <div className="mx-3 mb-2 text-[11px] text-accent-warning border border-accent-warning/30 rounded px-2 py-1">
                    {chatProvider
                        ? t('chat.unsupportedProvider', {provider: chatProvider})
                        : t('chat.configureProvider', {
                              defaultValue:
                                  'Select an AI provider in Settings → AI to start chatting.',
                          })}
                </div>
            )}

            <footer className="border-t border-default p-2">
                <div
                    className="mb-2 flex flex-wrap items-center gap-1.5"
                    aria-label={t('chat.attachedContext')}
                >
                    <span className="text-[10px] text-muted">{t('chat.contextLabel')}</span>
                    {activeConnectionId ? (
                        <span
                            className="inline-flex items-center gap-1 rounded border border-default bg-surface-secondary px-1.5 py-0.5 text-[10px] text-secondary"
                            title={connectionName}
                        >
                            <Database size={10} aria-hidden="true" />
                            {t('chat.contextConnection')}
                        </span>
                    ) : (
                        <span className="text-[10px] text-accent-warning">
                            {t('chat.contextNoConnection')}
                        </span>
                    )}
                    {includesCurrentSql && (
                        <span className="inline-flex items-center gap-1 rounded border border-default bg-surface-secondary px-1.5 py-0.5 text-[10px] text-secondary">
                            <Code2 size={10} aria-hidden="true" />
                            {t('chat.contextSql')}
                        </span>
                    )}
                    {includesLastResult && (
                        <span className="inline-flex items-center gap-1 rounded border border-accent-warning/40 bg-accent-warning/10 px-1.5 py-0.5 text-[10px] text-accent-warning">
                            <Table2 size={10} aria-hidden="true" />
                            {t('chat.contextResult', {
                                count: activeTab?.result?.rows.length ?? 0,
                            })}
                        </span>
                    )}
                    {includesLastPlan && (
                        <span className="inline-flex items-center gap-1 rounded border border-default bg-surface-secondary px-1.5 py-0.5 text-[10px] text-secondary">
                            <Code2 size={10} aria-hidden="true" />
                            {t('chat.contextPlan')}
                        </span>
                    )}
                </div>
                <div className="flex items-end gap-2">
                    <textarea
                        value={input}
                        onChange={(event) => setInput(event.target.value)}
                        onKeyDown={(event) => {
                            if (shouldSubmitChat(event, settings.dbaChatSendOnEnter !== false)) {
                                event.preventDefault();
                                void submit();
                            }
                        }}
                        rows={2}
                        placeholder={t('chat.placeholder', {
                            defaultValue: 'Ask a question or paste a query…',
                        })}
                        aria-label={t('chat.placeholder', {
                            defaultValue: 'Ask a question or paste a query…',
                        })}
                        disabled={sending || !providerReady}
                        className="flex-1 resize-none bg-base border border-default rounded-md px-2 py-1.5 text-xs text-primary placeholder:text-muted focus:outline-none focus:border-focus disabled:opacity-60"
                    />
                    <button
                        type="button"
                        onClick={() => (sending ? void cancel() : void submit())}
                        disabled={!sending && (!input.trim() || !providerReady)}
                        aria-label={
                            sending
                                ? t('chat.stop', {defaultValue: 'Stop'})
                                : t('chat.send', {defaultValue: 'Send'})
                        }
                        title={
                            sending
                                ? t('chat.stop', {defaultValue: 'Stop'})
                                : t('chat.send', {defaultValue: 'Send'})
                        }
                        className="p-2 rounded-md bg-accent-primary text-inverse disabled:opacity-40"
                    >
                        {sending ? <Square size={14} /> : <Send size={14} />}
                    </button>
                </div>
                <p className="mt-1 text-[10px] text-muted">
                    {t(
                        settings.dbaChatSendOnEnter !== false
                            ? 'chat.inputHintEnter'
                            : 'chat.inputHintModifiedEnter',
                    )}{' '}
                    {t('chat.readOnlyHint', {
                        defaultValue: 'Read-only tools. Writes stay in the editor.',
                    })}
                </p>
            </footer>
        </div>
    );
}
