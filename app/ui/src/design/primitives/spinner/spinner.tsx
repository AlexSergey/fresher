import type { JSX } from 'react';

import styles from './spinner.module.css';

export type SpinnerProps = {
  // What is under way, for assistive technology; without it the spinner decorates a control that says it is busy.
  readonly label?: string;
  // 12 or 16 pixels.
  readonly size?: 'medium' | 'small';
};

const PIXELS = { medium: 16, small: 12 } as const;

// An arc that turns while work runs: a transform loop, standing still under reduced motion and off.
export const Spinner = ({ label, size = 'medium' }: SpinnerProps): JSX.Element => (
  <svg
    {...(label === undefined ? { 'aria-hidden': true } : { 'aria-label': label, role: 'img' })}
    className={styles.spinner}
    focusable="false"
    height={PIXELS[size]}
    viewBox="0 0 16 16"
    width={PIXELS[size]}
  >
    <circle className={styles.track} cx="8" cy="8" r="6.25" />
    <circle className={styles.arc} cx="8" cy="8" r="6.25" />
  </svg>
);
