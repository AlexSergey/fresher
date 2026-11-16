import { type JSX, type ReactNode, useId } from 'react';

import styles from './sidebar.module.css';

export type NavSectionProps = {
  // `NavItem`s.
  readonly children: ReactNode;
  // A small uppercase heading; the first section of a sidebar may have none.
  readonly title?: string;
};

// A group of sidebar items under its heading.
export const NavSection = ({ children, title }: NavSectionProps): JSX.Element => {
  const id = useId();

  if (title === undefined) {
    return <div className={styles.section}>{children}</div>;
  }

  return (
    <section aria-labelledby={id} className={styles.section}>
      <h2 className={styles.heading} id={id}>
        {title}
      </h2>
      {children}
    </section>
  );
};
