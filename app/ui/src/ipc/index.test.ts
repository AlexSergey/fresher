import { describe, expect, it, vi } from 'vitest';

import type { DemoOutcome, DemoPlay, Event, FindingsQuery, Refusal, Sizes, TrashView } from './bindings.generated';

import {
  cancelCleanup,
  cancelScan,
  CommandFailed,
  COMMANDS,
  confirmCleanup,
  demoFinished,
  demoPlay,
  discardCleanup,
  failureOf,
  getAccessStatus,
  getApp,
  getApps,
  getCleanup,
  getCleanups,
  getDictionary,
  getFindings,
  getLargest,
  getLargestPlace,
  getModules,
  getOverview,
  getOwnData,
  getSettings,
  getStorageMap,
  getStorageMeaning,
  getTrash,
  ignore,
  openAccessSettings,
  openTrash,
  pickFolder,
  pickOwnItem,
  prepareCleanup,
  putBack,
  quitAfterCleanup,
  refreshAccess,
  relaunch,
  resetSettings,
  restoreIgnored,
  revealApp,
  revealFinding,
  revealInTrash,
  startScan,
  subscribe,
  updateSetting,
} from '.';
import { DEMO_PLAN, HISTORY, MOVED_DETAIL } from '../test/cleanup-plan';
import { allFiles, BUSY, DEMO_SCAN, IDLE } from '../test/demo-scan';
import { SAMPLE_STORAGE_MAP } from '../test/storage-map';
import { mockCore, pushEvents, refuse, sampleStorageAnswers } from './mock';

const SIZES: Sizes = { allocated: 4096, completeness: 'complete', logical: 100, reclaimable: 4096 };

const TRASH: TrashView = {
  trashes: [
    {
      fromCleanups: SIZES,
      goneBy: null,
      index: 0,
      volume: null,
      waiting: SIZES,
    },
  ],
};

const PLAY: DemoPlay = {
  private: ['/Users/me', '/private/tmp/fresher-demo/overview-1'],
  scenario: {
    name: 'overview',
    steps: [{ kind: 'click', target: { name: { kind: 'exact', text: 'Scan' }, role: 'button' } }],
    window: { height: 720, width: 1100 },
    world: 'developer-mac',
  },
};

// Answers that record the arguments each command was given, in order.
const recorder = (): {
  readonly answer: (value: unknown) => (args: unknown) => unknown;
  readonly seen: readonly unknown[];
} => {
  const seen: unknown[] = [];

  return {
    answer: (value) => (args) => {
      seen.push(args);

      return value;
    },
    seen,
  };
};

