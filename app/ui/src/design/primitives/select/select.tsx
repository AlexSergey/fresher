import { type ChangeEvent, type JSX, useId } from 'react';

import type { Forced } from '../forced';

import { Icon } from '../../icons/icon';
import { ChevronDown } from '../../icons/icons';
import fieldStyles from '../text-field/text-field.module.css';
import styles from './select.module.css';

export type SelectProps<T extends string> = Forced & {
  readonly disabled?: boolean;
  // The select's accessible name; visually hidden with `hideLabel`.
  readonly hideLabel?: boolean;
  readonly label: string;
  readonly onChange: (value: T) => void;
  readonly options: readonly SelectOption<T>[];
  readonly value: T;
};

type SelectOption<T extends string> = { readonly label: string; readonly value: T };

// A native select, restyled: the system menu opens, as on every Mac.
export const Select = <T extends string>({
  disabled = false,
  hideLabel = false,
  label,
  onChange,
  options,
  value,
  ...forced
}: SelectProps<T>): JSX.Element => {
  const selectId = useId();

  return (
    <div className={fieldStyles.field}>
      <label className={hideLabel ? 'u-visually-hidden' : fieldStyles.label} htmlFor={selectId}>
        {label}
      </label>
      <div className={styles.box}>
        <select
          {...forced}
          className={`${fieldStyles.input ?? ''} ${styles.select ?? ''}`}
          disabled={disabled}
          id={selectId}
          onChange={(event: ChangeEvent<HTMLSelectElement>) => {
            const chosen = options.find((option) => option.value === event.target.value);

            if (!disabled && chosen !== undefined) {
              onChange(chosen.value);
            }
          }}
          value={value}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <Icon className={styles.chevron ?? ''} icon={ChevronDown} />
      </div>
    </div>
  );
};
