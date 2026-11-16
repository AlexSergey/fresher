import { describe, expect, it } from 'vitest';

import { findingOf } from '../test/finding-fixtures';
import { checkedOf, NOTHING_SELECTED, reduceSelection } from './selection';

const EDITOR_CACHE = findingOf({ id: 1, level: 'safe', name: 'editor-cache' });
const INSTALLER = findingOf({ category: 'demo.downloads', id: 2, level: 'low', name: 'installer.dmg' });

describe('checkedOf', () => {
  describe('negative cases', () => {
    it('is unchecked with nothing selected', () => {
      expect(checkedOf(NOTHING_SELECTED, 'demo.caches')).toBe(false);
    });

    it("ignores another category's findings", () => {
      const selection = reduceSelection(NOTHING_SELECTED, { finding: INSTALLER, type: 'toggle-finding' });

      expect(checkedOf(selection, 'demo.caches')).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('is checked when the whole category is selected', () => {
      const selection = reduceSelection(NOTHING_SELECTED, { category: 'demo.caches', type: 'toggle-category' });

      expect(checkedOf(selection, 'demo.caches')).toBe(true);
    });

    it('is mixed when some of its findings are picked', () => {
      const selection = reduceSelection(NOTHING_SELECTED, { finding: EDITOR_CACHE, type: 'toggle-finding' });

      expect(checkedOf(selection, 'demo.caches')).toBe('mixed');
    });

    it('is mixed when a selected category has findings ticked off', () => {
      const whole = reduceSelection(NOTHING_SELECTED, { category: 'demo.caches', type: 'toggle-category' });
      const selection = reduceSelection(whole, { finding: EDITOR_CACHE, type: 'toggle-finding' });

      expect(checkedOf(selection, 'demo.caches')).toBe('mixed');
    });
  });
});
