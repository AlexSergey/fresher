import type { JSX, ReactNode } from 'react';

import styles from './page-header.module.css';

export type PageHeaderProps = {
  readonly actions?: ReactNode;
  readonly subtitle?: string;
  // The page's one first-level heading.
  readonly title: string;
};

// The top of a page: its title, a line under it, and its actions on the trailing side.
export const PageHeader = ({ actions, subtitle, title }: PageHeaderProps): JSX.Element => (
  <header className={styles.header}>
    <div className={styles.text}>
      <h1 className={styles.title}>{title}</h1>
      {subtitle === undefined ? null : <p className={styles.subtitle}>{subtitle}</p>}
    </div>
    {actions === undefined ? null : <div className={styles.actions}>{actions}</div>}
  </header>
);
