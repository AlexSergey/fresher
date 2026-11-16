import { describe, expect, it } from 'vitest';

import type { CategoryTotal, DangerLevel, Sizes } from '../ipc/bindings.generated';

import { findingOf } from '../test/finding-fixtures';
import { NOTHING_SELECTED, type Picked, type Selection } from './selection';
import { LARGEST_SIZE, summarize } from './selection-summary';

const sizes = (bytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: bytes,
  completeness,
  logical: bytes,
});

const total = (category: string, count: number, bytes: number, highestDanger: DangerLevel): CategoryTotal => ({
  category,
  count,
  highestDanger,
  sizes: sizes(bytes),
});

const pick = (category: string, bytes: number, danger: DangerLevel = 'safe'): Picked =>
  findingOf({ bytes, category, id: 0, level: danger, name: 'picked' });

const TOTALS: readonly CategoryTotal[] = [
  total('demo.caches', 100, 10_000, 'medium'),
  total('demo.downloads', 3, 3_000, 'low'),
];

const selected = (
  categories: readonly (readonly [string, readonly (readonly [number, Picked])[]])[],
  findings: readonly (readonly [number, Picked])[] = [],
): Selection => ({
  categories: new Map(categories.map(([id, excluded]) => [id, new Map(excluded)])),
  findings: new Map(findings),
});

describe('summarize', () => {
  describe('negative cases', () => {
    it('counts nothing when nothing is selected', () => {
      expect(summarize(NOTHING_SELECTED, TOTALS)).toEqual({ count: 0, highest: null, sizes: sizes(0) });
    });

    it('a partial size makes the summary partial', () => {
      const partial: Picked = { ...pick('demo.other', 5), sizes: sizes(5, 'partial') };

      const summary = summarize(selected([['demo.downloads', []]], [[7, partial]]), TOTALS);

      expect(summary.sizes).toEqual(sizes(3_005, 'partial'));
      expect(
        summarize(selected([], [[7, { ...partial, sizes: sizes(0, 'unknown') }]]), TOTALS).sizes.completeness,
      ).toBe('unknown');
    });

    it('a sum past the largest size saturates', () => {
      const huge = pick('demo.other', LARGEST_SIZE - 10);

      expect(summarize(selected([['demo.downloads', []]], [[7, huge]]), TOTALS).sizes).toEqual(sizes(LARGEST_SIZE));
    });

    it('an exclusion does not lower the category highest level', () => {
      const summary = summarize(selected([['demo.caches', [[1, pick('demo.caches', 10, 'medium')]]]]), TOTALS);

      expect(summary.highest).toBe('medium');
    });

    it('a category the overview no longer lists counts nothing', () => {
      expect(summarize(selected([['demo.gone', []]]), TOTALS)).toEqual(summarize(NOTHING_SELECTED, TOTALS));
    });
  });

  describe('positive cases', () => {
    it('selecting a category counts its total', () => {
      expect(summarize(selected([['demo.caches', []]]), TOTALS)).toEqual({
        count: 100,
        highest: 'medium',
        sizes: sizes(10_000),
      });
    });

    it('a category minus an exclusion counts the rest', () => {
      const summary = summarize(selected([['demo.caches', [[1, pick('demo.caches', 400)]]]]), TOTALS);

      expect(summary.count).toBe(99);
      expect(summary.sizes).toEqual(sizes(9_600));
    });

    it('the summary highest level is the highest selected', () => {
      const summary = summarize(
        selected(
          [['demo.downloads', []]],
          [
            [9, pick('demo.other', 10, 'high')],
            [8, pick('demo.other', 10, 'safe')],
          ],
        ),
        TOTALS,
      );

      expect(summary).toEqual({ count: 5, highest: 'high', sizes: sizes(3_020) });
    });
  });
});
