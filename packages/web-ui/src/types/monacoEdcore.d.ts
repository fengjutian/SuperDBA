/**
 * Type surface for the Monaco entry points `src/monacoLoader.ts` imports.
 *
 * `monaco-editor` ships `editor.main.d.ts` / `editor.api.d.ts` only; the
 * language-free `esm/vs/editor/edcore.main.js` and the two language
 * contributions are side-effect modules whose `.d.ts` files are just
 * `export {}`. Declaring them here keeps the app on the lean entry point
 * without pulling `editor.main` (which registers every language) in for types.
 */
declare module 'monaco-editor/esm/vs/editor/edcore.main' {
    export * from 'monaco-editor';
}

declare module 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution';

declare module 'monaco-editor/esm/vs/language/json/monaco.contribution';
