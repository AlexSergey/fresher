import { expect, type Locator, type Page, test } from '@playwright/test';

import { PLAN_GONE } from '../src/test/cleanup-plan';
import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { details, navScan, openDetails, scanButton, scanFinished } from './support/cards';
import { requestQuit } from './support/cleanup-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// Every screen in each of its states, motion off, in WebKit and Chromium.

const open = async (page: Page, script: CoreScript): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Scan' })).toBeVisible();
  await expect(scanButton(page)).toBeVisible();
};

// Scans and waits for the overview the core answers at the end: the size it found is on the screen.
const scan = async (page: Page, found: string): Promise<void> => {
  await scanButton(page).click();
  await expect(page.getByText(found).first()).toBeAttached();
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

// Scans `script` and opens the details of `category` from its card.
const detailsAfter = async (
  page: Page,
  script: (typeof SCRIPTS)[keyof typeof SCRIPTS],
  category: string,
): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  await openDetails(page, category);
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
  await mockCore(page, SCRIPTS.jetbrains);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
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
  test.describe('scan', () => {
    test('before a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await expect(navScan(page)).toHaveAttribute('aria-current', 'page');

      await looksTheSame(page, 'start-screen.png');
    });

    test('scanning, paused mid-stream', async ({ page }) => {
      await open(page, SCRIPTS.live);
      await scan(page, '2.3 GB');
      await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');
      await expect(page.getByText('18,432').first()).toBeVisible();
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);

      await looksTheSame(page, 'scan-scanning.png');
    });

    test('a finished scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);

      await looksTheSame(page, 'finished-scan.png');
    });

    test('a partial scan', async ({ page }) => {
      await open(page, SCRIPTS.partial);
      await scan(page, '2.6 GB');
      await expect(page.getByText('This scan is incomplete')).toBeVisible();

      await looksTheSame(page, 'scan-partial.png');
    });

    test('a failed scan', async ({ page }) => {
      await open(page, SCRIPTS.failed);
      // A failed scan shows no size: its error says the end.
      await scanButton(page).click();
      await expect(
        page.getByText('The demo module stopped with an error, as it was told to.', { exact: true }),
      ).toBeVisible();

      await looksTheSame(page, 'scan-failed.png');
    });

    test('the sandbox banner', async ({ page }) => {
      await open(page, SCRIPTS.sandboxed);
      await expect(page.getByRole('note')).toContainText(
        "Sandbox · /Users/dev/sandboxes/trial · Cleanups move items to the sandbox's Trash: " +
          '/Users/dev/sandboxes/trial/home/.Trash · Simulated: open apps.',
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

      await looksTheSame(page, 'scan-access-denied.png');
    });
  });

  test.describe("a category's details", () => {
    test('with a finding ticked and the line of what was left out', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
      await openDetails(page, 'Example caches');
      await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');
      await expect(details(page)).toContainText('1 item left out (157.3 MB): The app that made it is open.');

      await looksTheSame(page, 'category-details.png');
    });

    for (const [script, category, line, file] of [
      [
        'jetbrains',
        'Settings and plugins of older JetBrains IDE versions',
        '2 items left out (600 MB)',
        'category-details-jetbrains.png',
      ],
      ['temp', 'Old temporary files', '3 items left out (12 MB)', 'category-details-temp.png'],
      ['devCachesXcodeOpen', "Gradle's caches", '1 item left out (35 MB)', 'category-details-dev-caches.png'],
      ['appLeftovers', 'Caches of removed apps', '1 item left out (220 MB)', 'category-details-app-leftovers.png'],
      ['downloads', 'Unfinished downloads', '1 item left out (48 MB)', 'category-details-downloads.png'],
      ['staleProjects', 'Packages of old projects', '2 items left out', 'category-details-stale-projects.png'],
      ['localAiModels', 'Hugging Face models', '1 item left out (300 MB)', 'category-details-local-ai-models.png'],
    ] as const) {
      test(`of ${category}, with its left-out lines`, async ({ page }) => {
        await detailsAfter(page, SCRIPTS[script], category);
        await expect(details(page)).toContainText(line);

        await looksTheSame(page, file);
      });
    }

    test('the Scan screen with a category found but left out', async ({ page }) => {
      await mockCore(page, SCRIPTS.devCachesXcodeOpen);
      await page.goto('/');
      await scanButton(page).click();
      await scanFinished(page);
      const leftOut = page.getByRole('region', { name: 'Found, but left out' });
      await expect(leftOut).toContainText('2 items left out (4.3 GB)');
      // The totals have arrived: Gradle's caches are a card, not a left-out row.
      await expect(page.locator('[data-card]').filter({ hasText: "Gradle's caches" })).toBeVisible();
      await expect(leftOut).not.toContainText("Gradle's caches");
      await leftOut.scrollIntoViewIfNeeded();

      await looksTheSame(page, 'scan-left-out.png');
    });

    test('of a downloads category before Downloads is allowed', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.downloadsUnavailable, 'Unfinished downloads');
      await expect(page.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();

      await looksTheSame(page, 'category-details-downloads-unavailable.png');
    });

    test('of the caches of removed apps with Keep settings chosen', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.appLeftovers, 'Caches of removed apps');
      await page.getByRole('button', { name: 'Keep settings' }).click();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');

      await looksTheSame(page, 'category-details-app-leftovers-chosen.png');
    });
  });

  test.describe('review', () => {
    test('with a selection', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
      await openDetails(page, 'Example caches');
      await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
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
