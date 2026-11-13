import {useTranslation} from 'react-i18next';
import {Download} from 'lucide-react';
import type {QueryResult} from '@/types/editor';
import {formatResultForExport} from '@/utils/resultExport';
import {usePlatformCapabilities} from '@/hooks/usePlatformCapabilities';
import {downloadTextFile} from '@/utils/fileDownloads';
import {useAlert} from '@/hooks/useAlert';
import {useSettings} from '@/hooks/useSettings';

interface ResultExportButtonsProps {
    result: QueryResult;
}

export function ResultExportButtons({result}: ResultExportButtonsProps) {
    const {t} = useTranslation();
    const platform = usePlatformCapabilities();
    const {showAlert} = useAlert();
    const {settings} = useSettings();

    const handleExport = async (format: 'csv' | 'json') => {
        try {
            const downloaded = await downloadTextFile(platform, {
                fileName: `result.${format}`,
                contents: formatResultForExport(result, format, settings.csvDelimiter ?? ','),
                mimeType: format === 'json' ? 'application/json' : 'text/csv',
                filters: [{name: format.toUpperCase(), extensions: [format]}],
            });
            if (!downloaded) return;
            showAlert(t('editor.notebook.resultExportSuccess'), {kind: 'info'});
        } catch (error) {
            console.error(`${format.toUpperCase()} export failed:`, error);
            showAlert(t('editor.notebook.exportError'), {kind: 'error'});
        }
    };

    return (
        <div className="flex items-center gap-0.5 shrink-0">
            {(['csv', 'json'] as const).map((format) => {
                const label =
                    format === 'csv'
                        ? t('editor.notebook.exportCsv')
                        : t('editor.notebook.exportJson');
                return (
                    <button
                        key={format}
                        type="button"
                        onClick={() => handleExport(format)}
                        className="p-1 text-muted hover:text-secondary hover:bg-surface-secondary rounded transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                        title={label}
                        aria-label={format.toUpperCase()}
                    >
                        <span className="flex items-center gap-0.5">
                            <Download size={12} />
                            <span className="text-[9px]">{format.toUpperCase()}</span>
                        </span>
                    </button>
                );
            })}
        </div>
    );
}
