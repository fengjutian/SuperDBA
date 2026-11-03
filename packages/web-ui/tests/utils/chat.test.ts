import {describe, expect, it} from 'vitest';
import {extractSqlBlock, formatToolResult} from '@/utils/chat';

describe('chat utils', () => {
    describe('extractSqlBlock', () => {
        it('extracts and trims the first sql fence', () => {
            const content = 'Try this:\n\n```sql\nSELECT 1\n```\n\nThen check the plan.';
            expect(extractSqlBlock(content)).toBe('SELECT 1');
        });

        it('is case insensitive on the fence language', () => {
            expect(extractSqlBlock('```SQL\nselect 2\n```')).toBe('select 2');
        });

        it('returns null when there is no sql fence', () => {
            expect(extractSqlBlock('```js\nconsole.log(1)\n```')).toBeNull();
            expect(extractSqlBlock('no code here')).toBeNull();
        });

        it('returns null for empty content', () => {
            expect(extractSqlBlock('')).toBeNull();
        });
    });

    describe('formatToolResult', () => {
        it('pretty prints small results', () => {
            expect(formatToolResult({tables: ['a']})).toBe(
                JSON.stringify({tables: ['a']}, null, 2),
            );
        });

        it('truncates large results and marks the cut', () => {
            const result = formatToolResult({rows: 'x'.repeat(200)}, 40);
            expect(result.endsWith('…')).toBe(true);
            expect(result.length).toBeLessThan(60);
        });

        it('returns an empty string for undefined', () => {
            expect(formatToolResult(undefined)).toBe('');
        });
    });
});
