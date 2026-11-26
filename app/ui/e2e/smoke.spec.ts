import { expect, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { navScan, scanButton } from './support/cards';
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

    await expect(page.getByText('Fresher stopped answering. Quit it and open it again.')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();
  });

  test('a refused scan shows a toast with the core text', async ({ page }) => {
    await mockCore(page, SCRIPTS.refused);
    await page.goto('/');

    await scanButton(page).click();

    await expect(page.getByRole('region', { name: 'Notifications' })).toContainText(
      'The app is already running a scan. Wait for it to finish, or cancel it.',
    );
  });
});

test.describe('positive cases', () => {
  test('opens on Scan, current in the sidebar', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await expect(page.getByRole('heading', { level: 1, name: 'Scan' })).toBeVisible();
    await expect(navScan(page)).toHaveAttribute('aria-current', 'page');
  });

  test('streams a scan into its tiles', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await scanButton(page).click();

    await expect(page.getByRole('button', { exact: true, name: 'Scan again' })).toBeVisible();
    await expect(page.locator('[data-card]')).toHaveCount(2);
  });

  test('a nav item shows its screen and moves the focus to its heading', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await page.getByRole('button', { name: 'Settings' }).click();

    await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeFocused();
    await page.getByRole('button', { exact: true, name: 'About' }).click();
    await expect(page.getByText('Version 0.0.1')).toBeVisible();
  });
});
