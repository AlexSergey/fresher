import { type ChangeEvent, type JSX, useId } from 'react';

import type { Forced } from '../forced';

import { Icon } from '../../icons/icon';
import { CircleAlert } from '../../icons/icons';
import styles from './text-field.module.css';

export type TextFieldProps = Forced & {
  // A line under the field, linked to the input as its description.
  readonly description?: string;
  readonly disabled?: boolean;
  // What is wrong with the value: the field is marked invalid and the message is linked to it.
  readonly error?: string;
  // The input's accessible name; visually hidden with `hideLabel`.
  readonly hideLabel?: boolean;
  readonly label: string;
  readonly onChange: (value: string) => void;
  readonly placeholder?: string;
  readonly type?: 'number' | 'text';
  readonly value: string;
};

const joinIds = (...ids: readonly (false | string)[]): string | undefined => {
  const present = ids.filter((id) => id !== false);

  return present.length === 0 ? undefined : present.join(' ');
};

// A labelled text input, with an optional description and an error shown in words and with an icon, never in colour
// alone.
export const TextField = ({
  description,
  disabled = false,
  error,
  hideLabel = false,
  label,
  onChange,
  placeholder,
  type = 'text',
  value,
  ...forced
}: TextFieldProps): JSX.Element => {
  const inputId = useId();
  const descriptionId = useId();
  const errorId = useId();
  const invalid = error !== undefined;

  return (
    <div className={styles.field}>
      <label className={hideLabel ? 'u-visually-hidden' : styles.label} htmlFor={inputId}>
        {label}
      </label>
      <input
        {...forced}
        aria-describedby={joinIds(invalid && errorId, description !== undefined && descriptionId)}
        aria-invalid={invalid || undefined}
        className={styles.input}
        disabled={disabled}
        id={inputId}
        onChange={(event: ChangeEvent<HTMLInputElement>) => {
          onChange(event.target.value);
        }}
        placeholder={placeholder}
        type={type}
        value={value}
      />
      {invalid ? (
        <p className={styles.error} id={errorId}>
          <Icon className={styles.errorIcon ?? ''} icon={CircleAlert} />
          {error}
        </p>
      ) : null}
      {description === undefined ? null : (
        <p className={styles.description} id={descriptionId}>
          {description}
        </p>
      )}
    </div>
  );
};
