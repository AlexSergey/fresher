import { type JSX, useState } from 'react';

import type { Unit } from '../../ipc/bindings.generated';

import { TextField } from '../../design/primitives/text-field/text-field';
import { limitsText, notInLimitsText } from './texts';

type NumberFieldProps = {
  readonly disabled: boolean;
  readonly label: string;
  readonly max: number;
  readonly min: number;
  readonly onChange: (value: number) => void;
  // The stored value's problem, shown until something else is typed wrong.
  readonly problem: string | undefined;
  readonly unit: null | Unit;
  readonly value: number;
};

const WHOLE_NUMBER = /^-?\d+$/u;

// The number typed, when it is a whole number within its limits; `null` otherwise.
const parse = (typed: string, min: number, max: number): null | number => {
  if (!WHOLE_NUMBER.test(typed.trim())) {
    return null;
  }

  const number = Number(typed);

  return number >= min && number <= max ? number : null;
};

// A number setting, checked as it is typed: a number outside its limits stays in the field with its error and is
// never sent. A new value from the outside replaces what was typed.
export const NumberField = ({
  disabled,
  label,
  max,
  min,
  onChange,
  problem,
  unit,
  value,
}: NumberFieldProps): JSX.Element => {
  const [draft, setDraft] = useState({ shown: value, typed: String(value) });

  if (draft.shown !== value) {
    setDraft({ shown: value, typed: String(value) });
  }

  const error = parse(draft.typed, min, max) === null ? notInLimitsText(min, max) : problem;

  return (
    <TextField
      description={limitsText(min, max, unit)}
      disabled={disabled}
      hideLabel
      label={label}
      onChange={(typed) => {
        setDraft({ shown: value, typed });
        const number = parse(typed, min, max);

        if (number !== null) {
          onChange(number);
        }
      }}
      type="number"
      value={draft.typed}
      {...(error === undefined ? {} : { error })}
    />
  );
};
