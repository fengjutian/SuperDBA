/**
 * Bundles Monaco with the app instead of letting @monaco-editor/react fetch
 * it from the jsDelivr CDN at runtime, so the editor works offline and the
 * shipped version is the one pinned in package.json.
 *
 * Loaded through ensureMonaco only when an editor or SQL preview needs it.
 *
 * `monaco-editor`'s root entry (`editor.main`) registers every basic-language
 * tokenizer plus the TypeScript/CSS/HTML language services — and their
 * workers, which Vite then emits as separate assets (ts.worker alone is ~7 MB).
 * The app only edits SQL, JSON and plaintext, so import the editor core
 * without the language pack and register just the two languages it uses.
 */
import * as monaco from 'monaco-editor/esm/vs/editor/edcore.main';
import {loader} from '@monaco-editor/react';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import JsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
// Contributions must run after the core above so they attach to the same
// global Monaco instance it creates.
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution';
import 'monaco-editor/esm/vs/language/json/monaco.contribution';
import {installMonacoInputAreaSelectionFix} from './utils/monacoInputArea';

self.MonacoEnvironment = {
    getWorker(_workerId: string, label: string): Worker {
        // SQL and plaintext only need the base editor worker; JSON is the only
        // language service the app uses (config and MCP editors).
        return label === 'json' ? new JsonWorker() : new EditorWorker();
    },
};

loader.config({monaco});

installMonacoInputAreaSelectionFix();

export {monaco};
