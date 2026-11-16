import { useEffect, useMemo, useRef, useState } from 'react';

import { DURATION } from '../../design/tokens/motion';

const sameOrder = (first: readonly string[], second: readonly string[]): boolean =>
  first.length === second.length && first.every((key, index) => second[index] === key);

// Keys that never hold a line break (category ids), joined into one value React can compare.
const SEPARATOR = '\n';

const split = (joined: string): readonly string[] => (joined === '' ? [] : joined.split(SEPARATOR));

// The order to show `keys` in. At rest it is `keys`. While `coalescing` (a scan), a new order is shown at most once
// per --duration-move, so cards never chase each other: in between, the cards keep their places, a new key joins at
// the end and a gone key leaves.
export const useCoalescedOrder = (keys: readonly string[], coalescing: boolean): readonly string[] => {
  const wanted = keys.join(SEPARATOR);
  const [shown, setShown] = useState(wanted);
  const shownAtRef = useRef(Number.NEGATIVE_INFINITY);

  if (!coalescing && shown !== wanted) {
    setShown(wanted);
  }

  const kept = split(shown).filter((key) => keys.includes(key));
  const placed = coalescing ? [...kept, ...keys.filter((key) => !kept.includes(key))] : keys;
  const current = placed.join(SEPARATOR);
  const waiting = coalescing && !sameOrder(placed, keys);

  useEffect(() => {
    if (!waiting) {
      return;
    }

    const show = (): void => {
      shownAtRef.current = Date.now();
      setShown(wanted);
    };
    const wait = shownAtRef.current + DURATION.move - Date.now();

    if (wait <= 0) {
      show();

      return;
    }

    const timer = setTimeout(show, wait);

    return (): void => {
      clearTimeout(timer);
    };
  }, [waiting, wanted]);

  return useMemo(() => split(current), [current]);
};
