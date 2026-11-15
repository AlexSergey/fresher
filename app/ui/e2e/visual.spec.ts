import { expect, test } from '@playwright/test';

import { mockCore } from './core-mock';

test.describe('positive cases', () => {
  test('the start screen looks the same', async ({ page }) => {
    await mockCore(page, { appVersion: '0.0.1' });
    await page.goto('/');
    await expect(page.getByText('Version 0.0.1')).toBeVisible();

    await expect(page).toHaveScreenshot('start-screen.png');
  });
});
