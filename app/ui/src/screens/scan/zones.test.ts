import { describe, expect, it } from 'vitest';

import type { CategoryId, CategoryTotal, DangerLevel, Finding } from '../../ipc/bindings.generated';

import { NOTHING_SELECTED, type Selection } from '../../state/selection';
import { DEMO_FINDINGS } from '../../test/demo-scan';
import { zoneSelectAll, zonesOf } from './zones';

const total = (category: CategoryId, highestDanger: DangerLevel, count = 1): CategoryTotal => ({
  category,
  count,
  highestDanger,
  sizes: { allocated: 10, completeness: 'complete', logical: 10, reclaimable: 10 },
});

const whole = (...categories: readonly CategoryId[]): Selection => ({
  categories: new Map(categories.map((category) => [category, new Map()])),
  findings: new Map(),
  toCheck: new Set(),
});

// A real finding of the demo scan, placed in the zone's category `a`.
const PICKED: Finding = { ...(DEMO_FINDINGS[0] as Finding), category: 'a' };

const always = (): boolean => true;

describe('zonesOf', () => {
  describe('negative cases', () => {
    it('leaves out an empty zone', () => {
      const zones = zonesOf([total('a', 'safe'), total('b', 'high')]);

      expect(zones.map(({ level }) => level)).toStrictEqual(['safe', 'high']);
    });

    it('has no zone for no categories', () => {
      expect(zonesOf([])).toStrictEqual([]);
    });
  });

  describe('positive cases', () => {
    it('orders the zones Safe, Low, Medium, High', () => {
      const zones = zonesOf([total('d', 'high'), total('c', 'medium'), total('b', 'low'), total('a', 'safe')]);

      expect(zones.map(({ level }) => level)).toStrictEqual(['safe', 'low', 'medium', 'high']);
    });

    it('puts each category in the zone of its highest level, keeping their order', () => {
      const zones = zonesOf([total('x', 'low'), total('y', 'safe'), total('z', 'low')]);

      expect(zones.map(({ level, totals }) => [level, totals.map(({ category }) => category)])).toStrictEqual([
        ['safe', ['y']],
        ['low', ['x', 'z']],
      ]);
    });
  });
});

describe('zoneSelectAll', () => {
  const zone = {
    level: 'safe',
    totals: [total('a', 'safe'), total('b', 'safe'), total('c', 'safe', 0)],
  } as const;

  describe('negative cases', () => {
    it('gives no Select all to a zone with nothing selectable', () => {
      const empty = { level: 'safe', totals: [total('a', 'safe', 0)] } as const;

      expect(zoneSelectAll(NOTHING_SELECTED, empty, always)).toBeNull();
      expect(zoneSelectAll(NOTHING_SELECTED, zone, () => false)).toBeNull();
    });

    it('an unavailable category is never among the categories Select all ticks', () => {
      const all = zoneSelectAll(NOTHING_SELECTED, zone, (category) => category !== 'b');

      expect(all?.categories).toStrictEqual(['a']);
    });

    it('is unchecked when nothing of the zone is selected', () => {
      expect(zoneSelectAll(whole('other'), zone, always)?.checked).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('is checked when every selectable category is selected whole', () => {
      const all = zoneSelectAll(whole('a', 'b'), zone, always);

      expect(all).toStrictEqual({ categories: ['a', 'b'], checked: true });
    });

    it('is mixed when some categories are selected whole', () => {
      expect(zoneSelectAll(whole('a'), zone, always)?.checked).toBe('mixed');
    });

    it('is mixed when findings are left out of a selected category', () => {
      const selection: Selection = {
        categories: new Map([[PICKED.category, new Map([[PICKED.id, PICKED]])]]),
        findings: new Map(),
        toCheck: new Set(),
      };

      expect(zoneSelectAll(selection, zone, always)?.checked).toBe('mixed');
    });

    it('is mixed when findings of a category are picked one by one', () => {
      const selection: Selection = { ...NOTHING_SELECTED, findings: new Map([[PICKED.id, PICKED]]) };

      expect(zoneSelectAll(selection, zone, always)?.checked).toBe('mixed');
    });
  });
});
