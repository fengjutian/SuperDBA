import {beforeEach, describe, expect, it} from 'vitest';
import {
    buildLocalStarters,
    cachedStarters,
    clearStarterCache,
    rememberStarters,
    sanitizeGeneratedStarters,
    selectPromptTables,
    starterCacheKey,
    tableSignature,
} from '@/utils/chatStarters';

describe('chatStarters', () => {
    describe('selectPromptTables', () => {
        it('trims, drops blanks and de-duplicates case-insensitively', () => {
            expect(selectPromptTables([' users ', '', 'Users', 'orders', '  '])).toEqual([
                'users',
                'orders',
            ]);
        });

        it('keeps insertion order and caps the list', () => {
            const names = Array.from({length: 80}, (_, index) => `table_${index}`);
            const selected = selectPromptTables(names);
            expect(selected).toHaveLength(40);
            expect(selected[0]).toBe('table_0');
            expect(selected[39]).toBe('table_39');
        });
    });

    describe('buildLocalStarters', () => {
        it('always offers three questions when no tables are loaded', () => {
            const starters = buildLocalStarters([]);
            expect(starters.map((starter) => starter.key)).toEqual([
                'chat.local.noTables',
                'chat.starter.slow',
                'chat.starter.indexes',
            ]);
            for (const starter of starters) {
                expect(starter.fallback.length).toBeGreaterThan(0);
            }
        });

        it('names the only table in the scope', () => {
            const starters = buildLocalStarters(['events']);
            expect(starters).toHaveLength(3);
            expect(starters[0].params).toEqual({table: 'events'});
            expect(starters[1].params).toEqual({table: 'events'});
            expect(starters[2].params).toEqual({table: 'events'});
        });

        it('counts the scope and pairs the first two tables', () => {
            const starters = buildLocalStarters(['users', 'orders', 'invoices']);
            expect(starters).toHaveLength(3);
            expect(starters[0].params).toEqual({count: 3});
            expect(starters[2].params).toEqual({table: 'users', table2: 'orders'});
        });

        it('does not repeat a de-duplicated table as the second name', () => {
            const starters = buildLocalStarters(['users', 'Users']);
            // One real table: the single-table questions must be used.
            expect(starters[0].key).toBe('chat.local.singleTable');
        });
    });

    describe('tableSignature', () => {
        it('ignores order, case and duplicates', () => {
            expect(tableSignature(['users', 'Orders'])).toBe(
                tableSignature(['orders', 'users', 'USERS']),
            );
        });

        it('changes when a table is added', () => {
            expect(tableSignature(['users'])).not.toBe(tableSignature(['users', 'orders']));
        });
    });

    describe('starterCacheKey', () => {
        it('separates connections, providers and languages', () => {
            const base = starterCacheKey('openai', 'gpt-4', 'en', 'connection-1', ['users']);
            expect(starterCacheKey('openai', 'gpt-4', 'en', 'connection-2', ['users'])).not.toBe(
                base,
            );
            expect(starterCacheKey('deepseek', 'gpt-4', 'en', 'connection-1', ['users'])).not.toBe(
                base,
            );
            expect(starterCacheKey('openai', 'gpt-4', 'zh', 'connection-1', ['users'])).not.toBe(
                base,
            );
            expect(starterCacheKey('openai', 'gpt-4', 'en', null, ['users'])).not.toBe(base);
        });
    });

    describe('sanitizeGeneratedStarters', () => {
        it('strips numbering, bullets, quotes and extra whitespace', () => {
            expect(
                sanitizeGeneratedStarters([
                    '1. "Which tables should I start with?"',
                    '- What   does the users table hold?',
                    '• `Which indexes are missing?`',
                ]),
            ).toEqual([
                'Which tables should I start with?',
                'What does the users table hold?',
                'Which indexes are missing?',
            ]);
        });

        it('drops duplicates, non-strings and blanks, and keeps three', () => {
            expect(
                sanitizeGeneratedStarters([
                    'users?',
                    'USERS?',
                    '',
                    '   ',
                    42,
                    null,
                    {text: 'nope'},
                    'orders?',
                    'invoices?',
                    'fourth?',
                ]),
            ).toEqual(['users?', 'orders?', 'invoices?']);
        });

        it('returns nothing for a non-array payload', () => {
            expect(sanitizeGeneratedStarters('one question')).toEqual([]);
            expect(sanitizeGeneratedStarters(undefined)).toEqual([]);
        });

        it('bounds long questions', () => {
            const [starter] = sanitizeGeneratedStarters(['a'.repeat(400)]);
            expect(starter.length).toBe(120);
            expect(starter.endsWith('…')).toBe(true);
        });
    });

    describe('cache', () => {
        beforeEach(() => {
            clearStarterCache();
        });

        it('remembers sanitized questions per key', () => {
            const key = starterCacheKey('openai', 'gpt-4', 'en', 'connection-1', ['users']);
            expect(cachedStarters(key)).toBeNull();
            rememberStarters(key, ['1. Why is users slow?', 'Which indexes are missing?']);
            expect(cachedStarters(key)).toEqual(['Why is users slow?', 'Which indexes are missing?']);
        });

        it('does not cache an empty answer', () => {
            rememberStarters('key', []);
            expect(cachedStarters('key')).toBeNull();
        });
    });
});
