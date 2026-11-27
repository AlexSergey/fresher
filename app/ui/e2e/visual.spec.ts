import { expect, type Locator, type Page, test } from '@playwright/test';

import { PLAN_GONE } from '../src/test/cleanup-plan';
import { type CoreScript, firstRelease, mockCore, SCRIPTS } from './core-mock';
import { details, navScan, openDetails, scanButton, scanFinished, tickItemsOf } from './support/cards';
import { requestQuit } from './support/cleanup-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';
import { settingsPage } from './support/settings';

// Every screen in each of its states, motion off, in WebKit and Chromium.

const open = async (page: Page, script: CoreScript): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Scan' })).toBeVisible();
  await expect(scanButton(page)).toBeVisible();
};

// Scans and waits for the overview the core answers at the end and the disk card's sizes after it.
const scan = async (page: Page): Promise<void> => {
  await scanButton(page).click();
  await scanFinished(page);
  await expect(page.getByText('can be freed', { exact: true })).toBeVisible();
};

const go = async (page: Page, nav: RegExp | string, heading: string): Promise<void> => {
  await page.getByRole('button', { name: nav }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: heading })).toBeFocused();
};

// Ticks both categories of the demo scan and opens the review.
const reviewAll = async (page: Page, script: CoreScript): Promise<void> => {
  await open(page, script);
  await scan(page);
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await go(page, /^Review/, 'Review');
};

// The first cleanup of a world where the agreement was never accepted: the sheet is open.
const agreementSheet = async (page: Page): Promise<Locator> => {
  await reviewAll(page, { ...SCRIPTS.cleaning, agreement: 'not-accepted' });
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const sheet = page.getByRole('dialog', { name: 'Before your first cleanup' });
  await expect(sheet.getByRole('button', { name: 'Not now' })).toBeFocused();

  return sheet;
};

const confirmation = async (page: Page, script: CoreScript): Promise<Locator> => {
  await reviewAll(page, script);
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
};

// Ollama's models ticked on Scan, the review open and the confirmation asked for.
const ollamaConfirmation = async (page: Page): Promise<Locator> => {
  await open(page, SCRIPTS.localAiModelsOllama);
  await scan(page);
  await tickItemsOf(page, 'Ollama models');
  await go(page, /^Review/, 'Review');
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
};

// The downloads scanned on Scan: the Low, Medium and High zones.
const downloadsScanned = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.downloads);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
};

// The installers (Low) and the big untouched file (High) ticked, and the review open: both parts.
const reviewDownloads = async (page: Page): Promise<void> => {
  await downloadsScanned(page);
  await page.getByRole('checkbox', { name: 'Select Old installers' }).check();
  await tickItemsOf(page, 'Big unopened downloads');
  await go(page, /^Review/, 'Review');
};

// The confirmation of the downloads' High plan, its High tick off.
const highConfirmation = async (page: Page): Promise<Locator> => {
  await reviewDownloads(page);
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
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

const TEMP_PAGE = 'Temporary files';
const LOGS_PAGE = 'Logs and crash reports';

// Settings, with the temporary files module's number field in view.
const tempSettings = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeFocused();
  await settingsPage(page, TEMP_PAGE, 'System', TEMP_PAGE);
  const days = page.getByRole('spinbutton', { name: 'Offer temporary files unchanged for' });
  await expect(days).toHaveValue('7');
  await days.scrollIntoViewIfNeeded();
};

// Settings, with the logs module's days field in view.
const logsSettings = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.logs);
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeFocused();
  await settingsPage(page, LOGS_PAGE, 'System', LOGS_PAGE);
  const days = page.getByRole('spinbutton', { name: 'Offer logs and crash reports unchanged for' });
  await expect(days).toHaveValue('7');
  await days.scrollIntoViewIfNeeded();
};

// WebStorm's three categories ticked (its settings, Look before removing, one by one), the review open and the confirmation (it names Medium) asked for.
const jetbrainsConfirmation = async (page: Page): Promise<Locator> => {
  await mockCore(page, SCRIPTS.jetbrains);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  for (const name of ['caches', 'logs']) {
    await page.getByRole('checkbox', { name: new RegExp(`^Select Old JetBrains IDE ${name}`) }).check();
  }
  const settings = await page.getByRole('heading', { level: 3, name: /^Old JetBrains IDE settings/u }).textContent();
  await tickItemsOf(page, settings ?? '');
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
};

