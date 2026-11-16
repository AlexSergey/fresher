import { type JSX, type ReactNode, useId } from 'react';

import { Skeleton } from '../../primitives/skeleton/skeleton';
import styles from './stat-tile.module.css';

export type StatTileProps = {
  // A short line under the value ("in 6 modules").
  readonly hint?: string;
  readonly label: string;
  // While loading, a skeleton line takes the value's place and the tile is busy.
  readonly loading?: boolean;
  // Usually a `SizeText` or an `AnimatedNumber`.
  readonly value: ReactNode;
};

// One figure with its label above it and an optional hint below, named by its label.
export const StatTile = ({ hint, label, loading = false, value }: StatTileProps): JSX.Element => {
  const labelId = useId();

  return (
    <div aria-busy={loading ? true : undefined} aria-labelledby={labelId} className={styles.tile} role="group">
      <span className={styles.label} id={labelId}>
        {label}
      </span>
      <span className={styles.value}>{loading ? <Skeleton shape="line" /> : value}</span>
      {hint === undefined ? null : <span className={styles.hint}>{hint}</span>}
    </div>
  );
};
