import {useId} from 'react';
import {useTranslation} from 'react-i18next';
import type {DragEvent, MouseEvent, RefObject} from 'react';
import {
    BookOpen,
    ChevronLeft,
    ChevronRight,
    FileCode,
    FolderOpen,
    Network,
    Plus,
    Loader2,
    Table as TableIcon,
    UsersRound,
    X,
} from 'lucide-react';
import clsx from 'clsx';
import type {Tab} from '@/types/editor';
import {canRenameTab, getTabDisplayTitle, getTabFullTitle} from '@/utils/editor';
import {onActivationKey} from '@/utils/keyboardEvents';
import {resolveFlatMultiDbTabDatabase} from '@/utils/database';
import {TONE_SOFT_BG_CLASS, TONE_TEXT_CLASS} from '@/utils/tones';

export interface TabStripDrag {
    /** Id of the tab being dragged, or null when no drag is in progress. */
    tabId: string | null;
    /** Pixel offset of the insertion line, or null to hide it. */
    indicatorLeft: number | null;
    onStart: (e: DragEvent<HTMLDivElement>, tabId: string) => void;
    onEnd: () => void;
    onOver: (index: number) => (e: DragEvent<HTMLDivElement>) => void;
    onContainerOver: (e: DragEvent<HTMLDivElement>) => void;
    onDrop: (e: DragEvent<HTMLDivElement>) => void;
}

export interface TabStripRename {
    editingTabId: string | null;
    title: string;
    onTitleChange: (value: string) => void;
    onStart: (tabId: string) => void;
    onCommit: () => void;
    onCancel: () => void;
}

interface EditorTabStripProps {
    tabs: Tab[];
    activeTabId: string | null;
    /** Tabs that currently hold an open transaction (badge + screen-reader hint). */
    transactionTabIds: ReadonlySet<string>;
    isMultiDb: boolean;
    selectedDatabases: string[];
    scrollContainerRef: RefObject<HTMLDivElement | null>;
    canScrollLeft: boolean;
    canScrollRight: boolean;
    /** Recompute the arrow states from the container's scroll position. */
    onScrollTabs: () => void;
    /** Scroll to the neighbouring tab in `direction`. */
    onStepTabs: (direction: 'left' | 'right') => void;
    drag: TabStripDrag;
    rename: TabStripRename;
    onSelectTab: (tabId: string) => void;
    onCloseTab: (tabId: string) => void;
    onContextMenu: (e: MouseEvent<HTMLDivElement>, tabId: string) => void;
    onNewConsole: () => void;
    onOpenSqlFile: () => void;
    onNewQueryBuilder: () => void;
    onNewNotebook: () => void;
}

const TAB_BAR_ACTION_CLASS =
    'flex items-center justify-center w-9 h-full text-muted hover:text-primary hover:bg-surface-secondary border-l border-default transition-colors shrink-0';

/**
 * The tab strip: scroll arrows, the draggable tab list, and the "new …" actions.
 *
 * The strip owns no state — dragging, scrolling and renaming are driven by the
 * hooks in the editor, which pass them down grouped by concern.
 */