// Scans, opens the one card of the described section and waits for the insight screen's title to take the focus.
const insightAfter = async (page: Page, script: CoreScript, title: string): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  await page.getByRole('region', { name: 'Shown, never removed' }).getByRole('button', { name: 'Details' }).click();
  await expect(page.getByRole('heading', { level: 1, name: title })).toBeFocused();
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
      await scanButton(page).click();
      await expect(page.getByText('2.3 GB').first()).toBeAttached();
      await expect(page.getByRole('progressbar', { name: 'Scanning, 75 percent' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(0);

      await looksTheSame(page, 'scan-scanning.png');
    });

    test('a finished scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page);
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);

      await looksTheSame(page, 'finished-scan.png');
    });

    test('a finished scan while High is withheld', async ({ page }) => {
      await open(page, firstRelease(SCRIPTS.demo));
      await scan(page);
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);
      await expect(page.getByRole('navigation').getByRole('button', { name: 'Largest' })).toHaveCount(0);

      await looksTheSame(page, 'finished-scan-withheld.png');
    });

    test('a partial scan', async ({ page }) => {
      await open(page, SCRIPTS.partial);
      await scan(page);
      await expect(page.getByRole('heading', { name: 'Some folders were not scanned' })).toBeVisible();

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
      await scan(page);
      await expect(page.getByText('Fresher cannot see every folder')).toBeVisible();

      await looksTheSame(page, 'scan-access-denied.png');
    });
  });

  test.describe("a category's details", () => {
    test('with a finding ticked and the line of what was left out', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page);
      await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
      await openDetails(page, 'Example caches');
      await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');
      await expect(details(page)).toContainText('1 item left out (157.3 MB): The app that made it is open.');

      await looksTheSame(page, 'category-details.png');
    });

    for (const [script, category, line, file] of [
      ['jetbrains', 'Old JetBrains IDE settings', '2 items left out (600 MB)', 'category-details-jetbrains.png'],
      ['temp', 'Old temporary files', '3 items kept (12 MB)', 'category-details-temp.png'],
      ['logs', 'Old app logs', '1 item left out (3 MB)', 'category-details-logs.png'],
      ['devCachesXcodeOpen', "Gradle's caches", '1 item kept (35 MB)', 'category-details-dev-caches.png'],
      ['appCaches', 'App caches', '1 item left out (90 MB)', 'category-details-app-caches.png'],
      ['appCachesBrowsers', 'Browser caches', '1 item left out (30 MB)', 'category-details-app-caches-browsers.png'],
      [
        'appCachesBrowsers',
        'Web-based app caches',
        'Redis Insight: GPUCache',
        'category-details-app-caches-web-engine.png',
      ],
      ['appLeftovers', 'Caches of removed apps', '1 item left out (220 MB)', 'category-details-app-leftovers.png'],
      ['downloads', 'Unfinished downloads', '1 item left out (48 MB)', 'category-details-downloads.png'],
      ['staleProjects', 'Packages of old projects', '2 items left out', 'category-details-stale-projects.png'],
      ['localAiModels', 'Hugging Face models', '1 item kept (300 MB)', 'category-details-local-ai-models.png'],
      ['unusedToolchains', 'Unused Node versions', '1 item kept (110 MB)', 'category-details-unused-toolchains.png'],
    ] as const) {
      test(`of ${category}, with its left-out lines`, async ({ page }) => {
        await detailsAfter(page, SCRIPTS[script], category);
        await expect(details(page)).toContainText(line);
        await expect(details(page).getByRole('checkbox').first()).toBeVisible();

        await looksTheSame(page, file);
      });
    }

    test('of the Xcode archives, at the High level', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.xcode, 'Xcode archives');
      await expect(details(page).getByRole('checkbox').first()).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'category-details-xcode-archives.png');
    });

    test("of Ollama's models", async ({ page }) => {
      await detailsAfter(page, SCRIPTS.localAiModelsOllama, 'Ollama models');
      await expect(details(page).getByRole('checkbox').first()).toBeVisible();

      await looksTheSame(page, 'category-details-ollama.png');
    });

    test('the Scan screen with the described section', async ({ page }) => {
      await mockCore(page, SCRIPTS.docker);
      await page.goto('/');
      await scanButton(page).click();
      await scanFinished(page);
      const section = page.getByRole('region', { name: 'Shown, never removed' });
      await expect(section).toContainText('OrbStack');
      // The totals have arrived: the layout no longer moves.
      await expect(page.locator('[data-card]')).toHaveCount(4);
      await section.evaluate((element) => {
        element.scrollIntoView({ block: 'end' });
      });
      await expect(section.getByRole('button', { name: 'Details' })).toBeInViewport();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'scan-described.png');
    });

    test('the insight screen, answered', async ({ page }) => {
      await insightAfter(page, SCRIPTS.docker, 'OrbStack');
      await expect(page.getByRole('button', { name: 'Back' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'insight-answered.png');
    });

    test('the insight screen, stopped', async ({ page }) => {
      await insightAfter(page, SCRIPTS.dockerStopped, 'Docker Desktop');
      await expect(page.getByRole('button', { name: 'Back' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'insight-stopped.png');
    });

    test('the Scan screen with a category found but left out', async ({ page }) => {
      await mockCore(page, SCRIPTS.devCachesXcodeOpen);
      await page.goto('/');
      await scanButton(page).click();
      await scanFinished(page);
      const leftOut = page.getByRole('region', { name: 'Found, but left out' });
      await expect(leftOut).toContainText('2 items left out (4.3 GB)');
      // The line is folded: its items list stays hidden until the toggle is opened.
      await expect(leftOut.getByRole('list', { name: 'The items left out' })).toBeHidden();
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

    test('of an unused toolchains category before Documents is allowed', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.unusedToolchainsNoDocuments, 'Unused Rust versions');
      await expect(page.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();

      await looksTheSame(page, 'category-details-unused-toolchains-unavailable.png');
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
      await scan(page);
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

    test('with a Look tile partly checked', async ({ page }) => {
      await open(page, SCRIPTS.appLeftovers);
      await scan(page);
      await page.getByRole('checkbox', { exact: true, name: 'Select Caches of removed apps' }).check();
      await page.getByRole('checkbox', { exact: true, name: 'Select Settings of removed apps' }).check();
      await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
      const settings = page.getByRole('article', { name: 'Settings of removed apps' });
      await settings
        .getByRole('checkbox', { name: /^Select /u })
        .first()
        .check();
      await expect(settings).toContainText(/^.*1 of \d+ checked/u);
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'review-look.png');
    });

    test('with nothing selected', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page);
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

    test('the agreement before the first cleanup', async ({ page }) => {
      const sheet = await agreementSheet(page);
      await expect(sheet.getByRole('button', { name: 'I understand' })).toBeVisible();

      await looksTheSame(page, 'agreement-sheet.png');
    });

    test('the full licence', async ({ page }) => {
      const sheet = await agreementSheet(page);
      await sheet.getByRole('button', { name: 'Read the full licence' }).click();
      await expect(page.getByRole('dialog', { name: 'Licence' })).toBeVisible();

      await looksTheSame(page, 'licence.png');
    });

    test('the confirmation, permanent', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await dialog.getByRole('checkbox', { name: /^Delete permanently instead/ }).check();
      await expect(dialog.getByRole('button', { name: 'Delete permanently' })).toBeVisible();

      await looksTheSame(page, 'cleanup-confirm-permanent.png');
    });

    test('the confirmation with a model that goes with more files', async ({ page }) => {
      const dialog = await ollamaConfirmation(page);
      await dialog.getByRole('button', { name: /^and 3 more files that go with it/ }).click();
      await expect(
        dialog.getByRole('list', { name: 'The other files that go with llama3:8b, llama3:latest' }),
      ).toBeVisible();

      await looksTheSame(page, 'cleanup-confirm-together.png');
    });

    test('the confirmation with an item left out', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningLeftOut);
      const leftOut = dialog.getByRole('region', { name: 'These will not be removed' });
      await leftOut.scrollIntoViewIfNeeded();
      await expect(leftOut).toContainText('Quit Safari and try again.');

      await looksTheSame(page, 'cleanup-confirm-left-out.png');
    });

    test('the confirmation of a High plan, its tick off', async ({ page }) => {
      const dialog = await highConfirmation(page);
      await expect(dialog.getByRole('button', { name: 'Yes, move to the Trash' })).toBeDisabled();

      await looksTheSame(page, 'cleanup-confirm-high.png');
    });

    test('the confirmation of a High plan, ticked', async ({ page }) => {
      const dialog = await highConfirmation(page);
      await dialog.getByRole('checkbox', { name: 'I understand these may be the only copy of my data' }).check();
      await expect(dialog.getByRole('button', { name: 'Yes, move to the Trash' })).toBeEnabled();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'cleanup-confirm-high-ticked.png');
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
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByRole('alert')).toBeVisible();

      await looksTheSame(page, 'cleanup-refused.png');
    });

    test('the progress', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();

      await looksTheSame(page, 'cleanup-progress.png');
    });

    test('the quit question', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();
      await requestQuit(page);
      await expect(page.getByRole('button', { name: 'Keep cleaning' })).toBeFocused();

      await looksTheSame(page, 'cleanup-quit.png');
    });

    test('the result of a move to the Trash', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'cleanup-result.png');
    });

    test('the scan screen with the Trash line', async ({ page }) => {
      await open(page, SCRIPTS.trash);
      await scan(page);
      await expect(page.getByRole('region', { name: 'The Trash' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'scan-trash.png');
    });

    test('the scan screen with two Trash lines', async ({ page }) => {
      await open(page, SCRIPTS.trashTwo);
      await scan(page);
      await expect(page.getByRole('region', { name: 'The Trash' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'scan-trash-two.png');
    });

    test('the help when something was removed that is needed', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaning);
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await page.getByRole('button', { name: 'Removed something you need?' }).click();
      await expect(page.getByRole('dialog', { name: 'Removed something you need?' })).toBeVisible();

      await looksTheSame(page, 'recovery-help.png');
    });

    test('the result of a stopped cleanup', async ({ page }) => {
      const dialog = await confirmation(page, SCRIPTS.cleaningHeld);
      await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await page.getByRole('button', { name: 'Stop' }).click();
      await expect(page.getByText('Stopped. 2 items were not touched.')).toBeVisible();

      await looksTheSame(page, 'cleanup-result-stopped.png');
    });

    test('the largest files after a scan', async ({ page }) => {
      await open(page, SCRIPTS.largest);
      await scan(page);
      await go(page, 'Largest', 'Largest files and folders');
      await expect(page.getByText('ubuntu-24.04.dmg', { exact: true })).toBeVisible();
      await expect(page.getByRole('region', { name: 'Desktop folder' })).toBeVisible();
      await expect(page.getByText('Photos Library.photoslibrary', { exact: true })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'largest.png');
    });

    test('the largest folders after a scan', async ({ page }) => {
      await open(page, SCRIPTS.largest);
      await scan(page);
      await go(page, 'Largest', 'Largest files and folders');
      await page.getByRole('tab', { name: 'Folders' }).click();
      await expect(page.getByRole('list', { name: 'Folders, largest first' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'largest-folders.png');
    });

    test('a picked file in the review, on a tile of the High group', async ({ page }) => {
      await open(page, SCRIPTS.largest);
      await scan(page);
      await go(page, 'Largest', 'Largest files and folders');
      await page.getByRole('checkbox', { name: /^Choose ubuntu-24\.04\.dmg/ }).click();
      await expect(page.getByRole('region', { name: 'Selection' })).toContainText('1 item selected');
      await go(page, /^Review/, 'Review');
      await expect(page.getByRole('article', { name: 'Your own files' })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'largest-review.png');
    });

    test('the largest files before a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await go(page, 'Largest', 'Largest files and folders');
      await expect(page.getByText('Scan first to see your largest files.')).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'largest-no-scan.png');
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

  test.describe('apps', () => {
    const openApp = async (page: Page, script: CoreScript, name: string): Promise<void> => {
      await open(page, script);
      await scan(page);
      await go(page, 'Apps', 'Apps');
      await page
        .getByRole('list', { name: 'Apps, largest first' })
        .getByRole('listitem')
        .filter({ hasText: name })
        .first()
        .click();
      await expect(page.getByRole('heading', { level: 1, name })).toBeFocused();
    };

    test('the list of apps', async ({ page }) => {
      await open(page, SCRIPTS.apps);
      await scan(page);
      await go(page, 'Apps', 'Apps');
      await expect(page.getByRole('list', { name: 'Apps, largest first' })).toContainText('Disk Tool');
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'apps-list.png');
    });

    test('the page of an app that is offered', async ({ page }) => {
      await openApp(page, SCRIPTS.apps, 'Video Studio');
      await expect(page.getByRole('radio', { name: /keep its settings and data/ })).toBeChecked();
      await page.getByRole('button', { name: 'Uninstall…' }).scrollIntoViewIfNeeded();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'app-page-offered.png');
    });

    test('the page of an app that is not offered', async ({ page }) => {
      await openApp(page, SCRIPTS.apps, 'Terminal Pro');
      await expect(page.locator('code')).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'app-page-not-offered.png');
    });

    test('the confirmation of an uninstall with everything', async ({ page }) => {
      await openApp(page, SCRIPTS.apps, 'Video Studio');
      await page.getByRole('radio', { name: /everything it left/ }).check();
      await page.getByRole('button', { name: 'Uninstall…' }).click();
      const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
      await dialog.getByRole('button', { name: /more files that go with it/ }).click();
      await expect(dialog.getByRole('list', { name: /^The other files that go with/ })).toBeVisible();
      await page.mouse.move(0, 0);

      await looksTheSame(page, 'apps-confirmation.png');
    });
  });

  test.describe('settings', () => {
    test('the General page', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await expect(page.getByRole('button', { name: 'Reset all settings…' })).toBeVisible();

      await looksTheSame(page, 'settings.png');
    });

    test('the results of the search', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await page.getByRole('searchbox', { name: 'Search settings' }).fill('days');
      await expect(page.getByRole('heading', { level: 1, name: '1 setting matches “days”' })).toBeVisible();

      await looksTheSame(page, 'settings-search.png');
    });

    test('the page of a module, its switch last', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await settingsPage(page, 'Demo module', 'Debug', 'Demo module');
      await expect(page.getByRole('switch', { name: 'Look for Demo module' })).toBeChecked();

      await looksTheSame(page, 'settings-module.png');
    });

    for (const [agreement, file] of [
      ['accepted', 'settings-about.png'],
      ['not-accepted', 'settings-about-not-accepted.png'],
    ] as const) {
      test(`the About group, agreement ${agreement}`, async ({ page }) => {
        await open(page, { ...SCRIPTS.settings, agreement });
        await go(page, 'Settings', 'General');
        await settingsPage(page, 'About', 'About');
        const about = page.getByRole('region', { name: 'About' });
        await expect(about.getByRole('listitem')).toHaveCount(5);
        await about.scrollIntoViewIfNeeded();

        await expectNoCspViolation(page);
        await expect(about).toHaveScreenshot(file);
      });
    }

    test('the days field of Logs and crash reports', async ({ page }) => {
      await logsSettings(page);
      await expect(page.getByText('From 7 to 90 days')).toBeVisible();

      await looksTheSame(page, 'settings-logs.png');
    });

    test('the number field of Temporary files', async ({ page }) => {
      await tempSettings(page);
      await expect(page.getByText('From 7 to 30 days')).toBeVisible();

      await looksTheSame(page, 'settings-temp.png');
    });

    for (const [tab, file] of [
      ['Sections', 'settings-ignored-groups.png'],
      ['Not looked for', 'settings-ignored-modules.png'],
      ['Items', 'settings-ignored-items.png'],
    ] as const) {
      test(`the Ignored section of Settings, ${tab}`, async ({ page }) => {
        await open(page, SCRIPTS.ignoring);
        await go(page, 'Settings', 'General');
        await settingsPage(page, 'Ignored', 'Ignored');
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
      await go(page, 'Settings', 'General');
      await expect(page.getByText('Your settings were saved by a newer version of Fresher')).toBeVisible();
      await settingsPage(page, 'Demo module', 'Debug', 'Demo module');
      await expect(page.getByRole('switch', { name: 'Look for Demo module' })).toBeDisabled();

      await looksTheSame(page, 'settings-read-only.png');
    });
  });
});
