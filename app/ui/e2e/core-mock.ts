import type { Page } from '@playwright/test';

import { type CommandKey, COMMANDS } from '../src/ipc';

// The Rust core replaced in the browser, by the command keys of src/ipc/index.ts, injected before the page loads.
// `@tauri-apps/api` calls `window.__TAURI_INTERNALS__.invoke`, so that is all the page needs.
export const mockCore = async (page: Page, answers: Partial<Readonly<Record<CommandKey, unknown>>>): Promise<void> => {
  const table = Object.fromEntries((Object.keys(answers) as CommandKey[]).map((key) => [COMMANDS[key], answers[key]]));

  await page.addInitScript((byCommand: Readonly<Record<string, unknown>>) => {
    Object.defineProperty(globalThis, '__TAURI_INTERNALS__', {
      value: {
        invoke: (command: string): Promise<unknown> =>
          command in byCommand
            ? Promise.resolve(byCommand[command])
            : Promise.reject(new Error(`The core has no answer for "${command}"`)),
        transformCallback: (): number => 0,
      },
    });
  }, table);
};
