import type { CSSProperties, JSX } from 'react';

import { usePresence } from '../../motion';
import { DURATION } from '../../tokens/motion';
import styles from './scan-sweep.module.css';

export type ScanSweepProps = {
  readonly active: boolean;
  // The width of the ring's band the sweep is masked to, in pixels; the ring's stroke.
  readonly band?: number;
};

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

// A comet of Iris circling a ring's band while a scan runs: it fades in when it turns on and out when it turns off.
// It fills its container, which is the ring's square. Under reduced motion it stands still; under off, at its first
// frame.
export const ScanSweep = ({ active, band = 12 }: ScanSweepProps): JSX.Element | null => {
  const { mounted, ref } = usePresence<HTMLSpanElement>(active, { duration: DURATION.move, exit: FADE_OUT });

  if (!mounted) {
    return null;
  }

  return (
    <span
      aria-hidden="true"
      className={styles.sweep}
      data-sweep
      ref={ref}
      style={{ '--sweep-band': `${band}px` } as CSSProperties}
    >
      <span className={styles.comet} />
    </span>
  );
};
