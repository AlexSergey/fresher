import type { JSX } from 'react';

export type VisuallyHiddenProps = {
  readonly children: string;
};

// Text for assistive technology only: read by VoiceOver, not shown (the `u-visually-hidden` utility of effects.css).
export const VisuallyHidden = ({ children }: VisuallyHiddenProps): JSX.Element => (
  <span className="u-visually-hidden">{children}</span>
);
