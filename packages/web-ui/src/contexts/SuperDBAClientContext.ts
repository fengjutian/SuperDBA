import {createContext} from 'react';
import type {SuperDBAClient} from '@/api/client';

export const SuperDBAClientContext = createContext<SuperDBAClient | null>(null);
