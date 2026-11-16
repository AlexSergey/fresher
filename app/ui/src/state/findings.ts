import type { Failure } from '../ipc';
import type { Finding, FindingsPage, FindingsQuery } from '../ipc/bindings.generated';
import type { Clock } from './clock';
import type { CoreStore } from './store';

import { failureOf } from '../ipc';

// The findings of every list, asked of the core page by page and kept in a small cache, so the UI never holds
// more than `pageSize × maxPages` findings, whatever the size of the scan.
export type FindingsSource = {
  // Loads the pages the rows `first`…`last` (inclusive) of a view need, each with one request; remembers the range
  // as the view's rows on screen.
  readonly ensure: (view: FindingsView, first: number, last: number) => void;
  // Why the view's last page could not be loaded, until a page of it loads again.
  readonly failure: (view: FindingsView) => Failure | null;
  // Follows the store: a new scan drops every page; findings a cleanup removed drop every page and ask the rows on
  // screen again at once; a new overview asks the rows on screen again (at most twice a second), so they move to their
  // new order while the scan runs. Returns the function that stops following.
  readonly follow: (store: CoreStore) => () => void;
  // How many findings the cache holds now.
  readonly held: () => number;
  // The finding at `index` of the view, or `undefined` while its page loads.
  readonly rowAt: (view: FindingsView, index: number) => Finding | undefined;
  readonly subscribe: (listener: () => void) => () => void;
  // How many findings the view has, from the last page the core answered for it; `undefined` before the first.
  readonly total: (view: FindingsView) => number | undefined;
  // Changes whenever anything the source answers changes: the snapshot of `useSyncExternalStore`.
  readonly version: () => number;
};

// Which findings a list shows: a module, a category, an order. The source pages through them itself.
export type FindingsView = Pick<FindingsQuery, 'category' | 'module' | 'order'>;

type Range = { readonly first: number; readonly last: number };

type Shown = Range & { readonly view: FindingsView };

// The rows on screen are asked again at most twice a second while the overview changes.
const REFRESH_INTERVAL_MS = 500;

const keyOf = (view: FindingsView): string => JSON.stringify([view.module, view.category, view.order]);

const pageKey = (key: string, page: number): string => `${key}#${page}`;

export const createFindingsSource = (
  getFindings: (query: FindingsQuery) => Promise<FindingsPage>,
  { clock, maxPages, pageSize }: { readonly clock: Clock; readonly maxPages: number; readonly pageSize: number },
): FindingsSource => {
  // Least recently used first: a Map keeps the order its keys were set in.
  const pages = new Map<string, readonly Finding[]>();
  const totals = new Map<string, number>();
  const failures = new Map<string, Failure>();
  const loading = new Set<string>();
  // Pages asked again while a request of theirs was on its way: asked once more when it ends.
  const again = new Set<string>();
  const shown = new Map<string, Shown>();
  const listeners = new Set<() => void>();
  // A new scan starts a new generation: the answers of an older one are dropped.
  let generation = 0;
  let version = 0;
  let refreshedAt = Number.NEGATIVE_INFINITY;
  let cancelRefresh: (() => void) | null = null;

  const changed = (): void => {
    version += 1;
    for (const listener of listeners) {
      listener();
    }
  };

  const keep = (id: string, findings: readonly Finding[]): void => {
    pages.delete(id);
    pages.set(id, findings);

    for (const oldest of pages.keys()) {
      if (pages.size <= maxPages) {
        break;
      }
      pages.delete(oldest);
    }
  };

  // The page numbers the rows need, within the view's total when it is known.
  const pagesOf = (key: string, { first, last }: Range): readonly number[] => {
    const total = totals.get(key);
    const end = total === undefined ? last : Math.min(last, total - 1);

    if (first < 0 || first > end) {
      return [];
    }

    const from = Math.floor(first / pageSize);

    return Array.from({ length: Math.floor(end / pageSize) - from + 1 }, (_, at) => from + at);
  };

  const load = (view: FindingsView, key: string, page: number): void => {
    const id = pageKey(key, page);

    if (loading.has(id)) {
      return;
    }
    loading.add(id);
    const asked = generation;

    void getFindings({ ...view, limit: pageSize, offset: page * pageSize })
      .then(
        (answer) => {
          if (asked !== generation) {
            return;
          }
          keep(id, answer.findings);
          totals.set(key, answer.total);
          failures.delete(key);
          changed();
        },
        (error: unknown) => {
          // The rows the page had stay; the failure is told once, not once per failed request.
          if (asked !== generation || failures.has(key)) {
            return;
          }
          failures.set(key, failureOf(error));
          changed();
        },
      )
      .finally(() => {
        if (asked !== generation) {
          return;
        }
        loading.delete(id);
        if (again.delete(id)) {
          load(view, key, page);
        }
      });
  };

  const refresh = (): void => {
    cancelRefresh = null;
    refreshedAt = clock.now();

    for (const [key, range] of shown) {
      for (const page of pagesOf(key, range)) {
        const id = pageKey(key, page);

        if (loading.has(id)) {
          again.add(id);
        } else {
          load(range.view, key, page);
        }
      }
    }
  };

  const refreshSoon = (): void => {
    if (cancelRefresh !== null) {
      return;
    }

    const wait = refreshedAt + REFRESH_INTERVAL_MS - clock.now();

    if (wait <= 0) {
      refresh();
    } else {
      cancelRefresh = clock.setTimeout(refresh, wait);
    }
  };

  const reset = (): void => {
    generation += 1;
    pages.clear();
    totals.clear();
    failures.clear();
    loading.clear();
    again.clear();
    changed();
  };

  const rowAt = (view: FindingsView, index: number): Finding | undefined => {
    const page = Math.floor(index / pageSize);

    return pages.get(pageKey(keyOf(view), page))?.[index - page * pageSize];
  };

  return {
    ensure: (view, first, last): void => {
      const key = keyOf(view);
      const range = { first, last };
      shown.set(key, { ...range, view });

      for (const page of pagesOf(key, range)) {
        const id = pageKey(key, page);
        const cached = pages.get(id);

        if (cached === undefined) {
          load(view, key, page);
        } else {
          // Used again: the page moves to the end of the least recently used order.
          keep(id, cached);
        }
      }
    },
    failure: (view) => failures.get(keyOf(view)) ?? null,
    follow: (store): (() => void) => {
      let { overview, removed, scan } = store.getState();

      const unsubscribe = store.subscribe(() => {
        const state = store.getState();

        if (state.scan !== scan) {
          ({ overview, removed, scan } = state);
          reset();
        } else if (state.removed !== removed) {
          // A cleanup removed findings: every page after them moved, so the rows on screen are asked for at once.
          ({ overview, removed } = state);
          reset();
          refresh();
        } else if (state.overview !== overview) {
          ({ overview } = state);
          refreshSoon();
        }
      });

      return (): void => {
        unsubscribe();
        cancelRefresh?.();
        cancelRefresh = null;
      };
    },
    held: () => [...pages.values()].reduce((sum, page) => sum + page.length, 0),
    rowAt,
    subscribe: (listener): (() => void) => {
      listeners.add(listener);

      return (): void => {
        listeners.delete(listener);
      };
    },
    total: (view) => totals.get(keyOf(view)),
    version: () => version,
  };
};
