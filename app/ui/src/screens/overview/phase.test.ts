import { describe, expect, it } from 'vitest';

import type { LastScan, ScanOutcome } from '../../ipc/bindings.generated';

import { phaseOf, scanFailure } from './phase';

const REASON = { args: {}, key: 'core.scan.outcome.failed' };

const last = (outcome: ScanOutcome): LastScan => ({ finished: 1_735_689_600, outcome, scan: 1 });

describe('phaseOf', () => {
  describe('negative cases', () => {
    it('a scan the events started is scanning even if an older overview says idle', () => {
      expect(phaseOf({ scan: 2, state: 'scanning' }, last({ kind: 'completed' }), null)).toBe('scanning');
    });

    it('is idle before any scan', () => {
      expect(phaseOf({ state: 'idle' }, null, null)).toBe('idle');
    });

    it("the events' outcome wins over an overview that has not caught up", () => {
      expect(phaseOf({ state: 'idle' }, last({ kind: 'completed' }), { kind: 'partial' })).toBe('partial');
    });
  });

  describe('positive cases', () => {
    it("follows the last scan's outcome", () => {
      expect(phaseOf({ state: 'idle' }, last({ kind: 'completed' }), null)).toBe('finished');
      expect(phaseOf({ state: 'idle' }, last({ kind: 'partial' }), null)).toBe('partial');
      expect(phaseOf({ state: 'idle' }, last({ kind: 'failed', reason: REASON }), null)).toBe('failed');
    });

    it('a cleaning engine shows how its last scan ended', () => {
      expect(phaseOf({ state: 'cleaning' }, last({ kind: 'completed' }), null)).toBe('finished');
    });
  });
});

describe('scanFailure', () => {
  describe('negative cases', () => {
    it('is null when the scan did not fail', () => {
      expect(scanFailure(null, null)).toBeNull();
      expect(scanFailure(last({ kind: 'completed' }), null)).toBeNull();
      expect(scanFailure(last({ kind: 'failed', reason: REASON }), { kind: 'completed' })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it("is the core's reason of a failed scan", () => {
      expect(scanFailure(last({ kind: 'failed', reason: REASON }), null)).toBe(REASON);
      expect(scanFailure(null, { kind: 'failed', reason: REASON })).toBe(REASON);
    });
  });
});
