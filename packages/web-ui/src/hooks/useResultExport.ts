import {useCallback, useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import type {ExportStatus} from '@/components/modals/ExportProgressModal';
import {usePlatformCapabilities} from '@/hooks/usePlatformCapabilities';
import {useSuperDBAClient} from '@/hooks/useSuperDBAClient';
import type {QueryResult, QueryResultEntry, Tab} from '@/types/editor';
import type {DriverCapabilities} from '@/types/plugins';
import {isMultiDatabaseCapable} from '@/utils/database';
import {reconstructTableQuery} from '@/utils/editor';
import {buildExportFileName} from '@/utils/exportFileName';
import {downloadGeneratedFile, downloadTextFile} from '@/utils/fileDownloads';
import {
    formatResultForExport,
    getLoadedRowsExportLimit,
    type ResultExportFormat,
} from '@/utils/resultExport';

const LOADED_ROWS_EXPORT_MIME_TYPES: Record<ResultExportFormat, string> = {
    csv: 'text/csv',
    json: 'application/json',
    ndjson: 'application/x-ndjson',
    markdown: 'text/markdown',
    html: 'text/html',
};

export interface ExportState {
    isOpen: boolean;
    status: ExportStatus;
    rowsProcessed: number;
    fileName: string;
    errorMessage?: string;
    warningMessage?: string;
}

interface ResultExportOptions {
    tab: Tab | null;
    connectionId: string | null;
    /** Active schema, used to scope the streamed export to the browsed database. */
    schema: string | null;
    capabilities: DriverCapabilities | null;
    driver: string | null;
    /** The active result entry of a multi-result tab, if any. */
    resultEntry: QueryResultEntry | undefined;
    /** The single result shown for the tab, if any. */
    exportResult: QueryResult | null | undefined;
    /** True when the tab holds an open transaction the export must reuse. */
    inTransaction: boolean;
    csvDelimiter: string;
    /** Called once an export starts, so the caller can close its dropdown. */
    onExportStarted: () => void;
}

/**
 * The three ways a result leaves the app, plus the progress modal that reports
 * them: the streamed server-side exporter (`csv`/`json`/`markdown`), the loaded
 * grid (`html`, and `ndjson` which always writes what is on screen), and the
 * cancel path.
 *
 * The streamed exporter runs the tab's query again on the server; the loaded
 * grid path never does, so it can warn that only the loaded rows are written.
 */
export function useResultExport({
    tab,
    connectionId,
    schema,
    capabilities,
    driver,
    resultEntry,
    exportResult,
    inTransaction,
    csvDelimiter,
    onExportStarted,
}: ResultExportOptions) {
    const client = useSuperDBAClient();
    const platform = usePlatformCapabilities();
    const {t} = useTranslation();

    const [exportState, setExportState] = useState<ExportState>({
        isOpen: false,
        status: 'exporting',
        rowsProcessed: 0,
        fileName: '',
    });

    // The backend reports rows written while the streamed exporter runs.
    useEffect(() => {
        let disposed = false;
        let unsubscribe: (() => void) | undefined;
        void client
            .subscribe('export_progress', ({rows_processed: rowsProcessed}) => {
                setExportState((prev) => ({...prev, rowsProcessed}));
            })
            .then((stop) => {
                if (disposed) stop();
                else unsubscribe = stop;
            });
        return () => {
            disposed = true;
            unsubscribe?.();
        };
    }, [client]);

    const cancelExport = useCallback(async () => {
        if (!connectionId) return;
        try {
            await client.call('cancel_export', {connectionId});
            setExportState((prev) => ({
                ...prev,
                isOpen: false,
            }));
        } catch (e) {
            console.error('Failed to cancel export', e);
        }
    }, [connectionId, client]);

    const closeExportModal = useCallback(() => {
        setExportState((prev) => ({...prev, isOpen: false}));
    }, []);

    const getLoadedRowsWarning = useCallback(
        (result: QueryResult): string | undefined => {
            const limit = getLoadedRowsExportLimit(result);
            if (!limit) return undefined;
            const loaded = limit.loadedRows.toLocaleString();
            return limit.totalRows === null
                ? t('editor.exportLoadedRowsWarningUnknownTotal', {loaded})
                : t('editor.exportLoadedRowsWarning', {
                      loaded,
                      total: limit.totalRows.toLocaleString(),
                  });
        },
        [t],
    );

    const handleExportLoadedRows = useCallback(
        async (result: QueryResult, format: ResultExportFormat) => {
            if (!tab) return;

            const extension = format === 'markdown' ? 'md' : format;
            try {
                const warningMessage = getLoadedRowsWarning(result);

                const fileName = buildExportFileName({
                    tableName: tab.type === 'table' ? tab.activeTable : null,
                    extension,
                });
                setExportState({
                    isOpen: true,
                    status: 'exporting',
                    rowsProcessed: result.rows.length,
                    fileName,
                    errorMessage: undefined,
                    warningMessage,
                });
                onExportStarted();

                const downloaded = await downloadTextFile(platform, {
                    fileName,
                    contents: formatResultForExport(result, format, csvDelimiter),
                    mimeType: LOADED_ROWS_EXPORT_MIME_TYPES[format],
                    filters: [
                        {
                            name: format === 'markdown' ? 'Markdown' : format.toUpperCase(),
                            extensions: [extension],
                        },
                    ],
                });
                if (!downloaded) {
                    setExportState((prev) => ({...prev, isOpen: false}));
                    return;
                }

                setExportState((prev) => ({
                    ...prev,
                    status: 'completed',
                }));
            } catch (e) {
                setExportState((prev) => ({
                    ...prev,
                    status: 'error',
                    errorMessage: String(e),
                }));
            }
        },
        [tab, getLoadedRowsWarning, onExportStarted, platform, csvDelimiter],
    );

    // `ndjson` always writes the loaded rows, never the streaming exporter.
    const handleExportCommon = useCallback(
        async (format: Exclude<ResultExportFormat, 'html'>) => {
            if (!tab || !connectionId) return;

            if (format === 'ndjson') {
                if (!exportResult) return;
                await handleExportLoadedRows(exportResult, 'ndjson');
                return;
            }

            const extension = format === 'markdown' ? 'md' : format;
            const multiResult = resultEntry?.result;
            if (multiResult?.rows.length) {
                await handleExportLoadedRows(multiResult, format);
                return;
            }

            const effectiveSchema = capabilities?.schemas === true ? tab.schema : undefined;
            const tabForQuery = {...tab, schema: effectiveSchema};
            const query =
                tab.type === 'table' && tab.activeTable
                    ? reconstructTableQuery(tabForQuery, capabilities ?? driver ?? undefined)
                    : tab.query;

            if (!query || !query.trim()) return;

            try {
                const fileName = buildExportFileName({
                    tableName: tab.type === 'table' ? tab.activeTable : null,
                    extension,
                });
                const filter = {
                    name: format === 'markdown' ? 'Markdown' : format.toUpperCase(),
                    extensions: [extension],
                };
                const usesNativeSaveTarget =
                    platform.negotiation.environment === 'tauri' &&
                    platform.supports('chooseSaveTarget');
                const target = usesNativeSaveTarget
                    ? await platform.chooseSaveTarget({
                          suggestedName: fileName,
                          filters: [filter],
                      })
                    : null;
                if (usesNativeSaveTarget && !target) return;

                setExportState({
                    isOpen: true,
                    status: 'exporting',
                    rowsProcessed: 0,
                    fileName,
                    errorMessage: undefined,
                    warningMessage: undefined,
                });
                onExportStarted();

                // On multi-database connections (e.g. MySQL) scope the export to the
                // selected database so the query runs against the database the user is
                // viewing rather than the connection's primary database. The tab may not
                // carry its own schema (e.g. a console query), so fall back to the active
                // database — mirroring how execute_query resolves the schema. A
                // schema-based multi-db tab (Postgres) carries its own `database`
                // field instead, since `schema` there is a real Postgres schema, not
                // the database name.
                const targetDatabase = tab.schema ?? schema ?? undefined;
                const databaseParam = tab.database
                    ? {database: tab.database}
                    : isMultiDatabaseCapable(capabilities) && targetDatabase
                      ? {database: targetDatabase}
                      : {};

                const generated = await client.call(
                    'export_query_to_file',
                    {
                        connectionId,
                        query,
                        ...(target ? {filePath: target.reference} : {}),
                        format,
                        csvDelimiter: format === 'csv' ? csvDelimiter : undefined,
                        ...databaseParam,
                        sessionId: inTransaction ? tab.id : undefined,
                    },
                    {deadlineMs: 6 * 60 * 60 * 1000},
                );
                await downloadGeneratedFile(client, platform, generated);

                // Success: update modal state instead of showing toast
                setExportState((prev) => ({
                    ...prev,
                    status: 'completed',
                }));
            } catch (e) {
                // Error: update modal state
                setExportState((prev) => ({
                    ...prev,
                    status: 'error',
                    errorMessage: String(e),
                }));
            }
        },
        [
            tab,
            connectionId,
            exportResult,
            resultEntry?.result,
            capabilities,
            driver,
            schema,
            inTransaction,
            csvDelimiter,
            platform,
            client,
            handleExportLoadedRows,
            onExportStarted,
        ],
    );

    const handleExportCSV = useCallback(() => handleExportCommon('csv'), [handleExportCommon]);
    const handleExportJSON = useCallback(() => handleExportCommon('json'), [handleExportCommon]);
    const handleExportNDJSON = useCallback(
        () => handleExportCommon('ndjson'),
        [handleExportCommon],
    );
    const handleExportMarkdown = useCallback(
        () => handleExportCommon('markdown'),
        [handleExportCommon],
    );

    // Writes the grid already on screen. Never calls the streaming exporter.
    const handleExportHtml = useCallback(async () => {
        if (!connectionId || !exportResult) return;
        await handleExportLoadedRows(exportResult, 'html');
    }, [connectionId, exportResult, handleExportLoadedRows]);

    return {
        exportState,
        cancelExport,
        closeExportModal,
        handleExportCSV,
        handleExportJSON,
        handleExportNDJSON,
        handleExportMarkdown,
        handleExportHtml,
    };
}
