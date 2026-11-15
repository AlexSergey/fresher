import { mockIPC } from '@tauri-apps/api/mocks';

import { type CommandKey, COMMANDS } from '.';

// What the Rust core answers in a test, by the command keys of src/ipc/index.ts.
export type CoreAnswers = Readonly<Record<CommandKey, () => unknown>>;

const keyOf = (command: string): CommandKey | undefined =>
  (Object.keys(COMMANDS) as CommandKey[]).find((key) => COMMANDS[key] === command);

// Replaces the Rust core with the given answers; a command without an answer fails, as an unknown command would.
export const mockCore = (answers: Partial<CoreAnswers>): void => {
  mockIPC((command) => {
    const key = keyOf(command);
    const answer = key === undefined ? undefined : answers[key];

    if (answer === undefined) {
      throw new Error(`The core has no answer for "${command}" in this test`);
    }

    return answer();
  });
};
