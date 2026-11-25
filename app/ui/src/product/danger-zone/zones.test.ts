import { describe, expect, it } from 'vitest';

import { GROUP_ICONS, ZONE_ORDER } from './zones';

describe('GROUP_ICONS', () => {
  describe('negative cases', () => {
    it('gives no two levels the same icon', () => {
      expect(new Set(ZONE_ORDER.map((level) => GROUP_ICONS[level])).size).toBe(ZONE_ORDER.length);
    });
  });

  describe('positive cases', () => {
    it('has an icon for every level', () => {
      for (const level of ZONE_ORDER) {
        expect(GROUP_ICONS[level]).toBeDefined();
      }
    });
  });
});
