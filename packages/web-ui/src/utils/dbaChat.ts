import type {AiProvider} from '@/contexts/SettingsContext';

/** Providers whose wire adapters support the DBA agent's tool-call loop. */
export const SUPPORTED_DBA_PROVIDERS = [
    'openai',
    'anthropic',
    'deepseek',
    'minimax',
    'openrouter',
    'requesty',
    'custom-openai',
    'ollama',
] as const satisfies readonly AiProvider[];

const supportedProviders = new Set<AiProvider>(SUPPORTED_DBA_PROVIDERS);

export function isDbaChatProviderSupported(
    provider: AiProvider | null | undefined,
): provider is (typeof SUPPORTED_DBA_PROVIDERS)[number] {
    return provider != null && supportedProviders.has(provider);
}
