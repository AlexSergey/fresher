import { type RefObject, useEffect, useEffectEvent, useRef, useState } from 'react';

import { DURATION, EASING } from '../tokens/motion';
import { useMotion } from './use-motion';

export type Presence<E extends Element> = {
  // Whether the element is rendered: while open, and while its exit plays.
  readonly mounted: boolean;
  readonly ref: RefObject<E | null>;
};

export type PresenceOptions = {
  // Milliseconds.
  readonly duration: number;
  readonly exit: readonly Keyframe[];
};

// Under reduced motion an exit only fades.
const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

// Keeps an element mounted until its exit animation has finished (there is no `@starting-style` in the baseline):
// render it while `mounted`, with `ref` on it. Motion `off` unmounts at once; `reduced` fades out.
export const usePresence = <E extends Element>(open: boolean, options: PresenceOptions): Presence<E> => {
  const motion = useMotion();
  const ref = useRef<E>(null);
  const [leaving, setLeaving] = useState(false);
  const [wasOpen, setWasOpen] = useState(open);

  // Closing starts the exit in the same render, so the element never disappears for a frame.
  if (wasOpen !== open) {
    setWasOpen(open);
    setLeaving(!open);
  }

  const playExit = useEffectEvent((element: E): Animation => {
    const reduced = motion === 'reduced';

    return element.animate([...(reduced ? FADE_OUT : options.exit)], {
      duration: reduced ? Math.min(options.duration, DURATION.fast) : options.duration,
      easing: EASING.exit,
      fill: 'forwards',
    });
  });

  useEffect(() => {
    const element = ref.current;

    if (!leaving) {
      return;
    }

    // Nothing to play: the element leaves at once.
    if (motion === 'off' || element === null) {
      setLeaving(false);

      return;
    }

    let current = true;
    const animation = playExit(element);

    animation.finished.then(
      () => {
        if (current) {
          setLeaving(false);
        }
      },
      // Cancelled: the element was opened again, or is gone.
      () => undefined,
    );

    return (): void => {
      current = false;
      animation.cancel();
    };
  }, [leaving, motion]);

  return { mounted: open || (leaving && motion !== 'off'), ref };
};
