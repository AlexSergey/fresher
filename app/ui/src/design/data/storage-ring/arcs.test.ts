import { describe, expect, it } from 'vitest';

import type { Segment } from '../segment';

import { arcsOf, GAP, MIN_SWEEP } from './arcs';

const segment = (key: string, value: number, tone: Segment['tone'] = 'safe'): Segment => ({
  key,
  label: key,
  size: `${value} GB`,
  tone,
  value,
});

// The angle the arcs take with their gaps.
const taken = (arcs: readonly { readonly sweep: number }[]): number =>
  arcs.reduce((sum, { sweep }) => sum + sweep + GAP, 0);

describe('arcsOf', () => {
  describe('negative cases', () => {
    it('draws nothing without segments or without a total', () => {
      expect(arcsOf([], 100)).toStrictEqual([]);
      expect(arcsOf([segment('a', 10)], 0)).toStrictEqual([]);
    });

    it('a segment of 0 is not drawn', () => {
      expect(arcsOf([segment('a', 0), segment('b', 50), segment('c', -3)], 100).map(({ key }) => key)).toStrictEqual([
        'b',
      ]);
    });

    it('a tiny segment is drawn at least 2 degrees', () => {
      const [tiny] = arcsOf([segment('a', 1), segment('b', 50_000)], 100_000);

      expect(tiny?.sweep).toBe(MIN_SWEEP);
      expect(MIN_SWEEP).toBe(2);
    });

    it('never draws past the full circle, and keeps every segment', () => {
      const arcs = arcsOf([segment('a', 99), segment('b', 1), segment('c', 1)], 100);

      expect(arcs).toHaveLength(3);
      expect(taken(arcs)).toBeCloseTo(360);
      expect(arcs.at(-1)?.sweep).toBe(MIN_SWEEP);
    });

    it('drops the segments the circle cannot hold at their least', () => {
      const many = Array.from({ length: 100 }, (_, index) => segment(String(index), 1));

      expect(arcsOf(many, 100)).toHaveLength(80);
    });

    it('draws nothing for a free segment', () => {
      const arcs = arcsOf([segment('a', 60), segment('free', 40, 'free')], 100);

      expect(arcs.filter(({ drawn }) => drawn).map(({ key }) => key)).toStrictEqual(['a']);
      expect(arcs.find(({ key }) => key === 'free')?.drawn).toBe(false);
    });

    it('a single free segment draws no arc', () => {
      const arcs = arcsOf([segment('free', 100, 'free')], 100);

      expect(arcs.some(({ drawn }) => drawn)).toBe(false);
    });

    it('never passes the full circle with a free slot among tiny arcs', () => {
      const arcs = arcsOf([segment('a', 1), segment('free', 1, 'free'), segment('b', 1), segment('c', 100)], 103);

      expect(taken(arcs)).toBeLessThanOrEqual(360 + 1e-9);
    });
  });

  describe('positive cases', () => {
    it('segment angles add up to the share of the total', () => {
      const arcs = arcsOf([segment('a', 30), segment('b', 20)], 100);

      expect(arcs.map(({ sweep }) => sweep + GAP)).toStrictEqual([108, 72]);
      expect(taken(arcs)).toBe(180);
    });

    it('each arc starts where the one before it and its gap end', () => {
      const arcs = arcsOf([segment('a', 25), segment('b', 25), segment('c', 50)], 100);

      expect(arcs.map(({ start }) => start)).toStrictEqual([0, 90, 180]);
    });

    it('one segment that is the whole total is the whole circle', () => {
      expect(arcsOf([segment('a', 100)], 100)).toStrictEqual([
        { drawn: true, key: 'a', start: 0, sweep: 360, tone: 'safe' },
      ]);
    });

    it('keeps the order and the tones of the segments', () => {
      const arcs = arcsOf([segment('a', 10, 'high'), segment('b', 30, 'safe'), segment('c', 20, 'medium')], 100);

      expect(arcs.map(({ key, tone }) => [key, tone])).toStrictEqual([
        ['a', 'high'],
        ['b', 'safe'],
        ['c', 'medium'],
      ]);
    });

    it('leaves the free slot where it falls, the next arc after it', () => {
      const arcs = arcsOf([segment('a', 25), segment('free', 25, 'free'), segment('b', 50)], 100);

      expect(arcs.map(({ key, start }) => [key, start])).toStrictEqual([
        ['a', 0],
        ['free', 90],
        ['b', 180],
      ]);
    });

    it('a ring of parts and free closes only with the gap', () => {
      const arcs = arcsOf([segment('a', 40), segment('b', 20), segment('free', 40, 'free')], 100);
      const drawnSweeps = arcs.filter(({ drawn }) => drawn).reduce((sum, { sweep }) => sum + sweep, 0);
      const freeSlot = (arcs.find(({ drawn }) => !drawn)?.sweep ?? 0) + GAP;

      expect(drawnSweeps + 2 * GAP + freeSlot).toBeCloseTo(360, 9);
    });

    it('takes the gap and the least arc of the shape it is given', () => {
      const arcs = arcsOf([segment('a', 50), segment('b', 0.01)], 100, { gap: 4, minSweep: 6 });

      expect(arcs.map(({ sweep }) => sweep)).toStrictEqual([176, 6]);
      expect(arcs[1]?.start).toBe(180);
    });
  });
});
