import { createContext, use } from 'react';

import { type Clock, systemClock } from './clock';

// The time of day, for what says how long ago something happened: milliseconds since the epoch, with the timers of a
// `Clock`. Injected like the store's clock, so a test fixes it.
export type WallClock = Clock & { readonly epochMs: () => number };

export const systemWallClock: WallClock = { ...systemClock, epochMs: () => Date.now() };

export const WallClockContext = createContext<WallClock>(systemWallClock);

export const useWallClock = (): WallClock => use(WallClockContext);
