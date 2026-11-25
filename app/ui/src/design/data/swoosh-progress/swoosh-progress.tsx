import { type JSX, useEffect, useId, useRef, useState } from 'react';

import styles from './swoosh-progress.module.css';

export type SwooshProgressProps = {
  // The progress line's accessible name.
  readonly label: string;
  // How far, 0 to 1 (clamped); `null` when how much is left is not known.
  readonly value: null | number;
};

type Point = { readonly x: number; readonly y: number };

const PATH = 'M30 200 C220 110 480 70 800 50';
const STAR = 'M528 72 C529 81 531 83 540 84 C531 85 529 87 528 96 C527 87 525 85 516 84 C525 83 527 81 528 72 Z';
const STAR_CENTRE: Point = { x: 528, y: 84 };
const LENGTH = 1000;
const UNKNOWN_DASH = 160;

const clamp = (value: number): number => Math.min(Math.max(value, 0), 1);

// Where the fill ends; `null` where the DOM cannot measure a path (unit tests).
const headOf = (path: null | SVGPathElement, fraction: number): null | Point => {
  if (path === null || typeof path.getPointAtLength !== 'function') {
    return null;
  }
  const { x, y } = path.getPointAtLength(fraction * path.getTotalLength());

  return { x, y };
};

// The logo's swoosh as a progress line: a track, a glowing gradient fill and the logo's star at the fill's head.
export const SwooshProgress = ({ label, value }: SwooshProgressProps): JSX.Element => {
  const id = useId();
  const gradient = `${id}-gradient`;
  const glow = `${id}-glow`;
  const pathRef = useRef<SVGPathElement>(null);
  const [head, setHead] = useState<null | Point>(null);

  const known = value !== null;
  const fraction = known && Number.isFinite(value) ? clamp(value) : 0;
  // The known fill is one dash pushed back along the path: the offset moves (inside the motion budget), the length not.
  const dash = known
    ? { strokeDasharray: `${LENGTH} ${LENGTH}`, strokeDashoffset: (1 - fraction) * LENGTH }
    : { strokeDasharray: `${UNKNOWN_DASH} ${LENGTH}` };

  useEffect(() => {
    setHead(known && fraction > 0 ? headOf(pathRef.current, fraction) : null);
  }, [known, fraction]);

  const common = { d: PATH, fill: 'none', pathLength: LENGTH, strokeLinecap: 'round', strokeWidth: 14 } as const;
  const aria = known
    ? { 'aria-valuemax': 100, 'aria-valuemin': 0, 'aria-valuenow': Math.round(fraction * 100) }
    : { 'data-indeterminate': true };

  return (
    <svg
      aria-label={label}
      className={styles.swoosh}
      preserveAspectRatio="none"
      role="progressbar"
      viewBox="0 0 830 226"
      {...aria}
    >
      <defs>
        <linearGradient id={gradient} x1="0" x2="1" y1="1" y2="0">
          <stop className={styles.start} offset="0" />
          <stop className={styles.end} offset="1" />
        </linearGradient>
        <filter height="300%" id={glow} width="300%" x="-100%" y="-100%">
          <feGaussianBlur stdDeviation="6" />
        </filter>
      </defs>
      <path {...common} className={styles.track} ref={pathRef} />
      {known && fraction === 0 ? null : (
        <>
          <path
            {...common}
            className={known ? styles.dash : styles.sweep}
            filter={`url(#${glow})`}
            opacity="0.6"
            stroke={`url(#${gradient})`}
            {...dash}
          />
          <path {...common} className={known ? styles.dash : styles.sweep} stroke={`url(#${gradient})`} {...dash} />
        </>
      )}
      {known && fraction > 0 ? <circle className={styles.cap} cx="30" cy="200" r="7" /> : null}
      {head === null ? null : (
        <path
          className={styles.star}
          d={STAR}
          data-testid="swoosh-star"
          transform={`translate(${head.x - STAR_CENTRE.x} ${head.y - STAR_CENTRE.y})`}
        />
      )}
    </svg>
  );
};
