import type { JSX } from 'react';

import type { IconShape } from './icons';

export type IconProps = {
  readonly className?: string;
  readonly icon: IconShape;
  // Given, the icon means something on its own and is announced; without it, it decorates the text beside it.
  readonly label?: string;
  // 16 by default, 20 in cards, 24 in the hero.
  readonly size?: IconSize;
};

type IconSize = 16 | 20 | 24;

const STROKE = 1.75;

// An icon of the allowlist, drawn in the current colour at the design's stroke.
export const Icon = ({ className, icon: Shape, label, size = 16 }: IconProps): JSX.Element => {
  const meaning = label === undefined ? { 'aria-hidden': true } : { 'aria-label': label, role: 'img' };

  return (
    <Shape
      {...meaning}
      {...(className === undefined ? {} : { className })}
      color="currentColor"
      focusable="false"
      size={size}
      strokeWidth={STROKE}
    />
  );
};
