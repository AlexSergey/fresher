import type { Page } from '@playwright/test';

// The frame sampler of the performance spec: every `requestAnimationFrame` interval of the page, the long tasks where
// the engine reports them (Chromium) and the most virtual-list rows seen in the DOM.

export type Sample = {
  // Milliseconds between two frames, in order.
  readonly intervals: readonly number[];
  // Milliseconds of each long task; `null` where the engine reports none.
  readonly longTasks: null | readonly number[];
  // Every element of the document at the end.
  readonly nodes: number;
  // The most rows of virtual lists (`role="listitem"`) in the DOM at any frame.
  readonly rows: number;
};

type Sampled = {
  __frames__: number[];
  __longTasks__: null | number[];
  __rows__: number;
};

// Starts the sampler before the page's own scripts: it runs on every frame from the first one on.
export const sampleFrames = async (page: Page): Promise<void> => {
  await page.addInitScript(() => {
    const sampled = window as unknown as Sampled;
    let last: null | number = null;

    sampled.__frames__ = [];
    sampled.__rows__ = 0;
    sampled.__longTasks__ = null;
    if (PerformanceObserver.supportedEntryTypes.includes('longtask')) {
      sampled.__longTasks__ = [];
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          sampled.__longTasks__?.push(entry.duration);
        }
      }).observe({ type: 'longtask' });
    }

    const tick = (now: number): void => {
      if (last !== null) {
        sampled.__frames__.push(now - last);
      }
      last = now;
      sampled.__rows__ = Math.max(sampled.__rows__, document.querySelectorAll('[role="listitem"]').length);
      requestAnimationFrame(tick);
    };

    requestAnimationFrame(tick);
  });
};

// Forgets what was sampled so far: the measurement starts here.
export const beginSample = async (page: Page): Promise<void> => {
  await page.evaluate(() => {
    const sampled = window as unknown as Sampled;

    sampled.__frames__ = [];
    sampled.__rows__ = document.querySelectorAll('[role="listitem"]').length;
    if (sampled.__longTasks__ !== null) {
      sampled.__longTasks__ = [];
    }
  });
};

// What was sampled since `beginSample`.
export const endSample = (page: Page): Promise<Sample> =>
  page.evaluate(() => {
    const sampled = window as unknown as Sampled;

    return {
      intervals: [...sampled.__frames__],
      longTasks: sampled.__longTasks__ === null ? null : [...sampled.__longTasks__],
      nodes: document.querySelectorAll('*').length,
      rows: sampled.__rows__,
    };
  });

// Waits until no finite animation or transition runs (the endless loops of a scan are left running), and at least
// `min` milliseconds: an animation a change starts a frame later is still inside the measured window.
export const settle = (page: Page, min = 800): Promise<void> =>
  page.evaluate(
    (least) =>
      new Promise<void>((resolve) => {
        const start = performance.now();
        const check = (now: number): void => {
          const moving = document
            .getAnimations()
            .some(
              (animation) =>
                animation.playState === 'running' && animation.effect?.getComputedTiming().iterations !== Infinity,
            );

          if (now - start >= least && !moving) {
            resolve();
          } else {
            requestAnimationFrame(check);
          }
        };

        requestAnimationFrame(check);
      }),
    min,
  );

// Waits two frames: what a resize observer changes after the first layout (a long dialog body that becomes focusable
// once the dialog is open) is on the page when this returns.
export const nextFrames = (page: Page): Promise<void> =>
  page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            resolve();
          });
        });
      }),
  );
