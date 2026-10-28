import {renderHook} from '@testing-library/react';
import type {ReactNode} from 'react';
import {describe, expect, it} from 'vitest';
import {SuperDBAClient, type SuperDBATransport} from '@/api/client';
import {SuperDBAClientProvider} from '@/contexts/SuperDBAClientProvider';
import {useSuperDBAClient} from '@/hooks/useSuperDBAClient';

const createClient = () =>
    new SuperDBAClient({
        call: async () => {
            throw new Error('Unexpected command call');
        },
        callUnmigrated: async () => {
            throw new Error('Unexpected unmigrated command call');
        },
        subscribe: async () => {
            throw new Error('Unexpected event subscription');
        },
        emit: async () => {
            throw new Error('Unexpected event emission');
        },
    } as SuperDBATransport);

describe('SuperDBAClientProvider', () => {
    it('exposes the application client without exposing its transport', () => {
        const client = createClient();
        const wrapper = ({children}: {children: ReactNode}) => (
            <SuperDBAClientProvider client={client}>{children}</SuperDBAClientProvider>
        );

        const {result} = renderHook(() => useSuperDBAClient(), {wrapper});

        expect(result.current).toBe(client);
    });

    it('fails clearly when the hook is used outside the provider', () => {
        expect(() => renderHook(() => useSuperDBAClient())).toThrow(
            'useSuperDBAClient must be used within SuperDBAClientProvider',
        );
    });
});
