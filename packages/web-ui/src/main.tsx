// Import polyfills first to make Buffer available globally
import '@/polyfills';

import React, {Suspense} from 'react';
import ReactDOM from 'react-dom/client';
import {App} from '@/App';
import '@/index.css';
import '@/i18n/config';
import {DatabaseProvider} from '@/contexts/DatabaseProvider';
import {ToastProvider} from '@/contexts/ToastProvider';
import {SettingsProvider} from '@/contexts/SettingsProvider';
import {SavedQueriesProvider} from '@/contexts/SavedQueriesProvider';
import {QueryHistoryProvider} from '@/contexts/QueryHistoryProvider';
import {EditorProvider} from '@/contexts/EditorProvider';
import {ThemeProvider} from '@/contexts/ThemeProvider';
import {UpdateProvider} from '@/contexts/UpdateProvider';
import {ProductionGuardProvider} from '@/contexts/ProductionGuardContext';
import {bootstrapSuperDBARuntime} from '@/api/bootstrap';
import {SuperDBAClientProvider} from '@/contexts/SuperDBAClientProvider';
import {TauriPlatformCapabilities} from '@/platform/tauriCapabilities';
import {BrowserPlatformCapabilities} from '@/platform/browserCapabilities';
import {PlatformCapabilitiesProvider} from '@/contexts/PlatformCapabilitiesProvider';
import {detectPlatformEnvironment} from '@/platform/environment';
import {toErrorMessage} from '@/utils/errors';
import {BrowserCapabilityFallbacks} from '@/components/ui/BrowserCapabilityFallbacks';
import {registerActivePlatformCapabilities} from '@/platform/activeCapabilities';
import {LoadingState} from '@/components/ui/LoadingState';
import {UiStateProvider} from '@/contexts/UiStateProvider';

const rootElement = document.getElementById('root') as HTMLElement;
const environment = detectPlatformEnvironment();

async function startApplication() {
    const {client: superdbaClient, session} = await bootstrapSuperDBARuntime(environment);
    const platformCapabilities =
        environment === 'tauri'
            ? new TauriPlatformCapabilities()
            : new BrowserPlatformCapabilities(
                  superdbaClient,
                  undefined,
                  session?.capabilities.serverFileBrowser ?? false,
              );
    registerActivePlatformCapabilities(platformCapabilities);

    ReactDOM.createRoot(rootElement).render(
        <React.StrictMode>
            <Suspense fallback={<LoadingState />}>
                <SuperDBAClientProvider client={superdbaClient}>
                    <UiStateProvider>
                        <PlatformCapabilitiesProvider capabilities={platformCapabilities}>
                            <UpdateProvider session={session}>
                                <ThemeProvider>
                                    <SettingsProvider>
                                        <ToastProvider>
                                            <BrowserCapabilityFallbacks />
                                            <DatabaseProvider>
                                                <SavedQueriesProvider>
                                                    <QueryHistoryProvider>
                                                        <EditorProvider>
                                                            <ProductionGuardProvider>
                                                                <App />
                                                            </ProductionGuardProvider>
                                                        </EditorProvider>
                                                    </QueryHistoryProvider>
                                                </SavedQueriesProvider>
                                            </DatabaseProvider>
                                        </ToastProvider>
                                    </SettingsProvider>
                                </ThemeProvider>
                            </UpdateProvider>
                        </PlatformCapabilitiesProvider>
                    </UiStateProvider>
                </SuperDBAClientProvider>
            </Suspense>
        </React.StrictMode>,
    );
}

void startApplication().catch((error: unknown) => {
    console.error('Failed to start SuperDBA:', error);
    rootElement.textContent = `Unable to start SuperDBA: ${toErrorMessage(error)}`;
});
