import type { Segment, Tone } from '../segment';

// One segment's arc on the ring, in degrees clockwise from the top: where it starts and how far it is drawn. A gap
// of GAP follows every arc, except an arc that is the whole circle.
export type Arc = {
  readonly key: string;
  readonly start: number;
  readonly sweep: number;
  readonly tone: Tone;
};

// The gap between two arcs.
export const GAP = 2.5;

// The least a drawn arc takes, so a tiny segment stays visible.
export const MIN_SWEEP = 2;

const FULL = 360;

const sum = (values: readonly number[]): number => values.reduce((total, value) => total + value, 0);

// The angle each segment takes with its gap: its share of the circle, at least the least arc and its gap. When the
// raised tiny ones push the whole past the circle, the excess comes out of the larger ones, in proportion.
const slotsOf = (shares: readonly number[]): readonly number[] => {
  const least = MIN_SWEEP + GAP;
  const raised = shares.map((share) => Math.max(share, least));
  const excess = sum(raised) - FULL;
  const spare = sum(raised.map((slot) => slot - least));

  if (excess <= 0 || spare <= 0) {
    return raised;
  }

  const cut = Math.min(excess / spare, 1);

  return raised.map((slot) => slot - (slot - least) * cut);
};

// The arcs of the segments, in their order: each arc and the gap after it take the segment's share of the total, the
// arc at least MIN_SWEEP; nothing goes past the full circle. A segment of 0 or less is not drawn; one segment that is
// the whole total is the whole circle.
export const arcsOf = (segments: readonly Segment[], total: number): readonly Arc[] => {
  const drawn = total > 0 ? segments.filter(({ value }) => value > 0) : [];
  const [only] = drawn;

  if (drawn.length === 1 && only !== undefined && only.value >= total) {
    return [{ key: only.key, start: 0, sweep: FULL, tone: only.tone }];
  }

  const slots = slotsOf(drawn.map(({ value }) => (value / total) * FULL));
  const arcs: Arc[] = [];
  let start = 0;

  for (const [index, { key, tone }] of drawn.entries()) {
    const slot = slots[index] ?? 0;

    // Only with more segments than the circle holds at their least.
    if (start + slot > FULL + 1e-9) {
      break;
    }

    arcs.push({ key, start, sweep: slot - GAP, tone });
    start += slot;
  }

  return arcs;
};
