import { useCallback, useMemo, useRef, useState } from 'react';

import type { FindingId, NodeInfo } from '../../ipc/bindings.generated';
import type { Selection } from '../../state/selection';

import { failureOf, pickOwnItem } from '../../ipc';
import { useCore, useCoreStore } from '../../state/store';

export type OwnPick = {
  // The row is in the selection.
  readonly ticked: (row: Pickable) => boolean;
  // Ticks the row (the core picks it first and answers the finding) or unticks it (no question to the core).
  readonly toggle: (row: Pickable) => void;
};

// A row of either view: the node and the finding the core already made of it.
type Pickable = { readonly node: NodeInfo; readonly picked: FindingId | null };

const keyOf = ({ node }: Pickable): string => `${String(node.id.state)}:${String(node.id.index)}`;

const isEmpty = (selection: Selection): boolean => selection.findings.size === 0 && selection.categories.size === 0;

// What the user ticks on the Largest screen. The core turns an item into a finding only when it is ticked; the finding
// goes into the app's selection like any other. The ids of the findings picked in this session are kept by node, and
// forgotten when a new scan starts (the selection is cleared then).
export const useOwnPick = (): OwnPick => {
  const store = useCoreStore();
  const selection = useCore((state) => state.selection.findings);
  const scan = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const [kept, setKept] = useState<{ readonly ids: ReadonlyMap<string, FindingId>; readonly scan: null | number }>(
    () => ({ ids: new Map(), scan }),
  );
  // Nodes the core is picking now: a second click on the same row waits.
  const pickingRef = useRef(new Set<string>());
  const ids = useMemo(() => (kept.scan === scan ? kept.ids : new Map<string, FindingId>()), [kept, scan]);

  const idOf = (row: Pickable): FindingId | null => ids.get(keyOf(row)) ?? row.picked;

  const ticked = (row: Pickable): boolean => {
    const id = idOf(row);

    return id !== null && selection.has(id);
  };

  const toggle = useCallback(
    (row: Pickable): void => {
      const id = ids.get(keyOf(row)) ?? row.picked;
      const held = id === null ? undefined : selection.get(id);

      if (held !== undefined) {
        store.dispatch({ finding: held, type: 'toggle-finding' });

        return;
      }

      const key = keyOf(row);

      if (pickingRef.current.has(key)) {
        return;
      }
      pickingRef.current.add(key);
      const started = store.getState().selection;
      pickOwnItem(row.node.id).then(
        (finding) => {
          pickingRef.current.delete(key);
          const now = store.getState();

          // A new scan began (or ended) while the core was picking: the selection was cleared, this answer is old.
          if (now.status.state === 'scanning' || (now.overview?.lastScan?.scan ?? null) !== scan) {
            return;
          }
          // The selection was cleared while the core was picking: the user asked for nothing picked.
          if (isEmpty(now.selection) && !isEmpty(started)) {
            return;
          }
          setKept((before) => ({
            ids: new Map([...(before.scan === scan ? before.ids : []), [key, finding.id]]),
            scan,
          }));
          store.dispatch({ finding, type: 'toggle-finding' });
        },
        (error: unknown) => {
          pickingRef.current.delete(key);
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        },
      );
    },
    [ids, scan, selection, store],
  );

  return { ticked, toggle };
};
