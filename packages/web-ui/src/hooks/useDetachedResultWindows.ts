import {useCallback, useEffect, useRef, useState} from 'react';
import type {PlatformCapabilities} from '@/platform/capabilities';
import type {ResultsSessionSnapshot} from '@/platform/secondaryWindowSessions';
import type {Tab} from '@/types/editor';
import {buildSyncPayload, RESULTS_SYNC_EVENT} from '@/utils/resultsWindowSync';

type SyncOptions = Parameters<typeof buildSyncPayload>[1];

interface DetachedResultWindowsOptions {
    tabs: Tab[];
    activeTab: Tab | null;
    connectionId: SyncOptions['connectionId'];
    copyFormat: SyncOptions['copyFormat'];
    csvDelimiter: SyncOptions['csvDelimiter'];
    csvIncludeHeaders: SyncOptions['csvIncludeHeaders'];
    publishRouteEvent: PlatformCapabilities['publishRouteEvent'];
    openResultsWindow: (sessionId: string, title: string) => Promise<unknown>;
    closeResultsWindow: (sessionId: string) => Promise<unknown>;
}

/**
 * Owns the main window's side of the "detach results into their own window"
 * feature: which tabs are detached, the tab↔session mapping the event bridge
 * needs, and pushing each detached tab's result state to its window.
 *
 * The window-side handshake lives with the editor's event bridge; this hook
 * only exposes the two maps it must read (`resultTabIdsBySessionRef`,
 * `resultSessionIdsRef`) and the ids it must prune.
 */
export function useDetachedResultWindows({
    tabs,
    activeTab,
    connectionId,
    copyFormat,
    csvDelimiter,
    csvIncludeHeaders,
    publishRouteEvent,
    openResultsWindow,
    closeResultsWindow,
}: DetachedResultWindowsOptions) {
    // Ids of tabs whose results are detached into their own separate windows
    // (one window per tab). Each window keeps showing its tab even when the
    // user switches tabs in the main window.
    const [detachedTabIds, setDetachedTabIds] = useState<Set<string>>(() => new Set());
    // Mirror for use inside callbacks/refs without re-creating them or reading
    // stale closures.
    const detachedTabIdsRef = useRef(detachedTabIds);
    const resultSessionIdsRef = useRef(new Map<string, string>());
    const resultTabIdsBySessionRef = useRef(new Map<string, string>());

    const handleDetachResults = useCallback(async () => {
        if (!activeTab) return;
        const tabId = activeTab.id;
        const sessionId = crypto.randomUUID();
        resultSessionIdsRef.current.set(tabId, sessionId);
        resultTabIdsBySessionRef.current.set(sessionId, tabId);
        try {
            await openResultsWindow(sessionId, `${activeTab.title} — Query Results`);
            setDetachedTabIds((prev) => new Set(prev).add(tabId));
        } catch (e) {
            resultSessionIdsRef.current.delete(tabId);
            resultTabIdsBySessionRef.current.delete(sessionId);
            console.error('Failed to detach results', e);
        }
    }, [activeTab, openResultsWindow]);

    const handleReattachResults = useCallback(
        async (tabId: string) => {
            const sessionId = resultSessionIdsRef.current.get(tabId) ?? tabId;
            try {
                await closeResultsWindow(sessionId);
            } catch (e) {
                console.error('Failed to close results window', e);
            }
            resultSessionIdsRef.current.delete(tabId);
            resultTabIdsBySessionRef.current.delete(sessionId);
            setDetachedTabIds((prev) => {
                const next = new Set(prev);
                next.delete(tabId);
                return next;
            });
        },
        [closeResultsWindow],
    );

    // Push each detached tab's result state to its window whenever the tabs
    // change.
    useEffect(() => {
        if (detachedTabIds.size === 0) return;
        for (const id of detachedTabIds) {
            const tab = tabs.find((t) => t.id === id);
            const sessionId = resultSessionIdsRef.current.get(id);
            if (tab && sessionId) {
                void publishRouteEvent<ResultsSessionSnapshot>(RESULTS_SYNC_EVENT, {
                    sessionId,
                    payload: buildSyncPayload(tab, {
                        connectionId,
                        copyFormat,
                        csvDelimiter,
                        csvIncludeHeaders,
                    }),
                });
            }
        }
    }, [
        tabs,
        detachedTabIds,
        connectionId,
        copyFormat,
        csvDelimiter,
        csvIncludeHeaders,
        publishRouteEvent,
    ]);

    // If a detached tab is closed in the main window, close its orphaned
    // window. Closing the window emits RESULTS_CLOSED_EVENT, whose listener
    // owns pruning `detachedTabIds` — so this effect stays side-effect-only.
    useEffect(() => {
        for (const id of detachedTabIds) {
            if (!tabs.some((t) => t.id === id)) {
                const sessionId = resultSessionIdsRef.current.get(id) ?? id;
                void closeResultsWindow(sessionId).catch(() => {});
            }
        }
    }, [closeResultsWindow, tabs, detachedTabIds]);

    // Called by the event bridge once a results window reports that it closed.
    const forgetDetachedTab = useCallback((tabId: string) => {
        setDetachedTabIds((prev) => {
            if (!prev.has(tabId)) return prev;
            const next = new Set(prev);
            next.delete(tabId);
            return next;
        });
    }, []);

    return {
        detachedTabIds,
        detachedTabIdsRef,
        resultSessionIdsRef,
        resultTabIdsBySessionRef,
        handleDetachResults,
        handleReattachResults,
        forgetDetachedTab,
    };
}
