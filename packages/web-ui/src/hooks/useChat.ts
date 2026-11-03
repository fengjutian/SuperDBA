import {useCallback, useEffect, useRef, useState} from 'react';
import {useSuperDBAClient} from './useSuperDBAClient';
import {useSettings} from './useSettings';
import type {ChatContext, ChatSendRequest, ChatSession, ChatSessionSummary} from '@/types/chat';

/**
 * Conversation state for the chat panel.
 *
 * The backend owns persistence and the agent loop; this hook only keeps the
 * currently open conversation and the session list in sync with it.
 */
export function useChatAgent() {
    const client = useSuperDBAClient();
    const {settings} = useSettings();
    const [sessions, setSessions] = useState<ChatSessionSummary[]>([]);
    const [session, setSession] = useState<ChatSession | null>(null);
    const [loadingList, setLoadingList] = useState(true);
    const [sending, setSending] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const mounted = useRef(true);

    useEffect(() => {
        mounted.current = true;
        return () => {
            mounted.current = false;
        };
    }, []);

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

    const send = useCallback(
        async (message: string, context: ChatContext) => {
            const trimmed = message.trim();
            if (!trimmed || sending) return;
            const provider = settings.aiProvider;
            if (!provider) {
                setError('Configure an AI provider in Settings → AI before using chat.');
                return;
            }
            setSending(true);
            setError(null);
            const request: ChatSendRequest = {
                provider,
                model: settings.aiModel ?? '',
                ...(session?.id ? {sessionId: session.id} : {}),
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
                if (mounted.current) setSending(false);
            }
        },
        [client, refreshSessions, sending, session?.id, settings.aiModel, settings.aiProvider],
    );

    return {
        sessions,
        session,
        loadingList,
        sending,
        error,
        send,
        openSession,
        startNewSession,
        deleteSession,
        refreshSessions,
        setError,
    };
}
