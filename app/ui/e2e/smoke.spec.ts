import { expect, test } from '@playwright/test';

import { mockCore } from './core-mock';

test.describe('negative cases', () => {
  test('waits for the core when it does not answer', async ({ page }) => {
    await mockCore(page, {});
    await page.goto('/');

    await expect(page.getByText('Connecting to the core…')).toBeVisible();
  });
});

test.describe('positive cases', () => {
  test('shows the version from the core', async ({ page }) => {
    await mockCore(page, { appVersion: '0.0.1' });
    await page.goto('/');

    await expect(page.getByRole('heading', { name: 'Mac Fresher' })).toBeVisible();
    await expect(page.getByText('Version 0.0.1')).toBeVisible();
  });
});
