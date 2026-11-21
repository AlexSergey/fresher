import { describe, expect, it } from 'vitest';

import { ONE_ITEM_PLAN, reportOf } from './cleanup-plan';

describe('reportOf', () => {
  describe('negative cases', () => {
    it('counts an emptied folder as moved, not deleted, under the Trash', () => {
      const report = reportOf(ONE_ITEM_PLAN, { outcomes: [{ kind: 'emptied', moved: 2, skipped: 0 }] });

      expect(report.deleted.allocated).toBe(0);
    });

    it('counts an emptied folder as deleted, not moved, under a permanent delete', () => {
      const report = reportOf(ONE_ITEM_PLAN, {
        method: 'permanent',
        outcomes: [{ kind: 'emptied', moved: 2, skipped: 0 }],
      });

      expect(report.moved.allocated).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('adds an emptied folder to the moved size under the Trash', () => {
      const report = reportOf(ONE_ITEM_PLAN, { outcomes: [{ kind: 'emptied', moved: 2, skipped: 0 }] });

      expect(report.moved.allocated).toBe(ONE_ITEM_PLAN.summary.sizes.allocated);
    });
  });
});
