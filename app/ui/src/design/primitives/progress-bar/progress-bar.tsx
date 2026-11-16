import type { CSSProperties, JSX } from 'react';

import styles from './progress-bar.module.css';

export type ProgressBarProps = {
  // The bar's accessible name.
  readonly label: string;
  // Percent done, 0 to 100; `null` when how much is left is not known.
  readonly value: null | number;
};

const clamp = (value: number): number => Math.min(Math.max(value, 0), 100);

// How far work has gone: a fill grown by `transform: scaleX`, or, when unknown, a soft highlight sweeping across.
export const ProgressBar = ({ label, value }: ProgressBarProps): JSX.Element => {
  if (value === null) {
    return (
      <div aria-label={label} className={styles.bar} data-indeterminate role="progressbar">
        <span className={styles.sweep} />
      </div>
    );
  }

  const percent = clamp(value);

  return (
    <div
      aria-label={label}
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={Math.round(percent)}
      className={styles.bar}
      role="progressbar"
    >
      <span className={styles.fill} style={{ '--progress': percent / 100 } as CSSProperties} />
    </div>
  );
};
