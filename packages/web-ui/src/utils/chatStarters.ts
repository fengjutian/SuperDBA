/**
 * Starter questions for the empty chat panel.
 *
 * Two sources feed the same list: questions derived locally from the table
 * names that are already loaded (instant, free, never blank) and questions the
 * model writes for the same tables (nicer, but one provider round trip). The
 * panel renders the local set first and only swaps in the model set when the
 * call succeeds, so an offline provider degrades to slightly plainer wording.
 */

/** Tables sent to the model; long schemas keep their first names. */
const MAX_PROMPT_TABLES = 40;
/** The panel shows three buttons. */
const MAX_GENERATED = 3;
/** A longer question would not fit on one button line. */
const MAX_GENERATED_CHARS = 120;

/** An i18n key plus the literal English text, so a missing locale still reads. */
export interface StarterPrompt {
    key: string;
    fallback: string;
    params?: Record<string, string | number>;
}

/** Trims, de-duplicates (case-insensitively), and caps a table list. */
export function selectPromptTables(names: readonly string[]): string[] {
    const seen = new Set<string>();
    const selected: string[] = [];
    for (const raw of names) {
        const name = raw.trim();
        if (!name) continue;
        const key = name.toLowerCase();
        if (seen.has(key)) continue;
        seen.add(key);
        selected.push(name);
        if (selected.length === MAX_PROMPT_TABLES) break;
    }
    return selected;
}

/**
 * Three questions derived only from the active scope's table names.
 *
 * The result always has three entries: the empty state must never flicker
 * between one, two, or three placeholders while the schema loads.
 */
export function buildLocalStarters(names: readonly string[]): StarterPrompt[] {
    const tables = selectPromptTables(names);
    const first = tables[0];
    const second = tables[1];

    if (!first) {
        return [
            {key: 'chat.local.noTables', fallback: 'What tables are in this database?'},
            {key: 'chat.starter.slow', fallback: 'Why might my last query be slow?'},
            {key: 'chat.starter.indexes', fallback: 'Which indexes are missing?'},
        ];
    }

    if (!second) {
        return [
            {
                key: 'chat.local.singleTable',
                fallback: 'What does the "{{table}}" table hold?',
                params: {table: first},
            },
            {
                key: 'chat.local.singleColumns',
                fallback: 'What columns does "{{table}}" have?',
                params: {table: first},
            },
            {
                key: 'chat.local.indexes',
                fallback: 'Which indexes does "{{table}}" lack?',
                params: {table: first},
            },
        ];
    }

    return [
        {
            key: 'chat.local.manyTables',
            fallback: 'Of these {{count}} tables, which should I look at first?',
            params: {count: tables.length},
        },
        {
            key: 'chat.local.indexes',
            fallback: 'Which indexes does "{{table}}" lack?',
            params: {table: first},
        },
        {
            key: 'chat.local.relations',
            fallback: 'How are "{{table}}" and "{{table2}}" related?',
            params: {table: first, table2: second},
        },
    ];
}

/** Stable cache identity: same provider, model, language, connection, tables. */
export function starterCacheKey(
    provider: string,
    model: string,
    language: string,
    connectionId: string | null,
    names: readonly string[],
): string {
    return [provider, model, language, connectionId ?? '', tableSignature(names)].join('|');
}

/** Cheap, order-independent fingerprint of the table list. */
export function tableSignature(names: readonly string[]): string {
    const sorted = Array.from(
        new Set(names.map((name) => name.trim().toLowerCase()).filter(Boolean)),
    ).sort();
    let hash = 5381;
    for (const name of sorted) {
        for (let index = 0; index < name.length; index += 1) {
            hash = ((hash << 5) + hash + name.charCodeAt(index)) | 0;
        }
    }
    return `${sorted.length}:${(hash >>> 0).toString(36)}`;
}

/**
 * Accepts whatever a provider produced: a JSON array of strings, markdown
 * fences, numbering, or trailing prose. Anything unusable is dropped rather
 * than rendered as a button.
 */
export function sanitizeGeneratedStarters(values: unknown): string[] {
    if (!Array.isArray(values)) return [];
    const seen = new Set<string>();
    const starters: string[] = [];
    for (const value of values) {
        if (typeof value !== 'string') continue;
        const question = trimStarterText(value);
        if (!question) continue;
        const key = question.toLowerCase();
        if (seen.has(key)) continue;
        seen.add(key);
        starters.push(question);
        if (starters.length === MAX_GENERATED) break;
    }
    return starters;
}

function trimStarterText(value: string): string {
    let text = value.trim();
    // Providers often answer with `1. …`, `- …`, or quoted strings.
    text = text.replace(/^(?:[-*•>#]\s*)+/, '');
    text = text.replace(/^\d+\s*[.):、]\s*/, '');
    text = text.replace(/^["'`「]+|["'`」]+$/g, '');
    text = text.replace(/\s+/g, ' ').trim();
    if (text.length > MAX_GENERATED_CHARS) {
        text = `${text.slice(0, MAX_GENERATED_CHARS - 1)}…`;
    }
    return text;
}

const starterCache = new Map<string, string[]>();

/** Model questions already fetched for this key, if any. */
export function cachedStarters(key: string): string[] | null {
    return starterCache.get(key) ?? null;
}

/** Memoises the model questions for this key so reopening the panel is free. */
export function rememberStarters(key: string, values: readonly string[]): void {
    const starters = sanitizeGeneratedStarters(values);
    if (starters.length > 0) starterCache.set(key, starters);
}

/** Test hook: forget everything fetched so far. */
export function clearStarterCache(): void {
    starterCache.clear();
}
