import { describe, expect, it } from 'vitest';

import { chartLabel, type Segment } from './segment';

const segment = (key: string, value: number, size: string): Segment => ({
  key,
  label: key,
  level: 'safe',
  size,
  value,
});

describe('chartLabel', () => {
  describe('negative cases', () => {
    it('an empty chart is named by its name alone', () => {
      expect(chartLabel('Space found', [])).toBe('Space found');
    });

    it('leaves out a segment of 0', () => {
      expect(chartLabel('Space found', [segment('Caches', 0, '0 B'), segment('Logs', 3, '3 GB')])).toBe(
        'Space found: Logs 3 GB',
      );
    });
  });

  describe('positive cases', () => {
    it('lists every segment with its size, in order', () => {
      expect(chartLabel('Space found', [segment('Caches', 42, '42 GB'), segment('Logs', 3, '3 GB')])).toBe(
        'Space found: Caches 42 GB, Logs 3 GB',
      );
    });
  });
});
