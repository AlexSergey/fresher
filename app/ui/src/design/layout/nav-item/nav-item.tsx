import type { JSX, ReactNode } from 'react';

import type { Forced } from '../../primitives/forced';

import { Icon } from '../../icons/icon';
import { type IconShape } from '../../icons/icons';
import styles from './nav-item.module.css';

export type NavItemProps = Forced & {
  // The screen shown now: announced as the current page, and the sidebar's indicator slides under it.
  readonly active?: boolean;
  // Muted: what the item leads to is switched off (an ignored module).
  readonly dimmed?: boolean;
  readonly icon: IconShape;
  readonly label: string;
  readonly onClick: () => void;
  // After the label: a `Badge` count or a size.
  readonly trailing?: ReactNode;
};

// One entry of the sidebar: a link-like button to a screen. Its `--i` (its place in the entrance stagger) is set by the
// sidebar.
export const NavItem = ({
  active = false,
  dimmed = false,
  icon,
  label,
  onClick,
  trailing,
  ...forced
}: NavItemProps): JSX.Element => (
  <button
    {...forced}
    aria-current={active ? 'page' : undefined}
    className={styles.item}
    data-dimmed={dimmed || undefined}
    data-nav-item
    onClick={onClick}
    type="button"
  >
    <Icon className={styles.icon ?? ''} icon={icon} />
    <span className={styles.label}>{label}</span>
    {trailing === undefined ? null : <span className={styles.trailing}>{trailing}</span>}
  </button>
);
