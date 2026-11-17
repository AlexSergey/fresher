import { describe, expect, it } from 'vitest';

import type { ModuleInfo, Overview } from '../ipc/bindings.generated';

import { DEMO_MODULES, DEMO_OVERVIEW } from '../test/demo-scan';
import { hiddenBy, isGroupIgnored } from './use-ignored';

const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const hidden = (by: 'group' | 'module'): Overview => ({ ...DEMO_OVERVIEW, hiddenModules: [{ by, module: DEMO.id }] });

describe('use-ignored', () => {
  describe('negative cases', () => {
    it('hides nothing before the overview is known', () => {
      expect(hiddenBy(null, DEMO.id)).toBeUndefined();
      expect(isGroupIgnored(null, DEMO_MODULES, DEMO.group)).toBe(false);
    });

    it('does not hide a module the core does not list as hidden', () => {
      expect(hiddenBy(DEMO_OVERVIEW, DEMO.id)).toBeUndefined();
    });

    it('a module ignored on its own does not make its group ignored', () => {
      expect(isGroupIgnored(hidden('module'), DEMO_MODULES, DEMO.group)).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('names why the core hides a module', () => {
      expect(hiddenBy(hidden('module'), DEMO.id)).toBe('module');
      expect(hiddenBy(hidden('group'), DEMO.id)).toBe('group');
    });

    it('a module hidden by its group makes the group ignored', () => {
      expect(isGroupIgnored(hidden('group'), DEMO_MODULES, DEMO.group)).toBe(true);
    });
  });
});
