import {useCallback, useState} from 'react';
import type {RefObject} from 'react';
import {useClickOutside} from '@/hooks/useClickOutside';

/** The toolbar dropdowns, which share one "at most one is open" state. */
export type EditorMenu = 'run' | 'save' | 'export' | 'database';

interface EditorMenuRefs {
    run: RefObject<HTMLDivElement | null>;
    save: RefObject<HTMLDivElement | null>;
    export: RefObject<HTMLDivElement | null>;
    database: RefObject<HTMLDivElement | null>;
}

/**
 * Single source of truth for the toolbar dropdowns.
 *
 * Each dropdown used to own a boolean, so two menus could be open at once and
 * their outside-click listeners would fight over which one to close. One
 * `openMenu` makes that impossible.
 *
 * Outside clicks are still per-menu: only the open menu's listener reacts, so
 * clicking inside an unopened menu's element can never close another one.
 */
export function useEditorMenus(refs: EditorMenuRefs) {
    const [openMenu, setOpenMenu] = useState<EditorMenu | null>(null);

    const toggle = useCallback((menu: EditorMenu) => {
        setOpenMenu((previous) => (previous === menu ? null : menu));
    }, []);

    const close = useCallback((menu: EditorMenu) => {
        setOpenMenu((previous) => (previous === menu ? null : previous));
    }, []);

    const isOpen = useCallback((menu: EditorMenu) => openMenu === menu, [openMenu]);

    useClickOutside(refs.run, () => close('run'), openMenu === 'run');
    useClickOutside(refs.save, () => close('save'), openMenu === 'save');
    useClickOutside(refs.export, () => close('export'), openMenu === 'export');
    useClickOutside(refs.database, () => close('database'), openMenu === 'database');

    return {openMenu, isOpen, toggle, close};
}
