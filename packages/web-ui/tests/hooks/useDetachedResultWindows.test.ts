// @vitest-environment jsdom
import {renderHook, act, waitFor} from '@testing-library/react';
import {describe, it, expect, vi} from 'vitest';
import type {PlatformCapabilities} from '@/platform/capabilities';
import type {Tab} from '@/types/editor';
import {useDetachedResultWindows} from '@/hooks/useDetachedResultWindows';
import {RESULTS_SYNC_EVENT} from '@/utils/resultsWindowSync';

const makeTab = (id: string): Tab => ({
    id,
    title: `Tab ${id}`,
    type: 'console',
    query: 'select 1',
    result: null,
    error: '',
    executionTime: null,
    page: 1,
    activeTable: null,
    pkColumns: null,
    connectionId: 'conn-1',
});

const setup = (tabs: Tab[], activeTab: Tab | null = tabs[0] ?? null) => {
    const openResultsWindow = vi.fn(async () => undefined);
    const closeResultsWindow = vi.fn(async () => undefined);
    const publishRouteEvent = vi.fn(async () => undefined);

    const {result} = renderHook(() =>
        useDetachedResultWindows({
            tabs,
            activeTab,
            connectionId: 'conn-1',
            copyFormat: 'csv',
            csvDelimiter: ',',
            csvIncludeHeaders: true,
            publishRouteEvent:
                publishRouteEvent as unknown as PlatformCapabilities['publishRouteEvent'],
            openResultsWindow,
            closeResultsWindow,
        }),
    );

    return {result, openResultsWindow, closeResultsWindow, publishRouteEvent};
};

describe('useDetachedResultWindows', () => {
    it('detaches the active tab into a new window and marks it', async () => {
        const {result, openResultsWindow} = setup([makeTab('a'), makeTab('b')]);

        await act(async () => {
            await result.current.handleDetachResults();
        });

        expect(openResultsWindow).toHaveBeenCalledTimes(1);
        const [sessionId, title] = openResultsWindow.mock.calls[0] as unknown as [string, string];
        expect(sessionId).toMatch(/^[0-9a-f-]{36}$/);
        expect(title).toBe('Tab a — Query Results');
        expect(result.current.detachedTabIds.has('a')).toBe(true);
        expect(result.current.resultSessionIdsRef.current.get('a')).toBe(sessionId);
        expect(result.current.resultTabIdsBySessionRef.current.get(sessionId)).toBe('a');
    });

    it('pushes the tab state to its window once detached', async () => {
        const {result, publishRouteEvent} = setup([makeTab('a')]);

        await act(async () => {
            await result.current.handleDetachResults();
        });

        await waitFor(() => expect(publishRouteEvent).toHaveBeenCalled());
        const [event, payload] = publishRouteEvent.mock.calls[0] as unknown as [
            string,
            {sessionId: string; payload: {tabId: string; tabTitle: string}},
        ];
        expect(event).toBe(RESULTS_SYNC_EVENT);
        expect(payload.payload.tabId).toBe('a');
        expect(payload.payload.tabTitle).toBe('Tab a');
    });

    it('leaves the tab attached when the window fails to open', async () => {
        const {result, openResultsWindow} = setup([makeTab('a')]);
        openResultsWindow.mockRejectedValueOnce(new Error('nope'));
        const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

        await act(async () => {
            await result.current.handleDetachResults();
        });

        expect(result.current.detachedTabIds.size).toBe(0);
        expect(result.current.resultSessionIdsRef.current.size).toBe(0);
        expect(result.current.resultTabIdsBySessionRef.current.size).toBe(0);
        errorSpy.mockRestore();
    });

    it('reattaches by closing the window and clearing the mapping', async () => {
        const {result, closeResultsWindow} = setup([makeTab('a')]);

        await act(async () => {
            await result.current.handleDetachResults();
        });
        const sessionId = result.current.resultSessionIdsRef.current.get('a');

        await act(async () => {
            await result.current.handleReattachResults('a');
        });

        expect(closeResultsWindow).toHaveBeenCalledWith(sessionId);
        expect(result.current.detachedTabIds.has('a')).toBe(false);
        expect(result.current.resultSessionIdsRef.current.has('a')).toBe(false);
        expect(result.current.resultTabIdsBySessionRef.current.has(sessionId!)).toBe(false);
    });

    it('forgets a tab whose window reported that it closed', async () => {
        const {result, closeResultsWindow} = setup([makeTab('a')]);

        await act(async () => {
            await result.current.handleDetachResults();
        });

        act(() => {
            result.current.forgetDetachedTab('a');
        });

        expect(result.current.detachedTabIds.has('a')).toBe(false);
        expect(closeResultsWindow).not.toHaveBeenCalled();
    });

    it('ignores a detach when there is no active tab', async () => {
        const {result, openResultsWindow} = setup([makeTab('a')], null);

        await act(async () => {
            await result.current.handleDetachResults();
        });

        expect(openResultsWindow).not.toHaveBeenCalled();
        expect(result.current.detachedTabIds.size).toBe(0);
    });
});
