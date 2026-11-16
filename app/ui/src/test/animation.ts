import { act } from '@testing-library/react';
import { vi } from 'vitest';

// Test helpers for the motion hooks: frames on the fake clock, a recording Web Animations API, the motion mode.

// `requestAnimationFrame` and `performance.now()` on Vitest's fake clock: a frame runs every 16 ms of fake time.
export const fakeFrames = (): { readonly advance: (ms: number) => void } => {
  vi.useFakeTimers({ toFake: ['requestAnimationFrame', 'cancelAnimationFrame', 'performance', 'Date'] });

  return {
    advance: (ms: number): void => {
      act(() => {
        vi.advanceTimersByTime(ms);
      });
    },
  };
};

export type RecordedAnimation = {
  readonly cancelled: () => boolean;
  readonly element: Element;
  // Resolves `finished`, as the end of the animation would (a cancelled one stays cancelled).
  readonly finish: () => Promise<void>;
  readonly keyframes: readonly Keyframe[];
  readonly options: KeyframeAnimationOptions;
};

// Replaces `Element.prototype.animate` with a recorder: each call is kept, its `finished` resolves on demand and
// rejects on `cancel()`, as in a browser. Restored by `restoreMocks` after each test.
export const recordAnimations = (): readonly RecordedAnimation[] => {
  const recorded: RecordedAnimation[] = [];

  vi.spyOn(Element.prototype, 'animate').mockImplementation(function (
    this: Element,
    keyframes: Keyframe[] | null | PropertyIndexedKeyframes,
    options?: KeyframeAnimationOptions | number,
  ): Animation {
    let resolve: (value: Animation) => void = () => undefined;
    let reject: (reason: unknown) => void = () => undefined;
    const finished = new Promise<Animation>((onResolve, onReject) => {
      resolve = onResolve;
      reject = onReject;
    });
    let cancelled = false;
    const animation = {
      cancel: (): void => {
        cancelled = true;
        reject(new DOMException('The animation was cancelled.', 'AbortError'));
      },
      finished,
    } as unknown as Animation;
    // A rejection nobody waits for is not an error of the test.
    finished.catch(() => undefined);

    recorded.push({
      cancelled: () => cancelled,
      element: this,
      finish: async (): Promise<void> => {
        await act(async () => {
          resolve(animation);
          await finished.catch(() => undefined);
        });
      },
      keyframes: Array.isArray(keyframes) ? keyframes : [],
      options: typeof options === 'number' ? { duration: options } : (options ?? {}),
    });

    return animation;
  });

  return recorded;
};

// Sets the motion mode on <html>, as the app or Playwright would; `null` removes it. Awaited: the hooks hear of it
// through a MutationObserver, whose records are delivered in a microtask.
export const setMotion = async (mode: null | string): Promise<void> => {
  await act(async () => {
    if (mode === null) {
      delete document.documentElement.dataset.motion;
    } else {
      document.documentElement.dataset.motion = mode;
    }

    await Promise.resolve();
  });
};

// The system's "Reduce motion", as `matchMedia` reports it; `change(matches)` flips it as the system would, and
// `listeners()` counts who listens.
export const fakeReducedMotion = (
  initial: boolean,
): { readonly change: (matches: boolean) => void; readonly listeners: () => number } => {
  let matches = initial;
  const listeners = new Set<() => void>();
  const query = {
    addEventListener: (_type: string, listener: () => void): void => {
      listeners.add(listener);
    },
    get matches(): boolean {
      return matches;
    },
    removeEventListener: (_type: string, listener: () => void): void => {
      listeners.delete(listener);
    },
  } as unknown as MediaQueryList;

  vi.spyOn(window, 'matchMedia').mockReturnValue(query);

  return {
    change: (next: boolean): void => {
      act(() => {
        matches = next;
        for (const listener of listeners) {
          listener();
        }
      });
    },
    listeners: () => listeners.size,
  };
};
