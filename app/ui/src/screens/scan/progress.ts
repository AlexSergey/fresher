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

// The module the scan looks at: the first started one that has not finished, `null` when there is none.
export const lookingAt = (started: readonly ModuleId[], finished: readonly ModuleId[]): ModuleId | null =>
  started.find((module) => !finished.includes(module)) ?? null;
