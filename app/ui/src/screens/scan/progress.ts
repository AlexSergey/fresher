import type { ByteSize, ModuleId } from '../../ipc/bindings.generated';

type Modules = { readonly finished: number; readonly started: number };

// How far the scan is, 0 to 99 (100 is the scan's end, which the screen shows by leaving the card). With the core's
// estimate of what the scan will walk: the bytes walked so far against it; the walk can pass it, so it holds at 99.
// Without one: the modules finished of those started. Display math over the core's numbers, nothing it decides.
export const scanPercent = (bytes: ByteSize, expected: ByteSize | null, modules: Modules): number => {
  if (expected !== null && expected > 0) {
    return Math.min(Math.floor((100 * bytes) / expected), 99);
  }

  return modules.started > 0 ? Math.floor((99 * modules.finished) / modules.started) : 0;
};

// The modules the scan still runs: those started that have not finished, in the core's order.
export const stillRunning = (started: readonly ModuleId[], finished: readonly ModuleId[]): readonly ModuleId[] =>
  started.filter((module) => !finished.includes(module));

// The module the scan looks at on its `turn`: the modules still running take turns in the core's order, from the
// first again after the last; `null` when none runs.
export const lookingAt = (running: readonly ModuleId[], turn: number): ModuleId | null =>
  running.length === 0 ? null : (running[turn % running.length] ?? null);
