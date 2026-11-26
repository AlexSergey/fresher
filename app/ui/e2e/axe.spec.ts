import AxeBuilder from '@axe-core/playwright';
import { expect, type Locator, type Page, test } from '@playwright/test';

import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { details, openDetails, scanButton, scanFinished, tickItemsOf } from './support/cards';
import { requestQuit } from './support/cleanup-mock';
import { nextFrames } from './support/frames';
import { motionOff } from './support/motion';
import { settingsPage } from './support/settings';

// The automated checker over every screen in each of its states and every story of the gallery: no violation of
// WCAG 2.1 A and AA. Motion is off, so nothing is checked mid-animation.

const WCAG_21_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

type Violation = { readonly id: string; readonly nodes: readonly string[] };

// Whether the element is the text of an inactive control: a greyed-out row whose own control is disabled. WCAG 1.4.3
// sets no contrast for it; the checker cannot tell, since a row has no disabled state of its own in ARIA.
const inactive = (page: Page, selector: string): Promise<boolean> =>
  page.evaluate(
    (target) => document.querySelector(target)?.closest('[data-disabled]')?.querySelector(':disabled') != null,
    selector,
  );

// The violations of the page (or of `include`), each with the selectors of the elements it concerns.
const violations = async (page: Page, include?: string): Promise<readonly Violation[]> => {
  const builder = new AxeBuilder({ page }).withTags(WCAG_21_AA);
  const results = await (include === undefined ? builder : builder.include(include)).analyze();
  const found: Violation[] = [];

  for (const { id, nodes } of results.violations) {
    const targets: string[] = [];

    for (const { target } of nodes) {
      const selector = target.join(' ');

      if (id !== 'color-contrast' || !(await inactive(page, selector))) {
        targets.push(selector);
      }
    }
    if (targets.length > 0) {
      found.push({ id, nodes: targets });
    }
  }

  return found;
};

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

// The demo scan with one category ticked on its card and one finding on the details of the other.
const selectSome = async (page: Page): Promise<void> => {
  await open(page, SCRIPTS.demo);
  await scan(page, '2.6 GB');
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await openDetails(page, 'Example caches');
  await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
  await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');
};

// Ticks both categories of the demo scan and opens the review.
const reviewAll = async (page: Page, script: CoreScript): Promise<void> => {
  await open(page, script);
  await scan(page, '2.6 GB');
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await go(page, /^Review/, 'Review');
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
  await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeFocused();
  await settingsPage(page, 'Temporary files', 'System', 'Temporary files');
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
  await settingsPage(page, 'Logs and crash reports', 'System', 'Logs and crash reports');
  const days = page.getByRole('spinbutton', { name: 'Offer logs and crash reports unchanged for' });
  await expect(days).toHaveValue('7');
  await days.scrollIntoViewIfNeeded();
};

// Ollama's models ticked, the review open and the confirmation asked for, the first model's other files listed.
// The installers (Low) and the big untouched file (High) of the downloads ticked, and the review open: both parts.
const reviewDownloads = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.downloads);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  await page.getByRole('checkbox', { name: 'Select Old installers' }).check();
  await tickItemsOf(page, 'Big unopened downloads');
  await go(page, /^Review/, 'Review');
};

