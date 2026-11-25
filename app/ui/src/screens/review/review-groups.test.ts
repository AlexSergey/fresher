import { describe, expect, it } from 'vitest';

import type { CategoryTotal, DangerLevel } from '../../ipc/bindings.generated';

import { NOTHING_SELECTED } from '../../state/selection';
import { findingOf } from '../../test/finding-fixtures';
import { reviewGroups, reviewLevels } from './review-groups';

const total = (category: string, level: DangerLevel, count: number, allocated: number): CategoryTotal => ({
  category,
  count,
  highestDanger: level,
  sizes: { allocated, completeness: 'complete', logical: allocated, reclaimable: allocated },
});

// A "Look before removing" category of 3 findings, 3,000 bytes in all.
const PHOTOS = total('demo.photos', 'medium', 3, 3_000);

const CACHES = total('demo.caches', 'safe', 2, 5_000);

const LOGS = total('demo.logs', 'low', 1, 9_000);

const ARCHIVES = total('demo.archives', 'safe', 1, 8_000);

const PHOTO = findingOf({ bytes: 1_000, category: 'demo.photos', id: 7, level: 'medium', name: 'holiday.jpg' });

const CACHE = findingOf({ bytes: 2_000, category: 'demo.caches', id: 3, level: 'safe', name: 'cache' });

const TOTALS = [PHOTOS, CACHES, LOGS, ARCHIVES];

const selected = (...categories: readonly string[]): typeof NOTHING_SELECTED => ({
  ...NOTHING_SELECTED,
  categories: new Map(categories.map((category) => [category, new Map()])),
});

describe('reviewGroups', () => {
  describe('negative cases', () => {
    it('a category to check with no tick sums to nothing, all of it unchecked', () => {
      const [group] = reviewGroups({ ...NOTHING_SELECTED, toCheck: new Set(['demo.photos']) }, [PHOTOS]);

      expect(group).toMatchObject({
        category: 'demo.photos',
        count: 0,
        highest: 'medium',
        rows: { kind: 'to-check', listed: 3, ticked: new Map() },
        sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
        unchecked: 3,
      });
    });

    it('a category to check the overview no longer lists is not shown', () => {
      expect(reviewGroups({ ...NOTHING_SELECTED, toCheck: new Set(['demo.photos']) }, [])).toStrictEqual([]);
    });
  });

  describe('positive cases', () => {
    it('a category to check is listed with its ticked items', () => {
      const ticked = new Map([[PHOTO.id, PHOTO]]);
      const groups = reviewGroups({ ...NOTHING_SELECTED, findings: ticked, toCheck: new Set(['demo.photos']) }, [
        PHOTOS,
      ]);

      expect(groups).toHaveLength(1);
      expect(groups[0]?.rows).toStrictEqual({ kind: 'to-check', listed: 3, ticked });
      expect(groups[0]?.count).toBe(1);
      expect(groups[0]?.unchecked).toBe(2);
      expect(groups[0]?.sizes.allocated).toBe(1_000);
    });

    it('orders the groups Safe to High, largest first within one', () => {
      const groups = reviewGroups(
        { ...selected('demo.logs', 'demo.caches', 'demo.archives'), toCheck: new Set(['demo.photos']) },
        TOTALS,
      );

      expect(groups.map((group) => group.category)).toStrictEqual([
        'demo.archives',
        'demo.caches',
        'demo.logs',
        'demo.photos',
      ]);
    });

    it("a selected category's items ticked off count as unchecked", () => {
      const [group] = reviewGroups(
        { ...NOTHING_SELECTED, categories: new Map([['demo.caches', new Map([[CACHE.id, CACHE]])]]) },
        TOTALS,
      );

      expect(group?.count).toBe(1);
      expect(group?.unchecked).toBe(1);
      expect(group?.sizes.allocated).toBe(3_000);
    });
  });
});

describe('reviewLevels', () => {
  describe('negative cases', () => {
    it('leaves out a group with no category', () => {
      expect(
        reviewLevels(reviewGroups(selected('demo.logs'), TOTALS), TOTALS).map((level) => level.level),
      ).toStrictEqual(['low']);
      expect(reviewLevels([], TOTALS)).toStrictEqual([]);
    });
  });

  describe('positive cases', () => {
    it('sums each group with the store rule and counts what is not checked', () => {
      const selection = {
        ...selected('demo.caches', 'demo.archives', 'demo.logs'),
        findings: new Map([[PHOTO.id, PHOTO]]),
        toCheck: new Set(['demo.photos']),
      };

      expect(reviewLevels(reviewGroups(selection, TOTALS), TOTALS)).toStrictEqual([
        {
          level: 'safe',
          sizes: { allocated: 13_000, completeness: 'complete', logical: 13_000, reclaimable: 13_000 },
          unchecked: 0,
        },
        {
          level: 'low',
          sizes: { allocated: 9_000, completeness: 'complete', logical: 9_000, reclaimable: 9_000 },
          unchecked: 0,
        },
        {
          level: 'medium',
          sizes: { allocated: 1_000, completeness: 'complete', logical: 1_000, reclaimable: 1_000 },
          unchecked: 2,
        },
      ]);
    });
  });
});
