import {fireEvent, render, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';
import {Button, IconButton} from '@/components/ui/Button';

describe('Button', () => {
    it('defaults to a non-submitting button', () => {
        render(<Button>Save</Button>);

        expect(screen.getByRole('button', {name: 'Save'})).toHaveAttribute('type', 'button');
    });

    it('forwards clicks and supports an explicit submit type', () => {
        const onClick = vi.fn();
        render(
            <Button type="submit" onClick={onClick}>
                Save
            </Button>,
        );

        fireEvent.click(screen.getByRole('button', {name: 'Save'}));
        expect(onClick).toHaveBeenCalledOnce();
        expect(screen.getByRole('button', {name: 'Save'})).toHaveAttribute('type', 'submit');
    });

    it('disables interaction and exposes busy state while loading', () => {
        const onClick = vi.fn();
        render(
            <Button loading onClick={onClick}>
                Run
            </Button>,
        );

        const button = screen.getByRole('button', {name: 'Run'});
        fireEvent.click(button);
        expect(button).toBeDisabled();
        expect(button).toHaveAttribute('aria-busy', 'true');
        expect(onClick).not.toHaveBeenCalled();
    });

    it('applies semantic variants without palette colors', () => {
        render(<Button variant="danger">Delete</Button>);

        expect(screen.getByRole('button', {name: 'Delete'})).toHaveClass('bg-accent-error');
        expect(screen.getByRole('button', {name: 'Delete'})).toHaveClass('text-on-accent-error');
    });
});

describe('IconButton', () => {
    it('requires an accessible label and uses it as the default tooltip', () => {
        render(<IconButton label="Refresh" icon={<span>visible icon</span>} />);

        const button = screen.getByRole('button', {name: 'Refresh'});
        expect(button).toHaveAttribute('title', 'Refresh');
        expect(button).toHaveAttribute('aria-label', 'Refresh');
    });
});
