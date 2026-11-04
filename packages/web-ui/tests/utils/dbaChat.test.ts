import {describe, expect, it} from 'vitest';
import {isDbaChatProviderSupported, SUPPORTED_DBA_PROVIDERS} from '@/utils/dbaChat';

describe('DBA chat provider support', () => {
    it('matches the backend tool-calling adapters', () => {
        expect(SUPPORTED_DBA_PROVIDERS).toEqual([
            'openai',
            'anthropic',
            'deepseek',
            'minimax',
            'openrouter',
            'requesty',
            'custom-openai',
            'ollama',
        ]);
    });

    it('rejects ChatGPT plan access and missing providers', () => {
        expect(isDbaChatProviderSupported('chatgpt')).toBe(false);
        expect(isDbaChatProviderSupported(null)).toBe(false);
        expect(isDbaChatProviderSupported(undefined)).toBe(false);
    });
});
