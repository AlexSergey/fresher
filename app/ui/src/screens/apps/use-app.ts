import { useCallback, useEffect, useRef, useState } from 'react';

import type { AppDetail, AppId, Refusal } from '../../ipc/bindings.generated';

import { failureOf, getApp, isUnknownApp } from '../../ipc';
import { useCore, useCoreStore } from '../../state/store';

export type AppPage = {
  // The page the core answered; `null` until the first answer, and while no page can be asked for.
  readonly detail: AppDetail | null;
  // The core refused the page: its text is shown in place of the page.
  readonly refusal: null | Refusal;
  // Asks the core again (an uninstall it refused changed what the page offers).
  readonly reload: () => void;
  // The core no longer lists the app: a new scan made its id stale.
  readonly stale: boolean;
};

// An app's page, asked on mount, when a scan ends, when a cleanup ends (an uninstall moved the app) and on `reload`.
// Until a new answer arrives the page shown stays. An unreachable core goes to the store.
export const useApp = (app: AppId): AppPage => {
  const store = useCoreStore();
  const scan = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const scanning = useCore((state) => state.status.state === 'scanning');
  const removed = useCore((state) => state.removed);
  const ended = useCore((state) => state.cleanup.kind === 'finished');
  const [detail, setDetail] = useState<AppDetail | null>(null);
  const [refusal, setRefusal] = useState<null | Refusal>(null);
  const [reloads, setReloads] = useState(0);
  // Answers of an older question are dropped: it grows by one counter at every new question.
  const generationRef = useRef(0);
  const askable = scan !== null && !scanning;
  const { index, state } = app;

  useEffect(() => {
    generationRef.current += 1;
    const asked = generationRef.current;

    if (askable) {
      getApp({ index, state }).then(
        (answer) => {
          if (asked === generationRef.current) {
            setRefusal(null);
            setDetail(answer);
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
  }, [askable, ended, index, reloads, removed, scan, state, store]);

  const reload = useCallback((): void => {
    setReloads((count) => count + 1);
  }, []);

  const shown = askable ? refusal : null;

  return { detail: askable ? detail : null, refusal: shown, reload, stale: shown !== null && isUnknownApp(shown) };
};
