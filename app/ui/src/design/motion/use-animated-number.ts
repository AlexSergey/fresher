import { useEffect, useRef, useState } from 'react';

import { DURATION } from '../tokens/motion';
import { useMotion } from './use-motion';

export type AnimatedNumberOptions = {
  // Milliseconds.
  readonly duration?: number;
};

const easeOutCubic = (progress: number): number => 1 - (1 - progress) ** 3;

// The value to show while a number moves toward `target`: a tween on animation frames that starts from the value on
// screen, so a new target mid-way never jumps back. Under reduced or no motion, the target at once.
export const useAnimatedNumber = (target: number, { duration = DURATION.slow }: AnimatedNumberOptions = {}): number => {
  const motion = useMotion();
  const [shown, setShown] = useState(target);
  // The value on screen, read when a new tween starts.
  const onScreenRef = useRef(target);

  useEffect(() => {
    const from = onScreenRef.current;

    // `shown` and `onScreenRef` change together, only here: back in full motion, a number moves on from where it was.
    if (motion !== 'full' || from === target) {
      return;
    }

    let frame = 0;
    let start: null | number = null;

    const step = (now: number): void => {
      start ??= now;
      const progress = Math.min((now - start) / duration, 1);
      const value = progress === 1 ? target : from + (target - from) * easeOutCubic(progress);

      onScreenRef.current = value;
      setShown(value);

      if (progress < 1) {
        frame = requestAnimationFrame(step);
      }
    };

    frame = requestAnimationFrame(step);

    return (): void => {
      cancelAnimationFrame(frame);
    };
  }, [target, duration, motion]);

  return motion === 'full' ? shown : target;
};
