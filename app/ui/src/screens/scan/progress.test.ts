import { describe, expect, it } from 'vitest';

import { lookingAt, scanPercent } from './progress';

const NO_MODULES = { finished: 0, started: 0 };

describe('scanPercent', () => {
  describe('negative cases', () => {
    it('nothing walked is 0', () => {
      expect(scanPercent(0, 1000, NO_MODULES)).toBe(0);
    });

    it('an estimate smaller than the bytes holds at 99', () => {
      expect(scanPercent(5000, 1000, NO_MODULES)).toBe(99);
      expect(scanPercent(1000, 1000, NO_MODULES)).toBe(99);
    });

    it('without an estimate or modules it is 0', () => {
      expect(scanPercent(5000, null, NO_MODULES)).toBe(0);
    });

    it('an estimate of 0 counts as none', () => {
      expect(scanPercent(5000, 0, { finished: 1, started: 4 })).toBe(24);
    });
  });

  describe('positive cases', () => {
    it('bytes over the estimate', () => {
      expect(scanPercent(620, 1000, NO_MODULES)).toBe(62);
      expect(scanPercent(629, 1000, NO_MODULES)).toBe(62);
    });

    it('without an estimate, the modules finished', () => {
      expect(scanPercent(5000, null, { finished: 2, started: 4 })).toBe(49);
      expect(scanPercent(5000, null, { finished: 4, started: 4 })).toBe(99);
    });
  });
});

describe('lookingAt', () => {
  describe('negative cases', () => {
    it('is null when no module started', () => {
      expect(lookingAt([], [])).toBeNull();
    });

    it('is null when every started module finished', () => {
      expect(lookingAt(['demo', 'logs'], ['logs', 'demo'])).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('is the first started module that has not finished', () => {
      expect(lookingAt(['demo', 'logs', 'temp'], ['demo'])).toBe('logs');
    });
  });
});
