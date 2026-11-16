import { type RefObject, useEffect } from 'react';

import { useMotion } from './use-motion';

// Writes the pointer's place over the element into `--spot-x` and `--spot-y` (pixels from its top left corner), at
// most once per frame however often the pointer moves. Nothing under motion `off`.
export const useSpotlight = (ref: RefObject<HTMLElement | null>): void => {
  const motion = useMotion();

  useEffect(() => {
    const element = ref.current;

    if (motion === 'off' || element === null) {
      return;
    }

    let frame: null | number = null;
    let pointer = { x: 0, y: 0 };

    const write = (): void => {
      frame = null;
      const box = element.getBoundingClientRect();

      element.style.setProperty('--spot-x', `${pointer.x - box.left}px`);
      element.style.setProperty('--spot-y', `${pointer.y - box.top}px`);
    };

    const onMove = (event: PointerEvent): void => {
      pointer = { x: event.clientX, y: event.clientY };
      frame ??= requestAnimationFrame(write);
    };

    element.addEventListener('pointermove', onMove);

    return (): void => {
      element.removeEventListener('pointermove', onMove);

      if (frame !== null) {
        cancelAnimationFrame(frame);
      }
    };
  }, [ref, motion]);
};
