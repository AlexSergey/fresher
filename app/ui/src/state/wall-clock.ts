import { createContext, use } from 'react';

import type { AppInfo } from '../ipc/bindings.generated';

import { type Clock, systemClock } from './clock';

// The time of day, for what says how long ago something happened: milliseconds since the epoch, with the timers of a
// `Clock`. Injected like the store's clock, so a test fixes it.
export type WallClock = Clock & { readonly epochMs: () => number };

export const systemWallClock: WallClock = { ...systemClock, epochMs: () => Date.now() };

// The time of day the core counts on, from its answer `info` read when the page's clock said `pageMs`: a sandbox's
// core counts on its world's clock, which may be days apart from the Mac's, so the page's clock is moved by the same
// amount; every other build's core counts on the Mac's own clock, the page's.
export const wallClockOf = (info: Pick<AppInfo, 'build' | 'now'>, pageMs: number): WallClock => {
  if (info.build.kind !== 'sandbox') {
    return systemWallClock;
  }
  const offset = info.now * 1000 - pageMs;

  return { ...systemClock, epochMs: () => Date.now() + offset };
};

export const WallClockContext = createContext<WallClock>(systemWallClock);

export const useWallClock = (): WallClock => use(WallClockContext);
