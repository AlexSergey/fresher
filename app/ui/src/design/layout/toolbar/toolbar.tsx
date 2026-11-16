import { type JSX, type ReactNode, useEffect, useRef, useState } from 'react';

import styles from './toolbar.module.css';

export type ToolbarProps = {
  readonly actions?: ReactNode;
  readonly title?: ReactNode;
};

// The top 52 px of a screen: the first child of its `ScrollArea`, sticky, a drag region except over its controls. A
// sentinel just above it leaves the view once the content scrolls, and the toolbar shows its hairline and shade.
export const Toolbar = ({ actions, title }: ToolbarProps): JSX.Element => {
  const sentinelRef = useRef<HTMLDivElement>(null);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const sentinel = sentinelRef.current;

    if (sentinel === null) {
      return;
    }

    const observer = new IntersectionObserver((entries) => {
      const last = entries.at(-1);

      if (last !== undefined) {
        setScrolled(!last.isIntersecting);
      }
    });

    observer.observe(sentinel);

    return (): void => {
      observer.disconnect();
    };
  }, []);

  return (
    <>
      <div aria-hidden="true" className={styles.sentinel} ref={sentinelRef} />
      {/* `deep`: the whole bar drags the window; Tauri skips its buttons and the actions opt out explicitly. */}
      <header className={styles.toolbar} data-scrolled={scrolled || undefined} data-tauri-drag-region="deep">
        <div className={styles.title}>{title}</div>
        {actions === undefined ? null : (
          <div className={styles.actions} data-tauri-drag-region="false">
            {actions}
          </div>
        )}
      </header>
    </>
  );
};
