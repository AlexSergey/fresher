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
    // A dense row of a tile's list: no selection tint or edge, an unticked row dimmed instead; 44 px, the title in
    // body text over a small subtitle.
    readonly compact?: boolean;
    // Greyed out; a virtual list does not toggle it.
    readonly disabled?: boolean;
    // The selected row's leading edge in this level's colour instead of the accent.
    readonly edgeLevel?: Level;
    // A row of a framed list (a category's items): padded 14 px, no selection tint or edge, a divider under it.
    readonly flat?: boolean;
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
  compact = false,
  disabled = false,
  edgeLevel,
  flat = false,
  leading,
  selected = false,
  subtitle,
  title,
  trailing,
  ...rest
}: ListRowProps): JSX.Element => (
  // eslint-disable-next-line jsx-a11y-x/role-supports-aria-props -- an inactive row is exempt from contrast (WCAG 1.4.3) only when it says it is disabled
  <div
    {...rest}
    aria-disabled={disabled ? true : undefined}
    className={styles.row}
    data-compact={compact ? true : undefined}
    data-disabled={disabled ? true : undefined}
    data-flat={flat ? true : undefined}
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
