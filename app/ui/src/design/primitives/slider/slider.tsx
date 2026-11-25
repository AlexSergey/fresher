import { type JSX, useEffect, useRef } from 'react';

import type { Forced } from '../forced';

import styles from './slider.module.css';

export type SliderProps = Forced & {
  readonly describedBy?: string;
  readonly disabled?: boolean;
  readonly label: string;
  readonly max: number;
  readonly min: number;
  // Live: every whole number the thumb passes.
  readonly onChange: (value: number) => void;
  // Once, when the user lets go of the thumb or the key.
  readonly onCommit?: (value: number) => void;
  readonly value: number;
};

// A whole-number slider from `min` to `max`, both ends labelled. A value outside the bounds is shown at the nearest one.
export const Slider = ({
  describedBy,
  disabled = false,
  label,
  max,
  min,
  onChange,
  onCommit,
  value,
  ...forced
}: SliderProps): JSX.Element => {
  const inputRef = useRef<HTMLInputElement>(null);
  // Set by a change, cleared by the commit that follows: the pointer, the key and the browser's own `change` all
  // report the same release, which is sent once.
  const movedRef = useRef<number | undefined>(undefined);
  const commitRef = useRef(onCommit);

  commitRef.current = onCommit;

  const commit = (): void => {
    const latest = movedRef.current;

    if (latest === undefined) {
      return;
    }

    movedRef.current = undefined;
    commitRef.current?.(latest);
  };

  useEffect(() => {
    const element = inputRef.current;

    element?.addEventListener('change', commit);

    return (): void => {
      element?.removeEventListener('change', commit);
    };
  }, []);

  return (
    <div className={styles.slider}>
      <span className={styles.end}>{min}</span>
      <input
        {...forced}
        aria-describedby={describedBy}
        aria-label={label}
        className={styles.range}
        disabled={disabled}
        max={max}
        min={min}
        onChange={(event): void => {
          if (disabled) {
            return;
          }

          movedRef.current = event.currentTarget.valueAsNumber;
          onChange(event.currentTarget.valueAsNumber);
        }}
        onKeyUp={commit}
        onPointerUp={commit}
        ref={inputRef}
        step={1}
        type="range"
        value={Math.min(max, Math.max(min, value))}
      />
      <span className={styles.end}>{max}</span>
    </div>
  );
};
