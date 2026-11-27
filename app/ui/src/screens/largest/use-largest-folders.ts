import { useCallback, useEffect, useRef, useState } from 'react';

import type { FoldersPage } from '../../ipc/bindings.generated';

import { failureOf, getLargestFolders } from '../../ipc';
import { useCore, useCoreStore } from '../../state/store';

export type LargestFolders = {
  // The last answer failed: the frame shows the core's text, the tab an empty state.
  readonly failed: boolean;
  // Asks for the next page of the list.
  readonly more: () => void;
  // The folders asked for so far with the core's totals; `null` until the first page answered, and while no list can
  // be asked for (no scan yet, a scan running).
  readonly page: FoldersPage | null;
};

// Folders asked of the core at a time (the engine's `MAX_PAGE`).
const PAGE_SIZE = 100;

// The folders of the home left alone in the last scan, largest first, a page at a time. Asked when a scan ends and
// again when a cleanup removed findings or an ignore list changed; nothing is asked before a scan or while one runs. A
// refusal goes to the store: the frame shows the core's text.
export const useLargestFolders = (): LargestFolders => {
  const store = useCoreStore();
  const scan = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const scanning = useCore((state) => state.status.state === 'scanning');
  const removed = useCore((state) => state.removed);
  const ignoreChanges = useCore((state) => state.ignoreChanges);
  const [page, setPage] = useState<FoldersPage | null>(null);
  const [failed, setFailed] = useState(false);
  // Answers of an older list are dropped: it grows by one counter at every new list.
  const generationRef = useRef(0);
  // The next page is on its way: a second "Show more" waits for it.
  const loadingMoreRef = useRef(false);
  const askable = scan !== null && !scanning;

  const load = useCallback(
    (offset: number): void => {
      const asked = generationRef.current;

      if (offset > 0) {
        loadingMoreRef.current = true;
      }

      getLargestFolders({ limit: PAGE_SIZE, offset }).then(
        (answer) => {
          if (asked === generationRef.current) {
            loadingMoreRef.current = false;
            setFailed(false);
            // The rows from `offset` on are the answer's, so a page asked twice never lands twice.
            setPage((shown) =>
              shown === null || offset === 0
                ? answer
                : { ...answer, rows: [...shown.rows.slice(0, offset), ...answer.rows] },
            );
          }
        },
        (error: unknown) => {
          if (asked === generationRef.current) {
            loadingMoreRef.current = false;
            setFailed(true);
            store.dispatch({ failure: failureOf(error), type: 'command-failed' });
          }
        },
      );
    },
    [store],
  );

  // `scan`, `removed` and `ignoreChanges` are the signals that the list moved: they only restart it.
  useEffect(() => {
    generationRef.current += 1;
    loadingMoreRef.current = false;

    if (askable) {
      load(0);
    }

    return (): void => {
      generationRef.current += 1;
    };
  }, [askable, ignoreChanges, load, removed, scan]);

  return {
    // A failed "Show more" keeps the folders already in: only a list with no rows fails as a whole.
    failed: askable && failed && page === null,
    more: (): void => {
      if (page !== null && !loadingMoreRef.current && page.rows.length < page.total) {
        load(page.rows.length);
      }
    },
    page: askable ? page : null,
  };
};
