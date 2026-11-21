import { describe, expect, it } from 'vitest';

import { DEMO_MODULES } from '../test/demo-scan';
import { moduleOfCategory } from './use-modules';

const [DEMO] = DEMO_MODULES;

if (DEMO === undefined) {
  throw new Error('The demo module is missing from the fixtures');
}

describe('moduleOfCategory', () => {
  describe('negative cases', () => {
    it('finds nothing before the modules are known', () => {
      expect(moduleOfCategory(null, 'demo.caches')).toBeUndefined();
    });

    it('finds nothing for a category no module lists', () => {
      expect(moduleOfCategory(DEMO_MODULES, 'other.thing')).toBeUndefined();
    });
  });

  describe('positive cases', () => {
    it('finds the module that lists the category', () => {
      expect(moduleOfCategory(DEMO_MODULES, 'demo.downloads')).toBe(DEMO);
    });
  });
});
