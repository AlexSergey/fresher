import type { JSX, ReactNode } from 'react';

import type { FeedbackAction } from '../action';

import { Icon } from '../../icons/icon';
import { type IconShape, Info, OctagonAlert, TriangleAlert } from '../../icons/icons';
import { Button } from '../../primitives/button/button';
import styles from './notice.module.css';

export type NoticeProps = {
  readonly action?: FeedbackAction;
  // One or two sentences under the title.
  readonly children: ReactNode;
  // The notice appeared after the page did (a scan that ended partial): it is announced, as a status when it informs
  // and as an alert when it warns. A notice there from the first render is part of the page and is read with it.
  readonly live?: boolean;
  readonly title: string;
  readonly tone: NoticeTone;
};

type NoticeTone = 'danger' | 'info' | 'warning';

const ICONS: Readonly<Record<NoticeTone, IconShape>> = {
  danger: OctagonAlert,
  info: Info,
  warning: TriangleAlert,
};

// An inline banner on its tone's tint, with its icon, a title, its text and an optional action.
export const Notice = ({ action, children, live = false, title, tone }: NoticeProps): JSX.Element => {
  const role = tone === 'info' ? 'status' : 'alert';

  return (
    <div className={styles.notice} data-tone={tone} role={live ? role : undefined}>
      <span className={styles.icon}>
        <Icon icon={ICONS[tone]} />
      </span>
      <div className={styles.body}>
        <p className={styles.title}>{title}</p>
        <div className={styles.text}>{children}</div>
      </div>
      {action === undefined ? null : (
        <Button onClick={action.onClick} size="small">
          {action.label}
        </Button>
      )}
    </div>
  );
};
