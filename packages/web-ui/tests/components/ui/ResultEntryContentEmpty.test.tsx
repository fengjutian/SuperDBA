import {describe, it, expect, vi} from 'vitest';
import {render, screen} from '@testing-library/react';
import {ResultEntryContent} from '@/components/ui/ResultEntryContent';
import {StackedResultItem} from '@/components/ui/StackedResultItem';
import type {QueryResult, QueryResultEntry} from '@/types/editor';

vi.mock('@/components/ui/DataGrid', () => ({
    DataGrid: () => <div data-testid="data-grid" />,
}));

vi.mock('@/components/ui/ErrorDisplay', () => ({
    ErrorDisplay: ({error}: {error: string}) => <div>{error}</div>,
}));

function makeEntry(query: string, result: QueryResult): QueryResultEntry {
    return {
        id: 'tab1-result-0',
        queryIndex: 0,
        query,
        result,
        error: '',
        executionTime: 44,
        isLoading: false,
        page: 1,
        activeTable: null,
        pkColumns: null,
    };
}

const gridProps = {
    connectionId: 'c1',
    copyFormat: 'csv' as const,
    csvDelimiter: ',',
    csvIncludeHeaders: true,
    onPageChange: vi.fn(),
};

describe('ResultEntryContent empty results', () => {
    it('says so when a SELECT matched no rows instead of rendering a blank grid', () => {
        render(
            <ResultEntryContent
                {...gridProps}
                entry={makeEntry('SELECT id, name FROM users WHERE id = 1', {
                    columns: ['id', 'name'],
                    rows: [],
                    affected_rows: 0,
                })}
            />,
        );

        expect(screen.getByText('editor.noRowsReturned')).toBeInTheDocument();
        expect(screen.queryByTestId('data-grid')).not.toBeInTheDocument();
        // The empty state still reports the shape of the empty result set.
        expect(screen.getByText(/editor\.rowsRetrieved/)).toBeInTheDocument();
        expect(screen.getByText(/editor\.columnsCount/)).toBeInTheDocument();
    });

    it('reports an executed statement without a result set', () => {
        render(
            <ResultEntryContent
                {...gridProps}
                entry={makeEntry('UPDATE users SET name = $1', {
                    columns: [],
                    rows: [],
                    affected_rows: 2,
                })}
            />,
        );

        expect(screen.getByText('editor.queryExecuted')).toBeInTheDocument();
        expect(screen.getByText(/editor\.rowsAffected/)).toBeInTheDocument();
        expect(screen.queryByTestId('data-grid')).not.toBeInTheDocument();
    });

    it('renders the empty result as one status line inside a stacked result', () => {
        render(
            <StackedResultItem
                {...gridProps}
                entry={makeEntry('SELECT id FROM users WHERE id = 1', {
                    columns: ['id'],
                    rows: [],
                    affected_rows: 0,
                })}
                collapsed={false}
                aiEnabled={false}
                aiRenaming={false}
                onToggleCollapse={vi.fn()}
                onCopyAllRows={vi.fn()}
                onRename={vi.fn()}
                onRerun={vi.fn()}
                onAiRename={vi.fn()}
                onClose={vi.fn()}
            />,
        );

        expect(screen.getByText('editor.noRowsReturned')).toBeInTheDocument();
        expect(screen.queryByText('editor.noRowsReturnedHint')).not.toBeInTheDocument();
        expect(screen.queryByTestId('data-grid')).not.toBeInTheDocument();
    });
});
