import { describe, expect, it, vi } from 'vitest';

import type { Finding } from '../ipc/bindings.generated';

import { mockCore } from '../ipc/mock';
import { DEMO_FINDINGS, DEMO_SCAN, SCAN_STARTED } from '../test/demo-scan';
import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';
import { NOTHING_SELECTED } from './selection';

const run = (actions: readonly CoreAction[], from: CoreState = INITIAL_STATE): CoreState =>
  actions.reduce(reduce, from);

const at = (index: number): Finding => {
  const finding = DEMO_FINDINGS[index];

  if (finding === undefined) {
    throw new Error(`The demo scan has no finding ${index}`);
  }

  return finding;
};

// editor-cache and browser-cache are in `demo.caches`; installer.dmg is in `demo.downloads`.
const EDITOR_CACHE = at(0);
const INSTALLER = at(1);
const BROWSER_CACHE = at(2);

const finding = (picked: Finding): CoreAction => ({ finding: picked, type: 'toggle-finding' });

const category = (id: string): CoreAction => ({ category: id, type: 'toggle-category' });

describe('the selection', () => {
  describe('negative cases', () => {
    it('nothing is selected after a scan starts', () => {
      const state = run([...DEMO_SCAN, finding(INSTALLER), category('demo.caches'), SCAN_STARTED]);

      expect(state.selection).toBe(NOTHING_SELECTED);
    });

    it('ticking a finding off inside a selected category loads no other finding', () => {
      const getFindings = vi.fn();
      mockCore({ getFindings });

      const state = run([category('demo.caches'), finding(EDITOR_CACHE)]);

      expect(getFindings).not.toHaveBeenCalled();
      expect(state.selection.findings.size).toBe(0);
      expect(state.selection.categories.get('demo.caches')).toEqual(new Map([[EDITOR_CACHE.id, EDITOR_CACHE]]));
    });

    it('a category toggled off takes its exclusions with it', () => {
      const state = run([category('demo.caches'), finding(EDITOR_CACHE), category('demo.caches')]);

      expect(state.selection).toEqual(NOTHING_SELECTED);
    });

    it('clearing nothing leaves the state as it is', () => {
      expect(run([{ type: 'clear' }])).toBe(INITIAL_STATE);
    });
  });

  describe('positive cases', () => {
    it('toggling a finding twice leaves the selection as it was', () => {
      const once = run([finding(INSTALLER)]);

      expect(run([finding(EDITOR_CACHE), finding(EDITOR_CACHE)], once).selection).toEqual(once.selection);
      expect(run([category('demo.caches'), finding(EDITOR_CACHE), finding(EDITOR_CACHE)]).selection).toEqual(
        run([category('demo.caches')]).selection,
      );
    });

    it('picks a finding as the core sent it', () => {
      const state = run([finding(INSTALLER)]);

      expect(state.selection.findings).toEqual(new Map([[INSTALLER.id, INSTALLER]]));
    });

    it('a selected category takes in the findings picked one by one in it', () => {
      const state = run([finding(BROWSER_CACHE), finding(INSTALLER), category('demo.caches')]);

      expect([...state.selection.findings.keys()]).toEqual([INSTALLER.id]);
      expect([...state.selection.categories.keys()]).toEqual(['demo.caches']);
    });

    it('clear drops everything', () => {
      const state = run([finding(INSTALLER), category('demo.caches'), { type: 'clear' }]);

      expect(state.selection).toBe(NOTHING_SELECTED);
    });
  });
});
