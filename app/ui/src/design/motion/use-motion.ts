import { useSyncExternalStore } from 'react';

// How much the interface moves: everything (`full`), fades only (`reduced`), nothing (`off`, for screenshots and tests).
export type MotionMode = 'full' | 'off' | 'reduced';

const REDUCED_QUERY = '(prefers-reduced-motion: reduce)';

const isMotionMode = (value: string | undefined): value is MotionMode =>
  value === 'full' || value === 'reduced' || value === 'off';

// The motion mode now: the `data-motion` attribute on <html> wins; otherwise the system's "Reduce motion" decides.
export const readMotion = (): MotionMode => {
  const attribute = document.documentElement.dataset.motion;

  if (isMotionMode(attribute)) {
    return attribute;
  }

  return window.matchMedia(REDUCED_QUERY).matches ? 'reduced' : 'full';
};

const subscribe = (onChange: () => void): (() => void) => {
  const query = window.matchMedia(REDUCED_QUERY);
  const observer = new MutationObserver(onChange);

  query.addEventListener('change', onChange);
  observer.observe(document.documentElement, { attributeFilter: ['data-motion'], attributes: true });

  return () => {
    query.removeEventListener('change', onChange);
    observer.disconnect();
  };
};

// The motion mode, live: a component renders again when the attribute or the system setting changes.
export const useMotion = (): MotionMode => useSyncExternalStore(subscribe, readMotion);
