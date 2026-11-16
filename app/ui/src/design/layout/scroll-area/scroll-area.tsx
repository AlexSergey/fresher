import { type JSX, type ReactNode, useLayoutEffect, useRef } from 'react';

import styles from './scroll-area.module.css';
import { useOverflowFocus } from './use-overflow-focus';

export type ScrollAreaProps = {
  readonly children: ReactNode;
  // Its accessible name, read when the area itself takes the focus.
  readonly label: string;
  // The screen it shows: the scroll is kept per key and restored when the key comes back.
  readonly screenKey?: string;
};

// Where each screen was scrolled to, for the life of the page.
const positions = new Map<string, number>();

// The scrolling region of a screen, with thin scrollbars. It is focusable (so the keyboard can scroll it) only when
// it overflows and holds nothing focusable of its own.
export const ScrollArea = ({ children, label, screenKey }: ScrollAreaProps): JSX.Element => {
  const areaRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const focusable = useOverflowFocus(areaRef, contentRef);

  useLayoutEffect(() => {
    const area = areaRef.current;

    if (area !== null && screenKey !== undefined) {
      area.scrollTop = positions.get(screenKey) ?? 0;
    }
  }, [screenKey]);

  return (
    <div
      aria-label={focusable ? label : undefined}
      className={styles.area}
      onScroll={(event) => {
        if (screenKey !== undefined) {
          positions.set(screenKey, event.currentTarget.scrollTop);
        }
      }}
      ref={areaRef}
      role={focusable ? 'region' : undefined}
      tabIndex={focusable ? 0 : undefined}
    >
      <div className={styles.content} ref={contentRef}>
        {children}
      </div>
    </div>
  );
};
