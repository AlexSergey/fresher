import type { Clock } from '../state/clock';

export type FakeClock = Clock & {
  // Moves the time forward, running every timer that falls due on the way, in the order they fall due.
  readonly advance: (ms: number) => void;
  // How many timers wait to run.
  readonly waiting: () => number;
};

type Timer = { readonly at: number; readonly order: number; readonly run: () => void };

// A clock for tests: it starts at 0 and moves only when `advance` is called.
export const createFakeClock = (): FakeClock => {
  let now = 0;
  let order = 0;
  const timers = new Set<Timer>();

  const next = (until: number): Timer | undefined =>
    [...timers]
      .filter((timer) => timer.at <= until)
      .sort((a, b) => a.at - b.at || a.order - b.order)
      .at(0);

  return {
    advance: (ms): void => {
      const until = now + ms;

      for (let timer = next(until); timer !== undefined; timer = next(until)) {
        timers.delete(timer);
        now = timer.at;
        timer.run();
      }
      now = until;
    },
    now: () => now,
    setTimeout: (run, ms) => {
      const timer = { at: now + Math.max(ms, 0), order, run };
      order += 1;
      timers.add(timer);

      return (): void => {
        timers.delete(timer);
      };
    },
    waiting: () => timers.size,
  };
};
