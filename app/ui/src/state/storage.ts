import { useCallback, useEffect, useState, useSyncExternalStore } from 'react';

import type { Failure } from '../ipc';
import type {
  Descent,
  MeaningGroup,
  NodeId,
  NodeInfo,
  PlaceRow,
  PlaceView,
  StorageMap,
} from '../ipc/bindings.generated';

import { failureOf, getStorageMeaning, getStoragePlace } from '../ipc';
import { useCoreStore } from './store';

// The rows of a place view are asked page by page and kept in a small cache, as the finding lists are.
const PAGE_SIZE = 100;
const MAX_PAGES = 10;

// The map of the volume `volume` and where its numbers come from, as a string: it changes when the phase, the state
// id or the volume's parts do (an ignore or a restore, a scan's progress), and only then.
const mapKeyOf = (map: null | StorageMap, volume: null | number): null | string =>
  map === null
    ? null
    : JSON.stringify({ phase: map.phase, volume: volume === null ? null : (map.volumes[volume] ?? null) });

// What a place view shows now.
export type PlaceRows = {
  // The node last given to show: it differs from the node asked while the view has not taken it up yet.
  readonly asked: NodeId | null;
  readonly failure: Failure | null;
  // The node the rows belong to: the node asked for, or the root a stale answer reopened; `null` while not shown.
  readonly node: NodeId | null;
  readonly onRangeChange: (first: number, last: number) => void;
  // The opened node and the folders the automatic descent went through, the outermost first.
  readonly path: PlaceView['path'];
  readonly rowAt: (index: number) => PlaceRow | undefined;
  // How many rows the folder has; `undefined` until the core answered its first page.
  readonly total: number | undefined;
};

export type PlaceSource = {
  // The node last given to `show`, before a stale answer reopened the root.
  readonly asked: () => NodeId | null;
  readonly ensure: (first: number, last: number) => void;
  readonly failure: () => Failure | null;
  // How many rows the cache holds now.
  readonly held: () => number;
  readonly node: () => NodeId | null;
  readonly path: () => PlaceView['path'];
  // The map changed (its phase, its state id or the volume's parts): every page is dropped and the rows on screen are
  // asked again.
  readonly phaseChanged: (key: null | string) => void;
  readonly rowAt: (index: number) => PlaceRow | undefined;
  // Shows `node` (through its single-child folders, or exactly it), or nothing for `null`.
  readonly show: (node: NodeId | null, descent?: Descent) => void;
  readonly subscribe: (listener: () => void) => () => void;
  readonly total: () => number | undefined;
  readonly version: () => number;
};

type Range = { readonly first: number; readonly last: number };

const sameNode = (a: NodeId | null, b: NodeId | null): boolean =>
  a === b || (a !== null && b !== null && a.state === b.state && a.index === b.index);

// The rows of one place, asked of the core. `root` gives the node a stale answer reopens. The first page settles on a
// folder (the descent); the next pages are of that folder.
export const createPlaceSource = (
  getPlace: (node: NodeId, descent: Descent, offset: number, limit: number) => Promise<PlaceView>,
  {
    maxPages,
    pageSize,
    phase,
    root,
  }: {
    readonly maxPages: number;
    readonly pageSize: number;
    readonly phase: null | string;
    readonly root: () => NodeId | null;
  },
): PlaceSource => {
  // Least recently used first: a Map keeps the order its keys were set in.
  const pages = new Map<number, readonly PlaceRow[]>();
  const loading = new Set<number>();
  const listeners = new Set<() => void>();
  let node: NodeId | null = null;
  let shown: NodeId | null = null;
  let descent: Descent = 'through';
  // The folder the first page settled on: the last of its path.
  let folder: NodeId | null = null;
  let path: PlaceView['path'] = [];
  let total: number | undefined;
  let failure: Failure | null = null;
  let range: Range = { first: 0, last: 0 };
  let currentPhase = phase;
  // Whatever changes what is asked starts a new generation: the answers of an older one are dropped.
  let generation = 0;
  let version = 0;

  const changed = (): void => {
    version += 1;
    for (const listener of listeners) {
      listener();
    }
  };

  const keep = (page: number, rows: readonly PlaceRow[]): void => {
    pages.delete(page);
    pages.set(page, rows);

    for (const oldest of pages.keys()) {
      if (pages.size <= maxPages) {
        break;
      }
      pages.delete(oldest);
    }
  };

  const forget = (): void => {
    generation += 1;
    pages.clear();
    loading.clear();
    folder = null;
  };

  const load = (page: number): void => {
    if (node === null || loading.has(page)) {
      return;
    }
    // A next page waits for the first one: until then the folder it is of is not known.
    if (page > 0 && folder === null) {
      load(0);

      return;
    }
    const asked = node;
    const at = generation;
    loading.add(page);
    const answer =
      folder === null ? getPlace(asked, descent, 0, pageSize) : getPlace(folder, 'stay', page * pageSize, pageSize);

    void answer.then(
      (view) => {
        if (at !== generation) {
          return;
        }
        loading.delete(page);
        keep(page, view.rows);
        total = view.total;
        failure = null;

        if (page === 0) {
          path = view.path;
          folder = view.path.at(-1)?.id ?? asked;
          loadRange(range);
        }
        changed();
      },
      (error: unknown) => {
        if (at !== generation) {
          return;
        }
        loading.delete(page);
        const reason = failureOf(error);
        const stale = reason.kind === 'refused' && reason.refusal.error.code === 'stale-node' ? root() : null;

        if (stale !== null && !sameNode(stale, asked)) {
          // The node belongs to a state that is gone: the volume's first root is opened instead.
          forget();
          node = stale;
          descent = 'through';
          path = [];
          total = undefined;
          failure = null;
          range = { first: 0, last: 0 };
          changed();
          load(0);
        } else if (failure === null) {
          failure = reason;
          changed();
        }
      },
    );
  };

  const loadRange = ({ first, last }: Range): void => {
    const end = total === undefined ? last : Math.min(last, total - 1);

    if (first < 0 || first > end) {
      if (total === undefined) {
        load(0);
      }

      return;
    }

    for (let page = Math.floor(first / pageSize); page <= Math.floor(end / pageSize); page += 1) {
      const cached = pages.get(page);

      if (cached === undefined) {
        load(page);
      } else {
        // Used again: the page moves to the end of the least recently used order.
        keep(page, cached);
      }
    }
  };

  return {
    asked: () => shown,
    ensure: (first, last): void => {
      range = { first, last };
      loadRange(range);
    },
    failure: () => failure,
    held: () => [...pages.values()].reduce((sum, rows) => sum + rows.length, 0),
    node: () => node,
    path: () => path,
    phaseChanged: (key): void => {
      if (key === currentPhase) {
        return;
      }
      currentPhase = key;

      if (node === null) {
        return;
      }
      forget();
      path = [];
      total = undefined;
      failure = null;
      changed();
      loadRange(range);
    },
    rowAt: (index): PlaceRow | undefined => {
      const page = Math.floor(index / pageSize);

      return pages.get(page)?.[index - page * pageSize];
    },
    show: (next, nextDescent = 'through'): void => {
      if (sameNode(next, node) && nextDescent === descent) {
        return;
      }
      forget();
      node = next;
      shown = next;
      descent = nextDescent;
      path = [];
      total = undefined;
      failure = null;
      range = { first: 0, last: 0 };
      changed();
      load(0);
    },
    subscribe: (listener): (() => void) => {
      listeners.add(listener);

      return (): void => {
        listeners.delete(listener);
      };
    },
    total: () => total,
    version: () => version,
  };
};

