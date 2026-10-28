import type {ReactNode} from 'react';
import type {SuperDBAClient} from '@/api/client';
import {SuperDBAClientContext} from './SuperDBAClientContext';

interface SuperDBAClientProviderProps {
    children: ReactNode;
    client: SuperDBAClient;
}

export function SuperDBAClientProvider({children, client}: SuperDBAClientProviderProps) {
    return (
        <SuperDBAClientContext.Provider value={client}>{children}</SuperDBAClientContext.Provider>
    );
}
