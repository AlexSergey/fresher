import type { Channel } from '@tauri-apps/api/core';

import { mockIPC } from '@tauri-apps/api/mocks';

import type { Event, Refusal } from './bindings.generated';

import { type CommandKey, COMMANDS } from '.';
import { EMPTY_PLACE, SAMPLE_MEANING, SAMPLE_STORAGE_MAP } from '../test/storage-map';

// What the Rust core answers in a test, by the command keys of src/ipc/index.ts; an answer gets the command's args.
export type CoreAnswers = Readonly<Record<CommandKey, (args: unknown) => unknown>>;

// The storage map's three commands answered with the gallery's sample volume; spread into `mockCore`.
export const sampleStorageAnswers: Pick<CoreAnswers, 'getStorageMap' | 'getStorageMeaning' | 'getStoragePlace'> = {
  getStorageMap: () => SAMPLE_STORAGE_MAP,
  getStorageMeaning: () => SAMPLE_MEANING,
  getStoragePlace: () => EMPTY_PLACE,
};

const keyOf = (command: string): CommandKey | undefined =>
  (Object.keys(COMMANDS) as CommandKey[]).find((key) => COMMANDS[key] === command);

let channel: Channel<Event> | undefined;

// Replaces the Rust core with the given answers; a command without an answer fails, as an unknown command would.
// `subscribe` is always answered (by its answer when given, to delay it): its channel is kept for `pushEvents`.
export const mockCore = (answers: Partial<CoreAnswers>): void => {
  channel = undefined;
  mockIPC((command, args) => {
    const key = keyOf(command);

    if (key === 'subscribe') {
      channel = (args as { readonly channel: Channel<Event> }).channel;

      return answers.subscribe === undefined ? null : answers.subscribe(args);
    }

    const answer = key === undefined ? undefined : answers[key];

    if (answer === undefined) {
      throw new Error(`The core has no answer for "${command}" in this test`);
    }

    return answer(args);
  });
};

// An answer that refuses the command, as the core does: the promise rejects with the plain `Refusal`.
export const refuse =
  (refusal: Refusal): (() => Promise<never>) =>
  () => {
    // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
    return Promise.reject(refusal);
  };

// Sends events through the subscribed channel, as the core does during a scan.
export const pushEvents = (...events: readonly Event[]): void => {
  if (channel === undefined) {
    throw new Error('Nothing subscribed to the core in this test');
  }

  for (const event of events) {
    channel.onmessage(event);
  }
};
