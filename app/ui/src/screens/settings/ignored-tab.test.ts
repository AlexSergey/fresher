import { describe, expect, it } from 'vitest';

import { firstWithEntry } from './ignored-tab';

describe('firstWithEntry', () => {
  describe('negative cases', () => {
    it('opens the first tab when nothing is ignored', () => {
      expect(firstWithEntry({ groups: 0, items: 0, modules: 0 })).toBe('groups');
    });
  });

  describe('positive cases', () => {
    it('opens the modules tab when only modules are ignored', () => {
      expect(firstWithEntry({ groups: 0, items: 0, modules: 2 })).toBe('modules');
    });

    it('opens the items tab when only items are ignored', () => {
      expect(firstWithEntry({ groups: 0, items: 1, modules: 0 })).toBe('items');
    });

    it('prefers groups, then modules', () => {
      expect(firstWithEntry({ groups: 1, items: 1, modules: 1 })).toBe('groups');
      expect(firstWithEntry({ groups: 0, items: 1, modules: 1 })).toBe('modules');
    });
  });
});
