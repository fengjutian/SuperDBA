import {describe, expect, it} from 'vitest';
import {act, render} from '@testing-library/react';
import {useStableActions} from '@/hooks/useStableActions';

/** Renders the hook and exposes every result it produced to the test. */
function renderActions(
    buildActions: (suffix: string) => Record<string, (...args: never[]) => string>,
) {
    const seen: Array<Record<string, (...args: never[]) => string>> = [];
    const Probe = ({suffix}: {suffix: string}) => {
        seen.push(useStableActions(buildActions(suffix)));
        return null;
    };
    const view = render(<Probe suffix="a" />);
    return {seen, rerender: (suffix: string) => view.rerender(<Probe suffix={suffix} />)};
}

describe('useStableActions', () => {
    it('keeps the action object identical across renders', () => {
        const {seen, rerender} = renderActions((suffix) => ({
            greet: () => `hello ${suffix}`,
        }));

        rerender('b');

        expect(seen).toHaveLength(2);
        expect(seen[1]).toBe(seen[0]);
    });

    it('always calls the latest closure', () => {
        const {seen, rerender} = renderActions((suffix) => ({
            greet: () => `hello ${suffix}`,
        }));

        rerender('b');

        let result = '';
        act(() => {
            result = seen[0].greet();
        });
        expect(result).toBe('hello b');
    });

    it('forwards every argument', () => {
        const calls: unknown[][] = [];
        let record: ((...args: unknown[]) => void) | null = null;
        const Probe = () => {
            record = useStableActions({record: (...args: unknown[]) => calls.push(args)}).record;
            return null;
        };

        render(<Probe />);
        act(() => {
            record!('a', 1);
        });

        expect(calls).toEqual([['a', 1]]);
    });
});
