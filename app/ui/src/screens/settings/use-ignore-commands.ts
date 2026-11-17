import type { IgnoredEntry, IgnoreTarget } from '../../ipc/bindings.generated';

import { failureOf, ignore, restoreIgnored } from '../../ipc';
import { useCoreStore } from '../../state/store';

// The two commands of the ignore lists; a refusal reaches the store as a failure, as every command's does.
export const useIgnoreCommands = (): {
  readonly ignore: (target: IgnoreTarget) => void;
  readonly restore: (entry: IgnoredEntry) => void;
} => {
  const store = useCoreStore();
  const fail = (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };

  return {
    ignore: (target): void => {
      ignore(target).catch(fail);
    },
    restore: (entry): void => {
      restoreIgnored(entry).catch(fail);
    },
  };
};
