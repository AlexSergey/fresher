import type { Page } from '@playwright/test';

// Sets `data-motion="off"` on <html> before the app runs: every duration is 0 and every loop stands at its first frame,
// so a screenshot is the same on every run. Every visual spec calls it first.
export const motionOff = async (page: Page): Promise<void> => {
  await page.addInitScript(() => {
    // The init script may run before the parser has created <html>.
    const root = document.documentElement as HTMLElement | null;

    if (root !== null) {
      root.dataset.motion = 'off';

      return;
    }

    const observer = new MutationObserver(() => {
      const created = document.documentElement as HTMLElement | null;

      if (created !== null) {
        created.dataset.motion = 'off';
        observer.disconnect();
      }
    });

    observer.observe(document, { childList: true });
  });
};
