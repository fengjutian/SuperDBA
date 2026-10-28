import {useContext} from 'react';
import {SuperDBAClientContext} from '@/contexts/SuperDBAClientContext';

export function useSuperDBAClient() {
    const client = useContext(SuperDBAClientContext);

    if (!client) {
        throw new Error('useSuperDBAClient must be used within SuperDBAClientProvider');
    }

    return client;
}
