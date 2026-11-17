import type { Level } from './danger-badge/danger-badge';

// One part of a whole, as the storage ring and the usage bar draw it: in its tone's colour, named by its label, with
// its size already formatted by the caller.
export type Segment = {
  readonly key: string;
  readonly label: string;
  readonly size: string;
  readonly tone: Tone;
  // The part's share is `value / total`; a value of 0 or less is not drawn.
  readonly value: number;
};

// The colour of a part: a danger level keeps its colour; the neutral tones carry no light (goal, section 6): `plain`
// (your folders) a quiet graphite, `system` a darker graphite, `free` an outline with no fill, `unknown` hatched.
export type Tone = 'free' | 'plain' | 'system' | 'unknown' | Level;

// A chart's accessible name: the caller's name for the whole, then every drawn segment with its size.
export const chartLabel = (name: string, segments: readonly Segment[]): string => {
  const parts = segments.filter(({ value }) => value > 0).map(({ label, size }) => `${label} ${size}`);

  return parts.length === 0 ? name : `${name}: ${parts.join(', ')}`;
};
