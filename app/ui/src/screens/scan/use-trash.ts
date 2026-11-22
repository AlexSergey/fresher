import { useEffect, useRef, useState } from 'react';

import type { TrashLine } from '../../ipc/bindings.generated';
import type { CoreStore } from '../../state/store';

import { failureOf, getTrash, openTrash } from '../../ipc';
import { useCore } from '../../state/store';

// What waits in each Trash, asked for when the screen shows, after a new scan, after a cleanup ends and when the
// window gets focus. It is information: a failed answer shows no line and tells the frame nothing, and an answer to
// an older request never replaces a newer one.
export const useTrash = (): readonly TrashLine[] => {
  const [lines, setLines] = useState<readonly TrashLine[]>([]);
  const latestRef = useRef(0);
  const scan = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const cleanup = useCore((state) => (state.cleanup.kind === 'finished' ? state.cleanup.cleanup : null));

  useEffect(() => {
    const ask = (): void => {
      latestRef.current += 1;
      const request = latestRef.current;

      getTrash().then(
        (view) => {
          if (request === latestRef.current) {
            setLines(view.trashes);
          }
        },
        () => {
          if (request === latestRef.current) {
            setLines([]);
          }
        },
      );
    };

    ask();
    window.addEventListener('focus', ask);

    return (): void => {
      window.removeEventListener('focus', ask);
      // An answer that comes after the screen is gone is dropped.
      latestRef.current += 1;
    };
  }, [scan, cleanup]);

  return lines;
};

// Opens Trash `index` of the last answer in Finder; a refusal is the frame's toast, with the core's text.
export const openTrashLine =
  (store: CoreStore) =>
  (index: number): void => {
    openTrash(index).catch((error: unknown) => {
      store.dispatch({ failure: failureOf(error), type: 'command-failed' });
    });
  };