const ollamaConfirmation = async (page: Page): Promise<Locator> => {
  await mockCore(page, SCRIPTS.localAiModelsOllama);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
  await tickItemsOf(page, 'Ollama models');
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
  await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
  await dialog.getByRole('button', { name: /^and 3 more files that go with it/ }).click();
  await expect(dialog.getByRole('list', { name: /^The other files that go with/ })).toBeVisible();

  return dialog;
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

test.beforeEach(async ({ page }) => {
  await motionOff(page);
});

test.describe('positive cases', () => {
  test.describe('screens', () => {
    test('the Scan screen before a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen during a scan', async ({ page }) => {
      await open(page, SCRIPTS.live);
      await scan(page, '2.3 GB');
      await expect(page.getByRole('progressbar', { name: /^Scanning, \d+ percent$/u })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen after a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen with the Trash line', async ({ page }) => {
      await open(page, SCRIPTS.trashDated);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('region', { name: 'The Trash' })).toContainText('Finder removes them');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen with two Trash lines', async ({ page }) => {
      await open(page, SCRIPTS.trashTwo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('region', { name: 'The Trash' }).getByRole('button')).toHaveCount(2);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen after a partial scan', async ({ page }) => {
      await open(page, SCRIPTS.partial);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('heading', { name: 'Some folders were not scanned' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen after a failed scan', async ({ page }) => {
      await open(page, SCRIPTS.failed);
      // A failed scan shows no size: its error says the end.
      await scanButton(page).click();
      await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen in a sandbox, with its banner', async ({ page }) => {
      await open(page, SCRIPTS.sandboxed);
      await expect(page.getByRole('note')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen of a development build, with its banner', async ({ page }) => {
      await open(page, SCRIPTS.development);
      await expect(page.getByRole('note')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the Scan screen with Full Disk Access denied after a limited scan', async ({ page }) => {
      await open(page, SCRIPTS.denied);
      await scan(page, '2.6 GB');
      await expect(page.getByText('Fresher cannot see every folder')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test("a category's details with a selection", async ({ page }) => {
      await selectSome(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    for (const [script, category, line] of [
      ['jetbrains', 'Old JetBrains IDE settings', '2 items left out (600 MB)'],
      ['temp', 'Old temporary files', '4,712 items left out (790 MB)'],
      ['logs', 'Old app logs', '1 item left out (3 MB)'],
      ['devCachesXcodeOpen', "Gradle's caches", '1 item kept (35 MB)'],
      ['appCaches', 'App caches', '1 item left out (90 MB)'],
      ['appCachesBrowsers', 'Browser caches', '1 item left out (30 MB)'],
      ['appCachesBrowsers', 'Web-based app caches', 'Redis Insight: GPUCache'],
      ['appLeftovers', 'Caches of removed apps', '1 item left out (220 MB)'],
      ['downloads', 'Unfinished downloads', '1 item left out (48 MB)'],
      ['staleProjects', 'Packages of old projects', '2 items left out (1.3 GB)'],
      ['localAiModels', 'Hugging Face models', '1 item kept (300 MB)'],
      ['unusedToolchains', 'Unused Node versions', '1 item kept (110 MB)'],
    ] as const) {
      test(`the details of ${category}, with its left-out lines`, async ({ page }) => {
        await detailsAfter(page, SCRIPTS[script], category);
        await expect(details(page)).toContainText(line);
        await expect(details(page).getByRole('checkbox').first()).toBeVisible();

        expect(await violations(page)).toStrictEqual([]);
        // Opened, the lines' items pass too.
        for (const toggle of await details(page)
          .locator('button[aria-expanded="false"]', { hasText: 'left out' })
          .all()) {
          await toggle.click();
        }
        expect(await violations(page)).toStrictEqual([]);
      });
    }

    test('the Scan screen with the described section', async ({ page }) => {
      await mockCore(page, SCRIPTS.docker);
      await page.goto('/');
      await scanButton(page).click();
      await scanFinished(page);
      const section = page.getByRole('region', { name: 'Shown, never removed' });
      await expect(section).toContainText('OrbStack');
      // The totals have arrived: the layout no longer moves.
      await expect(page.locator('[data-card]')).toHaveCount(4);
      await section.scrollIntoViewIfNeeded();

      expect(await violations(page)).toStrictEqual([]);
    });

    for (const [script, title] of [
      ['docker', 'OrbStack'],
      ['dockerStopped', 'Docker Desktop'],
    ] as const) {
      test(`the insight screen of ${title}`, async ({ page }) => {
        await mockCore(page, SCRIPTS[script]);
        await page.goto('/');
        await scanButton(page).click();
        await scanFinished(page);
        await page
          .getByRole('region', { name: 'Shown, never removed' })
          .getByRole('button', { name: 'Details' })
          .click();
        await expect(page.getByRole('heading', { level: 1, name: title })).toBeFocused();

        expect(await violations(page)).toStrictEqual([]);
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

      expect(await violations(page)).toStrictEqual([]);
      // Opened, its items pass too.
      await leftOut.getByRole('button', { name: /^2 items left out/ }).click();
      // Each item held back is listed by its name and its path (cut in the middle, whole for assistive technology).
      const items = leftOut.getByRole('list', { name: 'The items left out' });
      await expect(items.getByRole('listitem').first()).toBeVisible();
      await expect(items).toContainText('Weather');
      await expect(items).toContainText('/Users/someone/Library/Developer/Xcode/DerivedData/Weather-abcdefghijklmn');
      expect(await violations(page)).toStrictEqual([]);
    });

    test('the details of the Xcode archives', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.xcode, 'Xcode archives');
      await expect(details(page).getByRole('checkbox').first()).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the details of an unused toolchains category before Documents is allowed', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.unusedToolchainsNoDocuments, 'Unused Rust versions');
      await expect(page.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the details of a downloads category before Downloads is allowed', async ({ page }) => {
      await detailsAfter(page, SCRIPTS.downloadsUnavailable, 'Unfinished downloads');
      await expect(page.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings with the days field of Logs and crash reports', async ({ page }) => {
      await logsSettings(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings with the number field of Temporary files', async ({ page }) => {
      await tempSettings(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the confirmation of a cleanup that names Medium', async ({ page }) => {
      await jetbrainsConfirmation(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the confirmation of a model that goes with more files', async ({ page }) => {
      await ollamaConfirmation(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test("the details of Ollama's models", async ({ page }) => {
      await detailsAfter(page, SCRIPTS.localAiModelsOllama, 'Ollama models');
      await expect(details(page).getByRole('checkbox').first()).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the review of a selection', async ({ page }) => {
      await selectSome(page);
      await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the review with a tile of each group, one a Look tile', async ({ page }) => {
      await reviewDownloads(page);
      await expect(page.getByRole('article')).toHaveCount(2);
      await expect(page.getByRole('region', { name: 'Ready to remove' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the confirmation of a High plan, its tick off and on', async ({ page }) => {
      await reviewDownloads(page);
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
      await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);

      await dialog.getByRole('checkbox', { name: 'I understand these may be the only copy of my data' }).check();
      await expect(dialog.getByRole('button', { name: 'Yes, move to the Trash' })).toBeEnabled();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the review with nothing selected', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await go(page, /^Review/, 'Review');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the confirmation of a cleanup', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaningLeftOut);
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await expect(page.getByRole('dialog').getByRole('button', { name: 'Cancel' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the agreement before the first cleanup', async ({ page }) => {
      await reviewAll(page, { ...SCRIPTS.cleaning, agreement: 'not-accepted' });
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await expect(page.getByRole('dialog', { name: 'Before your first cleanup' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the full licence', async ({ page }) => {
      await reviewAll(page, { ...SCRIPTS.cleaning, agreement: 'not-accepted' });
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await page.getByRole('button', { name: 'Read the full licence' }).click();
      await expect(page.getByRole('dialog', { name: 'Licence' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('a cleanup under way, with the quit question', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaningHeld);
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);

      await requestQuit(page);
      await expect(page.getByRole('button', { name: 'Keep cleaning' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the result of a cleanup', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaning);
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the help when something was removed that is needed', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaning);
      await page.getByRole('button', { exact: true, name: 'Move to the Trash' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
      await page.getByRole('button', { name: 'Removed something you need?' }).click();
      await expect(page.getByRole('dialog', { name: 'Removed something you need?' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the largest files', async ({ page }) => {
      await open(page, SCRIPTS.largest);
      await scan(page, '2.6 GB');
      await go(page, 'Largest', 'Largest files and folders');
      await expect(page.getByText('ubuntu-24.04.dmg', { exact: true })).toBeVisible();
      await expect(page.getByRole('region', { name: 'Desktop folder' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the largest folders', async ({ page }) => {
      await open(page, SCRIPTS.largest);
      await scan(page, '2.6 GB');
      await go(page, 'Largest', 'Largest files and folders');
      await page.getByRole('tab', { name: 'Folders' }).click();
      await expect(page.getByRole('list', { name: 'Folders, largest first' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the apps and an app’s page', async ({ page }) => {
      await open(page, SCRIPTS.apps);
      await scan(page, '1.1 GB');
      await go(page, 'Apps', 'Apps');
      await expect(page.getByRole('list', { name: 'Apps, largest first' })).toContainText('Disk Tool');

      expect(await violations(page)).toStrictEqual([]);

      await page.getByRole('listitem').filter({ hasText: 'Video Studio' }).first().click();
      await expect(page.getByRole('heading', { level: 1, name: 'Video Studio' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('an app that is not offered and the confirmation of an uninstall', async ({ page }) => {
      await open(page, SCRIPTS.appsDenied);
      await scan(page, '1.1 GB');
      await go(page, 'Apps', 'Apps');
      await page.getByRole('listitem').filter({ hasText: 'Terminal Pro' }).first().click();
      await expect(page.getByRole('heading', { level: 1, name: 'Terminal Pro' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);

      await page.getByRole('button', { name: 'All apps' }).click();
      await page.getByRole('listitem').filter({ hasText: 'Video Studio' }).first().click();
      await expect(page.getByRole('button', { name: 'Open System Settings' })).toBeVisible();
      await page.getByRole('button', { name: 'Uninstall…' }).click();
      await expect(page.getByRole('dialog')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the cleanups and one cleanup', async ({ page }) => {
      await open(page, SCRIPTS.cleaning);
      await go(page, 'Cleanups', 'Cleanups');
      await expect(page.getByText('Waiting in the Trash from cleanups: 4.1 GB')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);

      await page.getByRole('button', { name: /^Cleanup of Dec 31, 2024/ }).click();
      await expect(page.getByText(/Items marked Unknown/)).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the General page of the settings', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await expect(page.getByRole('button', { name: 'Reset all settings…' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the page of a module in the settings', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await settingsPage(page, 'Demo module', 'Debug', 'Demo module');
      await expect(page.getByRole('switch', { name: 'Look for Demo module' })).toBeChecked();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the results of the settings search, the tree counting and dimming', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'General');
      await page.getByRole('searchbox', { name: 'Search settings' }).fill('days');
      await expect(page.getByRole('heading', { level: 1, name: '1 setting matches “days”' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the About group, with the agreement not accepted', async ({ page }) => {
      await open(page, { ...SCRIPTS.settings, agreement: 'not-accepted' });
      await go(page, 'Settings', 'General');
      await settingsPage(page, 'About', 'About');
      await expect(page.getByText('Not accepted yet. Fresher asks before your first cleanup.')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    for (const tab of ['Sections', 'Not looked for', 'Items']) {
      test(`the Ignored section of Settings, ${tab}`, async ({ page }) => {
        await open(page, SCRIPTS.ignoring);
        await go(page, 'Settings', 'General');
        await settingsPage(page, 'Ignored', 'Ignored');
        await page.getByRole('switch', { name: 'Ignore Debug' }).click();
        await page.getByRole('tab', { name: new RegExp(`^${tab}`) }).click();
        await expect(page.getByRole('tab', { name: new RegExp(`^${tab}`) })).toHaveAttribute('aria-selected', 'true');

        expect(await violations(page)).toStrictEqual([]);
      });
    }

    test('the settings a newer version saved', async ({ page }) => {
      await open(page, SCRIPTS.readOnly);
      await go(page, 'Settings', 'General');
      await settingsPage(page, 'Demo module', 'Debug', 'Demo module');
      await expect(page.getByRole('switch', { name: 'Look for Demo module' })).toBeDisabled();

      expect(await violations(page)).toStrictEqual([]);
    });
  });

  // The stories are shared among this many tests, which run in parallel.
  const PARTS = 4;

  for (let part = 0; part < PARTS; part += 1) {
    test(`every story of the gallery, part ${String(part + 1)}`, async ({ page }) => {
      test.setTimeout(240_000);
      await page.goto('/gallery.html');
      const ids = await page.evaluate(() => (window as { __stories__?: readonly string[] }).__stories__ ?? []);

      expect(ids.length).toBeGreaterThan(0);

      for (const id of ids.filter((_, index) => index % PARTS === part)) {
        await test.step(id, async () => {
          const [component = '', state = ''] = id.split('/');
          await page.goto('about:blank');
          await page.goto(`/gallery.html#/${encodeURIComponent(component)}/${encodeURIComponent(state)}`);
          const selector = `[data-story="${id}"]`;
          await expect(page.locator(selector)).toBeVisible();
          await nextFrames(page);

          expect.soft(await violations(page, selector), id).toStrictEqual([]);
        });
      }
    });
  }
});
