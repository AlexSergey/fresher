import type { JSX, ReactNode, Ref } from 'react';

import styles from './surface.module.css';

export type SurfaceProps = {
  readonly children?: ReactNode;
  // Added to the surface's own class: a card's spotlight, for example.
  readonly className?: string;
  // 1: cards and the hero panel; 2: a hovered card; 3: dialogs, the selection bar, toasts.
  readonly elevation: SurfaceElevation;
  readonly padding?: SurfacePadding;
  readonly radius?: SurfaceRadius;
  readonly ref?: Ref<HTMLDivElement>;
};

type SurfaceElevation = 1 | 2 | 3;

type SurfacePadding = 'large' | 'medium' | 'none' | 'small';

type SurfaceRadius = 'lg' | 'md' | 'xl';

// A raised panel: the gradient fill, the hairline and the top highlight of its elevation; only level 3 casts a shadow
// that reads as an overlay.
export const Surface = ({
  children,
  className,
  elevation,
  padding = 'medium',
  radius = 'md',
  ref,
}: SurfaceProps): JSX.Element => (
  <div
    className={className === undefined ? styles.surface : `${styles.surface ?? ''} ${className}`}
    data-elevation={elevation}
    data-padding={padding}
    data-radius={radius}
    ref={ref}
  >
    {children}
  </div>
);
