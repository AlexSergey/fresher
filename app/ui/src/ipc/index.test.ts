import { describe, expect, it, vi } from 'vitest';

import type { Event, FindingsQuery, Refusal } from './bindings.generated';

import {
  cancelCleanup,
  cancelScan,
  CommandFailed,
  confirmCleanup,
  discardCleanup,
  failureOf,
  getAccessStatus,
  getCleanup,
  getCleanups,
  getDictionary,
  getFindings,
  getModules,
  getOverview,
  getSettings,
  prepareCleanup,
  putBack,
  quitAfterCleanup,
  revealInTrash,
  startScan,
  subscribe,
  updateSetting,
} from '.';
import { DEMO_PLAN, HISTORY, MOVED_DETAIL } from '../test/cleanup-plan';
import { BUSY, DEMO_SCAN, IDLE } from '../test/demo-scan';
import { mockCore, pushEvents, refuse } from './mock';

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
        getSettings: answer({ file: 'loaded', problems: [], settings: [], values: {} }),
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
      await getDictionary('en');
      await expect(getAccessStatus()).resolves.toEqual({
        capabilities: [{ access: 'unknown', capability: { kind: 'all-files' }, note: null }],
      });

      expect(seen).toContainEqual({ query });
      expect(seen).toContainEqual({ update });
      expect(seen).toContainEqual({ locale: 'en' });
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
      ]);
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
