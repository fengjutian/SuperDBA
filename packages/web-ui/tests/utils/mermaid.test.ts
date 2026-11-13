import {describe, expect, it} from 'vitest';
import {normalizeMermaidSource} from '@/utils/mermaid';

describe('normalizeMermaidSource', () => {
    it('adds a neutral label to ER relationships that omit the required colon', () => {
        const source = [
            'erDiagram',
            '    country ||--o{ state',
            '    customer }o--|| billgroup',
        ].join('\n');

        expect(normalizeMermaidSource(source)).toBe(
            [
                'erDiagram',
                '    country ||--o{ state : relates',
                '    customer }o--|| billgroup : relates',
            ].join('\n'),
        );
    });

    it('preserves explicit ER relationship labels', () => {
        const source = ['erDiagram', '    customer ||--o{ order : places'].join('\n');

        expect(normalizeMermaidSource(source)).toBe(source);
    });

    it('does not change other Mermaid diagram types', () => {
        const source = ['flowchart LR', '    A --> B'].join('\n');

        expect(normalizeMermaidSource(source)).toBe(source);
    });

    it('produces ER source accepted by the real Mermaid parser', async () => {
        const {default: mermaid} = await import('mermaid');
        const source = ['erDiagram', '    country ||--o{ state'].join('\n');

        await expect(mermaid.parse(normalizeMermaidSource(source))).resolves.toBeTruthy();
    }, 30_000);
});
