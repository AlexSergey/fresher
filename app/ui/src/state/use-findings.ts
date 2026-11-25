import { createContext, use, useCallback, useEffect, useSyncExternalStore } from 'react';

import type { Failure } from '../ipc';
import type { Finding } from '../ipc/bindings.generated';
import type { FindingsSource, FindingsView } from './findings';

// The rows of one finding list, made for `VirtualList`'s props: how many rows, the row at an index (`undefined`
// while its page loads), what to call when the rows on screen change, and why the rows could not be loaded.
export type FindingRows = {
  readonly failure: Failure | null;
  readonly onRangeChange: (first: number, last: number) => void;
  readonly rowAt: (index: number) => Finding | undefined;
  // How many findings the view has; `undefined` until the core answered its first page.
  readonly total: number | undefined;
};

export const FindingsContext = createContext<FindingsSource | null>(null);

// The rows of the findings of `view`; the component renders again when a page of any list arrives. The rows it shows
// stay in the source's cache until the component unmounts.
export const useFindings = (view: FindingsView): FindingRows => {
  const source = use(FindingsContext);

  if (source === null) {
    throw new Error('useFindings needs a FindingsContext around it');
  }

  // Every answer of the source renders again, and the rows are read anew: `rowAt` is a new function each time.
  useSyncExternalStore(source.subscribe, source.version);
  const { category, module, order } = view;
  // While the list is mounted its rows on screen stay in the cache; once it is gone they are dropped like any other.
  useEffect(() => source.hold({ category, module, order }), [source, category, module, order]);
  const onRangeChange = useCallback(
    (first: number, last: number): void => {
      source.ensure({ category, module, order }, first, last);
    },
    [source, category, module, order],
  );
  const shown = { category, module, order };

  return {
    failure: source.failure(shown),
    onRangeChange,
    rowAt: (index: number): Finding | undefined => source.rowAt(shown, index),
    total: source.total(shown),
  };
};
