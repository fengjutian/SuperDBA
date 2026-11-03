// @vitest-environment jsdom
import {renderHook, act, waitFor} from '@testing-library/react';
import {describe, it, expect, vi, beforeEach} from 'vitest';
import type {Tab} from '@/types/editor';
import {useResultExport} from '@/hooks/useResultExport';

const mocks = vi.hoisted(() => ({
    call: vi.fn(),
    subscribe: vi.fn(async () => () => {}),
    downloadTextFile: vi.fn(async () => true),
    downloadGeneratedFile: vi.fn(async () => undefined),
    supports: vi.fn(() => true),
    chooseSaveTarget: vi.fn(async () => ({reference: '/tmp/out.csv'}) as const),
}));

vi.mock('@/hooks/useSuperDBAClient', () => ({
    useSuperDBAClient: () => ({call: mocks.call, subscribe: mocks.subscribe}),
}));

vi.mock('@/hooks/usePlatformCapabilities', () => ({
    usePlatformCapabilities: () => ({
        negotiation: {environment: 'tauri'},
        supports: mocks.supports,
        chooseSaveTarget: mocks.chooseSaveTarget,
    }),
}));

vi.mock('@/utils/fileDownloads', () => ({
    downloadTextFile: mocks.downloadTextFile,
    downloadGeneratedFile: mocks.downloadGeneratedFile,
}));

const makeTab = (overrides: Partial<Tab> = {}): Tab => ({
    id: 'tab-a',
    title: 'Tab A',
    type: 'console',
    query: 'select * from users',
    result: null,
    error: '',
    executionTime: null,
    page: 1,
    activeTable: null,
    pkColumns: null,
    connectionId: 'conn-1',
    ...overrides,
});

const setup = (
    tab: Tab = makeTab(),
    options: Partial<Parameters<typeof useResultExport>[0]> = {},
) => {
    const onExportStarted = vi.fn();
    const {result} = renderHook(() =>
        useResultExport({
            tab,
            connectionId: 'conn-1',
            schema: null,
            capabilities: null,
            driver: null,
            resultEntry: undefined,
            exportResult: null,
            inTransaction: false,
            csvDelimiter: ',',
            onExportStarted,
            ...options,
        }),
    );
    return {result, onExportStarted};
};

describe('useResultExport', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mocks.call.mockResolvedValue({});
        mocks.downloadTextFile.mockResolvedValue(true);
    });

    it('streams the tab query to the server-side exporter', async () => {
        const {result, onExportStarted} = setup();

        await act(async () => {
            await result.current.handleExportCSV();
        });

        expect(mocks.call).toHaveBeenCalledWith(
            'export_query_to_file',
            expect.objectContaining({
                connectionId: 'conn-1',
                query: 'select * from users',
                format: 'csv',
                csvDelimiter: ',',
                filePath: '/tmp/out.csv',
            }),
            expect.anything(),
        );
        expect(onExportStarted).toHaveBeenCalled();
        expect(result.current.exportState.status).toBe('completed');
        expect(result.current.exportState.isOpen).toBe(true);
    });

    it('only passes the tab id as session when it holds a transaction', async () => {
        const {result} = setup(makeTab(), {inTransaction: true});

        await act(async () => {
            await result.current.handleExportJSON();
        });

        expect(mocks.call).toHaveBeenCalledWith(
            'export_query_to_file',
            expect.objectContaining({sessionId: 'tab-a'}),
            expect.anything(),
        );
    });

    it('writes the loaded rows instead of re-running the query', async () => {
        const resultPayload = {
            columns: ['id'],
            rows: [[1], [2]],
            affected_rows: 0,
        };
        const {result} = setup(makeTab(), {
            resultEntry: {
                id: 'entry-1',
                queryIndex: 0,
                query: 'select 1',
                result: resultPayload,
                error: '',
                executionTime: null,
            } as Parameters<typeof useResultExport>[0]['resultEntry'],
        });

        await act(async () => {
            await result.current.handleExportCSV();
        });

        expect(mocks.call).not.toHaveBeenCalledWith(
            'export_query_to_file',
            expect.anything(),
            expect.anything(),
        );
        expect(mocks.downloadTextFile).toHaveBeenCalledWith(
            expect.anything(),
            expect.objectContaining({fileName: expect.stringContaining('.csv')}),
        );
        expect(result.current.exportState.status).toBe('completed');
    });

    it('reports a failed export in the modal state', async () => {
        const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
        mocks.call.mockRejectedValueOnce(new Error('disk full'));
        const {result} = setup();

        await act(async () => {
            await result.current.handleExportMarkdown();
        });

        expect(result.current.exportState.status).toBe('error');
        expect(result.current.exportState.errorMessage).toContain('disk full');
        errorSpy.mockRestore();
    });

    it('cancels a running export and closes the modal', async () => {
        const {result} = setup();

        await act(async () => {
            await result.current.handleExportCSV();
        });
        await act(async () => {
            await result.current.cancelExport();
        });

        expect(mocks.call).toHaveBeenCalledWith('cancel_export', {connectionId: 'conn-1'});
        expect(result.current.exportState.isOpen).toBe(false);
    });

    it('closes the modal without cancelling', async () => {
        const {result} = setup();

        await act(async () => {
            await result.current.handleExportCSV();
        });
        act(() => {
            result.current.closeExportModal();
        });

        expect(result.current.exportState.isOpen).toBe(false);
        expect(mocks.call).not.toHaveBeenCalledWith('cancel_export', expect.anything());
    });

    it('ignores an export when there is nothing to export', async () => {
        const {result} = setup(makeTab({query: '   '}));

        await act(async () => {
            await result.current.handleExportCSV();
        });

        expect(mocks.call).not.toHaveBeenCalled();
        expect(result.current.exportState.isOpen).toBe(false);
    });

    it('subscribes to backend progress on mount', async () => {
        setup();
        await waitFor(() => expect(mocks.subscribe).toHaveBeenCalled());
        expect(mocks.subscribe.mock.calls[0][0]).toBe('export_progress');
    });
});
