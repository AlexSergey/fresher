import { useEffect, useRef, useState } from 'react';

import type { AppsView, Refusal, UnusedFor } from '../../ipc/bindings.generated';

import { failureOf, getApps } from '../../ipc';
import { useCore, useCoreStore } from '../../state/store';

export type Apps = {
  // The core refused the list: its text is shown in place of the list.
  readonly refusal: null | Refusal;
  // The apps the core listed under the filter; `null` until the first answer, and while no list can be asked for (no
  // scan yet).
  readonly view: AppsView | null;
};

// The Apps list under the filter, asked on mount, when the filter changes, when a scan ends and when a cleanup ended
// (an uninstall moved an app). Until a new answer arrives the rows shown stay. An unreachable core goes to the store.
export const useApps = (unused: null | UnusedFor): Apps => {
  const store = useCoreStore();
  const scan = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const scanning = useCore((state) => state.status.state === 'scanning');
  const removed = useCore((state) => state.removed);
  const [view, setView] = useState<AppsView | null>(null);
  const [refusal, setRefusal] = useState<null | Refusal>(null);
  // Answers of an older question are dropped: it grows by one counter at every new question.
  const generationRef = useRef(0);
  const askable = scan !== null && !scanning;

  useEffect(() => {
    generationRef.current += 1;
    const asked = generationRef.current;

    if (askable) {
      getApps({ unused }).then(
        (answer) => {
          if (asked === generationRef.current) {
            setRefusal(null);
            setView(answer);
          }
        },
        (error: unknown) => {
          if (asked !== generationRef.current) {
            return;
          }
          const failure = failureOf(error);

          if (failure.kind === 'refused') {
            setRefusal(failure.refusal);
          } else {
            store.dispatch({ failure, type: 'command-failed' });
          }
        },
      );
    }

    return (): void => {
      generationRef.current += 1;
    };
  }, [askable, removed, scan, store, unused]);

  return { refusal: askable ? refusal : null, view: askable ? view : null };
};
