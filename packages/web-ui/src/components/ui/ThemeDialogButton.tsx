import type {ComponentPropsWithRef, ReactNode} from 'react';
import {Button} from './Button';

interface ThemeDialogButtonProps extends ComponentPropsWithRef<'button'> {
    variant?: 'primary' | 'secondary' | 'danger';
    busy?: boolean;
    icon?: ReactNode;
}

/** Shared action styling for theme workflows, including pending and destructive states. */
export function ThemeDialogButton({
    variant = 'secondary',
    busy = false,
    disabled,
    icon,
    children,
    className,
    ...props
}: ThemeDialogButtonProps) {
    return (
        <Button
            {...props}
            variant={variant}
            loading={busy}
            disabled={disabled}
            icon={icon}
            className={className}
        >
            {children}
        </Button>
    );
}
