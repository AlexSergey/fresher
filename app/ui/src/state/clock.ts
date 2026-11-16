// Time as the store's timers see it: injected, so tests run on a fake clock and never on the wall clock.
export type Clock = {
  // Milliseconds from an arbitrary origin; only differences between two readings mean anything.
  readonly now: () => number;
  // Runs `run` once after `ms` milliseconds; the returned function cancels it.
  readonly setTimeout: (run: () => void, ms: number) => () => void;
};

export const systemClock: Clock = {
  now: () => performance.now(),
  setTimeout: (run, ms) => {
    const timer = globalThis.setTimeout(run, ms);

    return (): void => {
      globalThis.clearTimeout(timer);
    };
  },
};
