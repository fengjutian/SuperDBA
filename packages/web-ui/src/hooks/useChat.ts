import {useCallback, useEffect, useRef, useState} from 'react';
import {useSuperDBAClient} from './useSuperDBAClient';
import {useSettings} from './useSettings';
import {reduceChatStream, type ChatStreamState} from '@/utils/chat';
import {sanitizeGeneratedStarters} from '@/utils/chatStarters';
import type {
    ChatContext,
    ChatEvent,
    ChatSendRequest,
    ChatSession,
    ChatSessionSummary,
} from '@/types/chat';

function newRequestId(): string {
    if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
        return crypto.randomUUID();
    }
    return `chat-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

/**
 * Conversation state for the chat panel.
 *
 * The backend owns persistence and the agent loop; this hook keeps the open
 * conversation, the session list, and the live turn draft in sync.
 */
export function useChatAgent() {
    const client = useSuperDBAClient();
    const {settings} = useSettings();
    const [sessions, setSessions] = useState<ChatSessionSummary[]>([]);
    const [session, setSession] = useState<ChatSession | null>(null);
    const [loadingList, setLoadingList] = useState(true);
    const [sending, setSending] = useState(false);
    const [stream, setStream] = useState<ChatStreamState | null>(null);
    const [error, setError] = useState<string | null>(null);
    const mounted = useRef(true);
    const activeRequest = useRef<string | null>(null);

    useEffect(() => {
        mounted.current = true;
        return () => {
            mounted.current = false;
        };
    }, []);

    const applyEvent = useCallback((event: ChatEvent) => {
        const active = activeRequest.current;
        if (!active || event.requestId !== active) return;
        if (event.type === 'failed') {
            setError(event.message);
            return;
        }
        setStream((current) => reduceChatStream(current, event));
    }, []);

    useEffect(() => {
        const subscription = client.subscribe('ai://chat-event', applyEvent);
        return () => {
            void subscription.then((unsubscribe) => unsubscribe()).catch(() => {});
        };
    }, [client, applyEvent]);

    const refreshSessions = useCallback(async () => {
        try {
            const summaries = await client.call('chat_list_sessions', undefined);
            if (mounted.current) setSessions(summaries);
            return summaries;
        } catch (err) {
            if (mounted.current) setError(String(err));
            return [];
        } finally {
            if (mounted.current) setLoadingList(false);
        }
    }, [client]);

    const openSession = useCallback(
        async (sessionId: string) => {
            setError(null);
            try {
                const loaded = await client.call('chat_get_session', {sessionId});
                if (mounted.current) setSession(loaded);
            } catch (err) {
                if (mounted.current) setError(String(err));
            }
        },
        [client],
    );

    useEffect(() => {
        let cancelled = false;
        void (async () => {
            const summaries = await refreshSessions();
            if (cancelled) return;
            if (summaries.length > 0) {
                await openSession(summaries[0].id);
            }
        })();
        return () => {
            cancelled = true;
        };
    }, [refreshSessions, openSession]);

    const startNewSession = useCallback(() => {
        setSession(null);
        setStream(null);
        setError(null);
    }, []);

    const deleteSession = useCallback(
        async (sessionId: string) => {
            setError(null);
            try {
                await client.call('chat_delete_session', {sessionId});
                if (session?.id === sessionId) setSession(null);
                await refreshSessions();
            } catch (err) {
                setError(String(err));
            }
        },
        [client, refreshSessions, session?.id],
    );

    const cancel = useCallback(async () => {
        const requestId = activeRequest.current;
        if (!requestId) return;
        try {
            await client.call('chat_cancel', {requestId});
        } catch (err) {
            if (mounted.current) setError(String(err));
        }
    }, [client]);

    const send = useCallback(        async (message: string, context: ChatContext) => {
            const trimmed = message.trim();
            if (!trimmed || sending) return;
            const provider = settings.aiProvider;
            if (!provider) {
                setError('Configure an AI provider in Settings → AI before using chat.');
                return;
            }
            const requestId = newRequestId();
            activeRequest.current = requestId;
            setSending(true);
            setError(null);
            setStream({requestId, text: '', tools: [], cancelled: false});
            const request: ChatSendRequest = {
                provider,
                model: settings.aiModel ?? '',
                ...(session?.id ? {sessionId: session.id} : {}),
                requestId,
                message: trimmed,
                context,
            };
            try {
                const updated = await client.call('chat_send', {request});
                if (!mounted.current) return;
                setSession(updated);
                await refreshSessions();
            } catch (err) {
                if (mounted.current) setError(String(err));
            } finally {
                if (mounted.current) {
                    setSending(false);
                    setStream(null);
                }
                activeRequest.current = null;
            }
        },
        [client, refreshSessions, sending, session?.id, settings.aiModel, settings.aiProvider],
    );

    /**
     * Model-written opening questions for an empty conversation.
     *
     * Failures resolve to an empty list: the panel keeps its local questions,
     * so this never surfaces an error of its own.
     */
    const suggestStarters = useCallback(
        async (
            context: ChatContext,
            tables: string[],
            language: string,
        ): Promise<string[]> => {
            const provider = settings.aiProvider;
            if (!provider || tables.length === 0) return [];
            try {
                const values = await client.call('chat_suggest_starters', {
                    request: {
                        provider,
                        model: settings.aiModel ?? '',
                        language,
                        context,
                        tables,
                    },
                });
                return sanitizeGeneratedStarters(values);
            } catch {
                return [];
            }
        },
        [client, settings.aiModel, settings.aiProvider],
    );

    return {
        sessions,
        session,
        loadingList,
        sending,
        stream,
        error,
        send,
        suggestStarters,
        cancel,
        openSession,
        startNewSession,
        deleteSession,
        refreshSessions,
        setError,
    };
}
