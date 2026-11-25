import { type JSX, useId } from 'react';

import styles from './logo-mark.module.css';

export type LogoMarkProps = {
  // The side of the square mark, in px.
  readonly size?: number;
};

const LEAF = 'M8 92 C8 46 32 10 72 8 L88 8 Q92 8 92 12 L92 54 C92 80 70 92 44 92 Z';
const BAND = 'M6 96 C28 60 56 44 96 34 L96 48 C64 56 40 70 20 96 Z';
const STAR = 'M34 17 C35 28 37 30 48 31 C37 32 35 34 34 45 C33 34 31 32 20 31 C31 30 33 28 34 17 Z';

// The logo: a graphite leaf, an iris swoosh across it, a 1 px rim of the swoosh's gradient and a star. Decorative: the
// name beside it says what it is.
export const LogoMark = ({ size = 22 }: LogoMarkProps): JSX.Element => {
  const id = useId();
  const body = `${id}-body`;
  const band = `${id}-band`;
  const clip = `${id}-clip`;

  return (
    <svg aria-hidden="true" height={size} viewBox="0 0 100 100" width={size}>
      <defs>
        <linearGradient id={body} x1="0" x2="0.4" y1="0" y2="1">
          <stop className={styles.bodyTop} offset="0" />
          <stop className={styles.bodyBottom} offset="1" />
        </linearGradient>
        <linearGradient id={band} x1="0" x2="1" y1="1" y2="0">
          <stop className={styles.swooshStart} offset="0" />
          <stop className={styles.swooshEnd} offset="1" />
        </linearGradient>
        <clipPath id={clip}>
          <path d={LEAF} />
        </clipPath>
      </defs>
      <path d={LEAF} data-part="leaf" fill={`url(#${body})`} />
      <g clipPath={`url(#${clip})`}>
        <path d={BAND} data-part="band" fill={`url(#${band})`} />
      </g>
      <path
        d={LEAF}
        data-part="rim"
        fill="none"
        stroke={`url(#${band})`}
        strokeOpacity="0.5"
        strokeWidth="1"
        vectorEffect="non-scaling-stroke"
      />
      <path className={styles.star} d={STAR} data-part="star" />
    </svg>
  );
};
