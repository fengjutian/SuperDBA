import {describe, expect, it} from 'vitest';
import type {QueryResult, Tab} from '@/types/editor';
import {classifyEmptyResult} from '@/utils/resultPresentation';

function result(overrides: Partial<QueryResult> = {}): QueryResult {
    return {
        columns: [],
        rows: [],
        affected_rows: 0,
        ...overrides,
    };
}

function classify(overrides: Partial<Tab> = {}) {
    const tab: Tab = {
        id: 'tab-1',
        title: 'Result',
        type: 'console',
        query: '',
        result: result(),
        error: '',
        executionTime: null,
        page: 1,
        activeTable: null,
        pkColumns: null,
        connectionId: 'connection-1',
        ...overrides,
    };
    return classifyEmptyResult({
        query: tab.query,
        result: tab.result,
        tableTab: tab.type === 'table',
        pendingInsertionCount: Object.keys(tab.pendingInsertions ?? {}).length,
    });
}

describe('resultPresentation', () => {
    describe('classifyEmptyResult', () => {
        it('treats a command without a result set as an executed statement', () => {
            expect(
                classify({query: 'UPDATE users SET name = $1', result: result({affected_rows: 3})}),
            ).toBe('statement');
        });

        it('treats a DDL statement with no affected rows as an executed statement', () => {
            expect(classify({query: 'CREATE TABLE t (id integer)'})).toBe('statement');
        });

        it('treats a result-set query that matched nothing as no rows', () => {
            expect(classify({query: 'SELECT * FROM quote_detail WHERE id = 1'})).toBe('no-rows');
        });

        it('reports no rows when the driver returns columns but no rows', () => {
            expect(
                classify({
                    query: 'SELECT id, name FROM users',
                    result: result({columns: ['id', 'name']}),
                }),
            ).toBe('no-rows');
        });

        it('classifies a CTE query as a result set', () => {
            expect(classify({query: 'WITH recent AS (SELECT 1) SELECT * FROM recent'})).toBe(
                'no-rows',
            );
        });

        it('prefers the affected-row count over the leading keyword', () => {
            expect(
                classify({
                    query: 'WITH moved AS (DELETE FROM t RETURNING *) SELECT * FROM moved',
                    result: result({affected_rows: 2}),
                }),
            ).toBe('statement');
        });

        it('uses pagination metadata when the keyword alone is ambiguous', () => {
            expect(
                classify({
                    query: 'BEGIN',
                    result: result({
                        pagination: {page: 1, page_size: 100, total_rows: 0, has_more: false},
                    }),
                }),
            ).toBe('no-rows');
        });

        it('keeps an empty table tab in data mode so the first row can be added', () => {
            expect(classify({type: 'table', activeTable: 'users'})).toBeNull();
        });

        it('keeps rendering pending insertions', () => {
            expect(
                classify({
                    pendingInsertions: {
                        temporary: {tempId: 'temporary', data: {}, displayIndex: 0},
                    },
                }),
            ).toBeNull();
        });

        it('renders a grid when the result has rows', () => {
            expect(
                classify({
                    query: 'SELECT id FROM users',
                    result: result({columns: ['id'], rows: [[1]]}),
                }),
            ).toBeNull();
        });

        it('does not classify before a result exists', () => {
            expect(classify({result: null})).toBeNull();
            expect(classify({query: 'SELECT 1', result: undefined})).toBeNull();
        });
    });
});
