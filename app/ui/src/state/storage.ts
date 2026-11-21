import { useEffect, useState, useSyncExternalStore } from 'react';

import type { MeaningGroup, StorageMap } from '../ipc/bindings.generated';

import { getStorageMeaning } from '../ipc';
import { useCoreStore } from './store';

// The map of the volume `volume` and where its numbers come from, as a string: it changes when the phase, the state
// id or the volume's parts do (an ignore or a restore, a scan's progress), and only then.
const mapKeyOf = (map: null | StorageMap, volume: null | number): null | string =>
  map === null
    ? null
    : JSON.stringify({ phase: map.phase, volume: volume === null ? null : (map.volumes[volume] ?? null) });

// What the core's by-meaning query answered for the ring's split by level.
export type MeaningRows = {
  // The groups of the last answer, kept while the next one is on its way; `null` before the first.
  readonly groups: null | readonly MeaningGroup[];
};

// The groups of the volume `volume` (`null`: nothing is asked), asked again when the
// volume's map changes: a new state, an ignore or a restore, a scan's progress.
export const useStorageMeaning = (volume: null | number): MeaningRows => {
  const store = useCoreStore();
  const phase = useSyncExternalStore(store.subscribe, () => mapKeyOf(store.getState().storageMap, volume));
  const [answer, setAnswer] = useState<null | { readonly rows: MeaningRows; readonly volume: number }>(null);

  useEffect(() => {
    if (volume === null) {
      return;
    }
    let current = true;

    getStorageMeaning(volume).then(
      (groups) => {
        if (current) {
          setAnswer({ rows: { groups }, volume });
        }
      },
      () => {
        // A refused or unreachable core leaves the ring with the split it had; the frame shows the failure.
      },
    );

    return (): void => {
      current = false;
    };
  }, [volume, phase]);

  return volume !== null && answer?.volume === volume ? answer.rows : { groups: null };
};
