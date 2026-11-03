// @vitest-environment jsdom
import {renderHook, act} from '@testing-library/react';
import {describe, it, expect} from 'vitest';
import type {Tab} from '@/types/editor';
import {useRunTarget} from '@/hooks/useRunTarget';

const makeTab = (overrides: Partial<Tab> = {}): Tab => ({
    id: 'tab-a',
    title: 'Tab A',
    type: 'console',
    query: 'select 1',
    result: null,
    error: '',
    executionTime: null,
    page: 1,
    activeTable: null,
    pkColumns: null,
    connectionId: 'conn-1',
    ...overrides,
});

interface Options {
    tab?: Tab | null;
    isTableTab?: boolean;
    runStatementUnderCursor?: boolean;
    isMac?: boolean;
}

const setup = ({
    tab = makeTab(),
    isTableTab = false,
    runStatementUnderCursor = true,
    isMac = false,
}: Options = {}) =>
    renderHook(() => useRunTarget({tab, isTableTab, runStatementUnderCursor, isMac}));

describe('useRunTarget', () => {
    it('runs the whole tab when nothing is selected and the script is a single statement', () => {
        const {result} = setup();

        expect(result.current.runTitle).toBe(result.current.runButtonBase);
        expect(result.current.runButtonBase).toContain('Ctrl+Enter');
    });

    it('labels the selection variant differently from the plain one', () => {
        const plain = setup().result.current.runLabel;
        const {result} = setup();

        act(() => {
            result.current.handleRunContextChange({hasSelection: true, statementCount: 3});
        });

        expect(result.current.runLabel).not.toBe(plain);
    });

    it('switches to the statement variant once the script has several statements', () => {
        const plain = setup().result.current.runLabel;
        const {result} = setup();

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 3});
        });

        expect(result.current.runLabel).not.toBe(plain);
        // The escape hatch is only advertised while the button would run one statement.
        expect(result.current.runTitle).toContain('editor.runAll');
    });

    it('keeps running the whole script when the cursor-scoped setting is off', () => {
        const plain = setup().result.current.runLabel;
        const {result} = setup({runStatementUnderCursor: false});

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 3});
        });

        expect(result.current.runLabel).toBe(plain);
        expect(result.current.runTitle).toBe(result.current.runButtonBase);
    });

    it('always runs a table tab whole', () => {
        const plain = setup().result.current.runLabel;
        const {result} = setup({isTableTab: true});

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 3});
        });

        expect(result.current.runLabel).toBe(plain);
    });

    it('always runs a query-builder tab whole', () => {
        const plain = setup().result.current.runLabel;
        const {result} = setup({tab: makeTab({type: 'query_builder'})});

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 3});
        });

        expect(result.current.runLabel).toBe(plain);
    });

    it('uses the Cmd shortcut on macOS', () => {
        const {result} = setup({isMac: true});

        expect(result.current.runButtonBase).toContain('Cmd+Enter');
        expect(result.current.runButtonBase).not.toContain('Ctrl+Enter');
    });

    it('ignores statement-count changes that cannot affect the label', () => {
        const {result} = setup();

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 1});
        });
        const afterSingle = result.current.runLabel;

        act(() => {
            result.current.handleRunContextChange({hasSelection: false, statementCount: 1});
        });

        expect(result.current.runLabel).toBe(afterSingle);
    });
});
