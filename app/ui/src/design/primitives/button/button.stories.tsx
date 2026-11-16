import type { JSX } from 'react';

import { stories as storiesOf, type Story } from '../../../gallery/story';
import { type IconShape, RotateCw, Settings, Trash2 } from '../../icons/icons';
import { Button, type ButtonProps } from './button';
import styles from './button.stories.module.css';

type State = Pick<ButtonProps, 'busy' | 'data-focus' | 'data-hover' | 'data-pressed' | 'disabled'>;

type Variant = NonNullable<ButtonProps['variant']>;

const VARIANTS: Readonly<Record<Variant, { readonly icon: IconShape; readonly label: string }>> = {
  danger: { icon: Trash2, label: 'Delete permanently' },
  ghost: { icon: Settings, label: 'Show details' },
  primary: { icon: Trash2, label: 'Move to Trash' },
  secondary: { icon: RotateCw, label: 'Scan again' },
};

const STATES: Readonly<Record<string, State>> = {
  busy: { busy: true },
  disabled: { disabled: true },
  focus: { 'data-focus': true },
  hover: { 'data-hover': true },
  pressed: { 'data-pressed': true },
  rest: {},
};

// One variant in one state: the three sizes with their icon, and the medium one without.
const row = (variant: Variant, state: State): (() => JSX.Element) =>
  function Story(): JSX.Element {
    const { icon, label } = VARIANTS[variant];

    return (
      <div className={styles.row}>
        {(['small', 'medium', 'large'] as const).map((size) => (
          <Button icon={icon} key={size} size={size} variant={variant} {...state}>
            {label}
          </Button>
        ))}
        <Button variant={variant} {...state}>
          {label}
        </Button>
      </div>
    );
  };

export const stories: readonly Story[] = Object.keys(VARIANTS).flatMap((variant) =>
  storiesOf(
    'Button',
    Object.fromEntries(
      Object.entries(STATES).map(([state, props]) => [`${variant}-${state}`, row(variant as Variant, props)]),
    ),
  ),
);
