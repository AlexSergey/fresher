import type { JSX } from 'react';

import styles from './skeleton.module.css';

export type SkeletonProps = {
  // A line of text, a block (a card, a row) or a circle (an icon); a line and a block fill their container's width.
  readonly shape: 'block' | 'circle' | 'line';
};

// The shape of content still loading, with a shimmer crossing it; hidden from assistive technology, which hears the
// loading state from the component that shows it.
export const Skeleton = ({ shape }: SkeletonProps): JSX.Element => (
  <span aria-hidden="true" className={styles.skeleton} data-shape={shape} />
);
