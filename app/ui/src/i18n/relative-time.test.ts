import { describe, expect, it } from 'vitest';

import { agoText } from './relative-time';

// 1 January 2025, 00:00 UTC, in the core's seconds.
const AT = 1_735_689_600;
const after = (seconds: number): number => (AT + seconds) * 1000;

describe('agoText', () => {
  describe('negative cases', () => {
    it('a time in the future reads now', () => {
      expect(agoText(AT, after(-3600))).toBe('now');
    });

    it('under a minute reads now', () => {
      expect(agoText(AT, after(0))).toBe('now');
      expect(agoText(AT, after(59))).toBe('now');
    });
  });

  describe('positive cases', () => {
    it('reads minutes', () => {
      expect(agoText(AT, after(60))).toBe('1 minute ago');
      expect(agoText(AT, after(150))).toBe('2 minutes ago');
      expect(agoText(AT, after(59 * 60 + 59))).toBe('59 minutes ago');
    });

    it('reads hours', () => {
      expect(agoText(AT, after(3600))).toBe('1 hour ago');
      expect(agoText(AT, after(3 * 3600 + 1800))).toBe('3 hours ago');
    });

    it('reads yesterday', () => {
      expect(agoText(AT, after(24 * 3600))).toBe('yesterday');
      expect(agoText(AT, after(47 * 3600))).toBe('yesterday');
    });

    it('reads days', () => {
      expect(agoText(AT, after(2 * 24 * 3600))).toBe('2 days ago');
      expect(agoText(AT, after(40 * 24 * 3600))).toBe('40 days ago');
    });
  });
});
