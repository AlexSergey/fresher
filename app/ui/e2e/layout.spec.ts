import { expect, type Page, test } from '@playwright/test';

import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// The modal dialog in the real engines: what a unit test's DOM cannot do (the inert page, the focus trap).

const openStory = async (page: Page, id: string): Promise<void> => {
  await page.goto(`/gallery.html#/${id}`);
  await expect(page.locator(`[data-story="${id}"]`)).toBeVisible();
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('the page behind is inert while the dialog is open', async ({ page }) => {
    await openStory(page, 'Dialog/short-body');
    const behind = page.getByRole('button', { includeHidden: true, name: 'Open the Trash' });

    for (let press = 0; press < 6; press += 1) {
      await page.keyboard.press('Tab');
      await expect(behind).not.toBeFocused();
    }

    expect(await behind.evaluate((element) => element.matches(':focus'))).toBe(false);
  });
});

test.describe('positive cases', () => {
  test('the keyboard moves through the dialog from its first control', async ({ page }) => {
    await openStory(page, 'Dialog/short-body');
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });

    await expect(dialog.getByRole('button', { name: 'Close' })).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog.getByRole('button', { name: 'Move to Trash' })).toBeFocused();
  });
});
