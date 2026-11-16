import type { JSX } from 'react';

import type { FeedbackAction } from '../action';

import { Icon } from '../../icons/icon';
import { type IconShape } from '../../icons/icons';
import { Button } from '../../primitives/button/button';
import styles from './empty-state.module.css';

export type EmptyStateProps = {
  readonly action?: FeedbackAction;
  readonly icon: IconShape;
  // One sentence under the title.
  readonly text: string;
  readonly title: string;
  // `danger` draws the icon in High's colour, for a failure.
  readonly tone?: 'accent' | 'danger';
};

// An icon in a soft halo, a title, one sentence and an optional action, centred in its container.
export const EmptyState = ({ action, icon, text, title, tone = 'accent' }: EmptyStateProps): JSX.Element => (
  <div className={styles.state} data-tone={tone}>
    <span className={styles.halo}>
      <Icon icon={icon} size={24} />
    </span>
    <p className={styles.title}>{title}</p>
    <p className={styles.text}>{text}</p>
    {action === undefined ? null : <Button onClick={action.onClick}>{action.label}</Button>}
  </div>
);
