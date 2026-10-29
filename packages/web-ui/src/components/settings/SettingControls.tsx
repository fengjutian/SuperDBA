import {createContext, useContext, useId, type ReactNode} from 'react';
import {Minus, Plus} from 'lucide-react';
import clsx from 'clsx';
import {Button, IconButton} from '@/components/ui/Button';

/* ── Section ── */

interface SettingSectionProps {
    title: string;
    icon?: ReactNode;
    description?: string;
    action?: ReactNode;
    children: ReactNode;
}

export function SettingSection({title, icon, description, action, children}: SettingSectionProps) {
    return (
        <div className="mb-8">
            <div className="flex items-center gap-2 mb-1">
                {icon}
                <h3 className="text-xs font-semibold uppercase tracking-wider text-muted">
                    {title}
                </h3>
                {action && <div className="ml-auto">{action}</div>}
            </div>
            {description && <p className="text-xs text-muted mb-3">{description}</p>}
            <div className="border-t border-default pt-1">{children}</div>
        </div>
    );
}

/* ── Row ── */

/** Lets a control inside a SettingRow name itself after the row's label. */
const SettingRowContext = createContext<{labelId: string; descriptionId?: string} | null>(null);

interface SettingRowProps {
    label: string;
    description?: ReactNode;
    children: ReactNode;
    vertical?: boolean;
}

export function SettingRow({label, description, children, vertical}: SettingRowProps) {
    const labelId = useId();
    const descriptionId = useId();
    const context = {labelId, descriptionId: description ? descriptionId : undefined};
    const text = (
        <>
            <div id={labelId} className="text-sm text-primary">
                {label}
            </div>
            {description && (
                <div id={descriptionId} className="text-xs text-muted mt-0.5">
                    {description}
                </div>
            )}
        </>
    );
    if (vertical) {
        return (
            <div className="py-3">
                <div className="mb-2">{text}</div>
                <SettingRowContext.Provider value={context}>{children}</SettingRowContext.Provider>
            </div>
        );
    }
    return (
        <div className="flex items-center justify-between py-3 gap-8">
            <div className="min-w-0">{text}</div>
            <div className="shrink-0">
                <SettingRowContext.Provider value={context}>{children}</SettingRowContext.Provider>
            </div>
        </div>
    );
}

/* ── Toggle ── */

interface SettingToggleProps {
    checked: boolean;
    onChange: (checked: boolean) => void;
    disabled?: boolean;
}

export function SettingToggle({checked, onChange, disabled}: SettingToggleProps) {
    const row = useContext(SettingRowContext);
    return (
        <span
            className={clsx(
                'relative inline-flex items-center w-10 h-6 shrink-0',
                disabled && 'opacity-50',
            )}
        >
            {/* The native checkbox covers the whole switch, so it is both the click target and the named control. */}
            <input
                type="checkbox"
                checked={checked}
                disabled={disabled}
                onChange={(e) => onChange(e.target.checked)}
                aria-labelledby={row?.labelId}
                aria-describedby={row?.descriptionId}
                className={clsx(
                    'peer absolute inset-0 z-10 m-0 h-full w-full appearance-none opacity-0',
                    disabled ? 'cursor-not-allowed' : 'cursor-pointer',
                )}
            />
            <span
                aria-hidden="true"
                className="absolute inset-0 rounded-full bg-base border border-strong transition-colors peer-checked:bg-accent-primary peer-checked:border-accent-primary peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-focus"
            />
            <span
                aria-hidden="true"
                className="relative ml-1 w-4 h-4 rounded-full bg-primary transition-all peer-checked:bg-inverse peer-checked:translate-x-4"
            />
        </span>
    );
}

/* ── ButtonGroup ── */

interface ButtonGroupOption<T> {
    value: T;
    label: string;
}

interface SettingButtonGroupProps<T extends string | number> {
    value: T;
    onChange: (value: T) => void;
    options: Array<ButtonGroupOption<T>>;
    mono?: boolean;
}

