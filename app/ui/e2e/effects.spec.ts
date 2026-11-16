import { expect, type Page, test } from '@playwright/test';

import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// The light and the texture in the real engines, under the system's "Increase contrast" (the media query emulated).

const displays = async (page: Page): Promise<{ readonly light: string; readonly noise: string }> =>
  page.evaluate(() => {
    const light = document.querySelector('[data-ambient-light]');

    return {
      light: light === null ? 'missing' : getComputedStyle(light).display,
      noise: getComputedStyle(document.documentElement, '::after').display,
    };
  });

const openLight = async (page: Page): Promise<void> => {
  await page.goto('/gallery.html#/AmbientLight/rest');
  await expect(page.locator('[data-story="AmbientLight/rest"]')).toBeVisible();
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('the noise and the glows are off under increased contrast', async ({ page }) => {
    await page.emulateMedia({ contrast: 'more' });
    await openLight(page);

    expect(await displays(page)).toStrictEqual({ light: 'none', noise: 'none' });
  });
});

test.describe('positive cases', () => {
  test('the noise and the glows are drawn otherwise', async ({ page }) => {
    await page.emulateMedia({ contrast: 'no-preference' });
    await openLight(page);

    expect(await displays(page)).toStrictEqual({ light: 'block', noise: 'block' });
  });
});
