import { describe, expect, it } from 'vitest';

import type { Capability, Coverage } from '../ipc/bindings.generated';

import { accessStatus, FULL_COVERAGE } from '../test/demo-scan';
import { accessOf, notSeen } from './access';

const ALL_FILES: Capability = { kind: 'all-files' };
const DOCUMENTS: Capability = { kind: 'folder', value: 'documents' };

const LIMITED: Coverage = {
  notEntered: [{ access: 'denied', capability: DOCUMENTS }],
  unreadable: [
    { folders: 3, reason: 'no-permission', unlockedBy: ALL_FILES },
    { folders: 2, reason: 'no-permission', unlockedBy: DOCUMENTS },
    { folders: 5, reason: 'failed', unlockedBy: null },
  ],
};

describe('access', () => {
  describe('negative cases', () => {
    it('a complete coverage has nothing not seen', () => {
      expect(notSeen(FULL_COVERAGE)).toEqual([]);
      expect(notSeen(null)).toEqual([]);
    });

    it('an unreadable count without a capability is not listed', () => {
      const coverage: Coverage = {
        notEntered: [],
        unreadable: [{ folders: 4, reason: 'failed', unlockedBy: null }],
      };

      expect(notSeen(coverage)).toEqual([]);
    });

    it('a capability the core does not list has no access', () => {
      expect(accessOf(accessStatus([]), ALL_FILES)).toBeNull();
      expect(accessOf(null, ALL_FILES)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('finds the access of one capability', () => {
      const status = accessStatus([
        [ALL_FILES, 'denied'],
        [DOCUMENTS, 'granted'],
      ]);

      expect(accessOf(status, DOCUMENTS)).toBe('granted');
    });

    it('not seen groups by capability', () => {
      expect(notSeen(LIMITED)).toEqual([
        { access: 'denied', capability: DOCUMENTS, folders: 2 },
        { access: null, capability: ALL_FILES, folders: 3 },
      ]);
    });
  });
});
