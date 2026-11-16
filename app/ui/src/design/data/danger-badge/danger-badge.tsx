import type { JSX } from 'react';

import { Icon } from '../../icons/icon';
import { type IconShape, OctagonAlert, Shield, ShieldCheck, TriangleAlert } from '../../icons/icons';
import styles from './danger-badge.module.css';

export type DangerBadgeProps = (
  | {
      // The large badge's sentence: what the level means for the items it marks.
      readonly description?: string;
      readonly size: 'large';
    }
  | { readonly description?: never; readonly size?: 'medium' | 'small' }
) & {
  // The level's word, from the caller: the badge never rests on colour alone.
  readonly label: string;
  readonly level: Level;
};

// How much a removal can cost the user, from Safe to High.
export type Level = 'high' | 'low' | 'medium' | 'safe';

// The icons escalate in shape, like signage.
const ICONS: Readonly<Record<Level, IconShape>> = {
  high: OctagonAlert,
  low: Shield,
  medium: TriangleAlert,
  safe: ShieldCheck,
};

// A danger level: its icon and its word in the level's colour, on its tint with its border. The High badge glows at
// rest; no badge pulses.
export const DangerBadge = ({ description, label, level, size = 'medium' }: DangerBadgeProps): JSX.Element => (
  <span className={styles.badge} data-level={level} data-size={size}>
    <Icon className={styles.icon ?? ''} icon={ICONS[level]} size={size === 'large' ? 20 : 16} />
    <span className={styles.word}>{label}</span>
    {description === undefined ? null : <span className={styles.description}>{description}</span>}
  </span>
);
