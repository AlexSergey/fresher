import type { JSX } from 'react';

import { chartLabel, type Segment } from '../segment';
import styles from './usage-bar.module.css';

export type UsageBarProps = {
  // The caller's name for the whole; the bar's accessible name adds every segment and its size.
  readonly label: string;
  // Shows a legend under the bar: each segment's dot, label and size.
  readonly legend?: boolean;
  readonly segments: readonly Segment[];
  // The whole each segment takes its share of.
  readonly total: number;
};

const percentOf = (value: number, total: number): string => `${Math.round((value / total) * 10_000) / 100}%`;

// One row of segments in their tones' colours, each as wide as its share of the total, with an optional legend.
// A width is set once per update, never animated; a new segment enters by scaling from its left edge.
export const UsageBar = ({ label, legend = false, segments, total }: UsageBarProps): JSX.Element => {
  const drawn = total > 0 ? segments.filter(({ value }) => value > 0) : [];

  return (
    <div className={styles.usage}>
      <div aria-label={chartLabel(label, drawn)} className={styles.bar} role="img">
        {drawn.map(({ key, tone, value }) => (
          <span className={styles.segment} data-tone={tone} key={key} style={{ width: percentOf(value, total) }} />
        ))}
      </div>
      {legend ? (
        <ul className={styles.legend}>
          {drawn.map(({ key, label: name, size, tone }) => (
            <li className={styles.item} key={key}>
              <span className={styles.dot} data-tone={tone} />
              <span className={styles.name}>{name}</span>
              <span className={styles.size} data-numeric>
                {size}
              </span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
};
