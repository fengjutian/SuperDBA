import type {PlatformEnvironment} from '@/platform/capabilities';
import {SuperDBAClient, type SuperDBATransport} from './client';
import type {SessionNegotiation} from './session';
import {HttpTransport} from './transports/httpTransport';
import {TauriTransport} from './transports/tauriTransport';

interface InitializableBrowserTransport extends SuperDBATransport {
    initialize(): Promise<SessionNegotiation>;
}

interface ClientBootstrapDependencies {
    readonly createTauriTransport?: () => SuperDBATransport;
    readonly createHttpTransport?: () => InitializableBrowserTransport;
}

export interface SuperDBARuntimeBootstrap {
    readonly client: SuperDBAClient;
    readonly session: SessionNegotiation | null;
}

export async function bootstrapSuperDBARuntime(
    environment: PlatformEnvironment,
    dependencies: ClientBootstrapDependencies = {},
): Promise<SuperDBARuntimeBootstrap> {
    if (environment === 'tauri') {
        const transport = dependencies.createTauriTransport?.() ?? new TauriTransport();
        return {client: new SuperDBAClient(transport), session: null};
    }

    const transport = dependencies.createHttpTransport?.() ?? new HttpTransport();
    const session = await transport.initialize();
    return {client: new SuperDBAClient(transport), session};
}

export async function bootstrapSuperDBAClient(
    environment: PlatformEnvironment,
    dependencies: ClientBootstrapDependencies = {},
): Promise<SuperDBAClient> {
    return (await bootstrapSuperDBARuntime(environment, dependencies)).client;
}
