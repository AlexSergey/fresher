import type { JSX } from 'react';

import styles from './divider.module.css';

export type DividerProps = {
  // Horizontal by default; a vertical one stretches to the height of its row.
  readonly orientation?: 'horizontal' | 'vertical';
};

// A hairline between groups, a separator for assistive technology.
export const Divider = ({ orientation = 'horizontal' }: DividerProps): JSX.Element => (
  <hr aria-orientation={orientation} className={styles.divider} />
);
