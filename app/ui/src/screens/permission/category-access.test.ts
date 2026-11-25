import { beforeEach, describe, expect, it } from 'vitest';

import type { AccessStatus, Capability, ModuleInfo } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { ACCESS_DICTIONARY, DEMO_MODULES, readyCategory } from '../../test/demo-scan';
import { accessOfCategories, categoryAccess, missingText, unavailableTotals } from './category-access';

const DOCKER: Capability = { kind: 'tool', value: 'docker' };
const ALL_FILES: Capability = { kind: 'all-files' };

const STATUS: AccessStatus = {
  capabilities: [{ access: 'denied', capability: DOCKER, note: { args: {}, key: 'demo.docker.note' } }],
};

beforeEach(() => {
  setCoreDictionary({ ...ACCESS_DICTIONARY, 'demo.docker.note': 'Docker is installed but not running.' });
});

describe('categoryAccess', () => {
  describe('negative cases', () => {
    it('a_ready_category_has_no_access_text', () => {
      expect(categoryAccess({ kind: 'ready' }, STATUS)).toBeUndefined();
      expect(categoryAccess(undefined, STATUS)).toBeUndefined();
    });

    it('a_tool_without_a_note_still_says_what_is_missing', () => {
      expect(missingText([DOCKER], { capabilities: [] })).toBe('docker is not available.');
    });
  });

  describe('positive cases', () => {
    it('a_limited_category_names_each_missing_capability', () => {
      expect(
        categoryAccess({ kind: 'limited', missing: [ALL_FILES, { kind: 'folder', value: 'desktop' }] }, STATUS)?.text,
      ).toBe('Some of it could not be seen: Full Disk Access, Desktop folder.');
    });

    it('an_unavailable_tool_category_reads_the_tools_note', () => {
      expect(categoryAccess({ kind: 'unavailable', missing: [DOCKER] }, STATUS)).toStrictEqual({
        kind: 'unavailable',
        missing: [DOCKER],
        text: 'Docker is installed but not running.',
      });
    });

    it('lists_a_zero_total_only_for_an_unavailable_category_without_one', () => {
      const [demo] = DEMO_MODULES;
      const modules: ModuleInfo[] = [
        {
          ...(demo as ModuleInfo),
          categories: [
            readyCategory('demo.caches', 'safe'),
            {
              availability: { kind: 'unavailable', missing: [DOCKER] },
              descriptor: { danger: 'low', icon: 'download', id: 'demo.downloads', needs: [] },
            },
          ],
        },
      ];
      const access = accessOfCategories(modules, STATUS);
      const zero = unavailableTotals(modules, [], access);

      expect(zero.map(({ category, count }) => [category, count])).toStrictEqual([['demo.downloads', 0]]);
      expect(unavailableTotals(modules, zero, access)).toStrictEqual([]);
    });
  });
});
