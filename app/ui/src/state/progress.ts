import type { Clock } from './clock';
import type { CoreAction } from './reduce';

type Progress = Extract<CoreAction, { readonly type: 'scan-progress' }>;

// Lets `scan-progress` through at most once per `interval` (10 a second at 100 ms), so React renders no more than
// that during a scan. A dropped progress is kept and sent when the interval ends, so the last one always arrives;
// it is sent at once when the scan finishes or another starts. Every other action passes unchanged.
export const throttleProgress = (
  dispatch: (action: CoreAction) => void,
  { clock, interval }: { readonly clock: Clock; readonly interval: number },
): ((action: CoreAction) => void) => {
  let sentAt = Number.NEGATIVE_INFINITY;
  let held: null | Progress = null;
  let cancelTimer: (() => void) | null = null;

  const flush = (): void => {
    cancelTimer?.();
    cancelTimer = null;

    if (held !== null) {
      const progress = held;
      held = null;
      sentAt = clock.now();
      dispatch(progress);
    }
  };

  return (action): void => {
    if (action.type !== 'scan-progress') {
      if (action.type === 'scan-finished' || action.type === 'scan-started') {
        flush();
      }
      dispatch(action);

      return;
    }

    const now = clock.now();

    if (cancelTimer === null && now - sentAt >= interval) {
      sentAt = now;
      dispatch(action);

      return;
    }

    held = action;
    cancelTimer ??= clock.setTimeout(flush, sentAt + interval - now);
  };
};
