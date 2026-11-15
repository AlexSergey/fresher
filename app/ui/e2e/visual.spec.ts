import { expect, test } from '@playwright/test';

import { DEMO_DICTIONARY, DEMO_SCAN, IDLE } from '../src/test/demo-scan';
import { mockCore } from './core-mock';

const answers = { appVersion: '0.0.1', getDictionary: DEMO_DICTIONARY, getOverview: IDLE, startScan: 1 } as const;

test.describe('positive cases', () => {
  test('the start screen looks the same', async ({ page }) => {
    await mockCore(page, { answers });
    await page.goto('/');
    await expect(page.getByText('Version 0.0.1')).toBeVisible();

    await expect(page).toHaveScreenshot('start-screen.png');
  });

  test('a finished scan looks the same', async ({ page }) => {
    await mockCore(page, { answers, scan: DEMO_SCAN });
    await page.goto('/');
    await page.getByRole('button', { name: 'Scan' }).click();
    await expect(page.getByText('The scan is complete.')).toBeVisible();

    await expect(page).toHaveScreenshot('finished-scan.png');
  });
});
