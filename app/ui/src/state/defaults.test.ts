import { describe, expect, it } from 'vitest';

import type { CategoryTotal, DangerLevel } from '../ipc/bindings.generated';

import { findingOf } from '../test/finding-fixtures';
import { PRESELECTED, selectByLevel } from './defaults';
import { NOTHING_SELECTED, type Selection } from './selection';

const total = (category: string, highestDanger: DangerLevel): CategoryTotal => ({
  category,
  count: 1,
  highestDanger,
  sizes: { allocated: 1, completeness: 'complete', logical: 1 },
});

const TOTALS: readonly CategoryTotal[] = [
  total('demo.caches', 'safe'),
  total('demo.downloads', 'low'),
  total('demo.logs', 'safe'),
];

describe('the defaults of the selection', () => {
  describe('negative cases', () => {
    it('nothing is pre-selected', () => {
      expect(PRESELECTED).toBe(NOTHING_SELECTED);
    });

    it('selecting by level refuses anything above safe', () => {
      for (const level of ['low', 'medium', 'high'] as const) {
        expect(selectByLevel(NOTHING_SELECTED, TOTALS, level)).toEqual({ kind: 'refused', level });
      }
    });
  });

  describe('positive cases', () => {
    it('selects every category whose highest level is safe', () => {
      const answer = selectByLevel(NOTHING_SELECTED, TOTALS, 'safe');

      expect(answer.kind === 'selected' ? [...answer.selection.categories.keys()] : []).toEqual([
        'demo.caches',
        'demo.logs',
      ]);
    });

    it('keeps what was picked and the exclusions of a category already selected', () => {
      const excluded = new Map([[5, findingOf({ category: 'demo.caches', id: 5, level: 'safe', name: 'five' })]]);
      const before: Selection = {
        categories: new Map([['demo.caches', excluded]]),
        findings: new Map([
          [7, findingOf({ category: 'demo.downloads', id: 7, level: 'low', name: 'seven' })],
          [8, findingOf({ category: 'demo.logs', id: 8, level: 'safe', name: 'eight' })],
        ]),
      };

      const answer = selectByLevel(before, TOTALS, 'safe');

      expect(answer.kind === 'selected' ? answer.selection.categories.get('demo.caches') : undefined).toBe(excluded);
      expect(answer.kind === 'selected' ? [...answer.selection.findings.keys()] : []).toEqual([7]);
    });
  });
});
