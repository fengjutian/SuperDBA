import {describe, it, expect, vi} from 'vitest';
import {fireEvent, render, screen} from '@testing-library/react';
import {ChatMarkdown} from '@/components/chat/ChatMarkdown';

describe('ChatMarkdown', () => {
    it('allows users to select and copy rendered text', () => {
        const {container} = render(<ChatMarkdown content="Selectable response" />);

        expect(container.firstElementChild).toHaveClass('select-text');
    });

    it('renders a GitHub-flavoured table as a real table', () => {
        const content = [
            '| Schema | Tables | Domain |',
            '| --- | --- | --- |',
            '| ai_image_studio | 6 | image generation |',
            '| qb2026 | 143 | ordering ERP |',
        ].join('\n');

        render(<ChatMarkdown content={content} />);

        expect(screen.getByRole('table')).toBeInTheDocument();
        expect(screen.getByRole('columnheader', {name: 'Schema'})).toBeInTheDocument();
        expect(screen.getByRole('cell', {name: 'ai_image_studio'})).toBeInTheDocument();
        expect(screen.getByRole('cell', {name: '143'})).toBeInTheDocument();
        // Pipes are structural, never text the user reads.
        expect(screen.queryByText(/\|/)).not.toBeInTheDocument();
    });

    it('keeps a wide table inside a horizontally scrollable wrapper', () => {
        const content = ['| a | b |', '| --- | --- |', '| 1 | 2 |'].join('\n');

        render(<ChatMarkdown content={content} />);

        const wrapper = screen.getByRole('table').parentElement;
        expect(wrapper).toHaveClass('overflow-x-auto');
    });

    it('drops backslash escapes so snake_case identifiers read normally', () => {
        render(<ChatMarkdown content={'We looked at ai\\_image\\_studio and 1\\. runtime.'} />);

        expect(screen.getByText(/ai_image_studio/)).toBeInTheDocument();
        expect(screen.queryByText(/\\_/)).not.toBeInTheDocument();
    });

    it('renders fenced code as a code block instead of literal backticks', () => {
        render(<ChatMarkdown content={'```sql\nSELECT 1;\n```'} />);

        const code = screen.getByText('SELECT 1;');
        expect(code.tagName).toBe('CODE');
        expect(code.closest('pre')).not.toBeNull();
        expect(screen.queryByText(/```/)).not.toBeInTheDocument();
    });

    it('opens links in a new tab without leaking the opener', () => {
        render(<ChatMarkdown content={'See [the docs](https://example.com/docs).'} />);

        const link = screen.getByRole('link', {name: 'the docs'});
        expect(link).toHaveAttribute('href', 'https://example.com/docs');
        expect(link).toHaveAttribute('target', '_blank');
        expect(link).toHaveAttribute('rel', 'noopener noreferrer');
    });

    it('routes available evidence links inside the transcript', () => {
        const onEvidenceClick = vi.fn();
        render(
            <ChatMarkdown
                content={'Supported by [E2], but not [E3].'}
                evidenceCount={2}
                onEvidenceClick={onEvidenceClick}
            />,
        );

        const evidence = screen.getByRole('link', {name: 'E2'});
        expect(evidence).toHaveAttribute('href', '#chat-evidence-E2');
        expect(evidence).not.toHaveAttribute('target');
        fireEvent.click(evidence);
        expect(onEvidenceClick).toHaveBeenCalledWith('E2');
        expect(screen.queryByRole('link', {name: 'E3'})).not.toBeInTheDocument();
    });

    it('renders strikethrough and task list items', () => {
        render(<ChatMarkdown content={'- [x] done\n- ~~dropped~~ kept'} />);

        expect(screen.getByRole('checkbox')).toBeChecked();
        expect(screen.getByText('dropped').tagName).toBe('DEL');
    });
});
