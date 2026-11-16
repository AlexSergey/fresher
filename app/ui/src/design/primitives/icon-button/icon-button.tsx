import type { JSX } from 'react';

import type { Forced } from '../forced';

import { Icon } from '../../icons/icon';
import { type IconShape } from '../../icons/icons';
import { type ButtonSize } from '../button/button';
import buttonStyles from '../button/button.module.css';
import { Tooltip } from '../tooltip/tooltip';
import styles from './icon-button.module.css';

export type IconButtonProps = Forced & {
  readonly disabled?: boolean;
  readonly icon: IconShape;
  // The accessible name, and the text of its tooltip.
  readonly label: string;
  readonly onClick?: () => void;
  readonly size?: ButtonSize;
  readonly variant?: 'ghost' | 'secondary';
};

// A square button with one icon, named by its label, which its tooltip shows.
export const IconButton = ({
  disabled = false,
  icon,
  label,
  onClick,
  size = 'medium',
  variant = 'ghost',
  ...forced
}: IconButtonProps): JSX.Element => (
  <Tooltip content={label} repeatsName>
    {() => (
      <button
        {...forced}
        aria-label={label}
        className={`${buttonStyles.button ?? ''} ${styles.square ?? ''}`}
        data-size={size}
        data-variant={variant}
        disabled={disabled}
        onClick={onClick}
        type="button"
      >
        <Icon icon={icon} />
      </button>
    )}
  </Tooltip>
);
