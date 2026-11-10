import {describe, it, expect} from 'vitest';
import {render, screen} from '@testing-library/react';
import {EmptyResultState} from '@/components/ui/EmptyResultState';

describe('EmptyResultState', () => {
    it('explains that a result-set query matched no rows', () => {
        render(<EmptyResultState kind="no-rows" columnCount={2} executionTime={44} />);

        expect(screen.getByRole('status')).toBeInTheDocument();
        expect(screen.getByText('editor.noRowsReturned')).toBeInTheDocument();
        expect(screen.getByText('editor.noRowsReturnedHint')).toBeInTheDocument();
        // Shape of the empty result and the measured duration stay visible.
        expect(screen.getByText(/editor\.rowsRetrieved/)).toBeInTheDocument();
        expect(screen.getByText(/editor\.columnsCount/)).toBeInTheDocument();
        expect(screen.getByText(/44 ms/)).toBeInTheDocument();
    });

    it('reports an executed statement and its affected rows', () => {
        render(<EmptyResultState kind="statement" affectedRows={3} executionTime={12} />);

        expect(screen.getByText('editor.queryExecuted')).toBeInTheDocument();
        expect(screen.getByText(/editor\.rowsAffected/)).toBeInTheDocument();
        expect(screen.queryByText('editor.noRowsReturnedHint')).not.toBeInTheDocument();
    });

    it('omits the affected-row count when a statement changed nothing', () => {
        render(<EmptyResultState kind="statement" affectedRows={0} />);

        expect(screen.getByText('editor.queryExecuted')).toBeInTheDocument();
        expect(screen.queryByText(/editor\.rowsAffected/)).not.toBeInTheDocument();
    });

    it('collapses to a single status row in compact panes', () => {
        render(<EmptyResultState kind="no-rows" executionTime={8} compact />);

        expect(screen.getByRole('status')).toBeInTheDocument();
        expect(screen.getByText('editor.noRowsReturned')).toBeInTheDocument();
        expect(screen.queryByText('editor.noRowsReturnedHint')).not.toBeInTheDocument();
        expect(screen.getByText(/8 ms/)).toBeInTheDocument();
    });
});