describe('the IPC module', () => {
  describe('negative cases', () => {
    it('rejects a refused command with the refusal the core sent, its text untouched', async () => {
      mockCore({ startScan: refuse(BUSY) });

      const error: unknown = await startScan().catch((failure: unknown) => failure);

      expect(error).toBeInstanceOf(CommandFailed);
      expect(failureOf(error)).toEqual({ kind: 'refused', refusal: BUSY });
    });

    it('treats a failure that is not a refusal as an unreachable core', async () => {
      mockCore({ getFindings: refuse({ code: 'busy', running: 'scan' } as unknown as Refusal) });

      const notRefusal: unknown = await getFindings({
        category: null,
        limit: 1,
        module: null,
        offset: 0,
        order: 'found',
      }).catch((failure: unknown) => failure);
      const unanswered: unknown = await getOverview().catch((failure: unknown) => failure);

      expect(failureOf(notRefusal)).toEqual({ kind: 'unreachable' });
      expect(failureOf(unanswered)).toEqual({ kind: 'unreachable' });
      expect(failureOf('not even an error')).toEqual({ kind: 'unreachable' });
    });

    it('rejects a subscribe as an unreachable core when the page runs outside the app', async () => {
      Reflect.deleteProperty(window, '__TAURI_INTERNALS__');

      const error: unknown = await subscribe(vi.fn()).catch((failure: unknown) => failure);

      expect(failureOf(error)).toEqual({ kind: 'unreachable' });
    });

    it('a refusal of get_storage_meaning becomes a failure', async () => {
      const stale = {
        error: { code: 'stale-node' },
        text: { args: {}, key: 'core.error.stale-node' },
      } as unknown as Refusal;
      mockCore({ getStorageMeaning: refuse(stale) });

      const error: unknown = await getStorageMeaning(0).catch((failure: unknown) => failure);

      expect(error).toBeInstanceOf(CommandFailed);
      expect(failureOf(error)).toEqual({ kind: 'refused', refusal: stale });
    });

    it('a refusal of pick_own_item becomes a failure', async () => {
      const refusal = {
        error: { code: 'not-own-item' },
        text: { args: {}, key: 'core.error.not-own-item' },
      } as unknown as Refusal;
      mockCore({ pickOwnItem: refuse(refusal) });

      const error: unknown = await pickOwnItem({ index: 3, state: 1 }).catch((failure: unknown) => failure);

      expect(error).toBeInstanceOf(CommandFailed);
      expect(failureOf(error)).toEqual({ kind: 'refused', refusal });
    });

    it('opens the settings only for a capability object, never a string', async () => {
      // The IPC rejects arguments the command cannot read, as the core rejects a capability it does not know.
      mockCore({
        openAccessSettings: () => {
          throw new Error('invalid args `capability` for command `open_access_settings`');
        },
      });

      // @ts-expect-error -- a pane is named by its capability, never by an address or a string.
      const error: unknown = await openAccessSettings('all-files').catch((failure: unknown) => failure);

      expect(failureOf(error)).toEqual({ kind: 'unreachable' });
    });

    it('rejects the demo commands with `CommandFailed` when the shell refuses them', async () => {
      const outcome: DemoOutcome = { elapsedMs: 900, kind: 'finished' };
      mockCore({
        demoFinished: () => {
          throw new Error('the app is not in demo mode');
        },
      });

      const finished: unknown = await demoFinished(outcome).catch((failure: unknown) => failure);
      const played: unknown = await demoPlay().catch((failure: unknown) => failure);

      expect(finished).toBeInstanceOf(CommandFailed);
      expect(failureOf(finished)).toEqual({ kind: 'unreachable' });
      expect(played).toBeInstanceOf(CommandFailed);
    });

    it('cannot push events before anything subscribed', () => {
      mockCore({});

      expect(() => {
        pushEvents(...DEMO_SCAN);
      }).toThrow('Nothing subscribed');
    });
  });

  describe('positive cases', () => {
    it('passes every command its arguments by name', async () => {
      const query: FindingsQuery = { category: 'demo.caches', limit: 50, module: 'demo', offset: 0, order: 'largest' };
      const update = { key: 'demo.count', value: { kind: 'number', value: 3 } } as const;
      const { answer, seen } = recorder();
      mockCore({
        cancelScan: answer(null),
        getAccessStatus: answer({
          capabilities: [{ access: 'unknown', capability: { kind: 'all-files' }, note: null }],
        }),
        getDictionary: answer({}),
        getFindings: answer({ findings: [], offset: 0, total: 0 }),
        getModules: answer([]),
        getOverview: answer(IDLE),
        getOwnData: answer({ budget: 50, total: 1 }),
        getSettings: answer({ file: 'loaded', problems: [], settings: [], values: {} }),
        ignore: answer(null),
        pickFolder: answer('~/Projects'),
        resetSettings: answer(null),
        restoreIgnored: answer(null),
        startScan: answer(1),
        updateSetting: answer(null),
      });

      await expect(startScan()).resolves.toBe(1);
      await cancelScan();
      await expect(getOverview()).resolves.toEqual(IDLE);
      await expect(getModules()).resolves.toEqual([]);
      await getFindings(query);
      await expect(getSettings()).resolves.toMatchObject({ file: 'loaded' });
      await updateSetting(update);
      await resetSettings();
      await ignore({ category: 'demo.caches', kind: 'category' });
      await restoreIgnored({ kind: 'path', path: '/a' });
      await expect(pickFolder()).resolves.toBe('~/Projects');
      await expect(getOwnData()).resolves.toEqual({ budget: 50, total: 1 });
      await getDictionary('en');
      await expect(getAccessStatus()).resolves.toEqual({
        capabilities: [{ access: 'unknown', capability: { kind: 'all-files' }, note: null }],
      });

      expect(seen).toContainEqual({ query });
      expect(seen).toContainEqual({ update });
      expect(seen).toContainEqual({ target: { category: 'demo.caches', kind: 'category' } });
      expect(seen).toContainEqual({ entry: { kind: 'path', path: '/a' } });
      expect(seen).toContainEqual({ locale: 'en' });
    });

    it('get_storage_map calls its command', async () => {
      mockCore(sampleStorageAnswers);

      await expect(getStorageMap()).resolves.toEqual(SAMPLE_STORAGE_MAP);
    });

    it('passes the storage commands their arguments by name', async () => {
      const { answer, seen } = recorder();
      mockCore({ getStorageMeaning: answer([]) });

      await expect(getStorageMeaning(1)).resolves.toEqual([]);

      expect(seen).toStrictEqual([{ volume: 1 }]);
    });

    it('passes the Apps commands their arguments by name', async () => {
      const { answer, seen } = recorder();
      const app = { index: 2, state: 1 };
      mockCore({ getApp: answer(null), getApps: answer(null), revealApp: answer(null) });

      await getApps({ unused: 'a-year' });
      await getApp(app);
      await revealApp(app);

      expect(seen).toStrictEqual([{ query: { unused: 'a-year' } }, { app }, { app }]);
      expect([COMMANDS.getApps, COMMANDS.getApp, COMMANDS.revealApp]).toEqual(['get_apps', 'get_app', 'reveal_app']);
    });

    it('passes the Largest commands their arguments by name', async () => {
      const { answer, seen } = recorder();
      const node = { index: 3, state: 1 };
      const query = { kind: null, limit: 100, offset: 0, place: null, unchanged: 'any' } as const;
      mockCore({ getLargest: answer(null), getLargestPlace: answer(null), pickOwnItem: answer(null) });

      await getLargest(query);
      await getLargestPlace(node, 100, 50);
      await getLargestPlace(null, 0, 100);
      await pickOwnItem(node);

      expect(seen).toStrictEqual([
        { query },
        { folder: node, limit: 50, offset: 100 },
        { folder: null, limit: 100, offset: 0 },
        { node },
      ]);
    });

    it('passes every cleanup command its arguments by name', async () => {
      const { answer, seen } = recorder();
      const selection = { categories: [{ category: 'demo.caches', except: [3] }], findings: [2] };
      const confirmation = { allocated: 1, count: 3, highest: 'low', method: 'trash', plan: 1 } as const;
      mockCore({
        cancelCleanup: answer(null),
        confirmCleanup: answer(4),
        discardCleanup: answer(null),
        getCleanup: answer(MOVED_DETAIL),
        getCleanups: answer(HISTORY),
        getTrash: answer(TRASH),
        openTrash: answer(null),
        prepareCleanup: answer(DEMO_PLAN),
        putBack: answer({ items: [] }),
        quitAfterCleanup: answer(null),
        revealInTrash: answer(null),
      });

      await expect(prepareCleanup(selection)).resolves.toEqual(DEMO_PLAN);
      await discardCleanup(1);
      await expect(confirmCleanup(confirmation)).resolves.toBe(4);
      await cancelCleanup();
      await expect(getCleanups({ limit: 50, offset: 0 })).resolves.toEqual(HISTORY);
      await expect(getCleanup(3)).resolves.toEqual(MOVED_DETAIL);
      await expect(putBack({ cleanup: 3, items: null })).resolves.toEqual({ items: [] });
      await revealInTrash({ cleanup: 3, item: 0 });
      await expect(getTrash()).resolves.toEqual(TRASH);
      await openTrash(1);
      await quitAfterCleanup();

      expect(seen).toStrictEqual([
        { selection },
        { plan: 1 },
        { confirmation },
        {},
        { query: { limit: 50, offset: 0 } },
        { cleanup: 3 },
        { request: { cleanup: 3, items: null } },
        { item: { cleanup: 3, item: 0 } },
        {},
        { index: 1 },
        {},
      ]);
    });

    it('passes every access command its arguments by name', async () => {
      const { answer, seen } = recorder();
      const granted = allFiles('granted');
      const desktop = { kind: 'folder', value: 'desktop' } as const;
      mockCore({
        openAccessSettings: answer(null),
        refreshAccess: answer(granted),
        relaunch: answer(null),
        revealFinding: answer(null),
      });

      await expect(refreshAccess()).resolves.toEqual(granted);
      await openAccessSettings(desktop);
      await revealFinding(7);
      await relaunch();

      expect(seen).toStrictEqual([{}, { capability: desktop }, { finding: 7 }, {}]);
      expect([COMMANDS.refreshAccess, COMMANDS.openAccessSettings, COMMANDS.revealFinding, COMMANDS.relaunch]).toEqual([
        'refresh_access',
        'open_access_settings',
        'reveal_finding',
        'relaunch',
      ]);
    });

    it('passes the demo commands their arguments by name', async () => {
      const outcome: DemoOutcome = { kind: 'failed', reason: 'no button Scan', step: 3 };
      const { answer, seen } = recorder();
      mockCore({ demoFinished: answer(null), demoPlay: answer(PLAY) });

      await expect(demoPlay()).resolves.toEqual(PLAY);
      await demoFinished(outcome);

      expect(seen).toStrictEqual([{}, { outcome }]);
      expect([COMMANDS.demoPlay, COMMANDS.demoFinished]).toEqual(['demo_play', 'demo_finished']);
    });

    it('answers `null` from demo_play outside demo mode', async () => {
      mockCore({ demoPlay: () => null });

      await expect(demoPlay()).resolves.toBeNull();
    });

    it('delivers the events of the subscribed channel', async () => {
      mockCore({});
      const onEvent = vi.fn<(event: Event) => void>();

      await subscribe(onEvent);
      pushEvents(...DEMO_SCAN);

      expect(onEvent.mock.calls.map(([event]) => event)).toEqual(DEMO_SCAN);
    });
  });
});
