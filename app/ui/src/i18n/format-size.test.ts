import { describe, expect, it } from 'vitest';

import { formatBytes, formatSize } from './format-size';

describe('formatSize', () => {
  describe('negative cases', () => {
    it('formats an unknown size as a dash', () => {
      expect(formatSize(42_000_000_000, 'unknown')).toBe('—');
    });

    it('keeps a size under a kilobyte in bytes', () => {
      expect(formatBytes(0)).toBe('0 bytes');
      expect(formatBytes(1)).toBe('1 byte');
      expect(formatBytes(999)).toBe('999 bytes');
    });
  });

  describe('positive cases', () => {
    it('formats a size in decimal units', () => {
      expect(formatSize(1000, 'complete')).toBe('1 kB');
      expect(formatSize(1_500_000, 'complete')).toBe('1.5 MB');
      expect(formatSize(42_000_000_000, 'complete')).toBe('42 GB');
      expect(formatSize(3_000_000_000_000, 'complete')).toBe('3 TB');
      expect(formatSize(1_234_000_000_000_000_000, 'complete')).toBe('1,234 PB');
    });

    it('marks a partial size with a plus', () => {
      expect(formatSize(42_000_000_000, 'partial')).toBe('42 GB+');
    });
  });
});
