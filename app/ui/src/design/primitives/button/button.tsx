import type { JSX, MouseEvent, Ref } from 'react';

import type { Forced } from '../forced';

import { Icon } from '../../icons/icon';
import { type IconShape } from '../../icons/icons';
import { Spinner } from '../spinner/spinner';
import styles from './button.module.css';

export type ButtonProps = Forced & {
  // Work it started is under way: a spinner takes the icon's place, the label stays, clicks are ignored.
  readonly busy?: boolean;
  // The label, and the accessible name.
  readonly children: string;
  readonly disabled?: boolean;
  // A leading icon.
  readonly icon?: IconShape;
  readonly onClick?: () => void;
  // The native button: a dialog focuses its safe choice through it.
  readonly ref?: Ref<HTMLButtonElement>;
  readonly size?: ButtonSize;
  // `button` unless given: a button never submits a form by accident.
  readonly type?: 'button' | 'reset' | 'submit';
  // `danger` is the permanent-delete choice only.
  readonly variant?: ButtonVariant;
};

// 24, 28 or 36 pixels tall.
export type ButtonSize = 'large' | 'medium' | 'small';

type ButtonVariant = 'danger' | 'ghost' | 'primary' | 'secondary';

// A real button with its label, in one of four variants and three sizes.
export const Button = ({
  busy = false,
  children,
  disabled = false,
  icon,
  onClick,
  ref,
  size = 'medium',
  type = 'button',
  variant = 'secondary',
  ...forced
}: ButtonProps): JSX.Element => {
  const leading = icon === undefined ? null : <Icon icon={icon} />;

  return (
    <button
      {...forced}
      aria-busy={busy || undefined}
      aria-disabled={busy || undefined}
      className={styles.button}
      data-size={size}
      data-variant={variant}
      disabled={disabled}
      onClick={(event: MouseEvent<HTMLButtonElement>) => {
        if (busy) {
          // A busy submit button does not submit again.
          event.preventDefault();

          return;
        }

        onClick?.();
      }}
      ref={ref}
      type={type}
    >
      {busy ? <Spinner size={size === 'small' ? 'small' : 'medium'} /> : leading}
      <span className={styles.label}>{children}</span>
    </button>
  );
};
