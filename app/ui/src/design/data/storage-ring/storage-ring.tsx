import { type CSSProperties, type JSX, type ReactNode, useState } from 'react';

import { ScanSweep } from '../../effects/scan-sweep/scan-sweep';
import { useMotion } from '../../motion';
import { chartLabel, type Segment } from '../segment';
import { type Arc, arcsOf } from './arcs';
import styles from './storage-ring.module.css';

export type StorageRingProps = {
  // The centre, over the ring: the hero number, or the Scan button while idle.
  readonly children?: ReactNode;
  // The caller's name for the whole; the ring's accessible name adds every segment and its size.
  readonly label: string;
  readonly segments: readonly Segment[];
  // Pixels, the ring's width and height.
  readonly size?: number;
  // `empty` before a scan, `scanning` with the sweep, `done` after it, `partial` when part of it could not be read.
  readonly state: StorageRingState;
  // The whole each segment takes its share of.
  readonly total: number;
};

type StorageRingState = 'done' | 'empty' | 'partial' | 'scanning';

const STROKE = 12;
const DEGREES = 180 / Math.PI;

type Geometry = {
  readonly centre: number;
  readonly circumference: number;
  readonly radius: number;
};

// An arc as a dashed circle: a dash of the arc's length, shown from its start by a rotation. Round caps reach half a
// stroke past each end, so the dash is shortened by one stroke; an arc shorter than its caps is drawn square-ended.
const ArcCircle = ({ arc, geometry }: { readonly arc: Arc; readonly geometry: Geometry }): JSX.Element => {
  const { centre, circumference, radius } = geometry;
  const cap = (STROKE / radius) * DEGREES;
  const whole = arc.sweep === 360;
  const round = whole || arc.sweep >= cap;
  const drawn = whole || !round ? arc.sweep : arc.sweep - cap;
  const turn = arc.start + (whole || !round ? 0 : cap / 2) - 90;

  return (
    <circle
      className={styles.arc}
      cx={centre}
      cy={centre}
      data-cap={round ? 'round' : 'square'}
      data-tone={arc.tone}
      r={radius}
      style={{
        strokeDasharray: `${circumference} ${circumference}`,
        strokeDashoffset: circumference - (drawn / 360) * circumference,
        transform: `rotate(${turn}deg)`,
      }}
    />
  );
};

const isFinished = (state: StorageRingState): boolean => state === 'done' || state === 'partial';

// The storage ring: a faint track with one arc per segment in its tone's colour, over a blurred copy of the arcs as
// their glow. Arcs grow by stroke-dashoffset; a scan's sweep circles the band; a finished scan plays one sheen. The
// centre is an HTML layer over the ring.
export const StorageRing = ({ children, label, segments, size = 232, state, total }: StorageRingProps): JSX.Element => {
  const motion = useMotion();
  const [previous, setPrevious] = useState(state);
  // How many scans finished while shown: each one plays the sheen once.
  const [sheens, setSheens] = useState(0);

  if (previous !== state) {
    setPrevious(state);

    if (previous === 'scanning' && isFinished(state)) {
      setSheens(sheens + 1);
    }
  }

  const centre = size / 2;
  const radius = centre - STROKE / 2;
  const geometry = { centre, circumference: 2 * Math.PI * radius, radius };
  const arcs = state === 'empty' ? [] : arcsOf(segments, total);
  const drawn = arcs.map((arc) => <ArcCircle arc={arc} geometry={geometry} key={arc.key} />);
  const box = { height: size, viewBox: `0 0 ${size} ${size}`, width: size };

  return (
    <div
      className={styles.ring}
      style={
        {
          '--ring-circumference': `${geometry.circumference}px`,
          '--ring-size': `${size}px`,
          '--ring-stroke': `${STROKE}px`,
        } as CSSProperties
      }
    >
      <svg aria-hidden="true" className={styles.glow} focusable="false" {...box}>
        {drawn}
      </svg>
      <svg
        aria-label={chartLabel(label, state === 'empty' ? [] : segments)}
        className={styles.arcs}
        data-ring="arcs"
        focusable="false"
        role="img"
        {...box}
      >
        <circle
          className={styles.track}
          cx={centre}
          cy={centre}
          data-track={state === 'partial' ? 'partial' : 'whole'}
          r={radius}
        />
        {drawn}
      </svg>
      <ScanSweep active={state === 'scanning'} band={STROKE} />
      {sheens > 0 && motion === 'full' && isFinished(state) ? (
        <span aria-hidden="true" className={styles.sheen} data-sheen key={sheens} />
      ) : null}
      <div className={styles.centre}>{children}</div>
    </div>
  );
};
