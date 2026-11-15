import { describe, expect, it, vi } from 'vitest';

import type { Event, FindingsQuery, Refusal } from './bindings.generated';

import {
  cancelScan,
  CommandFailed,
  failureOf,
  getAccessStatus,
  getDictionary,
  getFindings,
  getModules,
  getOverview,
  getSettings,
  startScan,
  subscribe,
  updateSetting,
} from '.';
import { BUSY, DEMO_SCAN, IDLE } from '../test/demo-scan';
import { mockCore, pushEvents, refuse } from './mock';

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

      const notRefusal: unknown = await getFindings({ category: null, limit: 1, module: null, offset: 0 }).catch(
        (failure: unknown) => failure,
      );
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
      const query: FindingsQuery = { category: 'demo.caches', limit: 50, module: 'demo', offset: 0 };
      const update = { key: 'demo.count', value: { kind: 'number', value: 3 } } as const;
      const seen: unknown[] = [];
      const answer =
        (value: unknown) =>
        (args: unknown): unknown => {
          seen.push(args);

          return value;
        };
      mockCore({
        cancelScan: answer(null),
        getAccessStatus: answer({ fullDiskAccess: 'unknown' }),
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
      await expect(getAccessStatus()).resolves.toEqual({ fullDiskAccess: 'unknown' });

      expect(seen).toContainEqual({ query });
      expect(seen).toContainEqual({ update });
      expect(seen).toContainEqual({ locale: 'en' });
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
