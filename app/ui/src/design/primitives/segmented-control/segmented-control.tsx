import { type CSSProperties, type JSX, type KeyboardEvent, useRef } from 'react';

import type { Forced } from '../forced';

import styles from './segmented-control.module.css';

export type SegmentedControlProps<T extends string> = Forced & {
  readonly disabled?: boolean;
  // The group's accessible name.
  readonly label: string;
  readonly onChange: (value: T) => void;
  readonly options: readonly SegmentedOption<T>[];
  readonly value: T;
};

type SegmentedOption<T extends string> = { readonly label: string; readonly value: T };

// The option a key moves to from `index`, or `null` for a key that does not move the choice. It stops at both ends.
const targetOf = (key: string, index: number, last: number): null | number => {
  switch (key) {
    case 'ArrowDown':
    case 'ArrowRight': {
      return Math.min(index + 1, last);
    }
    case 'ArrowLeft':
    case 'ArrowUp': {
      return Math.max(index - 1, 0);
    }
    case 'End': {
      return last;
    }
    case 'Home': {
      return 0;
    }
    default: {
      return null;
    }
  }
};

// One choice of a few, as a radio group: the arrows, Home and End move the choice, as native radios do. The segments
// are equal, as macOS draws them, so one indicator under the chosen one slides by `transform` from its index alone.
export const SegmentedControl = <T extends string>({
  disabled = false,
  label,
  onChange,
  options,
  value,
  ...forced
}: SegmentedControlProps<T>): JSX.Element => {
  const buttonsRef = useRef<(HTMLButtonElement | null)[]>([]);
  const selected = Math.max(
    options.findIndex((option) => option.value === value),
    0,
  );

  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>): void => {
    const target = targetOf(event.key, selected, options.length - 1);

    if (target === null) {
      return;
    }

    event.preventDefault();
    const option = options[target];

    if (option !== undefined && target !== selected) {
      onChange(option.value);
      buttonsRef.current[target]?.focus();
    }
  };

  return (
    <div
      aria-disabled={disabled || undefined}
      aria-label={label}
      className={styles.control}
      role="radiogroup"
      style={{ '--segment-count': options.length, '--segment-index': selected } as CSSProperties}
    >
      <span aria-hidden="true" className={styles.indicator} />
      {options.map((option, index) => (
        <button
          key={option.value}
          {...(index === selected ? forced : {})}
          aria-checked={index === selected}
          className={styles.option}
          disabled={disabled}
          onClick={() => {
            if (index !== selected) {
              onChange(option.value);
            }
          }}
          onKeyDown={onKeyDown}
          ref={(element) => {
            buttonsRef.current[index] = element;
          }}
          role="radio"
          tabIndex={index === selected ? 0 : -1}
          type="button"
        >
          {option.label}
        </button>
      ))}
    </div>
  );
};
