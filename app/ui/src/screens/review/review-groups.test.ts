import { describe, expect, it } from 'vitest';

import type { DangerLevel } from '../../ipc/bindings.generated';
import type { ReviewGroup } from './review-groups';

import { reviewZones } from './review-groups';

const group = (category: string, highest: DangerLevel, allocated: number): ReviewGroup => ({
  category,
  count: 1,
  highest,
  rows: { findings: [], kind: 'picked' },
  sizes: { allocated, completeness: 'complete', logical: allocated, reclaimable: allocated },
});

describe('reviewZones', () => {
  describe('negative cases', () => {
    it('leaves out a zone with no group', () => {
      const zones = reviewZones([group('a', 'high', 1), group('b', 'safe', 2)]);

      expect(zones.map((zone) => zone.level)).toStrictEqual(['safe', 'high']);
    });

    it('never puts a group in a lower zone than its highest level', () => {
      const zones = reviewZones([group('a', 'medium', 1), group('b', 'low', 9)]);

      for (const zone of zones) {
        expect(zone.groups.every((member) => member.highest === zone.level)).toBe(true);
      }
    });

    it('has no zone for no groups', () => {
      expect(reviewZones([])).toStrictEqual([]);
    });
  });

  describe('positive cases', () => {
    it('orders the zones Safe to High', () => {
      const zones = reviewZones([
        group('a', 'high', 1),
        group('b', 'medium', 1),
        group('c', 'safe', 1),
        group('d', 'low', 1),
      ]);

      expect(zones.map((zone) => zone.level)).toStrictEqual(['safe', 'low', 'medium', 'high']);
    });

    it('keeps the groups largest first inside a zone', () => {
      const [zone] = reviewZones([group('big', 'low', 9), group('small', 'low', 1)]);

      expect(zone?.groups.map((member) => member.category)).toStrictEqual(['big', 'small']);
    });
  });
});
