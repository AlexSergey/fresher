import type { JSX } from 'react';

import type { Forced } from '../forced';

import styles from './switch.module.css';

export type SwitchProps = Forced & {
  readonly checked: boolean;
  readonly disabled?: boolean;
  // The switch's accessible name; visually hidden with `hideLabel` (a settings row whose title names it).
  readonly hideLabel?: boolean;
  readonly label: string;
  readonly onChange: (checked: boolean) => void;
};

// An on/off switch: a real button with the switch role, a pill track and a thumb that slides.
export const Switch = ({
  checked,
  disabled = false,
  hideLabel = false,
  label,
  onChange,
  ...forced
}: SwitchProps): JSX.Element => (
  <button
    {...forced}
    aria-checked={checked}
    className={styles.switch}
    disabled={disabled}
    onClick={() => {
      onChange(!checked);
    }}
    role="switch"
    type="button"
  >
    <span aria-hidden="true" className={styles.track}>
      <span className={styles.thumb} />
    </span>
    <span className={hideLabel ? 'u-visually-hidden' : styles.label}>{label}</span>
  </button>
);
