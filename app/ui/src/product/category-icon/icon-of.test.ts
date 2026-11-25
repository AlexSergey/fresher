import { describe, expect, it } from 'vitest';

import type { ModuleInfo } from '../../ipc/bindings.generated';

import { iconOf } from './icon-of';

const modules = [
  {
    categories: [{ descriptor: { icon: 'download', id: 'dev-caches.npm' } }],
    id: 'dev-caches',
  },
] as unknown as readonly ModuleInfo[];

describe('iconOf', () => {
  describe('negative cases', () => {
    it('a category no module lists is generic', () => {
      expect(iconOf(modules, 'core.unknown')).toBe('generic');
    });

    it('is generic while the modules are not loaded', () => {
      expect(iconOf(null, 'dev-caches.npm')).toBe('generic');
    });
  });

  describe('positive cases', () => {
    it("a module's category gives its icon", () => {
      expect(iconOf(modules, 'dev-caches.npm')).toBe('download');
    });
  });
});
