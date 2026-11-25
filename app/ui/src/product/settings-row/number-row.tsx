import { type JSX, useEffect, useState } from 'react';

import type { Unit } from '../../ipc/bindings.generated';

import { Slider } from '../../design/primitives/slider/slider';
import { t } from '../../i18n';
import { NumberField } from './number-field';
import styles from './settings-row.module.css';
import { unitText } from './texts';

type NumberRowProps = {
  readonly disabled: boolean;
  readonly label: string;
  readonly max: number;
  readonly min: number;
  readonly onChange: (value: number) => void;
  readonly problem: string | undefined;
  readonly unit: null | Unit;
  readonly value: number;
};

// A whole-number setting: a slider between the core's bounds, its number field and the unit. The slider moves the
// field as it goes and sends one value, when it is let go, and holds it until the stored value comes back as it; a
// typed number is checked by the field and moves the slider.
export const NumberRow = ({
  disabled,
  label,
  max,
  min,
  onChange,
  problem,
  unit,
  value,
}: NumberRowProps): JSX.Element => {
  const [moving, setMoving] = useState<number | undefined>(undefined);
  const unitName = unitText(unit);

  useEffect(() => {
    if (moving !== undefined && moving === value) {
      setMoving(undefined);
    }
  }, [moving, value]);

  return (
    <div className={styles.numberRow}>
      <div className={styles.sliderCell}>
        <Slider
          disabled={disabled}
          label={
            unitName === undefined
              ? label
              : t('ui.settings-row.slider', {
                  label: { kind: 'text', value: label },
                  unit: { kind: 'text', value: unitName },
                })
          }
          max={max}
          min={min}
          onChange={setMoving}
          onCommit={onChange}
          value={moving ?? value}
        />
      </div>
      <div className={styles.fieldCell}>
        <NumberField
          disabled={disabled}
          label={label}
          max={max}
          min={min}
          onChange={(typed) => {
            setMoving(undefined);
            onChange(typed);
          }}
          problem={problem}
          unit={unit}
          value={moving ?? value}
        />
      </div>
      {unitName === undefined ? null : <span className={styles.unit}>{unitName}</span>}
    </div>
  );
};
