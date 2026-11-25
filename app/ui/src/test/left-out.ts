import type { LeftOutCount, LeftOutEntry, Sizes, Text } from '../ipc/bindings.generated';

// The left-outs of the scan fixtures, as the engine counts them: shared by the unit tests and the e2e mock.

// A count kept for good (the version you use, a socket, the system's file): the engine lists none of its items.
export const keptCount = (category: string, key: string, items: number, sizes: Sizes): LeftOutCount => ({
  category,
  items,
  kind: 'kept',
  reason: { args: {}, key },
  shown: [],
  sizes,
});

// A count held back for now with the items the engine lists, the largest first; `items` counts the ones it did not
// list too (more than `LEFT_OUT_SHOWN`).
export const forNowCount = (
  category: string,
  key: string,
  sizes: Sizes,
  shown: readonly LeftOutEntry[],
  items: number = shown.length,
): LeftOutCount => ({ category, items, kind: 'for-now', reason: { args: {}, key }, shown: [...shown], sizes });

// One listed item of a count held back for now: its title as its finding would carry it, its path and its size.
export const leftOutEntry = (title: Text, path: string, sizes: Sizes): LeftOutEntry => ({ path, sizes, title });
