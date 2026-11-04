/** Pure helpers for the DBA chat panel. */

import type {ChatEvent, ChatToolCall} from '@/types/chat';
import type {QueryResult} from '@/types/editor';

const SQL_FENCE = /```sql\s*([\s\S]*?)```/i;

/**
 * Returns the first fenced `sql` block in an assistant message, trimmed.
 * Returns `null` when the message has no SQL block.
 */
export function extractSqlBlock(content: string): string | null {
    const match = content.match(SQL_FENCE);
    return match ? match[1].trim() : null;
}

/**
 * Renders a tool result for the details panel without dumping megabytes into
 * the DOM. The suffix marks that the text was cut.
 */
export function formatToolResult(value: unknown, limit = 1600): string {
    if (value === undefined) return '';
    const text = JSON.stringify(value, null, 2) ?? '';
    if (text.length <= limit) return text;
    return `${text.slice(0, limit)}\n…`;
}

/**
 * Produces a bounded JSON preview for the model without forwarding an entire
 * result set. BigInt cells are stringified because JSON does not support them.
 */
export function formatQueryResultSummary(
    result: QueryResult,
    rowLimit = 20,
    charLimit = 2000,
): string {
    const rows = result.rows.slice(0, rowLimit);
    const omittedRows = Math.max(0, result.rows.length - rows.length);
    const text = JSON.stringify(
        {
            columns: result.columns,
            rows,
            rowCount: result.rows.length,
            affectedRows: result.affected_rows,
            ...(omittedRows > 0 ? {omittedRows} : {}),
            ...(result.truncated ? {sourceTruncated: true} : {}),
        },
        (_key, value: unknown) => (typeof value === 'bigint' ? value.toString() : value),
        2,
    );
    if (text.length <= charLimit) return text;
    return `${text.slice(0, Math.max(0, charLimit))}\n… (truncated)`;
}

/** Resolves the configured chat submit shortcut without depending on React events. */
export function shouldSubmitChat(
    event: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'metaKey'>,
    sendOnEnter: boolean,
): boolean {
    if (event.key !== 'Enter' || event.shiftKey) return false;
    return sendOnEnter || event.ctrlKey || event.metaKey;
}

/** Draft of the assistant message currently streaming in. */
export interface ChatStreamState {
    requestId: string;
    text: string;
    tools: ChatToolCall[];
    cancelled: boolean;
    batchProgress?: {
        completedBatch: number;
        totalBatches: number;
        completedCalls: number;
        totalCalls: number;
    };
    /** Set while a write statement waits for the approval dialog. */
    awaitingApproval?: {approvalId: string; query: string; queryKind: string} | null;
}

function emptyStream(requestId: string): ChatStreamState {
    return {requestId, text: '', tools: [], cancelled: false, awaitingApproval: null};
}

/**
 * Folds one `ai://chat-event` into the live draft. Pure so the reducer can be
 * tested without a transport: the hook only checks that the event belongs to
 * the turn it started.
 */
export function reduceChatStream(
    current: ChatStreamState | null,
    event: ChatEvent,
): ChatStreamState | null {
    switch (event.type) {
        case 'started':
            return current ?? emptyStream(event.requestId);
        case 'delta': {
            const base = current ?? emptyStream(event.requestId);
            return {...base, text: base.text + event.text};
        }
        case 'toolStarted': {
            const base = current ?? emptyStream(event.requestId);
            return {
                ...base,
                tools: [
                    ...base.tools,
                    {
                        id: event.callId,
                        name: event.name,
                        arguments: event.arguments,
                        status: 'pending',
                        durationMs: 0,
                        summary: 'Running…',
                    },
                ],
            };
        }
        case 'toolFinished': {
            if (!current) return current;
            return {
                ...current,
                awaitingApproval: null,
                tools: current.tools.map((tool) =>
                    tool.id === event.callId
                        ? {
                              ...tool,
                              status: event.status,
                              durationMs: event.durationMs,
                              summary: event.summary,
                          }
                        : tool,
                ),
            };
        }
        case 'approvalRequired': {
            const base = current ?? emptyStream(event.requestId);
            return {
                ...base,
                awaitingApproval: {
                    approvalId: event.approvalId,
                    query: event.query,
                    queryKind: event.queryKind,
                },
            };
        }
        case 'approvalResolved': {
            if (!current) return current;
            return {...current, awaitingApproval: null};
        }
        case 'batchProgress': {
            const base = current ?? emptyStream(event.requestId);
            return {
                ...base,
                batchProgress: {
                    completedBatch: event.completedBatch,
                    totalBatches: event.totalBatches,
                    completedCalls: event.completedCalls,
                    totalCalls: event.totalCalls,
                },
            };
        }
        case 'cancelled':
            return current ? {...current, cancelled: true} : current;
        default:
            return current;
    }
}
