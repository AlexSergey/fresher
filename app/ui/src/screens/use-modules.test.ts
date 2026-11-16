import { describe, expect, it } from 'vitest';

import type { Overview } from '../ipc/bindings.generated';

import { DEMO_MODULES, DEMO_OVERVIEW, IDLE } from '../test/demo-scan';
import { moduleFound } from './use-modules';

const [DEMO] = DEMO_MODULES;

if (DEMO === undefined) {
  throw new Error('The demo module is missing from the fixtures');
}

describe('moduleFound', () => {
  describe('negative cases', () => {
    it('has no size before the overview is known', () => {
      expect(moduleFound(DEMO, null)).toBeNull();
    });

    it('has no size before the first scan', () => {
      expect(moduleFound(DEMO, IDLE)).toBeNull();
    });

    it('counts nothing of another module', () => {
      expect(moduleFound({ ...DEMO, categories: [], id: 'logs' }, DEMO_OVERVIEW)).toStrictEqual({
        allocated: 0,
        completeness: 'complete',
        logical: 0,
      });
    });
  });

  describe('positive cases', () => {
    it('sums the totals of its categories', () => {
      expect(moduleFound(DEMO, DEMO_OVERVIEW)).toStrictEqual({
        allocated: 2_570_000_000,
        completeness: 'complete',
        logical: 2_570_000_000,
      });
    });

    it('counts from the start of the first scan', () => {
      const scanning: Overview = { ...IDLE, status: { scan: 1, state: 'scanning' } };

      expect(moduleFound(DEMO, scanning)).toStrictEqual({ allocated: 0, completeness: 'complete', logical: 0 });
    });

    it('keeps the weakest completeness of its categories', () => {
      const [caches, downloads] = DEMO_OVERVIEW.totals;
      const partial: Overview =
        caches === undefined || downloads === undefined
          ? DEMO_OVERVIEW
          : {
              ...DEMO_OVERVIEW,
              totals: [{ ...caches, sizes: { ...caches.sizes, completeness: 'partial' } }, downloads],
            };

      expect(moduleFound(DEMO, partial)?.completeness).toBe('partial');
    });
  });
});
