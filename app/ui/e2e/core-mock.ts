import type { Page } from '@playwright/test';

import type { Event, Refusal } from '../src/ipc/bindings.generated';

import { type CommandKey, COMMANDS } from '../src/ipc';

// What the mocked core does: an answer per command key, a refusal per command key, and the events a scan streams.
export type CoreScript = {
  readonly answers?: Partial<Readonly<Record<CommandKey, unknown>>>;
  readonly refusals?: Partial<Readonly<Record<CommandKey, Refusal>>>;
  readonly scan?: readonly Event[];
};

type ByCommand = {
  readonly answers: Readonly<Record<string, unknown>>;
  readonly refusals: Readonly<Record<string, Refusal>>;
  readonly scan: readonly Event[];
  readonly startScan: string;
  readonly subscribe: string;
};

const byCommand = <T>(table: Partial<Readonly<Record<CommandKey, T>>> = {}): Readonly<Record<string, T>> =>
  Object.fromEntries((Object.entries(table) as [CommandKey, T][]).map(([key, value]) => [COMMANDS[key], value]));

// The Rust core replaced in the browser, by the command keys of src/ipc/index.ts, injected before the page loads.
// `@tauri-apps/api` calls `window.__TAURI_INTERNALS__`: `invoke` for a command, `transformCallback` for a channel.
// `subscribe` keeps the channel; `start_scan`, when answered, then streams `scan` through it.
export const mockCore = async (page: Page, script: CoreScript = {}): Promise<void> => {
  const setup: ByCommand = {
    answers: byCommand(script.answers),
    refusals: byCommand(script.refusals),
    scan: script.scan ?? [],
    startScan: COMMANDS.startScan,
    subscribe: COMMANDS.subscribe,
  };

  await page.addInitScript((core: ByCommand) => {
    const callbacks = new Map<number, (message: unknown) => void>();
    let channel: number | undefined;

    const stream = (): void => {
      const deliver = channel === undefined ? undefined : callbacks.get(channel);
      core.scan.forEach((message, index) => {
        setTimeout(() => deliver?.({ index, message }), 0);
      });
    };

    const invoke = (command: string, args: Readonly<Record<string, unknown>>): Promise<unknown> => {
      if (command === core.subscribe) {
        channel = (args.channel as { readonly id: number }).id;

        return Promise.resolve(null);
      }
      if (command in core.refusals) {
        // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
        return Promise.reject(core.refusals[command]);
      }
      if (!(command in core.answers)) {
        return Promise.reject(new Error(`The core has no answer for "${command}"`));
      }
      if (command === core.startScan) {
        stream();
      }

      return Promise.resolve(core.answers[command]);
    };

    let nextCallback = 1;
    Object.defineProperty(globalThis, '__TAURI_INTERNALS__', {
      value: {
        invoke,
        transformCallback: (callback: (message: unknown) => void): number => {
          const id = nextCallback;
          nextCallback += 1;
          callbacks.set(id, callback);

          return id;
        },
        unregisterCallback: (id: number): void => {
          callbacks.delete(id);
        },
      },
    });
  }, setup);
};
