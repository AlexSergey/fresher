import {
  type JSX,
  type ReactElement,
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import { type Position, useFlipList, useMotion } from '../../motion';
import { STAGGER } from '../../tokens/motion';
import { moveFocus, rowsInView, scrollToShow, withOverscan } from './range';
import styles from './virtual-list.module.css';

export type VirtualListProps<T> = {
  readonly count: number;
  // The item at an index, or undefined while its page is not loaded.
  readonly getItem: (index: number) => T | undefined;
  // A stable key per item: the focus and the moves follow it.
  readonly getKey: (index: number) => string;
  // The list's accessible name.
  readonly label: string;
  // The rows rendered changed (those in view and the overscan): load their pages.
  readonly onRangeChange?: (first: number, last: number) => void;
  // Space on a row; a row marked `data-disabled` (a disabled `ListRow`) is not toggled.
  readonly onToggle?: (index: number) => void;
  // Renders one row, spreading `row` on its root (`ListRow` takes it); `item` is undefined while its page loads, and
  // the caller renders a skeleton row.
  readonly renderRow: (index: number, item: T | undefined, row: VirtualRowProps) => ReactElement;
  // Pixels; every row is this tall.
  readonly rowHeight: number;
};

// What a row's root carries: its place in the whole list, and whether it is the list's one tab stop.
export type VirtualRowProps = {
  readonly 'aria-posinset': number;
  readonly 'aria-setsize': number;
  readonly tabIndex: -1 | 0;
};

type Focus = { readonly index: number; readonly key: null | string };

// Rows rendered above and below the view.
const OVERSCAN = 6;

// Every index from `first` to `last`; none when `last` is before `first`.
const indicesOf = (first: number, last: number): readonly number[] =>
  Array.from({ length: Math.max(0, last - first + 1) }, (_, offset) => first + offset);

// The slot of the row an event came from, and its index.
const slotOf = (target: EventTarget | null): HTMLElement | null =>
  target instanceof Element ? target.closest<HTMLElement>('[data-index]') : null;

// A row's root: the element the caller rendered inside the slot's motion layer.
const rowOf = (slot: Element): Element | null => slot.firstElementChild?.firstElementChild ?? null;

// A long list that renders only the rows in view and an overscan of 6 around them, each placed by its index. One tab
// stop, moved by the arrows, Home, End and Page Up and Down; Space toggles the focused row; a key on a control inside a
// row is the control's. A row whose index changes glides from its old place; rows new in view (not scrolled into it)
// fade and rise in with the stagger.
export const VirtualList = <T,>({
  count,
  getItem,
  getKey,
  label,
  onRangeChange,
  onToggle,
  renderRow,
  rowHeight,
}: VirtualListProps<T>): JSX.Element => {
  const motion = useMotion();
  const viewportRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<null | number>(null);
  const focusPendingRef = useRef(false);
  // A row of the list has the focus (or had it when it was removed).
  const hasFocusRef = useRef(false);
  const seenRef = useRef<ReadonlySet<string>>(new Set());
  const lastScrollRef = useRef(0);
  const [scrollTop, setScrollTop] = useState(0);
  const [height, setHeight] = useState(0);
  const [focus, setFocus] = useState<Focus>({ index: 0, key: null });

  const visible = rowsInView({ count, height, rowHeight, scrollTop });
  const rendered = visible === null ? null : withOverscan(visible, OVERSCAN, count);
  const first = rendered?.first ?? 0;
  const last = rendered?.last ?? -1;
  const visibleFirst = visible?.first ?? 0;
  const visibleLast = visible?.last ?? -1;
  const keys = useMemo(() => indicesOf(first, last).map(getKey), [first, last, getKey]);
  const positions = useMemo(
    () => new Map<string, Position>(keys.map((key, offset) => [key, { x: 0, y: (first + offset) * rowHeight }])),
    [keys, first, rowHeight],
  );
  const flip = useFlipList(keys, { positions });

  // The focused row follows its key through a re-render; a key gone from view leaves the focus at its index.
  const focused = ((): number => {
    const index = Math.min(focus.index, count - 1);

    if (index < 0 || focus.key === null || getKey(index) === focus.key) {
      return index;
    }

    const moved = keys.indexOf(focus.key);

    return moved === -1 ? index : first + moved;
  })();
  const tabStop = focused >= first && focused <= last ? focused : (visible?.first ?? -1);
  const pageSize = Math.max(1, Math.floor(height / rowHeight));

  useLayoutEffect(() => {
    const viewport = viewportRef.current;

    if (viewport === null) {
      return;
    }

    setHeight(viewport.clientHeight);
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setHeight(entry.contentRect.height);
      }
    });

    observer.observe(viewport);

    return (): void => {
      observer.disconnect();
    };
  }, []);

  const onScroll = useEffectEvent((): void => {
    if (frameRef.current !== null) {
      return;
    }

    // One read of the offset per frame, however many scroll events come.
    frameRef.current = requestAnimationFrame(() => {
      frameRef.current = null;
      setScrollTop(viewportRef.current?.scrollTop ?? 0);
    });
  });

  const onKeyDown = useEffectEvent((event: KeyboardEvent): void => {
    const viewport = viewportRef.current;
    const slot = slotOf(event.target);

    if (viewport === null || count === 0) {
      return;
    }

    const row = slot === null ? null : rowOf(slot);

    // Only the row itself: a control inside it handles its own keys.
    if (row === null || row !== event.target) {
      return;
    }

    if (event.key === ' ') {
      event.preventDefault();

      if (slot !== null && !row.hasAttribute('data-disabled')) {
        onToggle?.(Number(slot.dataset.index));
      }

      return;
    }

    const target = moveFocus(event.key, focused, count, pageSize);

    if (target === null) {
      return;
    }

    event.preventDefault();
    const top = scrollToShow(target, { height, rowHeight, scrollTop: viewport.scrollTop });

    viewport.scrollTop = top;
    setScrollTop(top);
    focusPendingRef.current = true;
    setFocus({ index: target, key: getKey(target) });
  });

  // A click or a Tab into a row makes it the focused one.
  const onFocusIn = useEffectEvent((event: FocusEvent): void => {
    const slot = slotOf(event.target);
    const index = slot === null ? null : Number(slot.dataset.index);

    hasFocusRef.current = true;

    if (index !== null && (index !== focused || getKey(index) !== focus.key)) {
      setFocus({ index, key: getKey(index) });
    }
  });

  useEffect(() => {
    const viewport = viewportRef.current;

    if (viewport === null) {
      return;
    }

    const scroll = (): void => {
      onScroll();
    };
    const keyDown = (event: KeyboardEvent): void => {
      onKeyDown(event);
    };
    const focusIn = (event: FocusEvent): void => {
      onFocusIn(event);
    };
    // The focus left the list for another element or for nothing. A row removed under the focus is not a leave: some
    // engines send a focusout while removing it, so a leave to nothing is judged once the removal is done.
    const onFocusOut = (event: FocusEvent): void => {
      const { relatedTarget: next, target } = event;

      if (next instanceof Node) {
        hasFocusRef.current = viewport.contains(next);

        return;
      }

      queueMicrotask(() => {
        if (target instanceof Node && target.isConnected) {
          hasFocusRef.current = false;
        }
      });
    };

    viewport.addEventListener('scroll', scroll, { passive: true });
    viewport.addEventListener('keydown', keyDown);
    viewport.addEventListener('focusin', focusIn);
    viewport.addEventListener('focusout', onFocusOut);

    return (): void => {
      viewport.removeEventListener('scroll', scroll);
      viewport.removeEventListener('keydown', keyDown);
      viewport.removeEventListener('focusin', focusIn);
      viewport.removeEventListener('focusout', onFocusOut);

      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
        frameRef.current = null;
      }
    };
  }, []);

  // A keyboard move focuses its row once it is rendered.
  useLayoutEffect(() => {
    if (!focusPendingRef.current) {
      return;
    }

    focusPendingRef.current = false;
    const slot = viewportRef.current?.querySelector(`[data-index="${String(focused)}"]`);
    const row = slot === null || slot === undefined ? null : rowOf(slot);

    if (row instanceof HTMLElement) {
      row.focus({ preventScroll: true });
    }
  });

  // The focused row was replaced under the focus (its page arrived, its key changed) and the focus fell to the page:
  // the row now at its index takes it back.
  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    const active = document.activeElement;

    if (!hasFocusRef.current || viewport === null || (active !== null && active !== document.body)) {
      return;
    }

    const slot = viewport.querySelector(`[data-index="${String(focused)}"]`);
    const row = slot === null ? null : rowOf(slot);

    if (row instanceof HTMLElement) {
      row.focus({ preventScroll: true });
    }
  });

  // Rows new in view enter with the stagger; rows that came in by scrolling, or before the view was measured, do not.
  useLayoutEffect(() => {
    const seen = seenRef.current;
    const scrolled = lastScrollRef.current !== scrollTop;

    lastScrollRef.current = scrollTop;

    if (height === 0) {
      return;
    }

    seenRef.current = new Set(keys);

    if (scrolled || motion === 'off') {
      return;
    }

    let step = 0;

    for (const slot of viewportRef.current?.querySelectorAll<HTMLElement>('[data-index]') ?? []) {
      const index = Number(slot.dataset.index);
      const layer = slot.firstElementChild;

      if (index < visibleFirst || index > visibleLast || seen.has(slot.dataset.key ?? '') || layer === null) {
        continue;
      }

      if (layer instanceof HTMLElement) {
        layer.style.setProperty('--i', String(Math.min(step, STAGGER.steps)));
        // Left in place after the animation: it plays once, and a row that comes back is a new element.
        layer.dataset.entering = '';
        step += 1;
      }
    }
  }, [keys, scrollTop, height, motion, visibleFirst, visibleLast]);

  const reportRange = useEffectEvent((): void => {
    if (last >= first) {
      onRangeChange?.(first, last);
    }
  });

  useEffect(() => {
    reportRange();
  }, [first, last]);

  return (
    <div aria-label={label} className={styles.viewport} ref={viewportRef} role="list">
      <div className={styles.spacer} style={{ height: count * rowHeight }}>
        {keys.map((key, offset) => {
          const index = first + offset;

          return (
            <div
              className={styles.slot}
              data-index={index}
              data-key={key}
              key={key}
              style={{ height: rowHeight, transform: `translateY(${String(index * rowHeight)}px)` }}
            >
              <div className={styles.motion} ref={flip(key)}>
                {renderRow(index, getItem(index), {
                  'aria-posinset': index + 1,
                  'aria-setsize': count,
                  tabIndex: index === tabStop ? 0 : -1,
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
