import { vi } from 'vitest';

// `ResizeObserver` as the browser calls it: once on observe, with the given height. A virtual list measures its
// viewport with it, so a test decides how many rows are in view.
export const fakeViewport = (height: number): void => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      private readonly onResize: ResizeObserverCallback;

      constructor(onResize: ResizeObserverCallback) {
        this.onResize = onResize;
      }

      disconnect(): void {
        // Nothing to stop.
      }

      observe(): void {
        this.onResize([{ contentRect: { height } } as ResizeObserverEntry], this as unknown as ResizeObserver);
      }
    },
  );
};
