import { describe, expect, it } from 'vitest';

import type { ModuleInfo } from '../ipc/bindings.generated';

import { inGroups, modulesByGroup } from './module-groups';

const module = (id: string, group: string): ModuleInfo =>
  ({ categories: [], group, id, settings: [] }) as unknown as ModuleInfo;

const ids = (modules: readonly ModuleInfo[]): readonly string[] => modules.map(({ id }) => id);

describe('modulesByGroup', () => {
  describe('negative cases', () => {
    it('has no section for a group without a module', () => {
      expect(modulesByGroup([module('xcode', 'development')]).map(({ group }) => group)).toStrictEqual(['development']);
    });

    it('has no section at all without modules', () => {
      expect(modulesByGroup([])).toStrictEqual([]);
    });

    it('never drops a module of a group this build does not know: it comes last, under its key', () => {
      const grouped = modulesByGroup([
        module('future', 'tomorrow'),
        module('demo', 'debug'),
        module('xcode', 'development'),
      ]);

      expect(grouped.map(({ group }) => group)).toStrictEqual(['development', 'debug', 'tomorrow']);
      expect(ids(grouped.flatMap(({ items }) => items))).toStrictEqual(['xcode', 'demo', 'future']);
    });
  });

  describe('positive cases', () => {
    it('puts Development before Debug', () => {
      const grouped = modulesByGroup([module('demo', 'debug'), module('xcode', 'development')]);

      expect(grouped.map(({ group }) => group)).toStrictEqual(['development', 'debug']);
    });

    it('keeps the registration order inside a group', () => {
      const grouped = modulesByGroup([
        module('node', 'development'),
        module('demo', 'debug'),
        module('xcode', 'development'),
      ]);

      expect(ids(grouped[0]?.items ?? [])).toStrictEqual(['node', 'xcode']);
    });
  });
});

describe('inGroups', () => {
  describe('positive cases', () => {
    it('groups any items by the group the caller names', () => {
      expect(inGroups(['a', 'bb', 'c'], (text) => (text.length > 1 ? 'debug' : 'development'))).toStrictEqual([
        { group: 'development', items: ['a', 'c'] },
        { group: 'debug', items: ['bb'] },
      ]);
    });
  });
});
