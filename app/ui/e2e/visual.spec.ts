import { expect, type Locator, type Page, test } from '@playwright/test';

import { PLAN_GONE } from '../src/test/cleanup-plan';
import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { requestQuit } from './support/cleanup-mock';
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

// Ticks both categories of the demo scan and opens the review.
const reviewAll = async (page: Page, script: CoreScript): Promise<void> => {
  await open(page, script);
  await scan(page, '2.6 GB');
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await go(page, /^Review/, 'Review');
};

const confirmation = async (page: Page, script: CoreScript): Promise<Locator> => {
  await reviewAll(page, script);
  await page.getByRole('button', { name: 'Move to Trash…' }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
};

const looksTheSame = async (page: Page, name: string): Promise<void> => {
  await expectNoCspViolation(page);
  await expect(page).toHaveScreenshot(name);
};

// The JetBrains scan, on the module's page; with `review`, WebStorm's three categories ticked and the review open.
const jetbrainsPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.jetbrains);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^JetBrains IDEs/ });
  await expect(nav).toContainText('1.7 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'JetBrains IDEs' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Caches of older JetBrains IDE versions' })).toContainText(
    '3 items left out (1.4 GB)',
  );
};

// The temporary files scan, on the module's page.
const tempPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Temporary files/ });
  await expect(nav).toContainText('837 MB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Temporary files' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Old temporary files' })).toContainText(
    '4,712 items left out (790 MB)',
  );
};

// The developer caches scan with Xcode open, on the module's page: the projects left out, one Gradle folder not the
// user's.
const devCachesPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.devCachesXcodeOpen);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Developer caches/ });
  await expect(nav).toContainText('12.9 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Developer caches' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Xcode build data' })).toContainText('2 items left out (4.3 GB)');
};

// The removed apps' leftovers scan, on the module's page: its two choices, and an open app's and a recent one's files
// left out.
const leftoversPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.appLeftovers);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Removed apps' leftovers/ });
  await expect(nav).toContainText('2.5 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: "Removed apps' leftovers" })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Caches of removed apps' })).toContainText('1 item left out (220 MB)');
};

// The downloads scan, on the module's page: five categories, a browser-open download and a recent big file left out.
const downloadsPage = async (
  page: Page,
  script: typeof SCRIPTS.downloads | typeof SCRIPTS.downloadsUnavailable = SCRIPTS.downloads,
  total = '15.8 GB',
): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Downloads and installers/ });
  await expect(nav).toContainText(total);
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Downloads and installers' })).toBeFocused();
};

// The old-projects scan, on the module's page: four categories and three left-out lines.
const staleProjectsPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.staleProjects);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Old projects/ });
  await expect(nav).toContainText('6.6 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Old projects' })).toBeFocused();
};

// Settings, with the temporary files module's number field in view.
const tempSettings = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeFocused();
  const days = page.getByRole('spinbutton', { name: 'Offer temporary files unchanged for' });
  await expect(days).toHaveValue('7');
  await days.scrollIntoViewIfNeeded();
};

