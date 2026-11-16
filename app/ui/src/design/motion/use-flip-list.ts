import { useCallback, useLayoutEffect, useRef } from 'react';

import { DURATION, EASING } from '../tokens/motion';
import { useMotion } from './use-motion';

export type FlipListOptions = {
  // Each key's position, when the caller knows it (a virtual list places rows by index); otherwise each element is
  // measured against its parent.
  readonly positions?: ReadonlyMap<string, Position>;
};

// Gives each keyed child its ref: `<li ref={flip(key)}>`.
export type FlipRef = (key: string) => (element: HTMLElement | null) => void;

export type Position = { readonly x: number; readonly y: number };

// One batched read of every element's place, relative to its parent: a scrolled page moves nothing.
const measure = (elements: ReadonlyMap<string, HTMLElement>): ReadonlyMap<string, Position> => {
  const positions = new Map<string, Position>();

  for (const [key, element] of elements) {
    const box = element.getBoundingClientRect();
    const parent = element.parentElement?.getBoundingClientRect();

    positions.set(key, { x: box.left - (parent?.left ?? 0), y: box.top - (parent?.top ?? 0) });
  }

  return positions;
};

const sameOrder = (before: readonly string[], after: readonly string[]): boolean =>
  before.length === after.length && before.every((key, index) => after[index] === key);

// FLIP for keyed children: after a reorder, each moved element starts where it was and glides to its new place
// (`translate(dx, dy)` → none, --duration-move, --ease-standard). A new key is left to its entrance animation.
export const useFlipList = (keys: readonly string[], { positions }: FlipListOptions = {}): FlipRef => {
  const motion = useMotion();
  const elementsRef = useRef(new Map<string, HTMLElement>());
  const callbacksRef = useRef(new Map<string, (element: HTMLElement | null) => void>());
  const runningRef = useRef(new Map<string, Animation>());
  const lastRef = useRef<{ readonly keys: readonly string[]; readonly positions: ReadonlyMap<string, Position> }>({
    keys: [],
    positions: new Map(),
  });

  useLayoutEffect(() => {
    const before = lastRef.current;

    // The same order: nothing moves. The places are kept fresh, unless a glide runs and would be measured mid-way.
    if (sameOrder(before.keys, keys)) {
      if (runningRef.current.size === 0) {
        lastRef.current = { keys, positions: positions ?? measure(elementsRef.current) };
      }

      return;
    }

    // A reorder during a glide: the glide ends first, so its element is measured where it really is.
    for (const animation of runningRef.current.values()) {
      animation.cancel();
    }

    runningRef.current.clear();

    const now = positions ?? measure(elementsRef.current);

    lastRef.current = { keys, positions: now };

    if (motion !== 'full') {
      return;
    }

    for (const [key, element] of elementsRef.current) {
      const from = before.positions.get(key);
      const to = now.get(key);

      if (from === undefined || to === undefined || (from.x === to.x && from.y === to.y)) {
        continue;
      }

      element.style.willChange = 'transform';
      const animation = element.animate(
        [{ transform: `translate(${from.x - to.x}px, ${from.y - to.y}px)` }, { transform: 'none' }],
        { duration: DURATION.move, easing: EASING.standard },
      );
      const done = (): void => {
        element.style.willChange = '';

        if (runningRef.current.get(key) === animation) {
          runningRef.current.delete(key);
        }
      };

      runningRef.current.set(key, animation);

      animation.finished.then(done, done);
    }
  }, [keys, positions, motion]);

  return useCallback((key: string) => {
    const known = callbacksRef.current.get(key);

    if (known !== undefined) {
      return known;
    }

    const ref = (element: HTMLElement | null): void => {
      if (element === null) {
        elementsRef.current.delete(key);
        callbacksRef.current.delete(key);
      } else {
        elementsRef.current.set(key, element);
      }
    };

    callbacksRef.current.set(key, ref);

    return ref;
  }, []);
};
