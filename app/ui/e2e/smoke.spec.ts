import { expect, test } from '@playwright/test';

import { BUSY, DEMO_DICTIONARY, DEMO_SCAN, IDLE } from '../src/test/demo-scan';
import { mockCore } from './core-mock';

const answers = { appVersion: '0.0.1', getDictionary: DEMO_DICTIONARY, getOverview: IDLE, startScan: 1 } as const;

test.describe('negative cases', () => {
  test('waits for the core when it does not answer', async ({ page }) => {
    await mockCore(page);
    await page.goto('/');

    await expect(page.getByText('Connecting to the core…')).toBeVisible();
  });

  test('shows the refusal when a scan is already running', async ({ page }) => {
    await mockCore(page, { answers, refusals: { startScan: BUSY } });
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();

    await expect(page.getByRole('alert')).toHaveText(
      'The app is already running a scan. Wait for it to finish, or cancel it.',
    );
  });
});

test.describe('positive cases', () => {
  test('shows the version from the core', async ({ page }) => {
    await mockCore(page, { answers });
    await page.goto('/');

    await expect(page.getByRole('heading', { name: 'Mac Fresher' })).toBeVisible();
    await expect(page.getByText('Version 0.0.1')).toBeVisible();
  });

  test('streams a scan into a list grouped by category', async ({ page }) => {
    await mockCore(page, { answers, scan: DEMO_SCAN });
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();

    await expect(page.getByText('The scan is complete.')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Example caches' })).toBeVisible();
    await expect(page.getByRole('region', { name: 'Example caches' }).getByRole('listitem')).toHaveCount(2);
    await expect(page.getByRole('region', { name: 'Example downloads' })).toContainText('750 MB · Low');
  });
});
