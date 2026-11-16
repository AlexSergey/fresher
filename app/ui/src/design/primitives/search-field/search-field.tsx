import { type ChangeEvent, type JSX, type KeyboardEvent, useId, useRef } from 'react';

import type { Forced } from '../forced';

import { Icon } from '../../icons/icon';
import { Search, X } from '../../icons/icons';
import { IconButton } from '../icon-button/icon-button';
import fieldStyles from '../text-field/text-field.module.css';
import styles from './search-field.module.css';

export type SearchFieldProps = Forced & {
  // The clear button's accessible name and tooltip ("Clear the search").
  readonly clearLabel: string;
  // The input's accessible name; visually hidden with `hideLabel` (a toolbar's search).
  readonly hideLabel?: boolean;
  readonly label: string;
  readonly onChange: (value: string) => void;
  readonly placeholder?: string;
  readonly value: string;
};

// A search input with its icon and, once it holds text, a clear button; Escape clears it too. The focus stays in it.
export const SearchField = ({
  clearLabel,
  hideLabel = false,
  label,
  onChange,
  placeholder,
  value,
  ...forced
}: SearchFieldProps): JSX.Element => {
  const inputRef = useRef<HTMLInputElement>(null);
  const inputId = useId();

  const clear = (): void => {
    onChange('');
    inputRef.current?.focus();
  };

  return (
    <div className={fieldStyles.field}>
      <label className={hideLabel ? 'u-visually-hidden' : fieldStyles.label} htmlFor={inputId}>
        {label}
      </label>
      <div className={styles.box}>
        <Icon className={styles.searchIcon ?? ''} icon={Search} />
        <input
          {...forced}
          className={`${fieldStyles.input ?? ''} ${styles.input ?? ''}`}
          id={inputId}
          onChange={(event: ChangeEvent<HTMLInputElement>) => {
            onChange(event.target.value);
          }}
          onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
            // An empty search lets Escape through, to whatever else listens for it (a dialog closing).
            if (event.key === 'Escape' && value !== '') {
              event.preventDefault();
              clear();
            }
          }}
          placeholder={placeholder}
          ref={inputRef}
          type="search"
          value={value}
        />
        {value === '' ? null : (
          <span className={styles.clear}>
            <IconButton icon={X} label={clearLabel} onClick={clear} size="small" />
          </span>
        )}
      </div>
    </div>
  );
};
