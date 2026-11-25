import { describe, expect, it } from 'vitest';

import { lookingAt, scanPercent, stillRunning } from './progress';

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

describe('stillRunning', () => {
  describe('negative cases', () => {
    it('is empty when no module started or every one finished', () => {
      expect(stillRunning([], [])).toStrictEqual([]);
      expect(stillRunning(['demo', 'logs'], ['logs', 'demo'])).toStrictEqual([]);
    });
  });

  describe('positive cases', () => {
    it('keeps the started modules that have not finished, in the core order', () => {
      expect(stillRunning(['demo', 'logs', 'temp'], ['logs'])).toStrictEqual(['demo', 'temp']);
    });
  });
});

describe('lookingAt', () => {
  describe('negative cases', () => {
    it('is null when no module runs', () => {
      expect(lookingAt([], 0)).toBeNull();
      expect(lookingAt([], 3)).toBeNull();
    });

    it('one module left stays on every turn', () => {
      expect(lookingAt(['logs'], 0)).toBe('logs');
      expect(lookingAt(['logs'], 7)).toBe('logs');
    });
  });

  describe('positive cases', () => {
    it('the modules take turns in the core order and wrap around', () => {
      const running = ['demo', 'logs', 'temp'];

      expect([0, 1, 2, 3, 4].map((turn) => lookingAt(running, turn))).toStrictEqual([
        'demo',
        'logs',
        'temp',
        'demo',
        'logs',
      ]);
    });
  });
});
