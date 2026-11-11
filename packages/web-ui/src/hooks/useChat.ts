import {useCallback, useEffect, useRef, useState} from 'react';
import {useSuperDBAClient} from './useSuperDBAClient';
import {useSettings} from './useSettings';
import {reduceChatStream, type ChatStreamState} from '@/utils/chat';
import {sanitizeGeneratedStarters} from '@/utils/chatStarters';
import {isDbaChatProviderSupported} from '@/utils/dbaChat';
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
    const [continuationRequired, setContinuationRequired] = useState<{
        completedCalls: number;
        totalCalls: number;
    } | null>(null);
    const mounted = useRef(true);
    const activeRequest = useRef<string | null>(null);
    const failedRequest = useRef<string | null>(null);

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
            failedRequest.current = event.requestId;
            setError(event.message);
            setStream((current) => reduceChatStream(current, event));
            return;
        }
        if (event.type === 'continuationRequired') {
            setContinuationRequired({
                completedCalls: event.completedCalls,
                totalCalls: event.totalCalls,
            });
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
            setContinuationRequired(null);
            try {
                const loaded = await client.call('chat_get_session', {sessionId});
                if (mounted.current) {
                    setSession(loaded);
                    setContinuationRequired(loaded.continuation ?? null);
                }
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
        setContinuationRequired(null);
    }, []);

    const deleteSession = useCallback(
        async (sessionId: string) => {
            setError(null);
            try {
                await client.call('chat_delete_session', {sessionId});
                if (session?.id === sessionId) {
                    setSession(null);
                    setContinuationRequired(null);
                }
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

    const send = useCallback(
        async (message: string, context: ChatContext): Promise<boolean> => {
            const trimmed = message.trim();
            if (!trimmed || sending) return false;
            const provider = settings.dbaChatProvider ?? settings.aiProvider;
            if (!isDbaChatProviderSupported(provider)) {
                setError(
                    provider
                        ? `${provider} does not support DBA tool calls. Choose a dedicated DBA provider in Settings.`
                        : 'Configure an AI provider in Settings before using DBA chat.',
                );
                return false;
            }
            const requestId = newRequestId();
            activeRequest.current = requestId;
            failedRequest.current = null;
            setSending(true);
            setError(null);
            setContinuationRequired(null);
            setStream({requestId, text: '', tools: [], cancelled: false});
            const request: ChatSendRequest = {
                provider,
                model: settings.dbaChatModel?.trim() || settings.aiModel || '',
                ...(session?.id ? {sessionId: session.id} : {}),
                requestId,
                message: trimmed,
                context,
                autoContinue: settings.dbaChatAutoContinue === true,
                temperature: settings.dbaChatTemperature ?? 0,
                maxOutputTokens: settings.dbaChatMaxOutputTokens ?? 2048,
            };
            try {
                const updated = await client.call('chat_send', {request});
                if (!mounted.current) return false;
                setSession(updated);
                setContinuationRequired(updated.continuation ?? null);
                await refreshSessions();
                return failedRequest.current !== requestId;
            } catch (err) {
                if (mounted.current) setError(String(err));
                return false;
            } finally {
                if (mounted.current) {
                    setSending(false);
                    setStream(null);
                }
                activeRequest.current = null;
                if (failedRequest.current === requestId) failedRequest.current = null;
            }
        },
        [
            client,
            refreshSessions,
            sending,
            session?.id,
            settings.aiModel,
            settings.aiProvider,
            settings.dbaChatProvider,
            settings.dbaChatModel,
            settings.dbaChatAutoContinue,
            settings.dbaChatTemperature,
            settings.dbaChatMaxOutputTokens,
        ],
    );

    /**
     * Model-written opening questions for an empty conversation.
     *
     * Failures resolve to an empty list: the panel keeps its local questions,
     * so this never surfaces an error of its own.
     */
    const suggestStarters = useCallback(
        async (context: ChatContext, tables: string[], language: string): Promise<string[]> => {
            const provider = settings.dbaChatProvider ?? settings.aiProvider;
            if (!isDbaChatProviderSupported(provider) || tables.length === 0) return [];
            try {
                const values = await client.call('chat_suggest_starters', {
                    request: {
                        provider,
                        model: settings.dbaChatModel?.trim() || settings.aiModel || '',
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
        [
            client,
            settings.aiModel,
            settings.aiProvider,
            settings.dbaChatModel,
            settings.dbaChatProvider,
        ],
    );

    return {
        sessions,
        session,
        loadingList,
        sending,
        stream,
        error,
        continuationRequired,
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
