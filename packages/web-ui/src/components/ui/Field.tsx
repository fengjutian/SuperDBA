import {cloneElement, isValidElement} from 'react';
import type {ReactElement, ReactNode} from 'react';
import clsx from 'clsx';

interface FieldProps {
    label: ReactNode;
    htmlFor: string;
    children: ReactNode;
    description?: ReactNode;
    error?: ReactNode;
    required?: boolean;
    className?: string;
}

interface FieldControlProps {
    'aria-describedby'?: string;
    'aria-invalid'?: boolean;
    'aria-required'?: boolean;
}

export function Field({
    label,
    htmlFor,
    children,
    description,
    error,
    required = false,
    className,
}: FieldProps) {
    const descriptionId = description ? `${htmlFor}-description` : undefined;
    const errorId = error ? `${htmlFor}-error` : undefined;
    const describedBy = [descriptionId, errorId].filter(Boolean).join(' ') || undefined;
    const control = isValidElement<FieldControlProps>(children)
        ? cloneElement(children as ReactElement<FieldControlProps>, {
              'aria-describedby': describedBy,
              'aria-invalid': error ? true : undefined,
              'aria-required': required || undefined,
          })
        : children;

    return (
        <div className={clsx('space-y-1.5', className)}>
            <label htmlFor={htmlFor} className="block text-sm font-medium text-primary">
                {label}
                {required && (
                    <span className="ml-1 text-accent-error" aria-hidden="true">
                        *
                    </span>
                )}
            </label>
            {description && (
                <p id={descriptionId} className="text-xs text-muted">
                    {description}
                </p>
            )}
            {control}
            {error && (
                <p id={errorId} role="alert" className="text-xs text-accent-error">
                    {error}
                </p>
            )}
        </div>
    );
}
