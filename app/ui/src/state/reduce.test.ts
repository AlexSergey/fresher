import { describe, expect, it } from 'vitest';

import type { Event, Overview, ScanId, SettingsView, SettingValue, Sizes } from '../ipc/bindings.generated';

import {
  allFiles,
  DEMO_DICTIONARY,
  DEMO_FINDINGS,
  DEMO_OVERVIEW,
  DEMO_SCAN,
  FIRST_BATCH,
  FULL_COVERAGE,
  IDLE,
} from '../test/demo-scan';
import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';

const run = (actions: readonly CoreAction[], from: CoreState = INITIAL_STATE): CoreState =>
  actions.reduce(reduce, from);

const started: Event = { modules: ['demo'], scan: 1, type: 'scan-started' };

const scanning = (scan: ScanId): Overview => ({ ...IDLE, status: { scan, state: 'scanning' } });

const requested: CoreAction = { type: 'overview-requested' };

const loaded = (overview: Overview): CoreAction => ({ overview, type: 'overview-loaded' });

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

const changed = (key: string, value: SettingValue): CoreAction => ({ key, type: 'setting-changed', value });

const SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [
    { key: 'demo.count', kind: 'out-of-range', max: 50, min: 1 },
    { key: 'demo.enabled', kind: 'wrong-type' },
  ],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'demo.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 8 },
      effect: 'next-scan',
      group: 'module',
      key: 'demo.count',
      kind: { kind: 'number', max: 50, min: 1, unit: null },
    },
  ],
  values: { 'demo.count': { kind: 'number', value: 8 }, 'demo.enabled': { kind: 'switch', value: true } },
};

