import type { JSX, ReactNode } from 'react';

import type { Forced } from '../../primitives/forced';
import type { Level } from '../danger-badge/danger-badge';
import type { VirtualRowProps } from '../virtual-list/virtual-list';

import styles from './list-row.module.css';

export type ListRowProps = Forced &
  Partial<VirtualRowProps> & {
    // What a finding row reads instead of its content ("Xcode caches, Safe, 12.4 GB") and why it is listed.
    readonly 'aria-describedby'?: string;
    readonly 'aria-label'?: string;
    // Greyed out; a virtual list does not toggle it.
    readonly disabled?: boolean;
    // The selected row's leading edge in this level's colour instead of the accent.
    readonly edgeLevel?: Level;
    // A checkbox or an icon. A selectable row's checkbox is what tells assistive technology it is selected.
    readonly leading?: ReactNode;
    readonly selected?: boolean;
    // A second line under the title: a path, a description.
    readonly subtitle?: ReactNode;
    readonly title: string;
    // A badge, a size.
    readonly trailing?: ReactNode;
  };

// A 44 px row of a list: a leading slot, a title with its subtitle, a trailing slot. Hover lightens it, selection tints
// it and lights its leading edge; the focus ring is drawn inside, so a scrolling list never clips it.
export const ListRow = ({
  disabled = false,
  edgeLevel,
  leading,
  selected = false,
  subtitle,
  title,
  trailing,
  ...rest
}: ListRowProps): JSX.Element => (
  <div
    {...rest}
    className={styles.row}
    data-disabled={disabled ? true : undefined}
    data-level={edgeLevel}
    data-selected={selected ? true : undefined}
    role="listitem"
  >
    {leading === undefined ? null : <span className={styles.leading}>{leading}</span>}
    <span className={styles.content}>
      <span className={styles.title}>{title}</span>
      {subtitle === undefined ? null : <span className={styles.subtitle}>{subtitle}</span>}
    </span>
    {trailing === undefined ? null : <span className={styles.trailing}>{trailing}</span>}
  </div>
);
