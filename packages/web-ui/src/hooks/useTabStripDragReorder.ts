import {useCallback, useEffect, useRef, useState} from 'react';
import type {DragEvent, RefObject} from 'react';
import {computeAutoScrollSpeed} from '@/utils/notebookDnd';

interface TabStripDragReorderOptions {
    /** Scroll container that holds the tab elements. */
    tabScrollRef: RefObject<HTMLDivElement | null>;
    /** How many tabs the strip currently renders. */
    tabCount: number;
    /** Apply a reorder: move `fromTabId` to gap index `insertAt`. */
    onReorder: (fromTabId: string, insertAt: number) => void;
}

interface TabStripDragReorder {
    /** Id of the tab being dragged, or `null` when no drag is in progress. */
    dragTabId: string | null;
    /** Pixel offset of the insertion line inside the scroll container. */
    dropIndicatorLeft: number | null;
    handleTabDragStart: (e: DragEvent<HTMLDivElement>, tabId: string) => void;
    handleTabDragEnd: () => void;
    handleTabDragOver: (index: number) => (e: DragEvent<HTMLDivElement>) => void;
    handleTabsContainerDragOver: (e: DragEvent<HTMLDivElement>) => void;
    handleTabsDrop: (e: DragEvent<HTMLDivElement>) => void;
}

/**
 * Tab reordering via native HTML5 drag-and-drop.
 *
 * `dropPos` is a gap index into the tab strip (0 = before the first tab,
 * `tabCount` = after the last); `dropIndicatorLeft` is its pixel offset within
 * the scroll container, used to render the insertion line.
 *
 * The strip also auto-scrolls while the cursor is held near an edge: native
 * DnD does not scroll the container on its own, and no further `dragover`
 * events fire while the pointer is still.
 */
export function useTabStripDragReorder({
    tabScrollRef,
    tabCount,
    onReorder,
}: TabStripDragReorderOptions): TabStripDragReorder {
    const [dragTabId, setDragTabId] = useState<string | null>(null);
    const [dropPos, setDropPos] = useState<number | null>(null);
    const [dropIndicatorLeft, setDropIndicatorLeft] = useState<number | null>(null);
    const autoScrollRafRef = useRef<number | null>(null);
    const autoScrollSpeedRef = useRef(0);

    const stopTabAutoScroll = useCallback(() => {
        if (autoScrollRafRef.current !== null) {
            cancelAnimationFrame(autoScrollRafRef.current);
            autoScrollRafRef.current = null;
        }
        autoScrollSpeedRef.current = 0;
    }, []);

    const stepTabAutoScroll = useCallback(() => {
        const tick = () => {
            const el = tabScrollRef.current;
            const speed = autoScrollSpeedRef.current;
            if (!el || speed === 0) {
                autoScrollRafRef.current = null;
                return;
            }
            el.scrollLeft += speed;
            autoScrollRafRef.current = requestAnimationFrame(tick);
        };
        tick();
    }, [tabScrollRef]);

    useEffect(() => stopTabAutoScroll, [stopTabAutoScroll]);

    const handleTabDragStart = useCallback((e: DragEvent<HTMLDivElement>, tabId: string) => {
        e.dataTransfer.effectAllowed = 'move';
        // WebKitGTK (and Firefox) won't start a drag with an empty data store.
        e.dataTransfer.setData('text/plain', tabId);
        setDragTabId(tabId);
    }, []);

    const handleTabDragEnd = useCallback(() => {
        setDragTabId(null);
        setDropPos(null);
        setDropIndicatorLeft(null);
        stopTabAutoScroll();
    }, [stopTabAutoScroll]);

    const handleTabDragOver = useCallback(
        (index: number) => (e: DragEvent<HTMLDivElement>) => {
            if (!dragTabId) return;
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            // Left half of the tab drops before it, right half drops after it.
            const target = e.currentTarget;
            const rect = target.getBoundingClientRect();
            const after = e.clientX > rect.left + rect.width / 2;
            setDropPos(after ? index + 1 : index);
            setDropIndicatorLeft(
                after ? target.offsetLeft + target.offsetWidth : target.offsetLeft,
            );
        },
        [dragTabId],
    );

    // Drive edge auto-scroll from the scroll container, and fall back to
    // "drop after the last tab" when hovering empty space past the last tab.
    const handleTabsContainerDragOver = useCallback(
        (e: DragEvent<HTMLDivElement>) => {
            if (!dragTabId) return;
            const container = tabScrollRef.current;
            if (!container) return;
            e.preventDefault();
            const containerRect = container.getBoundingClientRect();
            const speed = computeAutoScrollSpeed(
                {top: containerRect.left, bottom: containerRect.right},
                e.clientX,
            );
            autoScrollSpeedRef.current = speed;
            if (speed !== 0 && autoScrollRafRef.current === null) {
                autoScrollRafRef.current = requestAnimationFrame(stepTabAutoScroll);
            }
            if (e.target === container) {
                const lastTab = container.children[tabCount - 1] as HTMLElement | undefined;
                setDropPos(tabCount);
                setDropIndicatorLeft(lastTab ? lastTab.offsetLeft + lastTab.offsetWidth : 0);
            }
        },
        [dragTabId, tabCount, stepTabAutoScroll, tabScrollRef],
    );

    const handleTabsDrop = useCallback(
        (e: DragEvent<HTMLDivElement>) => {
            e.preventDefault();
            stopTabAutoScroll();
            const fromTabId = dragTabId;
            const insertAt = dropPos;
            setDragTabId(null);
            setDropPos(null);
            setDropIndicatorLeft(null);
            if (fromTabId === null || insertAt === null) return;
            onReorder(fromTabId, insertAt);
        },
        [dragTabId, dropPos, stopTabAutoScroll, onReorder],
    );

    return {
        dragTabId,
        dropIndicatorLeft,
        handleTabDragStart,
        handleTabDragEnd,
        handleTabDragOver,
        handleTabsContainerDragOver,
        handleTabsDrop,
    };
}
