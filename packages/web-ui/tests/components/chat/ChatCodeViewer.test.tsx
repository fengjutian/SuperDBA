import {render, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';
import {ChatCodeViewer} from '@/components/chat/ChatCodeViewer';

const editorProps = vi.hoisted(() => ({current: null as Record<string, unknown> | null}));

vi.mock('@/components/ui/CellCodeEditor', () => ({
    CellCodeEditor: (props: Record<string, unknown>) => {
        editorProps.current = props;
        return <div data-testid="cell-code-editor" />;
    },
}));

describe('ChatCodeViewer', () => {
    it('renders JSON in the shared VS Code editor as read-only content', () => {
        render(
            <ChatCodeViewer
                value={'{\n  "database": "qb2026"\n}'}
                label="Result"
                maxHeight={208}
            />,
        );

        expect(screen.getByRole('region', {name: 'Result'})).toHaveStyle({height: '73px'});
        expect(screen.getByTestId('cell-code-editor')).toBeInTheDocument();
        expect(editorProps.current).toMatchObject({
            value: '{\n  "database": "qb2026"\n}',
            height: '100%',
            language: 'json',
            readOnly: true,
        });
    });

    it('caps tall output at the requested maximum height', () => {
        render(
            <ChatCodeViewer
                value={Array.from({length: 20}, (_, index) => String(index)).join('\n')}
                label="Arguments"
                maxHeight={160}
            />,
        );

        expect(screen.getByRole('region', {name: 'Arguments'})).toHaveStyle({height: '160px'});
    });
});