// The rows of the place `node` of volume `volume` (`null`: no place is shown yet, so nothing is asked), opened through
// its single-child folders or at `node` itself; a stale node reopens that volume's first root. Asked again when the
// volume's map changes.
export const useStoragePlace = (node: NodeId | null, volume: number, descent: Descent = 'through'): PlaceRows => {
  const store = useCoreStore();
  const [source] = useState(() =>
    createPlaceSource(getStoragePlace, {
      maxPages: MAX_PAGES,
      pageSize: PAGE_SIZE,
      phase: mapKeyOf(store.getState().storageMap, volume),
      root: () => store.getState().storageMap?.volumes[volume]?.roots[0] ?? null,
    }),
  );
  const phase = useSyncExternalStore(store.subscribe, () => mapKeyOf(store.getState().storageMap, volume));

  useSyncExternalStore(source.subscribe, source.version);
  useEffect(() => {
    source.show(node, descent);
  }, [source, node, descent]);
  useEffect(() => {
    source.phaseChanged(phase);
  }, [source, phase]);

  const onRangeChange = useCallback(
    (first: number, last: number): void => {
      source.ensure(first, last);
    },
    [source],
  );

  return {
    asked: source.asked(),
    failure: source.failure(),
    node: source.node(),
    onRangeChange,
    path: source.path(),
    rowAt: (index: number): PlaceRow | undefined => source.rowAt(index),
    total: source.total(),
  };
};

// What the by-meaning view shows now.
export type MeaningRows = {
  readonly failure: Failure | null;
  // The groups of the last answer, kept while the next one is on its way; `null` before the first.
  readonly groups: null | readonly MeaningGroup[];
};

// The groups of the volume `volume` (`null`: the view is not shown, so nothing is asked), asked again when the
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
          setAnswer({ rows: { failure: null, groups }, volume });
        }
      },
      (error: unknown) => {
        if (current) {
          setAnswer((last) => ({
            rows: { failure: failureOf(error), groups: last?.volume === volume ? last.rows.groups : null },
            volume,
          }));
        }
      },
    );

    return (): void => {
      current = false;
    };
  }, [volume, phase]);

  return volume !== null && answer?.volume === volume ? answer.rows : { failure: null, groups: null };
};

// The scan roots of the volume `volume` when it holds several, each as the core describes it (`null`: the view is not
// shown, so nothing is asked): `undefined` until every root answered, `null` when the volume holds one root or the core
// refused (the place view of its first root then says why). Asked again when the volume's map changes.
export const useStorageRoots = (volume: null | number): null | readonly NodeInfo[] | undefined => {
  const store = useCoreStore();
  // The volume's roots and its map as one string: it changes when either does, and only then.
  const key = useSyncExternalStore(store.subscribe, () => {
    const map = store.getState().storageMap;
    const roots = volume === null ? undefined : map?.volumes[volume]?.roots;

    return roots === undefined || roots.length < 2 ? null : JSON.stringify({ map: mapKeyOf(map, volume), roots });
  });
  const [answer, setAnswer] = useState<null | { readonly key: string; readonly nodes: null | readonly NodeInfo[] }>(
    null,
  );

  useEffect(() => {
    if (key === null) {
      return;
    }
    const { roots } = JSON.parse(key) as { readonly roots: readonly NodeId[] };
    let current = true;

    // A root's own description is the first step of its path: no row is asked.
    Promise.all(roots.map((root) => getStoragePlace(root, 'stay', 0, 0))).then(
      (views) => {
        if (current) {
          setAnswer({ key, nodes: views.flatMap(({ path }) => path.slice(0, 1)) });
        }
      },
      () => {
        if (current) {
          setAnswer({ key, nodes: null });
        }
      },
    );

    return (): void => {
      current = false;
    };
  }, [key]);

  if (key === null) {
    return null;
  }

  return answer?.key === key ? answer.nodes : undefined;
};
