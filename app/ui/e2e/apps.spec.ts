import { expect, type Locator, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { REVEALED } from './support/apps-mock';
import { scanButton, scanFinished } from './support/cards';

// The Apps screen and an app's page on the mocked core: the list and its filter, the uninstall with each choice through
// the confirmation, the reason an app is not offered, and put back of an uninstall from Cleanups.

type Script = (typeof SCRIPTS)[keyof typeof SCRIPTS];

const heading = (page: Page, name: string): Locator => page.getByRole('heading', { level: 1, name });

const appsList = (page: Page): Locator => page.getByRole('list', { name: 'Apps, largest first' });

const appRow = (page: Page, name: string): Locator =>
  appsList(page).getByRole('listitem').filter({ hasText: name }).first();

// Scans the script and opens the Apps screen.
const openApps = async (page: Page, script: Script = SCRIPTS.apps): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  await page.getByRole('navigation').getByRole('button', { name: 'Apps' }).click();
  await expect(heading(page, 'Apps')).toBeFocused();
  await expect(appsList(page)).toBeVisible();
};

// Opens an app's page from its row.
const openApp = async (page: Page, name: string, script: Script = SCRIPTS.apps): Promise<void> => {
  await openApps(page, script);
  await appRow(page, name).click();
  await expect(heading(page, name)).toBeFocused();
};

const uninstallButton = (page: Page): Locator => page.getByRole('button', { name: 'Uninstall…' });

const KEEP = /Remove the app, keep its settings and data/;

const EVERYTHING = /Remove the app and everything it left/;

test.describe('negative cases', () => {
  test('an app a manager installed shows its reason, its command and no button', async ({ page }) => {
    await openApp(page, 'Terminal Pro');

    await expect(page.getByRole('region', { name: 'Why it cannot be uninstalled' })).toContainText(
      'Homebrew installed this app',
    );
    await expect(page.locator('code')).toHaveText('brew uninstall --cask terminal-pro');
    await expect(uninstallButton(page)).toHaveCount(0);
    await expect(page.getByRole('radio')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Show in Finder' })).toHaveCount(0);
  });

  test('an app that is open shows its reason and no button', async ({ page }) => {
    await openApps(page);

    await expect(appRow(page, 'Chat')).toContainText('This app, or a part of it, is open.');
    await expect(uninstallButton(page)).toHaveCount(0);
  });

  test('cancelling the confirmation removes nothing and stays on the page', async ({ page }) => {
    await openApp(page, 'Video Studio');
    await uninstallButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });

    await dialog.getByRole('button', { name: 'Cancel' }).click();

    await expect(dialog).toHaveCount(0);
    await expect(heading(page, 'Video Studio')).toBeVisible();
    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toHaveCount(0);
  });

  test('nothing is chosen for removal before the user asks', async ({ page }) => {
    await openApp(page, 'Video Studio');

    await expect(page.getByRole('dialog')).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Review/ })).not.toContainText('selected');
  });
});

test.describe('positive cases', () => {
  test('the filter keeps the unused apps', async ({ page }) => {
    await openApps(page);
    await expect(appsList(page).getByRole('listitem').filter({ hasText: 'takes' })).not.toHaveCount(0);

    await page.getByRole('radio', { name: 'A year' }).click();

    await expect(appRow(page, 'Old Editor')).toBeVisible();
    await expect(appsList(page)).not.toContainText('Video Studio');
    await expect(appsList(page)).not.toContainText('Terminal Pro');
  });

  test('uninstall keeps the data by default', async ({ page }) => {
    await openApp(page, 'Video Studio');
    await expect(page.getByRole('radio', { name: KEEP })).toBeChecked();
    await expect(page.getByRole('radio', { name: EVERYTHING })).not.toBeChecked();

    await uninstallButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('Video Studio');
    await expect(dialog).toContainText('Medium');
    await expect(dialog.getByRole('button', { name: 'and 4 more files that go with it: Video Studio' })).toBeVisible();
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 1 item to the Trash');
  });

  test('uninstall everything lists the files in the confirmation', async ({ page }) => {
    await openApp(page, 'Video Studio');
    await page.getByRole('radio', { name: EVERYTHING }).check();

    await uninstallButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('High');
    await dialog.getByRole('button', { name: /and 8 more files that go with it/ }).click();
    const files = dialog.getByRole('list', { name: /^The other files that go with/ });
    await expect(files.getByRole('listitem')).toHaveCount(8);
    await expect(files).toContainText('/Library/Preferences/com.example.VideoStudio.plist');
    await expect(files).toContainText('/Library/Application Support/com.example.VideoStudio');
    await expect(files).not.toContainText('Group Containers');
  });

  test('the page lists every file, the kept ones with their reason, and links to the app’s categories', async ({
    page,
  }) => {
    await openApp(page, 'Video Studio');

    const files = page.getByRole('region', { name: 'What it would lose' });
    await expect(files.getByRole('region', { name: 'Caches' })).toContainText('Goes with both choices');
    await expect(files.getByRole('region', { name: 'Settings' })).toContainText('Only with everything');
    await expect(files).toContainText('Other apps by the same maker may use this folder');
    await page.getByRole('button', { name: 'Caches of your apps' }).click();
    await expect(heading(page, 'Caches of your apps')).toBeVisible();
  });

  test('the App Management notice asks nothing before macOS refused', async ({ page }) => {
    await openApp(page, 'Video Studio');

    await expect(page.getByText(/macOS may ask you to let Fresher manage apps/)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Open System Settings' })).toHaveCount(0);
  });

  test('the App Management notice offers System Settings once macOS refused', async ({ page }) => {
    await openApp(page, 'Video Studio', SCRIPTS.appsDenied);

    await expect(page.getByText(/macOS did not let Fresher move an app/)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Open System Settings' })).toBeVisible();
  });

  test('an app that needs an administrator offers Show in Finder', async ({ page }) => {
    await openApp(page, 'Disk Tool');

    await expect(page.getByRole('region', { name: 'Why it cannot be uninstalled' })).toContainText(
      'Removing this app needs an administrator',
    );
    await expect(uninstallButton(page)).toHaveCount(0);
    await page.getByRole('button', { name: 'Show in Finder' }).click();

    await expect.poll(() => page.evaluate((key) => (globalThis as never)[key] as number[], REVEALED)).toEqual([4]);
  });

  test('an uninstall comes back by put back from Cleanups', async ({ page }) => {
    await openApps(page);
    await page.getByRole('navigation').getByRole('button', { name: 'Cleanups' }).click();
    await page
      .getByRole('list', { name: 'Cleanups, newest first' })
      .getByRole('button', { name: /Jan 2, 2025/ })
      .click();

    const item = page.getByRole('list', { name: 'The items of this cleanup' }).getByRole('listitem').first();
    await expect(item).toContainText('Video Studio');
    await expect(item).toContainText('In the Trash.');
    await item.getByRole('button', { name: 'Put back' }).click();

    await expect(item).toContainText('Put back where it was.');
  });
});
