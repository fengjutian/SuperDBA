// @vitest-environment jsdom
import {renderHook, act} from '@testing-library/react';
import {describe, it, expect, afterEach} from 'vitest';
import {useEditorMenus} from '@/hooks/useEditorMenus';

const setup = () => {
    const runElement = document.createElement('div');
    const saveElement = document.createElement('div');
    document.body.append(runElement, saveElement);

    const {result} = renderHook(() =>
        useEditorMenus({
            run: {current: runElement},
            save: {current: saveElement},
            export: {current: null},
            database: {current: null},
        }),
    );

    return {result, runElement, saveElement};
};

afterEach(() => {
    document.body.innerHTML = '';
});

describe('useEditorMenus', () => {
    it('toggles a menu open and closed', () => {
        const {result} = setup();

        act(() => {
            result.current.toggle('run');
        });
        expect(result.current.isOpen('run')).toBe(true);

        act(() => {
            result.current.toggle('run');
        });
        expect(result.current.isOpen('run')).toBe(false);
    });

    it('keeps at most one menu open', () => {
        const {result} = setup();

        act(() => {
            result.current.toggle('run');
        });
        act(() => {
            result.current.toggle('save');
        });

        expect(result.current.isOpen('save')).toBe(true);
        expect(result.current.isOpen('run')).toBe(false);
        expect(result.current.openMenu).toBe('save');
    });

    it('closes only the menu it is told to close', () => {
        const {result} = setup();

        act(() => {
            result.current.toggle('save');
        });
        act(() => {
            result.current.close('run');
        });
        expect(result.current.isOpen('save')).toBe(true);

        act(() => {
            result.current.close('save');
        });
        expect(result.current.openMenu).toBeNull();
    });

    it('closes the open menu on an outside mousedown', () => {
        const {result} = setup();

        act(() => {
            result.current.toggle('run');
        });
        act(() => {
            document.body.dispatchEvent(new MouseEvent('mousedown', {bubbles: true}));
        });

        expect(result.current.openMenu).toBeNull();
    });

    it('stays open while the mousedown lands inside the menu', () => {
        const {result, runElement} = setup();

        act(() => {
            result.current.toggle('run');
        });
        act(() => {
            runElement.dispatchEvent(new MouseEvent('mousedown', {bubbles: true}));
        });

        expect(result.current.isOpen('run')).toBe(true);
    });
});
