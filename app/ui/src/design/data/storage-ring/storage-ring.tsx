import { type CSSProperties, type JSX, type ReactNode, useState } from 'react';

import { ScanSweep } from '../../effects/scan-sweep/scan-sweep';
import { useMotion } from '../../motion';
import { chartLabel, type Segment } from '../segment';
import { type Arc, arcsOf, GAP } from './arcs';
import styles from './storage-ring.module.css';

export type StorageRingProps = {
  // The centre, over the ring: the hero number, or the Scan button while idle.
  readonly children?: ReactNode;
  // The label at the free segment's slot, just outside the band (the legend carries it for assistive technology).
  // Without it the ring takes no room for one.
  readonly gapLabel?: string;
  // The key of the segment the pointer or the legend points at: the other arcs step back.
  readonly highlighted?: null | string;
  // The caller's name for the whole; the ring's accessible name adds every segment and its size.
  readonly label: string;
  // Called with an arc's key when the pointer enters it, and with `null` when it leaves.
  readonly onHighlight?: (key: null | string) => void;
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
const ArcCircle = ({
  arc,
  dimmed,
  geometry,
  onHighlight,
}: {
  readonly arc: Arc;
  readonly dimmed: boolean;
  readonly geometry: Geometry;
  readonly onHighlight: ((key: null | string) => void) | undefined;
}): JSX.Element => {
  const { centre, circumference, radius } = geometry;
  const cap = (STROKE / radius) * DEGREES;
  const whole = arc.sweep === 360;
  const round = whole || arc.sweep >= cap;
  const drawn = whole || !round ? arc.sweep : arc.sweep - cap;
  const turn = arc.start + (whole || !round ? 0 : cap / 2) - 90;

  const enter =
    onHighlight === undefined
      ? undefined
      : (): void => {
          onHighlight(arc.key);
        };
  const leave =
    onHighlight === undefined
      ? undefined
      : (): void => {
          onHighlight(null);
        };

  return (
    <circle
      className={styles.arc}
      cx={centre}
      cy={centre}
      data-cap={round ? 'round' : 'square'}
      data-dimmed={dimmed ? '' : undefined}
      data-tone={arc.tone}
      onPointerEnter={enter}
      onPointerLeave={leave}
      r={radius}
      style={{
        strokeDasharray: `${circumference} ${circumference}`,
        strokeDashoffset: circumference - (drawn / 360) * circumference,
        transform: `rotate(${turn}deg)`,
      }}
    />
  );
};

// How far past the band's outer edge the gap label's anchor sits, in pixels.
const LABEL_OFFSET = 6;

// The label of the free slot: at the middle of the slot, just outside the band, on the side facing away from the centre.
const GapLabel = ({
  arc,
  geometry,
  highlighted,
  text,
}: {
  readonly arc: Arc;
  readonly geometry: Geometry;
  readonly highlighted: boolean;
  readonly text: string;
}): JSX.Element => {
  const angle = (arc.start + (arc.sweep + GAP) / 2) / DEGREES;
  const reach = geometry.radius + STROKE / 2 + LABEL_OFFSET;
  const sine = Math.sin(angle);
  const cosine = Math.cos(angle);

  return (
    <span
      aria-hidden="true"
      className={styles.gapLabel}
      data-gap-label
      data-highlighted={highlighted ? '' : undefined}
      style={{
        left: geometry.centre + sine * reach,
        top: geometry.centre - cosine * reach,
        transform: `translate(${String(-50 + sine * 50)}%, ${String(-50 - cosine * 50)}%)`,
      }}
    >
      {text}
    </span>
  );
};

const isFinished = (state: StorageRingState): boolean => state === 'done' || state === 'partial';

// The storage ring: a faint track with one arc per segment in its tone's colour, over a blurred copy of the arcs as
// their glow. Arcs grow by stroke-dashoffset; a scan's sweep circles the band; a finished scan plays one sheen. The
// centre is an HTML layer over the ring.
export const StorageRing = ({
  children,
  gapLabel,
  highlighted = null,
  label,
  onHighlight,
  segments,
  size = 232,
  state,
  total,
}: StorageRingProps): JSX.Element => {
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
  const gap = gapLabel === undefined ? undefined : arcs.find((arc) => !arc.drawn);
  const circles = (listening: boolean): readonly JSX.Element[] =>
    arcs
      .filter((arc) => arc.drawn)
      .map((arc) => (
        <ArcCircle
          arc={arc}
          dimmed={highlighted !== null && highlighted !== arc.key}
          geometry={geometry}
          key={arc.key}
          onHighlight={listening ? onHighlight : undefined}
        />
      ));
  const box = { height: size, viewBox: `0 0 ${size} ${size}`, width: size };

  return (
    <div
      className={styles.ring}
      data-labelled={gap === undefined ? undefined : ''}
      style={
        {
          '--ring-circumference': `${geometry.circumference}px`,
          '--ring-size': `${size}px`,
          '--ring-stroke': `${STROKE}px`,
        } as CSSProperties
      }
    >
      <svg aria-hidden="true" className={styles.glow} focusable="false" {...box}>
        {circles(false)}
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
        {circles(true)}
      </svg>
      <ScanSweep active={state === 'scanning'} band={STROKE} />
      {sheens > 0 && motion === 'full' && isFinished(state) ? (
        <span aria-hidden="true" className={styles.sheen} data-sheen key={sheens} />
      ) : null}
      {gap === undefined || gapLabel === undefined ? null : (
        <GapLabel arc={gap} geometry={geometry} highlighted={highlighted === gap.key} text={gapLabel} />
      )}
      <div className={styles.centre}>{children}</div>
    </div>
  );
};
