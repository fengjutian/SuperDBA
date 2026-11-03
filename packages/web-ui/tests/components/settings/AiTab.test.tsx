import {fireEvent, render, screen, waitFor} from '@testing-library/react';
import {beforeEach, describe, expect, it, vi} from 'vitest';
import {AiTab} from '@/components/settings/AiTab';

const {invoke, updateSetting, showToast} = vi.hoisted(() => ({
    invoke: vi.fn(),
    updateSetting: vi.fn(),
    showToast: vi.fn(),
}));
vi.unmock('lucide-react');
vi.mock('@tauri-apps/api/core', () => ({invoke}));
vi.mock('@/hooks/useToast', () => ({useToast: () => ({showToast})}));
vi.mock('@/hooks/useSettings', () => ({
    useSettings: () => ({
        settings: {aiEnabled: true, aiProvider: 'openrouter', aiModel: ''},
        updateSetting,
    }),
}));
vi.mock('react-i18next', () => ({
    useTranslation: () => ({t: (key: string) => key}),
    Trans: () => null,
}));

describe('AiTab provider selection', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        invoke.mockImplementation(async (cmd: string) => {
            if (cmd === 'check_ai_key_status') return {configured: false, fromEnv: false};
            if (cmd === 'get_ai_models') return {openrouter: [], deepseek: []};
            return '';
        });
        updateSetting.mockResolvedValue(undefined);
    });

    it('offers DeepSeek and checks its key status', async () => {
        render(<AiTab />);
        const deepseek = await screen.findByRole('button', {name: 'DeepSeek'});

        expect(deepseek).toHaveAttribute('aria-pressed', 'false');
        await waitFor(() =>
            expect(invoke).toHaveBeenCalledWith('check_ai_key_status', {provider: 'deepseek'}),
        );

        fireEvent.click(deepseek);
        expect(updateSetting).toHaveBeenCalledWith('aiProvider', 'deepseek');
    });
});

describe('AiTab proxy configuration', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        invoke.mockImplementation(async (cmd: string) => {
            if (cmd === 'check_ai_key_status') return {configured: true, fromEnv: false};
            if (cmd === 'get_ai_models') return {openrouter: []};
            return '';
        });
        updateSetting.mockResolvedValue(undefined);
    });

    it('keeps a blank custom endpoint editable and persists it after entering a host', async () => {
        render(<AiTab />);
        await screen.findByText('settings.ai.configured');
        fireEvent.click(screen.getByRole('button', {name: 'settings.network.modeCustom'}));
        const host = await screen.findByPlaceholderText('127.0.0.1');
        expect(host).toHaveValue('');
        expect(updateSetting).toHaveBeenLastCalledWith('aiProviderProxies', {});
        fireEvent.change(host, {target: {value: 'proxy.example.test'}});
        await waitFor(() =>
            expect(updateSetting).toHaveBeenLastCalledWith('aiProviderProxies', {
                openrouter: {
                    mode: 'custom',
                    endpoint: {
                        protocol: 'http',
                        host: 'proxy.example.test',
                        port: 7890,
                        username: undefined,
                    },
                },
            }),
        );
        fireEvent.click(screen.getByRole('button', {name: 'settings.network.modeInherit'}));
        expect(screen.queryByPlaceholderText('127.0.0.1')).not.toBeInTheDocument();
        expect(updateSetting).toHaveBeenLastCalledWith('aiProviderProxies', {});
    });
});

describe('AiTab model refresh', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        invoke.mockImplementation(async (cmd: string) => {
            if (cmd === 'check_ai_key_status') return {configured: false, fromEnv: false};
            if (cmd === 'get_ai_models') return {openrouter: ['openrouter/auto']};
            return '';
        });
        updateSetting.mockResolvedValue(undefined);
    });

    it('confirms a refresh with a toast and keeps a single request in flight', async () => {
        render(<AiTab />);
        const refresh = await screen.findByRole('button', {name: 'settings.ai.refresh'});

        fireEvent.click(refresh);
        fireEvent.click(refresh);

        await waitFor(() =>
            expect(showToast).toHaveBeenCalledWith('settings.ai.refreshSuccess', {
                kind: 'success',
            }),
        );
        expect(showToast).toHaveBeenCalledTimes(1);
        const forcedRefreshes = invoke.mock.calls.filter(
            (call) =>
                call[0] === 'get_ai_models' &&
                (call[1] as {forceRefresh?: boolean} | undefined)?.forceRefresh === true,
        );
        expect(forcedRefreshes).toHaveLength(1);
        await waitFor(() => expect(refresh).toBeEnabled());
    });
});

vi.mock('@/hooks/useSuperDBAClient', () => {
    const client = {call: (...args: unknown[]) => invoke(...args)};
    return {useSuperDBAClient: () => client};
});

vi.mock('@/hooks/usePlatformCapabilities', () => ({
    usePlatformCapabilities: () => ({negotiation: {environment: 'tauri'}}),
}));
