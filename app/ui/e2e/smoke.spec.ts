import { expect, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('an unreachable core shows its error state', async ({ page }) => {
    await mockCore(page);
    await page.goto('/');

    await expect(page.getByText('The app could not reach its core. Restart the app.')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();
  });

  test('a refused scan shows a toast with the core text', async ({ page }) => {
    await mockCore(page, SCRIPTS.refused);
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();

    await expect(page.getByRole('region', { name: 'Notifications' })).toContainText(
      'The app is already running a scan. Wait for it to finish, or cancel it.',
    );
  });
});

test.describe('positive cases', () => {
  test('opens on the overview with the modules in the sidebar', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
    await expect(page.getByRole('button', { name: /^Demo module/ })).toContainText('—');
  });

  test("streams a scan into the module's size", async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();

    await expect(page.getByRole('button', { name: /^Demo module/ })).toContainText('2.6 GB');
  });

  test('a nav item shows its screen and moves the focus to its heading', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await page.getByRole('button', { name: 'Settings' }).click();

    await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeFocused();
    await expect(page.getByText('Version 0.0.1')).toBeVisible();
  });
});
