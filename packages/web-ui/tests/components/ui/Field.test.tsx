import {render, screen} from '@testing-library/react';
import {describe, expect, it} from 'vitest';
import {Field} from '@/components/ui/Field';

describe('Field', () => {
    it('associates the label with its control', () => {
        render(
            <Field label="Database name" htmlFor="database-name">
                <input id="database-name" />
            </Field>,
        );

        expect(screen.getByLabelText('Database name')).toHaveAttribute('id', 'database-name');
    });

    it('renders supporting text and an announced error', () => {
        render(
            <Field
                label="Port"
                htmlFor="port"
                description="The server port"
                error="Enter a valid port"
                required
            >
                <input id="port" />
            </Field>,
        );

        expect(screen.getByText('The server port')).toHaveAttribute('id', 'port-description');
        expect(screen.getByRole('alert')).toHaveAttribute('id', 'port-error');
        const input = screen.getByLabelText('Port');
        expect(input).toHaveAttribute('aria-required', 'true');
        expect(input).toHaveAttribute('aria-invalid', 'true');
        expect(input).toHaveAttribute('aria-describedby', 'port-description port-error');
    });
});
