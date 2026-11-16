import type { JSX, ReactNode } from 'react';

import styles from './app-shell.module.css';

export type AppShellProps = {
  // The light behind the main pane's content: an `AmbientLight`.
  readonly ambient?: ReactNode;
  // The screen: a `ScrollArea` whose first child is its `Toolbar`.
  readonly children: ReactNode;
  // A `Sidebar`.
  readonly sidebar: ReactNode;
};

// The window's frame: the sidebar beside the main pane. The main pane is the opaque window surface; its content keeps
// to the content column through `--content-inset`.
export const AppShell = ({ ambient, children, sidebar }: AppShellProps): JSX.Element => (
  <div className={styles.shell}>
    <div className={styles.sidebar}>{sidebar}</div>
    <main className={styles.main}>
      {ambient === undefined ? null : (
        <div aria-hidden="true" className={styles.ambient}>
          {ambient}
        </div>
      )}
      <div className={styles.content}>{children}</div>
    </main>
  </div>
);
