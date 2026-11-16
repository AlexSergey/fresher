import { expect, type Locator, type Page, test } from '@playwright/test';

import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// Every screen in each of its states, motion off, in WebKit and Chromium.

// The demo module's nav item, with its found size: the overview the core answered last.
const moduleNav = (page: Page): Locator => page.getByRole('button', { name: /^Demo module/ });

const open = async (page: Page, script: CoreScript): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await expect(moduleNav(page)).toBeVisible();
};

// Scans and waits for the overview the core answers at the end: the module's nav item shows its size.
const scan = async (page: Page, found: string): Promise<void> => {
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(moduleNav(page)).toContainText(found);
};

const go = async (page: Page, nav: RegExp | string, heading: string): Promise<void> => {
  await page.getByRole('button', { name: nav }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: heading })).toBeFocused();
};

const looksTheSame = async (page: Page, name: string): Promise<void> => {
  await expectNoCspViolation(page);
  await expect(page).toHaveScreenshot(name);
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.describe('positive cases', () => {
  test.describe('overview', () => {
    test('before a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await expect(moduleNav(page)).toContainText('—');

      await looksTheSame(page, 'start-screen.png');
    });

    test('scanning, paused mid-stream', async ({ page }) => {
      await open(page, SCRIPTS.live);
      await scan(page, '2.3 GB');
      await expect(page.getByRole('list', { name: 'Parts of the scan' })).toContainText('scanning');
      await expect(page.getByText('18,432').first()).toBeVisible();
      await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);

      await looksTheSame(page, 'overview-scanning.png');
    });

    test('a finished scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);

      await looksTheSame(page, 'finished-scan.png');
    });

    test('a partial scan', async ({ page }) => {
      await open(page, SCRIPTS.partial);
      await scan(page, '2.6 GB');
      await expect(page.getByText('This scan is incomplete')).toBeVisible();

      await looksTheSame(page, 'overview-partial.png');
    });

    test('a failed scan', async ({ page }) => {
      await open(page, SCRIPTS.failed);
      await scan(page, '2.3 GB');
      await expect(
        page.getByText('The demo module stopped with an error, as it was told to.', { exact: true }),
      ).toBeVisible();

      await looksTheSame(page, 'overview-failed.png');
    });

    test('Full Disk Access denied', async ({ page }) => {
      await open(page, SCRIPTS.denied);
      await expect(page.getByText('Fresher cannot see every folder')).toBeVisible();

      await looksTheSame(page, 'overview-access-denied.png');
    });
  });

  test.describe('module page', () => {
    test('a category expanded with rows selected', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('button', { name: 'Open Example caches' }).click();
      const caches = page.getByRole('region', { name: 'Example caches' });
      await caches.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
      await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');

      await looksTheSame(page, 'module-page.png');
    });
  });

  test.describe('review', () => {
    test('with a selection', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('button', { name: 'Open Example caches' }).click();
      const caches = page.getByRole('region', { name: 'Example caches' });
      await caches.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
      await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
      await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
      await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toContainText(
        'Example: installer.dmg',
      );

      await looksTheSame(page, 'review-selection.png');
    });

    test('with nothing selected', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await go(page, /^Review/, 'Review');
      await expect(page.getByText('Nothing is selected')).toBeVisible();

      await looksTheSame(page, 'review-empty.png');
    });
  });

  test.describe('settings', () => {
    test('settings of every kind', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByText('Version 0.0.1')).toBeVisible();
      await expect(page.getByRole('switch', { name: 'Demo module' })).toBeChecked();

      await looksTheSame(page, 'settings.png');
    });

    test('a settings file only a newer version can change', async ({ page }) => {
      await open(page, SCRIPTS.readOnly);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByText('Your settings were saved by a newer version of Fresher')).toBeVisible();
      await expect(page.getByRole('switch', { name: 'Demo module' })).toBeDisabled();

      await looksTheSame(page, 'settings-read-only.png');
    });
  });
});