export function SettingButtonGroup<T extends string | number>({
    value,
    onChange,
    options,
    mono,
}: SettingButtonGroupProps<T>) {
    return (
        <div className="flex gap-2">
            {options.map((opt) => (
                <Button
                    key={String(opt.value)}
                    variant={value === opt.value ? 'primary' : 'secondary'}
                    aria-pressed={value === opt.value}
                    onClick={() => onChange(opt.value)}
                    className={clsx(
                        mono && 'font-mono',
                        value === opt.value
                            ? 'border-accent-primary shadow-lg shadow-accent-primary/20'
                            : 'border-default text-muted enabled:hover:border-strong',
                    )}
                >
                    {opt.label}
                </Button>
            ))}
        </div>
    );
}

/* ── Slider ── */

interface SettingSliderProps {
    value: number;
    onChange: (value: number) => void;
    min: number;
    max: number;
    step: number;
    formatValue?: (value: number) => string;
    /** Adds −/+ buttons around the value so exact steps are reachable without dragging. */
    stepper?: {decreaseLabel: string; increaseLabel: string};
}

interface StepperButtonProps {
    label: string;
    disabled: boolean;
    onClick: () => void;
    children: ReactNode;
}

function StepperButton({label, disabled, onClick, children}: StepperButtonProps) {
    return (
        <IconButton
            label={label}
            icon={children}
            size="sm"
            variant="ghost"
            disabled={disabled}
            onClick={onClick}
            className={clsx(
                'rounded-md border-default',
                disabled
                    ? 'cursor-not-allowed text-muted opacity-40'
                    : 'text-muted enabled:hover:border-strong enabled:hover:text-primary',
            )}
        />
    );
}

export function SettingSlider({
    value,
    onChange,
    min,
    max,
    step,
    formatValue,
    stepper,
}: SettingSliderProps) {
    const row = useContext(SettingRowContext);
    const display = formatValue ? formatValue(value) : String(value);
    // Round away binary floating point noise (e.g. 1.5 + 0.1) for sub-unit steps.
    const changeBy = (direction: 1 | -1) => {
        const next = Math.min(max, Math.max(min, value + direction * step));
        onChange(Number(next.toFixed(4)));
    };
    return (
        <div
            className={clsx('flex items-center gap-4', stepper ? 'min-w-[280px]' : 'min-w-[200px]')}
        >
            <input
                type="range"
                min={min}
                max={max}
                step={step}
                value={value}
                aria-labelledby={row?.labelId}
                aria-describedby={row?.descriptionId}
                onChange={(e) =>
                    onChange(step < 1 ? parseFloat(e.target.value) : parseInt(e.target.value))
                }
                className="flex-1 h-2 bg-surface-tertiary rounded-lg appearance-none cursor-pointer accent-accent-primary"
            />
            {stepper ? (
                <div className="flex items-center gap-1">
                    <StepperButton
                        label={stepper.decreaseLabel}
                        disabled={value <= min}
                        onClick={() => changeBy(-1)}
                    >
                        <Minus size={13} aria-hidden="true" />
                    </StepperButton>
                    <span className="text-sm font-mono text-primary w-12 text-center tabular-nums">
                        {display}
                    </span>
                    <StepperButton
                        label={stepper.increaseLabel}
                        disabled={value >= max}
                        onClick={() => changeBy(1)}
                    >
                        <Plus size={13} aria-hidden="true" />
                    </StepperButton>
                </div>
            ) : (
                <span className="text-sm font-mono text-primary w-16 text-right">{display}</span>
            )}
        </div>
    );
}

/* ── NumberInput ── */

interface SettingNumberInputProps {
    value: number;
    onChange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    suffix?: string;
    fallback?: number;
}

export function SettingNumberInput({
    value,
    onChange,
    min,
    max,
    step,
    suffix,
    fallback = 0,
}: SettingNumberInputProps) {
    return (
        <div className="flex items-center gap-2">
            <input
                autoCorrect="off"
                autoCapitalize="off"
                autoComplete="off"
                spellCheck={false}
                type="number"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => onChange(parseInt(e.target.value) || fallback)}
                className="bg-base border border-strong rounded px-3 py-2 text-primary w-24 focus:outline-none focus:border-focus transition-colors"
            />
            {suffix && <span className="text-sm text-muted">{suffix}</span>}
        </div>
    );
}
