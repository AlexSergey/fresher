import { expect, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// The modal dialog and the sidebar in the real engines: what a unit test's DOM cannot do (the inert page, the focus
// trap, the scroll of a short window).

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

const NAV_ITEMS = ['Scan', 'Largest', 'Apps', 'Review', 'Cleanups', 'Settings'] as const;

test.describe('negative cases', () => {
  test("at the window's minimum, 900 × 600, no sidebar item is hidden", async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.setViewportSize({ height: 600, width: 900 });
    await page.goto('/');
    const sidebar = page.getByRole('navigation', { name: 'Screens' });

    for (const name of NAV_ITEMS) {
      await expect(sidebar.getByRole('button', { name: new RegExp(`^${name}`, 'u') })).toBeInViewport({ ratio: 1 });
    }
  });

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

  test('in a short frame the sidebar scrolls and Settings can be reached and opened by keyboard', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.setViewportSize({ height: 420, width: 900 });
    await page.goto('/');
    const sidebar = page.getByRole('navigation', { name: 'Screens' });
    const settings = sidebar.getByRole('button', { name: 'Settings' });

    await sidebar.getByRole('button', { exact: true, name: 'Scan' }).focus();
    for (let press = 0; press < NAV_ITEMS.length - 1; press += 1) {
      await page.keyboard.press('Tab');
    }

    await expect(settings).toBeFocused();
    await expect(settings).toBeInViewport({ ratio: 1 });

    await page.keyboard.press('Enter');

    await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeVisible();
  });
});
