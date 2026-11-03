// @vitest-environment jsdom
import {renderHook, act} from '@testing-library/react';
import {describe, it, expect, vi} from 'vitest';
import type {DragEvent, RefObject} from 'react';
import {useTabStripDragReorder} from '@/hooks/useTabStripDragReorder';

interface TabGeometry {
    left: number;
    width: number;
}

const makeTabElement = ({left, width}: TabGeometry): HTMLDivElement => {
    const element = document.createElement('div');
    element.getBoundingClientRect = () =>
        ({
            left,
            width,
            right: left + width,
            top: 0,
            bottom: 0,
            height: 0,
            x: left,
            y: 0,
            toJSON: () => ({}),
        }) as DOMRect;
    Object.defineProperty(element, 'offsetLeft', {value: left});
    Object.defineProperty(element, 'offsetWidth', {value: width});
    return element;
};

const makeDragEvent = (init: {
    clientX?: number;
    currentTarget?: HTMLElement;
    target?: EventTarget | null;
}): DragEvent<HTMLDivElement> =>
    ({
        clientX: init.clientX ?? 0,
        currentTarget: init.currentTarget,
        target: init.target ?? null,
        preventDefault: vi.fn(),
        dataTransfer: {
            effectAllowed: '',
            dropEffect: '',
            setData: vi.fn(),
        },
    }) as unknown as DragEvent<HTMLDivElement>;

const setup = (tabCount = 3) => {
    const container = document.createElement('div');
    const tabScrollRef = {current: container} as RefObject<HTMLDivElement | null>;
    const onReorder = vi.fn();

    const {result} = renderHook(() => useTabStripDragReorder({tabScrollRef, tabCount, onReorder}));

    return {result, container, onReorder};
};

describe('useTabStripDragReorder', () => {
    it('records the dragged tab and clears it when the drag ends', () => {
        const {result} = setup();

        act(() => {
            result.current.handleTabDragStart(
                makeDragEvent({currentTarget: document.createElement('div')}),
                'tab-b',
            );
        });
        expect(result.current.dragTabId).toBe('tab-b');

        act(() => {
            result.current.handleTabDragEnd();
        });
        expect(result.current.dragTabId).toBeNull();
        expect(result.current.dropIndicatorLeft).toBeNull();
    });

    it('writes the dragged id to the data store so WebKitGTK starts the drag', () => {
        const {result} = setup();
        const event = makeDragEvent({currentTarget: document.createElement('div')});

        act(() => {
            result.current.handleTabDragStart(event, 'tab-a');
        });

        expect(event.dataTransfer.setData).toHaveBeenCalledWith('text/plain', 'tab-a');
    });

    it('drops before the tab when the cursor is on its left half', () => {
        const {result, onReorder} = setup();
        const tab = makeTabElement({left: 100, width: 80});

        act(() => {
            result.current.handleTabDragStart(makeDragEvent({currentTarget: tab}), 'tab-c');
        });
        act(() => {
            result.current.handleTabDragOver(1)(makeDragEvent({clientX: 110, currentTarget: tab}));
        });
        expect(result.current.dropIndicatorLeft).toBe(100);

        act(() => {
            result.current.handleTabsDrop(
                makeDragEvent({currentTarget: document.createElement('div')}),
            );
        });
        expect(onReorder).toHaveBeenCalledWith('tab-c', 1);
    });

    it('drops after the tab when the cursor is on its right half', () => {
        const {result, onReorder} = setup();
        const tab = makeTabElement({left: 100, width: 80});

        act(() => {
            result.current.handleTabDragStart(makeDragEvent({currentTarget: tab}), 'tab-a');
        });
        act(() => {
            result.current.handleTabDragOver(1)(makeDragEvent({clientX: 170, currentTarget: tab}));
        });
        expect(result.current.dropIndicatorLeft).toBe(180);

        act(() => {
            result.current.handleTabsDrop(makeDragEvent({}));
        });
        expect(onReorder).toHaveBeenCalledWith('tab-a', 2);
    });

    it('falls back to the end of the strip when hovering past the last tab', () => {
        const {result, container, onReorder} = setup(3);
        container.appendChild(makeTabElement({left: 200, width: 30}));
        container.appendChild(makeTabElement({left: 230, width: 30}));
        container.appendChild(makeTabElement({left: 260, width: 60}));

        act(() => {
            result.current.handleTabDragStart(makeDragEvent({}), 'tab-a');
        });
        act(() => {
            result.current.handleTabsContainerDragOver(
                makeDragEvent({clientX: 900, target: container}),
            );
        });
        expect(result.current.dropIndicatorLeft).toBe(320);

        act(() => {
            result.current.handleTabsDrop(makeDragEvent({}));
        });
        expect(onReorder).toHaveBeenCalledWith('tab-a', 3);
    });

    it('ignores a drop that no drag started', () => {
        const {result, onReorder} = setup();

        act(() => {
            result.current.handleTabsDrop(makeDragEvent({}));
        });

        expect(onReorder).not.toHaveBeenCalled();
    });
});
