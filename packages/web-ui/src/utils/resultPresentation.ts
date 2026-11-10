import {returnsResultSet} from '@/utils/sqlSplitter';
import type {QueryResult} from '@/types/editor';

/**
 * What an empty query result actually means.
 *
 * A driver only returns column metadata together with a row, so a `SELECT`
 * that matches nothing arrives with `columns: []` exactly like an
 * `INSERT`/`UPDATE`/DDL statement. The two still need different wording: one
 * "the query matched no rows", the other "the statement ran".
 */
export type EmptyResultKind = 'statement' | 'no-rows';

export interface EmptyResultContext {
    /** SQL that produced `result` (used to tell result-set statements apart). */
    query: string;
    result: QueryResult | null | undefined;
    /** Table tabs stay in data mode so the first row can still be added. */
    tableTab?: boolean;
    /** Pending insertions render an editable row even without result rows. */
    pendingInsertionCount?: number;
}

/**
 * Classifies a result that has no rows worth putting in a grid, so callers can
 * show an explicit empty state instead of a blank grid.
 *
 * Returns `null` when a grid should render (rows, or a table tab being
 * edited).
 */
export function classifyEmptyResult({
    query,
    result,
    tableTab = false,
    pendingInsertionCount = 0,
}: EmptyResultContext): EmptyResultKind | null {
    if (tableTab || !result || pendingInsertionCount > 0) return null;
    if (result.rows.length > 0) return null;
    if (result.columns.length > 0) return 'no-rows';

    // No columns means the driver sent no result set. Statements that write
    // rows are unambiguous; otherwise fall back to the SQL keyword, with the
    // pagination metadata (only set for paginated result-set queries) as a
    // second signal for drivers that do not paginate `SHOW`/`PRAGMA`-style
    // statements.
    if (result.affected_rows > 0) return 'statement';
    return result.pagination || returnsResultSet(query) ? 'no-rows' : 'statement';
}
