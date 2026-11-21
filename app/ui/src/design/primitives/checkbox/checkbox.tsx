import { type ChangeEvent, type JSX, useEffect, useId, useRef } from 'react';

import type { Forced } from '../forced';

import styles from './checkbox.module.css';

export type CheckboxProps = Forced & {
  // The id of another text that says more, read after the description: a heading the checkbox acts for.
  readonly 'aria-describedby'?: string;
  // `mixed` when some, not all, of what it stands for is chosen.
  readonly checked: 'mixed' | boolean;
  // A second line under the label, linked to the input as its description.
  readonly description?: string;
  readonly disabled?: boolean;
  // The input's accessible name; visually hidden with `hideLabel` (a row whose title names it).
  readonly hideLabel?: boolean;
  readonly label: string;
  readonly onChange: (checked: boolean) => void;
  // -1 inside a row that is its list's one tab stop: the row toggles it with Space, a click still reaches it.
  readonly tabIndex?: -1;
};

// A real checkbox, visually replaced: the input stays the focus and click target, a box with a drawn mark shows it.
export const Checkbox = ({
  'aria-describedby': describedBy,
  checked,
  description,
  disabled = false,
  hideLabel = false,
  label,
  onChange,
  tabIndex,
  ...forced
}: CheckboxProps): JSX.Element => {
  const inputRef = useRef<HTMLInputElement>(null);
  const inputId = useId();
  const descriptionId = useId();
  const mixed = checked === 'mixed';

  // `indeterminate` is a property, not an attribute; the browser clears it on a click, so every render sets it again.
  useEffect(() => {
    if (inputRef.current !== null) {
      inputRef.current.indeterminate = mixed;
    }
  });

  return (
    <div className={styles.checkbox} data-disabled={disabled || undefined}>
      <span className={styles.control}>
        <input
          {...forced}
          aria-describedby={
            [description === undefined ? undefined : descriptionId, describedBy].filter(Boolean).join(' ') || undefined
          }
          checked={checked === true}
          className={styles.input}
          disabled={disabled}
          id={inputId}
          onChange={(event: ChangeEvent<HTMLInputElement>) => {
            // The parent decides: until it renders again, the box keeps showing what it was given.
            event.target.indeterminate = mixed;
            onChange(event.target.checked);
          }}
          ref={inputRef}
          tabIndex={tabIndex}
          type="checkbox"
        />
        <span aria-hidden="true" className={styles.box}>
          <svg className={styles.glyph} viewBox="0 0 16 16">
            <path className={styles.mark} d="M3.5 8.5 6.5 11.5 12.5 4.5" />
            <path className={styles.bar} d="M4.5 8h7" />
          </svg>
        </span>
      </span>
      <span className={hideLabel ? 'u-visually-hidden' : styles.text}>
        <label className={styles.label} htmlFor={inputId}>
          {label}
        </label>
        {description === undefined ? null : (
          <span className={styles.description} id={descriptionId}>
            {description}
          </span>
        )}
      </span>
    </div>
  );
};
