import { describe, expect, it, vi } from 'vitest';

import type { Finding } from '../ipc/bindings.generated';

import { mockCore } from '../ipc/mock';
import { DEMO_FINDINGS, DEMO_OVERVIEW, DEMO_SCAN, SCAN_STARTED } from '../test/demo-scan';
import { findingOf } from '../test/finding-fixtures';
import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';
import { checkedOf, cleanupSelectionOf, NOTHING_SELECTED } from './selection';

// The demo totals, a Safe category outside the demo module, and a "Look before removing" one (Medium) of 3 findings.
const TOTALS_LOADED: CoreAction = {
  overview: {
    ...DEMO_OVERVIEW,
    totals: [
      ...DEMO_OVERVIEW.totals,
      { category: 'other.cache', count: 1, highestDanger: 'safe', sizes: DEMO_OVERVIEW.found },
      { category: 'demo.photos', count: 3, highestDanger: 'medium', sizes: DEMO_OVERVIEW.found },
    ],
  },
  type: 'overview-loaded',
};

const WITH_TOTALS = reduce(INITIAL_STATE, TOTALS_LOADED);

const run = (actions: readonly CoreAction[], from: CoreState = WITH_TOTALS): CoreState => actions.reduce(reduce, from);

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

const PHOTO = findingOf({ category: 'demo.photos', id: 40, level: 'medium', name: 'holiday.jpg' });
const OTHER_PHOTO = findingOf({ category: 'demo.photos', id: 41, level: 'medium', name: 'party.jpg' });

const finding = (picked: Finding): CoreAction => ({ finding: picked, type: 'toggle-finding' });

const choose = (categories: readonly string[], within: readonly string[]): CoreAction => ({
  categories,
  type: 'choose',
  within,
});

const category = (id: string): CoreAction => ({ category: id, type: 'toggle-category' });

const tickAll = (id: string): CoreAction => ({ category: id, type: 'tick-all' });

