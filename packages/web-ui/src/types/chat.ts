/**
 * Wire types for the conversational DBA agent (`application::chat`).
 *
 * The backend serialises with `rename_all = "camelCase"` and skips empty
 * optional fields, so absent values never arrive as `null` here.
 */

export type ChatRole = 'system' | 'user' | 'assistant' | 'tool';

export type ChatToolStatus = 'pending' | 'ok' | 'error' | 'denied';

export interface ChatToolCall {
    id: string;
    name: string;
    arguments: Record<string, unknown>;
    status: ChatToolStatus;
    durationMs: number;
    summary: string;
    /** Structured tool result, bounded by the backend before it is stored. */
    result?: unknown;
}

export interface ChatMessage {
    id: string;
    role: ChatRole;
    content: string;
    /** Provider thinking, replayed to the model; not shown as an answer. */
    reasoning?: string;
    createdAt: string;
    toolCalls?: ChatToolCall[];
    toolCallId?: string;
    notice?: string;
}

export interface ChatSession {
    id: string;
    title: string;
    provider: string;
    model: string;
    connectionId?: string;
    createdAt: string;
    updatedAt: string;
    messages: ChatMessage[];
}

export interface ChatSessionSummary {
    id: string;
    title: string;
    provider: string;
    model: string;
    connectionId?: string;
    createdAt: string;
    updatedAt: string;
    messageCount: number;
}

export interface ChatContext {
    connectionId?: string;
    connectionName?: string;
    schema?: string;
    database?: string;
    selectedSql?: string;
    lastResult?: string;
    lastPlan?: string;
}

export interface ChatSendRequest {
    provider: string;
    model: string;
    sessionId?: string;
    /** Client-generated id used to correlate `ai://chat-event` progress and cancellation. */
    requestId?: string;
    message: string;
    context: ChatContext;
}

/**
 * Progress emitted while a turn runs. `finished`, `cancelled`, and `failed`
 * close the stream; the authoritative transcript still comes from the
 * `chat_send` response.
 */
export type ChatEvent =
    | {type: 'started'; requestId: string; sessionId: string}
    | {type: 'delta'; requestId: string; sessionId: string; text: string}
    | {
          type: 'toolStarted';
          requestId: string;
          callId: string;
          name: string;
          arguments: Record<string, unknown>;
      }
    | {
          type: 'toolFinished';
          requestId: string;
          callId: string;
          name: string;
          status: ChatToolStatus;
          durationMs: number;
          summary: string;
      }
    | {type: 'finished'; requestId: string; sessionId: string}
    | {type: 'cancelled'; requestId: string; sessionId: string}
    | {
          type: 'batchProgress';
          requestId: string;
          sessionId: string;
          completedBatch: number;
          totalBatches: number;
          completedCalls: number;
          totalCalls: number;
      }
    | {type: 'failed'; requestId: string; sessionId: string; message: string}
    | {
          type: 'approvalRequired';
          requestId: string;
          approvalId: string;
          query: string;
          queryKind: string;
      }
    | {
          type: 'approvalResolved';
          requestId: string;
          approvalId: string;
          decision: string;
      };
