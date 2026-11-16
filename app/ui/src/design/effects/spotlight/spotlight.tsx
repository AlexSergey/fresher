import { type RefObject, useRef } from 'react';

import { useSpotlight } from '../../motion';
import styles from './spotlight.module.css';

export type SpotlightProps = {
  readonly className: string;
  readonly ref: RefObject<HTMLDivElement | null>;
};

// A card's hover light: a soft highlight that follows the pointer. The card spreads these props on its root (adding
// the class to its own), so the light is the root's `::before` and adds no element.
export const useSpotlightProps = (): SpotlightProps => {
  const ref = useRef<HTMLDivElement>(null);

  useSpotlight(ref);

  return { className: styles.spotlight ?? '', ref };
};