const untickAll = (id: string): CoreAction => ({ category: id, type: 'untick-all' });

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
      expect(run([{ type: 'clear' }], INITIAL_STATE)).toBe(INITIAL_STATE);
    });

    it('a Look before removing category ticked on Scan selects none of its items', () => {
      const state = run([category('demo.photos')]);

      expect(state.selection.categories.size).toBe(0);
      expect(state.selection.findings.size).toBe(0);
      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('unticking it drops its items ticked one by one', () => {
      const state = run([category('demo.photos'), finding(PHOTO), category('demo.photos')]);

      expect(state.selection).toEqual(NOTHING_SELECTED);
    });

    it('a category whose level the totals do not tell is never selected whole', () => {
      const state = run([category('demo.unknown')], INITIAL_STATE);

      expect(state.selection.categories.size).toBe(0);
      expect([...state.selection.toCheck]).toEqual(['demo.unknown']);
    });

    it('a module’s choice never selects a Look category whole', () => {
      const state = run([choose(['demo.caches', 'demo.photos'], ['demo.caches', 'demo.photos'])]);

      expect([...state.selection.categories.keys()]).toEqual(['demo.caches']);
      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('a choice that no longer includes a Look category leaves it no longer to check', () => {
      const state = run([
        choose(['demo.caches', 'demo.photos'], ['demo.caches', 'demo.photos']),
        finding(PHOTO),
        choose(['demo.caches'], ['demo.caches', 'demo.photos']),
      ]);

      expect(state.selection.toCheck.size).toBe(0);
      expect(state.selection.findings.size).toBe(0);
    });

    it('a choice leaves a category to check outside its module to check', () => {
      const state = run([category('demo.photos'), choose(['demo.caches'], ['demo.caches', 'demo.downloads'])]);

      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('Clear empties the categories to check', () => {
      const state = run([category('demo.photos'), finding(PHOTO), { type: 'clear' }]);

      expect(state.selection).toBe(NOTHING_SELECTED);
    });

    it('a category to check adds nothing to the core’s selection', () => {
      const state = run([category('demo.photos')]);

      expect(cleanupSelectionOf(state.selection)).toEqual({ categories: [], findings: [] });
    });

    it('untick-all drops a category selected whole and its exclusions', () => {
      const state = run([tickAll('demo.photos'), finding(PHOTO), untickAll('demo.photos')]);

      expect(cleanupSelectionOf(state.selection)).toEqual({ categories: [], findings: [] });
    });

    it('untick-all leaves a Safe category not to check', () => {
      const state = run([category('demo.caches'), untickAll('demo.caches')]);

      expect(state.selection).toEqual(NOTHING_SELECTED);
    });
  });

  describe('positive cases', () => {
    it('a choice leaves a selected category outside its module selected', () => {
      const state = run([category('other.cache'), choose(['demo.caches'], ['demo.caches', 'demo.downloads'])]);

      expect([...state.selection.categories.keys()]).toEqual(['other.cache', 'demo.caches']);
    });

    it('a choice leaves a finding picked outside its module picked', () => {
      const outside: Finding = { ...INSTALLER, category: 'other.cache' };

      const state = run([finding(outside), choose(['demo.caches'], ['demo.caches', 'demo.downloads'])]);

      expect(state.selection.findings).toEqual(new Map([[outside.id, outside]]));
    });

    it('a choice drops the module’s other categories and the findings picked in them', () => {
      const state = run([
        category('demo.downloads'),
        finding(INSTALLER),
        finding(EDITOR_CACHE),
        choose(['demo.caches'], ['demo.caches', 'demo.downloads']),
      ]);

      expect([...state.selection.categories]).toEqual([['demo.caches', new Map()]]);
      expect(state.selection.findings.size).toBe(0);
    });

    it('a choice selects its categories whole, ticks off included', () => {
      const state = run([
        category('demo.caches'),
        finding(EDITOR_CACHE),
        choose(['demo.caches'], ['demo.caches', 'demo.downloads']),
      ]);

      expect(state.selection.categories.get('demo.caches')).toEqual(new Map());
    });

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

    it('an item ticked one by one goes', () => {
      const state = run([category('demo.photos'), finding(PHOTO)]);

      expect(cleanupSelectionOf(state.selection)).toEqual({ categories: [], findings: [PHOTO.id] });
      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('tick-all selects the category whole and it leaves the list to check', () => {
      const state = run([category('demo.photos'), finding(PHOTO), tickAll('demo.photos')]);

      expect(cleanupSelectionOf(state.selection)).toEqual({
        categories: [{ category: 'demo.photos', except: [] }],
        findings: [],
      });
      expect(state.selection.toCheck.size).toBe(0);
    });

    it('untick-all keeps a Look category to check', () => {
      const state = run([category('demo.photos'), tickAll('demo.photos'), untickAll('demo.photos')]);

      expect(state.selection.categories.size).toBe(0);
      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('a Look category ticked on its items keeps them when it is brought to Review', () => {
      const state = run([finding(PHOTO), category('demo.photos')]);

      expect([...state.selection.findings.keys()]).toEqual([PHOTO.id]);
      expect([...state.selection.toCheck]).toEqual(['demo.photos']);
    });

    it('a Look category selected whole is unticked whole', () => {
      const state = run([tickAll('demo.photos'), category('demo.photos')]);

      expect(state.selection).toEqual(NOTHING_SELECTED);
    });

    it('a Safe category ticked on Scan goes whole, as before', () => {
      const state = run([category('demo.caches')]);

      expect(cleanupSelectionOf(state.selection)).toEqual({
        categories: [{ category: 'demo.caches', except: [] }],
        findings: [],
      });
      expect(state.selection.toCheck.size).toBe(0);
    });

    it('checkedOf reads a category to check as checked, mixed once some items are ticked', () => {
      const brought = run([category('demo.photos')]);
      const ticked = run([finding(PHOTO), finding(OTHER_PHOTO)], brought);

      expect(checkedOf(brought.selection, 'demo.photos')).toBe(true);
      expect(checkedOf(ticked.selection, 'demo.photos')).toBe('mixed');
    });

    it('clear drops everything', () => {
      const state = run([finding(INSTALLER), category('demo.caches'), { type: 'clear' }]);

      expect(state.selection).toBe(NOTHING_SELECTED);
    });
  });
});