describe('reduce', () => {
  describe('negative cases', () => {
    it('a findings batch does not grow the state', () => {
      const once = run([started, FIRST_BATCH]);

      const state = run(
        Array.from({ length: 1000 }, (): CoreAction => FIRST_BATCH),
        once,
      );

      expect(state).toBe(once);
      expect(JSON.stringify(state)).not.toContain('editor-cache');
    });

    it('an event of another scan does not mark the overview stale', () => {
      const state = run([
        started,
        { findings: [...DEMO_FINDINGS], scan: 2, type: 'findings-added' },
        { module: 'demo', outcome: { kind: 'completed' }, scan: 2, type: 'module-finished' },
        { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 2, type: 'scan-finished' },
      ]);

      expect(state.stale).toBe(false);
      expect(state.modules).toEqual([]);
      expect(state.status).toEqual({ scan: 1, state: 'scanning' });
    });

    it('a scan learned of from an overview only has no module list', () => {
      expect(run([loaded(scanning(4))]).scanModules).toBeNull();
    });

    it('an overview of an older scan is ignored', () => {
      const finished = run([...DEMO_SCAN, requested]);

      expect(run([loaded({ ...DEMO_OVERVIEW, lastScan: null })], finished)).toBe(finished);
      expect(
        run([{ modules: ['demo'], scan: 2, type: 'scan-started' }, loaded(DEMO_OVERVIEW)], finished).overview,
      ).toBeNull();
    });

    it('keeps the overview stale when an event comes after the request', () => {
      const state = run([started, FIRST_BATCH, requested, FIRST_BATCH]);

      expect(state.stale).toBe(true);
    });

    it('keeps findings counted when the scan is cancelled and marks it partial', () => {
      const state = run([
        started,
        FIRST_BATCH,
        { module: 'demo', outcome: { kind: 'cancelled' }, scan: 1, type: 'module-finished' },
        { kind: { kind: 'full' }, outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' },
        requested,
        loaded({
          ...DEMO_OVERVIEW,
          lastScan: {
            coverage: FULL_COVERAGE,
            finished: 1,
            kind: { kind: 'full' },
            outcome: { kind: 'partial' },
            scan: 1,
            settingsChanged: false,
          },
        }),
      ]);

      expect(state.overview?.totals).toEqual(DEMO_OVERVIEW.totals);
      expect(state.outcome).toEqual({ kind: 'partial' });
      expect(state.modules).toEqual([{ module: 'demo', outcome: { kind: 'cancelled' } }]);
      expect(state.status).toEqual({ state: 'idle' });
    });

    it('keeps a failure until the next scan starts', () => {
      const failed = run([{ failure: { kind: 'unreachable' }, type: 'command-failed' }]);

      expect(failed.failure).toEqual({ kind: 'unreachable' });
      expect(run([started], failed).failure).toBeNull();
    });

    it('keeps a scan the events started when a stale idle overview arrives', () => {
      const state = run([started, loaded(IDLE)]);

      expect(state.status).toEqual({ scan: 1, state: 'scanning' });
      expect(state.scan).toBe(1);
    });

    it('keeps a scan the events finished when a stale overview says it still runs', () => {
      const state = run([
        { bytes: 2048, entries: 10, scan: 4, type: 'scan-progress' },
        { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 4, type: 'scan-finished' },
        loaded(scanning(4)),
      ]);

      expect(state.status).toEqual({ state: 'idle' });
      expect(state.overview).toBeNull();
    });

    it('keeps the scan of the events when the overview reports an older one', () => {
      const state = run([{ modules: ['demo'], scan: 5, type: 'scan-started' }, loaded(scanning(4))]);

      expect(state.status).toEqual({ scan: 5, state: 'scanning' });
      expect(state.scan).toBe(5);
    });

    it('leaves the state as it is for a coverage and a request of a fresh overview', () => {
      const state = run([{ coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' }, requested]);

      expect(state).toBe(INITIAL_STATE);
    });

    it.each<Event>([
      { cleanup: 1, done: 1, items: [{ finding: 1, index: 0, outcome: { kind: 'moved' } }], type: 'cleanup-items' },
      {
        cleanup: 1,
        report: {
          counts: { deleted: 0, emptied: 0, failed: 0, moved: 1, notReached: 0, skipped: 0, toolDone: 0, unknown: 0 },
          deleted: NOTHING,
          ended: 'completed',
          measured: null,
          method: 'trash',
          moved: NOTHING,
        },
        type: 'cleanup-finished',
      },
    ])('leaves the state as it is for a $type event of a cleanup it does not follow', (event) => {
      expect(run([event])).toBe(INITIAL_STATE);
    });

    it('an access status answered after an access-changed event does not undo it', () => {
      const state = run([
        { status: allFiles('granted'), type: 'access-changed' },
        { status: allFiles('denied'), type: 'access-loaded' },
      ]);

      expect(state.access).toEqual(allFiles('granted'));
    });

    it('a settings view answered again does not undo the changes since the first', () => {
      const first = run([
        { settings: SETTINGS, type: 'settings-loaded' },
        changed('demo.count', { kind: 'number', value: 3 }),
      ]);

      expect(run([{ settings: SETTINGS, type: 'settings-loaded' }], first)).toBe(first);
    });
  });

  describe('positive cases', () => {
    it('findings, a module and the end of the scan mark the overview stale', () => {
      expect(run([started, FIRST_BATCH]).stale).toBe(true);
      expect(
        run([started, { module: 'demo', outcome: { kind: 'completed' }, scan: 1, type: 'module-finished' }]).stale,
      ).toBe(true);
      expect(
        run([started, { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' }])
          .stale,
      ).toBe(true);
    });

    it('a request clears the stale mark', () => {
      expect(run([started, FIRST_BATCH, requested]).stale).toBe(false);
    });

    it('keeps the overview of the finished scan', () => {
      const state = run([...DEMO_SCAN, requested, loaded(DEMO_OVERVIEW)]);

      expect(state.overview).toBe(DEMO_OVERVIEW);
      expect(state.stale).toBe(false);
      expect(state.outcome).toEqual({ kind: 'completed' });
      expect(state.modules).toEqual([{ module: 'demo', outcome: { kind: 'completed' } }]);
    });

    it('keeps the totals counted so far during a scan', () => {
      const sofar: Overview = { ...DEMO_OVERVIEW, status: { scan: 1, state: 'scanning' } };

      const state = run([started, FIRST_BATCH, requested, loaded(sofar)]);

      expect(state.overview).toBe(sofar);
      expect(state.status).toEqual({ scan: 1, state: 'scanning' });
    });

    it('starts a new scan from nothing but the last scan', () => {
      const state = run([...DEMO_SCAN, loaded(DEMO_OVERVIEW), { modules: ['demo'], scan: 2, type: 'scan-started' }]);

      expect(state.overview).toEqual({
        ...DEMO_OVERVIEW,
        found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
        status: { scan: 2, state: 'scanning' },
        totals: [],
      });
      expect(state.stale).toBe(false);
      expect(state.outcome).toBeNull();
      expect(state.scan).toBe(2);
    });

    it('keeps the modules a scan runs, as its scan-started listed them', () => {
      const state = run([...DEMO_SCAN, { modules: ['logs'], scan: 2, type: 'scan-started' }]);

      expect(run([started]).scanModules).toStrictEqual(['demo']);
      expect(state.scanModules).toStrictEqual(['logs']);
    });

    it('keeps the progress, the dictionary and the state of the engine', () => {
      const state = run([
        { dictionary: DEMO_DICTIONARY, type: 'dictionary-loaded' },
        loaded(scanning(4)),
        { bytes: 2048, entries: 10, scan: 4, type: 'scan-progress' },
      ]);

      expect(state.dictionary).toBe(DEMO_DICTIONARY);
      expect(state.scan).toBe(4);
      expect(state.progress).toEqual({ bytes: 2048, entries: 10 });
      const cleaning = run([loaded({ ...IDLE, status: { cleanup: 3, state: 'cleaning' } })]);
      expect(cleaning.status).toEqual({ cleanup: 3, state: 'cleaning' });
    });

    it('keeps the settings view and the access status the core answered', () => {
      const state = run([
        { settings: SETTINGS, type: 'settings-loaded' },
        { status: allFiles('denied'), type: 'access-loaded' },
      ]);

      expect(state.settings).toBe(SETTINGS);
      expect(state.access).toEqual(allFiles('denied'));
    });

    it('a setting-changed event replaces its value and clears its problem', () => {
      const state = run([
        { settings: SETTINGS, type: 'settings-loaded' },
        changed('demo.count', { kind: 'number', value: 3 }),
      ]);

      expect(state.settings?.values['demo.count']).toEqual({ kind: 'number', value: 3 });
      expect(state.settings?.values['demo.enabled']).toEqual({ kind: 'switch', value: true });
      expect(state.settings?.problems).toEqual([{ key: 'demo.enabled', kind: 'wrong-type' }]);
    });

    it('a setting-changed event asks for the overview again, even before the settings arrived', () => {
      const state = run([changed('core.ignore.categories', { kind: 'ids', value: ['demo.caches'] })]);

      expect(state.stale).toBe(true);
      expect(state.settings).toBeNull();
    });

    it('an access-changed event replaces the access status', () => {
      const state = run([
        { status: allFiles('denied'), type: 'access-loaded' },
        { status: allFiles('granted'), type: 'access-changed' },
      ]);

      expect(state.access).toEqual(allFiles('granted'));
    });

    it('follows the scan an overview reports after a reload during that scan', () => {
      const state = run([
        { bytes: 1024, entries: 5, scan: 4, type: 'scan-progress' },
        loaded(scanning(4)),
        { findings: DEMO_FINDINGS.slice(0, 1).map((finding) => ({ ...finding })), scan: 4, type: 'findings-added' },
        { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 4, type: 'scan-finished' },
      ]);

      expect(state.scan).toBe(4);
      expect(state.stale).toBe(true);
      expect(state.status).toEqual({ state: 'idle' });
      expect(state.outcome).toEqual({ kind: 'completed' });
    });
  });
});
