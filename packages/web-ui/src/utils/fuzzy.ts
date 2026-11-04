import Fuse from 'fuse.js';

interface CachedIndex {
    getText: unknown;
    length: number;
    fuse: Fuse<string>;
}

/**
 * Fuse builds its index from every item's text, which is too expensive to redo
 * on each keystroke in the sidebar filters. Callers pass the same array
 * reference between renders (render data comes from React state), so the index
 * is kept per array and rebuilt only when the array or its accessor changes.
 */
const indexCache = new WeakMap<object, CachedIndex>();

export function fuzzyFilter<T>(items: T[], query: string, getText: (item: T) => string): T[] {
    const trimmed = query.trim();
    if (trimmed === '') return items;

    let cached = indexCache.get(items);
    if (!cached || cached.getText !== getText || cached.length !== items.length) {
        cached = {
            getText,
            length: items.length,
            fuse: new Fuse(items.map(getText), {threshold: 0.4, ignoreLocation: true}),
        };
        indexCache.set(items, cached);
    }

    return cached.fuse.search(trimmed).map((result) => items[result.refIndex]);
}
