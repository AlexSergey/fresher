import type { JSX } from 'react';

import type { FeedbackAction } from '../action';

import { Icon } from '../../icons/icon';
import { CircleAlert, CircleCheck, type IconShape, Info, X } from '../../icons/icons';
import { Button } from '../../primitives/button/button';
import { IconButton } from '../../primitives/icon-button/icon-button';
import styles from './toast.module.css';

// What a toast says: its tone, one sentence and an optional action.
export type ToastContent = {
  readonly action?: FeedbackAction;
  readonly text: string;
  readonly tone: ToastTone;
};

export type ToastProps = ToastContent & {
  // The close button's accessible name ("Close").
  readonly closeLabel: string;
  readonly onClose: () => void;
};

type ToastTone = 'error' | 'info' | 'success';

const ICONS: Readonly<Record<ToastTone, IconShape>> = {
  error: CircleAlert,
  info: Info,
  success: CircleCheck,
};

// A solid card at elevation 3: the tone's icon, the text, the action and a close button. The action closes it too.
export const Toast = ({ action, closeLabel, onClose, text, tone }: ToastProps): JSX.Element => (
  <div className={styles.toast} data-tone={tone}>
    <span className={styles.icon}>
      <Icon icon={ICONS[tone]} />
    </span>
    <p className={styles.text}>{text}</p>
    {action === undefined ? null : (
      <Button
        onClick={() => {
          action.onClick();
          onClose();
        }}
        size="small"
        variant="ghost"
      >
        {action.label}
      </Button>
    )}
    <IconButton icon={X} label={closeLabel} onClick={onClose} size="small" />
  </div>
);
