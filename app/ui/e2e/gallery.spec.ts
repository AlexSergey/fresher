import { expect, test } from '@playwright/test';

import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

type Listed = { readonly __stories__?: readonly string[] };

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.describe('positive cases', () => {
  test('every story of the gallery looks the same', async ({ page }) => {
    // Well over a hundred stories, each photographed: longer than one test's default, under a full parallel run.
    test.setTimeout(120_000);
    await page.goto('/gallery.html');
    const ids = await page.evaluate(() => (window as Listed).__stories__ ?? []);

    expect(ids.length).toBeGreaterThan(0);

    for (const id of ids) {
      await test.step(id, async () => {
        const [component = '', state = ''] = id.split('/');
        // A new document for each story, so its policy check sees only its own page.
        await page.goto('about:blank');
        await page.goto(`/gallery.html#/${encodeURIComponent(component)}/${encodeURIComponent(state)}`);
        const story = page.locator(`[data-story="${id}"]`);

        await expect(story).toBeVisible();
        await expectNoCspViolation(page);
        await expect.soft(story).toHaveScreenshot([component, `${state}.png`]);
      });
    }
  });
});
