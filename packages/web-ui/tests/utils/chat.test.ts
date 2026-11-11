import {describe, expect, it} from 'vitest';
import {
    extractSqlBlock,
    formatQueryResultSummary,
    formatToolResult,
    reduceChatStream,
    classifyChatError,
    summarizeChatTools,
    isDuplicateLiveChatFailure,
    buildChatEvidenceIndex,
    linkChatEvidenceReferences,
    buildChatEvidenceStatuses,
    validateChatEvidenceReferences,
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

        it('masks sensitive columns before building model context', () => {
            const summary = formatQueryResultSummary(
                {
                    columns: ['id', 'email', 'access_token'],
                    rows: [[1, 'ada@example.com', 'secret-value']],
                    affected_rows: 0,
                },
                20,
                2000,
                {enabled: true},
            );
            expect(JSON.parse(summary).rows).toEqual([[1, '••••••', '••••••']]);
            expect(summary).not.toContain('ada@example.com');
            expect(summary).not.toContain('secret-value');
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

        it('marks the draft failed without dropping partial evidence', () => {
            let state = reduceChatStream(null, {
                type: 'delta',
                requestId: 'r1',
                sessionId: 's1',
                text: 'partial',
            });
            state = reduceChatStream(state, {
                type: 'failed',
                requestId: 'r1',
                sessionId: 's1',
                message: '401 Unauthorized',
            });
            expect(state).toMatchObject({text: 'partial', failed: '401 Unauthorized'});
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

    describe('classifyChatError', () => {
        it('recognizes authentication failures from provider responses', () => {
            expect(classifyChatError('401 Unauthorized: invalid api key')).toBe('authentication');
        });

        it('recognizes rate limits and network failures', () => {
            expect(classifyChatError('429 rate limit exceeded')).toBe('rateLimit');
            expect(classifyChatError('error sending request for url')).toBe('network');
        });

        it('keeps unknown failures generic', () => {
            expect(classifyChatError('provider returned an unexpected response')).toBe('unknown');
        });
    });

    describe('tool activity summaries', () => {
        it('treats a resumable query budget checkpoint as completed work', () => {
            const summary =
                'Stopped after 12 queries in one turn to protect database stability. Ask a follow-up to continue.';
            expect(
                summarizeChatTools([
                    {
                        id: '1',
                        name: 'run_query',
                        arguments: {},
                        status: 'limitReached',
                        durationMs: 0,
                        summary,
                    },
                ]),
            ).toEqual({total: 1, completed: 1, pending: 0, issues: 0, durationMs: 0});
        });

        it('counts active, completed, and failed calls separately', () => {
            expect(
                summarizeChatTools([
                    {id: '1', name: 'a', arguments: {}, status: 'ok', durationMs: 10, summary: ''},
                    {
                        id: '2',
                        name: 'b',
                        arguments: {},
                        status: 'pending',
                        durationMs: 0,
                        summary: '',
                    },
                    {
                        id: '3',
                        name: 'c',
                        arguments: {},
                        status: 'error',
                        durationMs: 5,
                        summary: '',
                    },
                ]),
            ).toEqual({total: 3, completed: 1, pending: 1, issues: 1, durationMs: 15});
        });
    });

    describe('live failure deduplication', () => {
        const failedMessage = {
            id: 'message-1',
            role: 'assistant' as const,
            content: '',
            createdAt: '2026-11-11T00:00:00Z',
            toolCalls: [],
            notice: '401 Unauthorized',
        };

        it('hides an empty persisted failure while the matching error card is visible', () => {
            expect(isDuplicateLiveChatFailure(failedMessage, '401 Unauthorized')).toBe(true);
        });

        it('keeps historical notices and assistant content visible', () => {
            expect(isDuplicateLiveChatFailure(failedMessage, null)).toBe(false);
            expect(
                isDuplicateLiveChatFailure(
                    {...failedMessage, content: 'Partial answer'},
                    '401 Unauthorized',
                ),
            ).toBe(false);
        });
    });

    describe('evidence indexing', () => {
        it('assigns stable positions across message tool batches', () => {
            const messages = [
                {
                    id: 'm1',
                    role: 'assistant' as const,
                    content: '',
                    createdAt: '2026-11-11T00:00:00Z',
                    toolCalls: [
                        {
                            id: 'c1',
                            name: 'a',
                            arguments: {},
                            status: 'ok' as const,
                            durationMs: 1,
                            summary: '',
                        },
                        {
                            id: 'c2',
                            name: 'b',
                            arguments: {},
                            status: 'ok' as const,
                            durationMs: 1,
                            summary: '',
                        },
                    ],
                },
                {
                    id: 'm2',
                    role: 'assistant' as const,
                    content: '',
                    createdAt: '2026-11-11T00:00:01Z',
                    toolCalls: [
                        {
                            id: 'c3',
                            name: 'c',
                            arguments: {},
                            status: 'ok' as const,
                            durationMs: 1,
                            summary: '',
                        },
                    ],
                },
            ];

            expect(buildChatEvidenceIndex(messages)).toEqual({
                starts: {m1: 1, m2: 3},
                next: 4,
            });
        });

        it('links only available evidence references outside code', () => {
            expect(
                linkChatEvidenceReferences(
                    'Use [E1], not [E3]. `literal [E2]`\n```sql\nSELECT \'[E2]\'\n```',
                    2,
                ),
            ).toBe(
                'Use [E1](#chat-evidence-E1), not [E3]. `literal [E2]`\n```sql\nSELECT \'[E2]\'\n```',
            );
        });

        it('does not rewrite an existing evidence link', () => {
            expect(linkChatEvidenceReferences('[E1](https://example.com)', 1)).toBe(
                '[E1](https://example.com)',
            );
        });

        it('indexes persisted and live evidence statuses in display order', () => {
            const messages = [
                {
                    id: 'm1',
                    role: 'assistant' as const,
                    content: '',
                    createdAt: '2026-11-11T00:00:00Z',
                    toolCalls: [
                        {
                            id: 'c1',
                            name: 'a',
                            arguments: {},
                            status: 'ok' as const,
                            durationMs: 1,
                            summary: '',
                        },
                    ],
                },
            ];
            const live = [
                {
                    id: 'c2',
                    name: 'b',
                    arguments: {},
                    status: 'pending' as const,
                    durationMs: 0,
                    summary: '',
                },
            ];

            expect(buildChatEvidenceStatuses(messages, live)).toEqual({E1: 'ok', E2: 'pending'});
        });

        it('reports missing, pending, and unsuccessful citations but ignores code', () => {
            expect(
                validateChatEvidenceReferences(
                    'Valid [E1], waiting [E2], failed [E3], missing [E9], `code [E8]`.',
                    {E1: 'ok', E2: 'pending', E3: 'error'},
                ),
            ).toEqual([
                {evidenceId: 'E2', kind: 'pending'},
                {evidenceId: 'E3', kind: 'unreliable'},
                {evidenceId: 'E9', kind: 'missing'},
            ]);
        });

        it('does not let later evidence validate an earlier answer', () => {
            expect(validateChatEvidenceReferences('Premature [E2].', {E1: 'ok', E2: 'ok'}, 1)).toEqual(
                [{evidenceId: 'E2', kind: 'missing'}],
            );
        });
    });
});
