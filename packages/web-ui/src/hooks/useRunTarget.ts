import {useCallback, useState} from 'react';
import {useTranslation} from 'react-i18next';
import type {Tab} from '@/types/editor';
import {resolveRunTarget, type RunContext} from '@/utils/runTarget';

interface RunTargetOptions {
    tab: Tab | null;
    /** Table tabs always run their whole (generated) query. */
    isTableTab: boolean;
    /** Setting: with nothing selected, run only the statement under the cursor. */
    runStatementUnderCursor: boolean;
    isMac: boolean;
}

/**
 * What the Run button would execute right now, and the label that says so.
 *
 * With no selection a pasted multi-statement script only runs the statement
 * under the cursor, and a button that just reads "Run" gives no hint that the
 * rest was skipped — so the label switches between Run / Run Statement /
 * Run Selection, and the tooltip surfaces the whole-script escape hatch
 * exactly when the button would run one statement out of several.
 *
 * `handleRunButton` dispatches on the same `resolveRunTarget` result.
 */
export function useRunTarget({tab, isTableTab, runStatementUnderCursor, isMac}: RunTargetOptions) {
    const {t} = useTranslation();
    // Reported by the editor on every cursor, selection and content change.
    const [runContext, setRunContext] = useState<RunContext>({
        hasSelection: false,
        statementCount: 0,
    });

    // The label only distinguishes ≤1 from >1 statements, so bail out of count
    // changes that can't affect it (typing a ';' mid-way through a script) —
    // each accepted update re-renders the whole editor page.
    const handleRunContextChange = useCallback((context: RunContext) => {
        setRunContext((previous) =>
            previous.hasSelection === context.hasSelection &&
            previous.statementCount > 1 === context.statementCount > 1
                ? previous
                : context,
        );
    }, []);

    // Query-builder tabs always run their generated query; `runContext` also
    // still describes the last SQL editor tab on a builder tab.
    const runTarget =
        isTableTab || tab?.type === 'query_builder'
            ? 'whole'
            : resolveRunTarget({
                  hasSelection: runContext.hasSelection,
                  statementCount: runContext.statementCount,
                  runStatementUnderCursor,
              });

    const runLabel =
        runTarget === 'selection'
            ? t('editor.runSelection')
            : runTarget === 'statement'
              ? t('editor.runStatement')
              : t('editor.run');

    const runButtonBase = `${runLabel} (${isMac ? 'Cmd+Enter' : 'Ctrl+Enter'})`;
    const runTitle =
        runTarget === 'statement'
            ? `${runButtonBase} · ${t('editor.runAll')} (${isMac ? 'Cmd+Shift+Enter' : 'Ctrl+Shift+Enter'})`
            : runButtonBase;

    return {
        runContext,
        handleRunContextChange,
        runTarget,
        runLabel,
        runButtonBase,
        runTitle,
    };
}
