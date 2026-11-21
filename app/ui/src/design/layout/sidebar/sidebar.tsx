import { type JSX, type ReactNode, useLayoutEffect, useRef } from 'react';

import styles from './sidebar.module.css';

export type SidebarProps = {
  // `NavSection`s.
  readonly children: ReactNode;
  // At the bottom: Settings.
  readonly footer?: ReactNode;
  // The navigation's accessible name.
  readonly label: string;
};

// The sidebar's content: the strip the traffic lights sit in (a drag region), then a scroll container with the
// sections and a footer, so a short window never hides an item. One indicator per sidebar slides under the active
// item; it is placed from the item's offset in the scroll container after every render.
export const Sidebar = ({ children, footer, label }: SidebarProps): JSX.Element => {
  const navRef = useRef<HTMLElement>(null);
  const indicatorRef = useRef<HTMLSpanElement>(null);
  const placedRef = useRef(false);

  // The items stagger in once, in their order.
  useLayoutEffect(() => {
    const items = navRef.current?.querySelectorAll<HTMLElement>('[data-nav-item]') ?? [];

    for (const [index, item] of [...items].entries()) {
      item.style.setProperty('--i', String(index));
    }
  }, []);

  useLayoutEffect(() => {
    const active = navRef.current?.querySelector<HTMLElement>('[data-nav-item][aria-current="page"]') ?? null;
    const indicator = indicatorRef.current;

    if (indicator === null) {
      return;
    }

    if (active === null) {
      delete indicator.dataset.shown;
      placedRef.current = false;

      return;
    }

    // The first placement jumps; every later one slides.
    indicator.toggleAttribute('data-instant', !placedRef.current);
    indicator.style.setProperty('--indicator-y', `${active.offsetTop}px`);
    indicator.dataset.shown = '';
    placedRef.current = true;
  });

  return (
    <nav aria-label={label} className={styles.sidebar} ref={navRef}>
      <div className={styles.strip} data-tauri-drag-region />
      <div className={styles.scroll}>
        <span aria-hidden="true" className={styles.indicator} data-nav-indicator ref={indicatorRef} />
        <div className={styles.sections}>{children}</div>
        {footer === undefined ? null : <div className={styles.footer}>{footer}</div>}
      </div>
    </nav>
  );
};
