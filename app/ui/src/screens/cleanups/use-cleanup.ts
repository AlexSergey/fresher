import { useCallback, useEffect, useState } from 'react';

import type { CleanupDetail, CleanupId } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { failureOf, getCleanup, putBack, revealInTrash } from '../../ipc';
import { useCoreStore } from '../../state/store';

export type OneCleanup = {
  // How putting each item back just ended, by its index, with the core's text.
  readonly answers: ReadonlyMap<number, string>;
  // `null` until the core answered.
  readonly detail: CleanupDetail | null;
  // Puts back these items, or (`null`) every item still in the Trash.
  readonly putBack: (items: null | readonly number[]) => void;
  // The core's text of the last refused command.
  readonly refusal: null | string;
  // Shows the first item still in the Trash.
  readonly showInTrash: () => void;
};

// One cleanup of the history, and putting its items back. A put back where every item came back is told to
// `onAllBack` with their count; any other keeps each item's answer to show. A refusal is shown with its text; an
// unreachable core is the frame's error.
export const useCleanup = (cleanup: CleanupId, onAllBack: (count: number) => void): OneCleanup => {
  const store = useCoreStore();
  const [detail, setDetail] = useState<CleanupDetail | null>(null);
  const [answers, setAnswers] = useState<ReadonlyMap<number, string>>(() => new Map());
  const [refusal, setRefusal] = useState<null | string>(null);

  const failed = useCallback(
    (error: unknown): void => {
      const failure = failureOf(error);

      if (failure.kind === 'refused') {
        setRefusal(tText(failure.refusal.text));
      } else {
        store.dispatch({ failure, type: 'command-failed' });
      }
    },
    [store],
  );

  const load = useCallback((): void => {
    getCleanup(cleanup).then(setDetail, failed);
  }, [cleanup, failed]);

  useEffect(load, [load]);

  return {
    answers,
    detail,
    putBack: (items): void => {
      setRefusal(null);
      putBack({ cleanup, items: items === null ? null : [...items] }).then((report) => {
        if (report.items.length > 0 && report.items.every((item) => item.outcome.kind === 'put-back')) {
          onAllBack(report.items.length);

          return;
        }
        setAnswers(
          (shown) => new Map([...shown, ...report.items.map((item) => [item.index, tText(item.text)] as const)]),
        );
        load();
      }, failed);
    },
    refusal,
    showInTrash: (): void => {
      const item = detail?.items.find((one) => one.trash === 'in-trash');

      if (item !== undefined) {
        setRefusal(null);
        revealInTrash({ cleanup, item: item.index }).catch(failed);
      }
    },
  };
};