// WebStorm's three categories ticked, the review open and the confirmation (it names Medium) asked for.
const jetbrainsConfirmation = async (page: Page): Promise<Locator> => {
  await jetbrainsPage(page);
  for (const name of ['Caches', 'Logs', 'Settings and plugins']) {
    await page.getByRole('checkbox', { name: new RegExp(`^Select ${name} of older`) }).check();
  }
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
  await page.getByRole('button', { name: 'Move to Trash…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
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
      await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');
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

    test('a finished scan, by place', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('radio', { name: 'By place' }).click();
      await expect(page.getByRole('list', { name: 'Inside home' }).getByRole('listitem')).toHaveCount(6);

      await looksTheSame(page, 'overview-by-place.png');
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

    test('the sandbox banner', async ({ page }) => {
      await open(page, SCRIPTS.sandboxed);
      await expect(page.getByRole('note')).toContainText(
        'Sandbox · /Users/dev/sandboxes/trial · Simulated: open apps.',
      );

      await looksTheSame(page, 'banner-sandbox.png');
    });

    test('the development banner', async ({ page }) => {
      await open(page, SCRIPTS.development);
      await expect(page.getByRole('note')).toContainText('nothing can be removed outside a sandbox');

      await looksTheSame(page, 'banner-development.png');
    });

    test('Full Disk Access denied', async ({ page }) => {
      await open(page, SCRIPTS.denied);
      await scan(page, '2.6 GB');
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

    test('a category with the line of what was left out of it', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('button', { name: 'Open Example caches' }).click();
      const caches = page.getByRole('region', { name: 'Example caches' });
      await expect(caches).toContainText('1 item left out (157.3 MB): The app that made it is open.');

      await looksTheSame(page, 'module-page-left-out.png');
    });

    test('the page of a module with its left-out lines and a Medium category', async ({ page }) => {
      await jetbrainsPage(page);
      await expect(
        page.getByRole('region', { name: 'Settings and plugins of older JetBrains IDE versions' }),
      ).toContainText('2 items left out (600 MB)');

      await looksTheSame(page, 'module-page-jetbrains.png');
    });

    test('the page of Temporary files with its left-out lines', async ({ page }) => {
      await tempPage(page);
      await expect(page.getByRole('region', { name: 'Old temporary files' })).toContainText('3 items left out (12 MB)');

      await looksTheSame(page, 'module-page-temp.png');
    });

    test('the page of Developer caches with its left-out lines', async ({ page }) => {
      await devCachesPage(page);
      await expect(page.getByRole('region', { name: "Gradle's caches" })).toContainText('1 item left out (35 MB)');
      await page.getByRole('region', { name: 'Xcode build data' }).scrollIntoViewIfNeeded();

      await looksTheSame(page, 'module-page-dev-caches.png');
    });

    test('the page of Removed apps’ leftovers with its choices and left-out lines', async ({ page }) => {
      await leftoversPage(page);
      await expect(page.getByRole('button', { name: 'Remove everything' })).toBeEnabled();

      await looksTheSame(page, 'module-page-app-leftovers.png');
    });

    test('the page of Downloads and installers with its five categories and left-out lines', async ({ page }) => {
      await downloadsPage(page);
      await expect(page.getByRole('region', { name: 'Unfinished downloads' })).toContainText('1 item left out (48 MB)');

      await looksTheSame(page, 'module-page-downloads.png');
    });

    test('the page of Downloads and installers before Downloads is allowed', async ({ page }) => {
      await downloadsPage(page, SCRIPTS.downloadsUnavailable, '0 bytes');
      await expect(page.getByRole('button', { name: 'Let Fresher look there' }).first()).toBeVisible();

      await looksTheSame(page, 'module-page-downloads-unavailable.png');
    });

    test('the page of Old projects with its four categories and left-out lines', async ({ page }) => {
      await staleProjectsPage(page);
      await expect(page.getByRole('region', { name: 'Packages of old projects' })).toContainText('2 items left out');

      await looksTheSame(page, 'module-page-stale-projects.png');
    });

    test('the page of Removed apps’ leftovers with Keep settings chosen', async ({ page }) => {
      await leftoversPage(page);
      await page.getByRole('button', { name: 'Keep settings' }).click();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');

      await looksTheSame(page, 'module-page-app-leftovers-chosen.png');
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

  test.describe('cleanup', () => {
    test('the confirmation', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await expect(dialog).toHaveAccessibleName('Move 3 items to the Trash?');

      await looksTheSame(page, 'cleanup-confirm.png');
    });

    test('the confirmation, permanent', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await dialog.getByRole('checkbox', { name: /^Delete permanently instead/ }).check();
      await expect(dialog.getByRole('button', { name: 'Delete permanently' })).toBeVisible();

      await looksTheSame(page, 'cleanup-confirm-permanent.png');
    });

    test('the confirmation with an item left out', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningLeftOut);
      const leftOut = dialog.getByRole('region', { name: 'These will not be removed' });
      await leftOut.scrollIntoViewIfNeeded();
      await expect(leftOut).toContainText('Quit Safari and try again.');

      await looksTheSame(page, 'cleanup-confirm-left-out.png');
    });

    test('the confirmation that names Medium', async ({ page }) => {
      const dialog = await jetbrainsConfirmation(page);
      await expect(dialog).toContainText('Medium');

      await looksTheSame(page, 'cleanup-confirm-medium.png');
    });

    test('a refused confirmation', async ({ page }) => {
      const dialog = await confirmation(page, {
        ...SCRIPTS.cleaning,
        refusals: { confirmCleanup: PLAN_GONE },
      });
      await dialog.getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByRole('alert')).toBeVisible();

      await looksTheSame(page, 'cleanup-refused.png');
    });

    test('the progress', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();

      await looksTheSame(page, 'cleanup-progress.png');
    });

    test('the quit question', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();
      await requestQuit(page);
      await expect(page.getByRole('button', { name: 'Keep cleaning' })).toBeFocused();

      await looksTheSame(page, 'cleanup-quit.png');
    });

    test('the result of a move to the Trash', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await dialog.getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();

      await looksTheSame(page, 'cleanup-result.png');
    });

    test('the result of a stopped cleanup', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Move to Trash' }).click();
      await page.getByRole('button', { name: 'Stop' }).click();
      await expect(page.getByText('Stopped. 2 items were not touched.')).toBeVisible();

      await looksTheSame(page, 'cleanup-result-stopped.png');
    });

    test('the cleanups', async ({ page }) => {
      await open(page, SCRIPTS.cleaning);
      await go(page, 'Cleanups', 'Cleanups');
      await expect(page.getByText('Waiting in the Trash from cleanups: 4.1 GB')).toBeVisible();

      await looksTheSame(page, 'cleanups.png');
    });

    test('no cleanup yet', async ({ page }) => {
      await open(page, SCRIPTS.cleanupsEmpty);
      await go(page, 'Cleanups', 'Cleanups');
      await expect(page.getByText('Nothing cleaned yet')).toBeVisible();

      await looksTheSame(page, 'cleanups-empty.png');
    });

    test('one cleanup', async ({ page }) => {
      await open(page, SCRIPTS.cleaning);
      await go(page, 'Cleanups', 'Cleanups');
      await page.getByRole('button', { name: /^Cleanup of Jan 2, 2025/ }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Cleanup of Jan 2, 2025' })).toBeFocused();
      await expect(page.getByRole('list', { name: 'The items of this cleanup' })).toContainText('editor-cache');

      await looksTheSame(page, 'cleanup-detail.png');
    });

    test('a cleanup Fresher quit during', async ({ page }) => {
      await open(page, SCRIPTS.cleaning);
      await go(page, 'Cleanups', 'Cleanups');
      await page.getByRole('button', { name: /^Cleanup of Dec 31, 2024/ }).click();
      await expect(page.getByText(/Items marked Unknown may be in the Trash/)).toBeVisible();

      await looksTheSame(page, 'cleanup-detail-interrupted.png');
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

    test('the number field of Temporary files', async ({ page }) => {
      await tempSettings(page);
      await expect(page.getByText('From 7 to 30 days')).toBeVisible();

      await looksTheSame(page, 'settings-temp.png');
    });

    test('the page of an ignored module', async ({ page }) => {
      await open(page, SCRIPTS.ignoring);
      await scan(page, '2.6 GB');
      await go(page, /^Demo module/, 'Demo module');
      await page.getByRole('button', { name: 'Ignore this module' }).click();
      await expect(page.getByText('You ignore this module.')).toBeVisible();

      await looksTheSame(page, 'module-ignored.png');
    });

    for (const [tab, file] of [
      ['Ignored groups', 'settings-ignored-groups.png'],
      ['Ignored modules', 'settings-ignored-modules.png'],
      ['Ignored items', 'settings-ignored-items.png'],
    ] as const) {
      test(`the Ignored section of Settings, ${tab}`, async ({ page }) => {
        await open(page, SCRIPTS.ignoring);
        await go(page, 'Settings', 'Settings');
        await page.getByRole('switch', { name: 'Ignore Debug' }).click();
        const section = page.getByRole('region', { exact: true, name: 'Ignored' });
        await section.getByRole('tab', { name: new RegExp(`^${tab}`) }).click();
        await section.scrollIntoViewIfNeeded();

        await expectNoCspViolation(page);
        await expect(section).toHaveScreenshot(file);
      });
    }

    test('a settings file only a newer version can change', async ({ page }) => {
      await open(page, SCRIPTS.readOnly);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByText('Your settings were saved by a newer version of Fresher')).toBeVisible();
      await expect(page.getByRole('switch', { name: 'Demo module' })).toBeDisabled();

      await looksTheSame(page, 'settings-read-only.png');
    });
  });
});
