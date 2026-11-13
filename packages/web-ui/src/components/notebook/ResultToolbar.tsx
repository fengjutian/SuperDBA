import {useTranslation} from 'react-i18next';
import type {QueryResult} from '@/types/editor';
import {ResultExportButtons} from '@/components/ui/ResultExportButtons';

interface ResultToolbarProps {
    result: QueryResult;
    executionTime?: number | null;
}

/**
 * Row-count / timing summary plus CSV/JSON export buttons, rendered inside the
 * result section header.
 */
export function ResultToolbar({result, executionTime}: ResultToolbarProps) {
    const {t} = useTranslation();

    return (
        <>
            <span>
                {t('editor.notebook.cellResult', {
                    count: result.rows.length,
                    time: executionTime != null ? Math.round(executionTime) : '—',
                })}
            </span>
            <ResultExportButtons result={result} />
        </>
    );
}
