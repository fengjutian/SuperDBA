import {useTranslation} from 'react-i18next';
import clsx from 'clsx';
import {CheckCircle2, SearchX} from 'lucide-react';
import {formatDuration} from '@/utils/formatTime';
import type {EmptyResultKind} from '@/utils/resultPresentation';

interface EmptyResultStateProps {
    kind: EmptyResultKind;
    /** Rows written by a statement without a result set. */
    affectedRows?: number;
    /** Shapes the reported result set; omitted when the driver sent no columns. */
    columnCount?: number;
    executionTime?: number | null;
    timingTooltip?: string;
    /** Single-line layout for stacked results and narrow panes. */
    compact?: boolean;
    className?: string;
}

/**
 * Explicit empty state for a query that produced nothing to show in a grid.
 *
 * A successful statement (`INSERT`/`UPDATE`/DDL) and a result-set query that
 * matched no rows are both "no grid", but mean different things, so they read
 * differently instead of sharing one blank panel.
 */
export function EmptyResultState({
    kind,
    affectedRows = 0,
    columnCount,
    executionTime = null,
    timingTooltip,
    compact = false,
    className,
}: EmptyResultStateProps) {
    const {t} = useTranslation();
    const isStatement = kind === 'statement';
    const Icon = isStatement ? CheckCircle2 : SearchX;
    const title = isStatement ? t('editor.queryExecuted') : t('editor.noRowsReturned');

    const meta: string[] = [];
    if (isStatement) {
        if (affectedRows > 0) meta.push(t('editor.rowsAffected', {count: affectedRows}));
    } else {
        meta.push(t('editor.rowsRetrieved', {count: 0}));
        if (columnCount !== undefined && columnCount > 0) {
            meta.push(t('editor.columnsCount', {count: columnCount}));
        }
    }
    if (executionTime !== null) meta.push(formatDuration(executionTime));
    const metaText = meta.join(' · ');

    if (compact) {
        return (
            <div
                role="status"
                className={clsx('flex items-center gap-2 px-3 py-2 text-xs min-w-0', className)}
            >
                <Icon
                    size={14}
                    aria-hidden="true"
                    className={clsx('shrink-0', isStatement ? 'text-accent-success' : 'text-muted')}
                />
                <span className="text-primary truncate">{title}</span>
                {metaText && (
                    <span className="text-muted font-mono shrink-0" title={timingTooltip}>
                        {metaText}
                    </span>
                )}
            </div>
        );
    }

    return (
        <div
            role="status"
            className={clsx(
                'flex-1 min-h-0 flex flex-col items-center justify-center gap-2 px-6 text-center',
                className,
            )}
        >
            <Icon
                size={28}
                aria-hidden="true"
                className={isStatement ? 'text-accent-success' : 'text-muted'}
            />
            <p className="text-sm font-medium text-primary">{title}</p>
            {!isStatement && (
                <p className="text-xs text-secondary max-w-[420px]">
                    {t('editor.noRowsReturnedHint')}
                </p>
            )}
            {metaText && (
                <p className="text-xs text-muted font-mono" title={timingTooltip}>
                    {metaText}
                </p>
            )}
        </div>
    );
}
