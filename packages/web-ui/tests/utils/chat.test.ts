import {describe, expect, it} from 'vitest';
import {
    extractSqlBlock,
    formatQueryResultSummary,
    formatToolResult,
    reduceChatStream,
    shouldSubmitChat,
} from '@/utils/chat';

describe('chat utils', () => {
    describe('extractSqlBlock', () => {
        it('extracts and trims the first sql fence', () => {
            const content = 'Try this:\n\n```sql\nSELECT 1\n```\n\nThen check the plan.';
            expect(extractSqlBlock(content)).toBe('SELECT 1');
        });

        it('is case insensitive on the fence language', () => {
            expect(extractSqlBlock('```SQL\nselect 2\n```')).toBe('select 2');
        });

        it('returns null when there is no sql fence', () => {
            expect(extractSqlBlock('```js\nconsole.log(1)\n```')).toBeNull();
            expect(extractSqlBlock('no code here')).toBeNull();
        });

        it('returns null for empty content', () => {
            expect(extractSqlBlock('')).toBeNull();
        });
    });

    describe('formatToolResult', () => {
        it('pretty prints small results', () => {
            expect(formatToolResult({tables: ['a']})).toBe(
                JSON.stringify({tables: ['a']}, null, 2),
            );
        });

        it('truncates large results and marks the cut', () => {
            const result = formatToolResult({rows: 'x'.repeat(200)}, 40);
            expect(result.endsWith('…')).toBe(true);
            expect(result.length).toBeLessThan(60);
        });

        it('returns an empty string for undefined', () => {
            expect(formatToolResult(undefined)).toBe('');
        });
    });

    describe('formatQueryResultSummary', () => {
        it('includes columns, rows and result metadata', () => {
            const summary = formatQueryResultSummary({
                columns: ['id', 'name'],
                rows: [[1, 'Ada']],
                affected_rows: 0,
            });
            expect(JSON.parse(summary)).toEqual({
                columns: ['id', 'name'],
                rows: [[1, 'Ada']],
                rowCount: 1,
                affectedRows: 0,
            });
        });

        it('limits rows and reports omissions', () => {
            const summary = formatQueryResultSummary(
                {columns: ['id'], rows: [[1], [2], [3]], affected_rows: 0},
                2,
            );
            expect(JSON.parse(summary)).toMatchObject({
                rows: [[1], [2]],
                rowCount: 3,
                omittedRows: 1,
            });
        });

        it('serializes bigint cells', () => {
            const summary = formatQueryResultSummary({
                columns: ['value'],
                rows: [[123n]],
                affected_rows: 0,
            });
            expect(JSON.parse(summary).rows).toEqual([['123']]);
        });

        it('bounds the character count', () => {
            const summary = formatQueryResultSummary(
                {columns: ['value'], rows: [['x'.repeat(100)]], affected_rows: 0},
                20,
                60,
            );
            expect(summary.endsWith('… (truncated)')).toBe(true);
        });
    });

    describe('shouldSubmitChat', () => {
        const keyEvent = (overrides: Partial<KeyboardEvent> = {}) => ({
            key: 'Enter',
            shiftKey: false,
            ctrlKey: false,
            metaKey: false,
            ...overrides,
        });

        it('sends with plain Enter when enabled', () => {
            expect(shouldSubmitChat(keyEvent(), true)).toBe(true);
        });

        it('keeps Shift+Enter for a new line', () => {
            expect(shouldSubmitChat(keyEvent({shiftKey: true}), true)).toBe(false);
        });

        it('requires Ctrl or Command when plain Enter is disabled', () => {
            expect(shouldSubmitChat(keyEvent(), false)).toBe(false);
            expect(shouldSubmitChat(keyEvent({ctrlKey: true}), false)).toBe(true);
            expect(shouldSubmitChat(keyEvent({metaKey: true}), false)).toBe(true);
        });
    });

    describe('reduceChatStream', () => {
        it('starts a draft from a started event', () => {
            const state = reduceChatStream(null, {
                type: 'started',
                requestId: 'r1',
                sessionId: 's1',
            });
            expect(state).toEqual({
                requestId: 'r1',
                text: '',
                tools: [],
                cancelled: false,
                awaitingApproval: null,
            });
        });

        it('appends deltas in order', () => {
            let state = reduceChatStream(null, {
                type: 'started',
                requestId: 'r1',
                sessionId: 's1',
            });
            state = reduceChatStream(state, {
                type: 'delta',
                requestId: 'r1',
                sessionId: 's1',
                text: 'Hello',
            });
            state = reduceChatStream(state, {
                type: 'delta',
                requestId: 'r1',
                sessionId: 's1',
                text: ' world',
            });
            expect(state?.text).toBe('Hello world');
        });

        it('tracks tool lifecycle by call id', () => {
            let state = reduceChatStream(null, {
                type: 'toolStarted',
                requestId: 'r1',
                callId: 'c1',
                name: 'list_tables',
                arguments: {schema: 'public'},
            });
            expect(state?.tools[0]).toMatchObject({
                id: 'c1',
                name: 'list_tables',
                status: 'pending',
            });

            state = reduceChatStream(state, {
                type: 'toolFinished',
                requestId: 'r1',
                callId: 'c1',
                name: 'list_tables',
                status: 'ok',
                durationMs: 12,
                summary: '3 tables',
            });
            expect(state?.tools[0]).toMatchObject({
                status: 'ok',
                durationMs: 12,
                summary: '3 tables',
            });
            expect(state?.tools).toHaveLength(1);
        });

        it('marks the draft cancelled without dropping partial text', () => {
            let state = reduceChatStream(null, {
                type: 'delta',
                requestId: 'r1',
                sessionId: 's1',
                text: 'partial',
            });
            state = reduceChatStream(state, {
                type: 'cancelled',
                requestId: 'r1',
                sessionId: 's1',
            });
            expect(state).toMatchObject({text: 'partial', cancelled: true});
        });

        it('tracks automatic batch progress', () => {
            const state = reduceChatStream(null, {
                type: 'batchProgress',
                requestId: 'r1',
                sessionId: 's1',
                completedBatch: 1,
                totalBatches: 4,
                completedCalls: 16,
                totalCalls: 64,
            });
            expect(state?.batchProgress).toEqual({
                completedBatch: 1,
                totalBatches: 4,
                completedCalls: 16,
                totalCalls: 64,
            });
        });

        it('tracks the approval gate for write statements', () => {
            let state = reduceChatStream(null, {
                type: 'approvalRequired',
                requestId: 'r1',
                approvalId: 'a1',
                query: 'DELETE FROM users WHERE id = 1',
                queryKind: 'DELETE',
            });
            expect(state?.awaitingApproval).toEqual({
                approvalId: 'a1',
                query: 'DELETE FROM users WHERE id = 1',
                queryKind: 'DELETE',
            });

            state = reduceChatStream(state, {
                type: 'approvalResolved',
                requestId: 'r1',
                approvalId: 'a1',
                decision: 'approve',
            });
            expect(state?.awaitingApproval).toBeNull();
        });

        it('ignores finished events without a draft', () => {
            expect(
                reduceChatStream(null, {type: 'finished', requestId: 'r1', sessionId: 's1'}),
            ).toBeNull();
        });
    });
});
