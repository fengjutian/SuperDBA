import {useLayoutEffect, useRef, useState} from 'react';

type AnyAction = (...args: never[]) => unknown;

/**
 * Keeps the identity of a bag of callbacks stable while always invoking the
 * latest closure.
 *
 * Context providers re-render far more often than the state they publish
 * changes (a toast, a theme switch, a settings update), and callbacks created
 * during those renders get a new identity each time. Once they are part of a
 * memoized context value they invalidate it — and with it every consumer. The
 * wrappers built here never change; each call reads the newest closure.
 */
export function useStableActions<T extends Record<string, AnyAction>>(actions: T): T {
    const latest = useRef(actions);
    // Written after commit rather than during render: a render-phase ref write
    // is a side effect React may replay or discard (and `react-hooks/refs`
    // rejects it). An action is only ever called from an event handler, effect
    // or timer, i.e. after this has run.
    useLayoutEffect(() => {
        latest.current = actions;
    });

    const [stable] = useState(() => {
        const wrappers: Record<string, AnyAction> = {};
        for (const key of Object.keys(actions)) {
            wrappers[key] = (...args: never[]) => latest.current[key](...args);
        }
        return wrappers as T;
    });

    return stable;
}
