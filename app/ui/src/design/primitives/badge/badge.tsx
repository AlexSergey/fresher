import type { JSX } from 'react';

import { VisuallyHidden } from '../visually-hidden/visually-hidden';
import styles from './badge.module.css';

export type BadgeProps = (
  | { readonly children: string; readonly count?: never; readonly label?: never }
  | {
      readonly children?: never;
      // A count: shown up to 999, then "999+".
      readonly count: number;
      // What the count says in full, for assistive technology ("1,204 items").
      readonly label: string;
    }
) & {
  readonly tone?: BadgeTone;
};

type BadgeTone = 'accent' | 'neutral';

const CAP = 999;

// A small pill: a word, or a count in tabular figures.
export const Badge = ({ children, count, label, tone = 'neutral' }: BadgeProps): JSX.Element => {
  if (count === undefined) {
    return (
      <span className={styles.badge} data-tone={tone}>
        {children}
      </span>
    );
  }

  return (
    <span className={styles.badge} data-numeric data-tone={tone}>
      <span aria-hidden="true">{count > CAP ? `${CAP}+` : String(count)}</span>
      <VisuallyHidden>{label}</VisuallyHidden>
    </span>
  );
};
