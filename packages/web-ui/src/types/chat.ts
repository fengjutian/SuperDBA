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
    message: string;
    context: ChatContext;
}