export const EditorTabStrip = ({
    tabs,
    activeTabId,
    transactionTabIds,
    isMultiDb,
    selectedDatabases,
    scrollContainerRef,
    canScrollLeft,
    canScrollRight,
    onScrollTabs,
    onStepTabs,
    drag,
    rename,
    onSelectTab,
    onCloseTab,
    onContextMenu,
    onNewConsole,
    onOpenSqlFile,
    onNewQueryBuilder,
    onNewNotebook,
}: EditorTabStripProps) => {
    const {t} = useTranslation();
    // Split panes mount one strip each; the ids below must stay unique per instance.
    const tabStripId = useId();
    // Full name behind a truncated tab title (tooltip and accessible name).
    const tabFullTitle = (tab: Tab) =>
        getTabFullTitle(
            tab,
            tabs,
            tab.type === 'console' && isMultiDb ? tab.schema || selectedDatabases[0] : undefined,
        );

    return (
        <div className="flex items-center bg-elevated border-b border-default h-9 shrink-0">
            <button
                onClick={() => onStepTabs('left')}
                disabled={!canScrollLeft}
                className="flex items-center justify-center w-7 h-full text-muted border-r border-default shrink-0 transition-colors disabled:opacity-30 disabled:cursor-not-allowed hover:enabled:text-primary hover:enabled:bg-surface-secondary"
            >
                <ChevronLeft size={14} />
            </button>
            <button
                onClick={() => onStepTabs('right')}
                disabled={!canScrollRight}
                className="flex items-center justify-center w-7 h-full text-muted border-r border-default shrink-0 transition-colors disabled:opacity-30 disabled:cursor-not-allowed hover:enabled:text-primary hover:enabled:bg-surface-secondary"
            >
                <ChevronRight size={14} />
            </button>
            <div
                ref={scrollContainerRef}
                onScroll={onScrollTabs}
                onDragOver={drag.onContainerOver}
                onDrop={drag.onDrop}
                className="flex flex-1 overflow-x-auto no-scrollbar h-full relative"
            >
                {/* Always mounted, so a screen reader announces when the active tab opens a transaction. */}
                <span role="status" className="sr-only">
                    {activeTabId && transactionTabIds.has(activeTabId)
                        ? t('editor.transactionOpenHint')
                        : ''}
                </span>
                {tabs.map((tab, index) => (
                    <div
                        key={tab.id}
                        draggable
                        onDragStart={(e) => drag.onStart(e, tab.id)}
                        onDragEnd={drag.onEnd}
                        onDragOver={drag.onOver(index)}
                        role="button"
                        tabIndex={0}
                        title={tabFullTitle(tab)}
                        aria-label={tabFullTitle(tab)}
                        aria-describedby={
                            [
                                tab.sourceFileDirty ? `${tabStripId}-dirty-${tab.id}` : null,
                                transactionTabIds.has(tab.id)
                                    ? `${tabStripId}-transaction-${tab.id}`
                                    : null,
                            ]
                                .filter(Boolean)
                                .join(' ') || undefined
                        }
                        aria-current={activeTabId === tab.id ? 'true' : undefined}
                        onClick={() => onSelectTab(tab.id)}
                        onKeyDown={onActivationKey(() => onSelectTab(tab.id))}
                        onContextMenu={(e) => onContextMenu(e, tab.id)}
                        onAuxClick={(e) => {
                            if (e.button === 1) {
                                e.preventDefault();
                                onCloseTab(tab.id);
                            }
                        }}
                        className={clsx(
                            'flex items-center gap-2 px-3 h-full border-r border-default cursor-pointer min-w-[140px] max-w-[220px] text-xs transition-all duration-150 group relative select-none focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus',
                            activeTabId === tab.id
                                ? 'bg-accent-primary/10 text-primary font-medium'
                                : 'text-muted hover:bg-accent-primary/10 hover:text-secondary',
                            drag.tabId === tab.id && 'opacity-40',
                        )}
                    >
                        {activeTabId === tab.id && (
                            <div className="absolute top-0 left-0 right-0 h-[2px] rounded-b-sm bg-accent-primary shadow-[0_0_5px_color-mix(in_srgb,var(--accent-primary)_35%,transparent)]" />
                        )}
                        {tab.isLoading ? (
                            // Running indicator: the type icon gives way to a spinner so
                            // the tab whose query is executing stands out even when it is
                            // not the active one.
                            <span
                                className="flex shrink-0"
                                title={t('editor.executingQuery')}
                                aria-label={t('editor.executingQuery')}
                                role="status"
                            >
                                <Loader2 size={12} className="animate-spin text-accent" />
                            </span>
                        ) : tab.type === 'table' ? (
                            <TableIcon size={12} className="text-accent shrink-0" />
                        ) : tab.type === 'query_builder' ? (
                            <Network size={12} className="text-accent-secondary shrink-0" />
                        ) : tab.type === 'notebook' ? (
                            <BookOpen size={12} className="text-accent-warning shrink-0" />
                        ) : tab.type === 'users' ? (
                            <UsersRound size={12} className="text-accent-success shrink-0" />
                        ) : (
                            <FileCode size={12} className="text-accent-secondary shrink-0" />
                        )}
                        {rename.editingTabId === tab.id ? (
                            <input
                                autoCorrect="off"
                                autoCapitalize="off"
                                autoComplete="off"
                                spellCheck={false}
                                type="text"
                                draggable={false}
                                value={rename.title}
                                autoFocus
                                onClick={(e) => e.stopPropagation()}
                                onChange={(e) => rename.onTitleChange(e.target.value)}
                                aria-label={t('editor.renameTab')}
                                onBlur={rename.onCommit}
                                onKeyDown={(e) => {
                                    e.stopPropagation();
                                    if (e.key === 'Enter') rename.onCommit();
                                    if (e.key === 'Escape') rename.onCancel();
                                }}
                                className="flex-1 min-w-0 bg-surface-secondary border border-accent-primary/50 rounded px-1 py-0.5 text-xs text-primary focus:outline-none"
                            />
                        ) : (
                            <span
                                className="truncate flex-1 flex items-center gap-1"
                                onDoubleClick={
                                    canRenameTab(tab)
                                        ? (e) => {
                                              e.stopPropagation();
                                              rename.onStart(tab.id);
                                          }
                                        : undefined
                                }
                            >
                                <span className="truncate" title={tab.sourceFilePath}>
                                    {getTabDisplayTitle(tab, tabs)}
                                    {tab.sourceFileDirty && (
                                        // The tab's aria-label replaces its text, so the marker
                                        // reaches screen readers as a description instead.
                                        <>
                                            <span aria-hidden="true"> •</span>
                                            <span
                                                id={`${tabStripId}-dirty-${tab.id}`}
                                                className="sr-only"
                                            >
                                                {t('editor.sqlFileUnsavedChanges')}
                                            </span>
                                        </>
                                    )}
                                </span>
                                {tab.type === 'console' && isMultiDb && (
                                    <span className="text-muted shrink-0">
                                        ({resolveFlatMultiDbTabDatabase(tab, selectedDatabases)})
                                    </span>
                                )}
                                {transactionTabIds.has(tab.id) && (
                                    // This tab is holding a pooled connection open, and its
                                    // uncommitted changes are invisible to every other tab.
                                    <span
                                        id={`${tabStripId}-transaction-${tab.id}`}
                                        className={`shrink-0 px-1 rounded text-[9px] font-semibold uppercase tracking-wide ${TONE_SOFT_BG_CLASS.warning} ${TONE_TEXT_CLASS.warning}`}
                                        title={t('editor.transactionOpenHint')}
                                        aria-label={t('editor.transactionOpenHint')}
                                    >
                                        {t('editor.transactionOpen')}
                                    </span>
                                )}
                            </span>
                        )}
                        <button
                            draggable={false}
                            onClick={(e) => {
                                e.stopPropagation();
                                onCloseTab(tab.id);
                            }}
                            aria-label={t('editor.closeTab')}
                            className={clsx(
                                'p-0.5 rounded hover:bg-surface-secondary hover:text-primary hover:scale-110 transition-all duration-150 shrink-0',
                                activeTabId === tab.id
                                    ? 'opacity-100'
                                    : 'opacity-0 group-hover:opacity-100',
                            )}
                        >
                            <X size={12} />
                        </button>
                        {tab.isLoading && (
                            <div
                                className="absolute bottom-0 left-0 h-0.5 w-full animate-pulse"
                                style={{
                                    backgroundImage:
                                        'linear-gradient(90deg, transparent, var(--accent-primary), transparent)',
                                }}
                            />
                        )}
                    </div>
                ))}
                {drag.tabId && drag.indicatorLeft !== null && (
                    <div
                        className="absolute top-0 bottom-0 w-0.5 pointer-events-none z-10 bg-accent-primary"
                        style={{left: drag.indicatorLeft}}
                    />
                )}
            </div>
            <button
                onClick={onNewConsole}
                className={TAB_BAR_ACTION_CLASS}
                title={t('editor.newConsole')}
            >
                <Plus size={16} />
            </button>
            <button
                onClick={onOpenSqlFile}
                className="flex items-center justify-center w-9 h-full text-accent-info hover:text-primary hover:bg-surface-secondary border-l border-default transition-colors shrink-0"
                title={t('editor.openSqlFile')}
            >
                <FolderOpen size={16} />
            </button>
            <button
                onClick={onNewQueryBuilder}
                className="flex items-center justify-center w-9 h-full text-accent-secondary hover:text-primary hover:bg-surface-secondary border-l border-default transition-colors shrink-0"
                title={t('editor.newVisualQuery')}
            >
                <Network size={16} />
            </button>
            <button
                onClick={onNewNotebook}
                className="flex items-center justify-center w-9 h-full text-accent-warning hover:text-primary hover:bg-surface-secondary border-l border-default transition-colors shrink-0"
                title={t('editor.newNotebook')}
            >
                <BookOpen size={16} />
            </button>
        </div>
    );
};
