import type {ButtonHTMLAttributes, ReactNode} from 'react';
import {Loader2} from 'lucide-react';
import clsx from 'clsx';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'success' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: ButtonVariant;
    size?: ButtonSize;
    loading?: boolean;
    icon?: ReactNode;
}

const VARIANT_CLASS: Record<ButtonVariant, string> = {
    primary:
        'border-transparent bg-accent-primary text-inverse enabled:hover:bg-accent-primary/90',
    secondary:
        'border-strong bg-base text-secondary enabled:hover:bg-surface-secondary enabled:hover:text-primary',
    ghost: 'border-transparent bg-transparent text-secondary enabled:hover:bg-surface-hover enabled:hover:text-primary',
    success:
        'border-transparent bg-accent-success text-on-accent-success enabled:hover:bg-accent-success/90',
    danger:
        'border-transparent bg-accent-error text-on-accent-error enabled:hover:bg-accent-error/90',
};

const SIZE_CLASS: Record<ButtonSize, string> = {
    sm: 'min-h-7 gap-1.5 px-2.5 py-1 text-xs',
    md: 'min-h-9 gap-2 px-4 py-2 text-sm',
    lg: 'min-h-11 gap-2.5 px-5 py-2.5 text-base',
};

export function Button({
    variant = 'secondary',
    size = 'md',
    loading = false,
    icon,
    disabled,
    type = 'button',
    className,
    children,
    ...props
}: ButtonProps) {
    return (
        <button
            {...props}
            type={type}
            disabled={disabled || loading}
            aria-busy={loading || undefined}
            className={clsx(
                'inline-flex items-center justify-center rounded-lg border font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:cursor-not-allowed disabled:opacity-50',
                VARIANT_CLASS[variant],
                SIZE_CLASS[size],
                className,
            )}
        >
            {loading ? (
                <Loader2 className="size-4 shrink-0 animate-spin" aria-hidden="true" />
            ) : (
                icon
            )}
            {children}
        </button>
    );
}

interface IconButtonProps extends Omit<ButtonProps, 'children' | 'icon'> {
    label: string;
    icon: ReactNode;
}

const ICON_SIZE_CLASS: Record<ButtonSize, string> = {
    sm: 'size-7 p-1.5',
    md: 'size-9 p-2',
    lg: 'size-11 p-2.5',
};

export function IconButton({label, icon, size = 'md', className, ...props}: IconButtonProps) {
    return (
        <Button
            {...props}
            size={size}
            icon={<span aria-hidden="true">{icon}</span>}
            className={clsx(ICON_SIZE_CLASS[size], className)}
            aria-label={label}
            title={props.title ?? label}
        />
    );
}
