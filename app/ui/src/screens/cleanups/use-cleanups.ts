import { useCallback, useEffect, useState } from 'react';

import type { CleanupsPage } from '../../ipc/bindings.generated';

import { failureOf, getCleanups } from '../../ipc';
import { useCoreStore } from '../../state/store';

export type History = {
  // Asks for the next page of older cleanups.
  readonly more: () => void;
  // Every cleanup asked for so far, newest first, with the core's totals; `null` until the first page answered.
  readonly page: CleanupsPage | null;
};

// Cleanups asked of the core at a time.
const PAGE_SIZE = 50;

// The history of cleanups, newest first, a page at a time; read anew whenever the screen opens. A failure is the
// frame's: the core never refuses this read.
export const useCleanups = (): History => {
  const store = useCoreStore();
  const [page, setPage] = useState<CleanupsPage | null>(null);

  const load = useCallback(
    (offset: number): (() => void) => {
      let current = true;

      getCleanups({ limit: PAGE_SIZE, offset }).then(
        (answer) => {
          if (current) {
            setPage((shown) =>
              shown === null || offset === 0
                ? answer
                : { ...answer, cleanups: [...shown.cleanups, ...answer.cleanups] },
            );
          }
        },
        (error: unknown) => {
          if (current) {
            store.dispatch({ failure: failureOf(error), type: 'command-failed' });
          }
        },
      );

      return (): void => {
        current = false;
      };
    },
    [store],
  );

  useEffect(() => load(0), [load]);

  return {
    more: (): void => {
      if (page !== null && page.cleanups.length < page.total) {
        load(page.cleanups.length);
      }
    },
    page,
  };
};
