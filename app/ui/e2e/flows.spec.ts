import { expect, type Locator, type Page, test } from '@playwright/test';

import { COMMANDS } from '../src/ipc';
import { APP_CACHES_DICTIONARY } from '../src/test/app-caches-scan';
import { APP_LEFTOVERS_DICTIONARY } from '../src/test/app-leftovers-scan';
import { CLEANUP_DICTIONARY, PLAN_GONE, PLAN_MISMATCH } from '../src/test/cleanup-plan';
import { DEV_CACHES_DICTIONARY } from '../src/test/dev-caches-scan';
import { DOCKER_DICTIONARY } from '../src/test/docker-scan';
import { DOWNLOADS_DICTIONARY } from '../src/test/downloads-scan';
import { JETBRAINS_DICTIONARY } from '../src/test/jetbrains-scan';
import { LOCAL_AI_MODELS_DICTIONARY } from '../src/test/local-ai-models-scan';
import { LOGS_DICTIONARY } from '../src/test/logs-scan';
import { STALE_PROJECTS_DICTIONARY } from '../src/test/stale-projects-scan';
import { TEMP_DICTIONARY } from '../src/test/temp-scan';
import { UNUSED_TOOLCHAINS_DICTIONARY } from '../src/test/unused-toolchains-scan';
import { XCODE_DICTIONARY } from '../src/test/xcode-scan';
import { mockCore, releaseScan, SCRIPTS } from './core-mock';
import {
  cardNames,
  details,
  detailsOf,
  fromCardMenu,
  navScan,
  openDetails,
  scanButton,
  scanFinished,
} from './support/cards';
import { releaseCleanup, requestQuit } from './support/cleanup-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The user's paths through the screens on the mocked core, by pointer and by keyboard alone.

const BUSY_TEXT = 'The app is already running a scan. Wait for it to finish, or cancel it.';

// Presses Tab until `target` has the focus: the keyboard's way to a control, whatever comes before it.
const tabTo = async (page: Page, target: Locator): Promise<void> => {
  for (let presses = 0; presses < 40; presses += 1) {
    if (await target.evaluate((element) => element === document.activeElement)) {
      return;
    }
    await page.keyboard.press('Tab');
  }
  await expect(target).toBeFocused();
};

const heading = (page: Page, name: string): Locator => page.getByRole('heading', { level: 1, name });

const cards = (page: Page): Locator => page.locator('[data-card]');

const zone = (page: Page, name: string): Locator => page.getByRole('region', { name });

const selectionBar = (page: Page): Locator => page.getByRole('region', { name: 'Selection' });

const removeButton = (page: Page): Locator => page.getByRole('button', { name: 'Move to Trash…' });

const notifications = (page: Page): Locator => page.getByRole('region', { name: 'Notifications' });

const QUIT = 'A cleanup is running. Stop after the current item and quit?';

// Scans, ticks both categories and opens the review: the demo plan's three findings.
const reviewAll = async (page: Page): Promise<void> => {
  await page.goto('/');
  await scanButton(page).click();
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await selectionBar(page).getByRole('button', { name: 'Review' }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

// Scans the demo, then opens Settings (the Ignored tabs).
const openIgnoring = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.ignoring);
  await page.goto('/');
  await scanButton(page).click();
  await expect(cards(page)).toHaveCount(2);
};

const openSettings = (page: Page): Promise<void> => page.getByRole('button', { exact: true, name: 'Settings' }).click();

const JB = {
  caches: 'Caches of older JetBrains IDE versions',
  logs: 'Logs of older JetBrains IDE versions',
  settings: 'Settings and plugins of older JetBrains IDE versions',
  uninstalled: 'Data of JetBrains IDEs that are no longer installed',
} as const;

// The module's own texts, as its locale file has them.
const jb = (key: string): string => JETBRAINS_DICTIONARY[`jetbrains.${key}`] ?? '';

// Scans `script` and waits on the overview for its end.
const scanned = async (page: Page, script: (typeof SCRIPTS)[keyof typeof SCRIPTS]): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
};

// Scans the JetBrains script.
const openJetbrains = (page: Page): Promise<void> => scanned(page, SCRIPTS.jetbrains);

// Ticks WebStorm's three categories on their cards and opens the review.
const reviewWebStorm = async (page: Page): Promise<void> => {
  await openJetbrains(page);
  for (const name of [JB.caches, JB.logs, JB.settings]) {
    await page.getByRole('checkbox', { name: `Select ${name}` }).check();
  }
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

// The temporary files module's own texts, as its locale file has them.
const tmp = (key: string): string => TEMP_DICTIONARY[`temp.${key}`] ?? '';

const TEMP_CATEGORY = tmp('old-files.name');

// The logs module's own texts, as its locale file has them.
const lg = (key: string): string => LOGS_DICTIONARY[`logs.${key}`] ?? '';

const LOGS_CATEGORY = lg('app-logs.name');

// Counts the setting updates the page sends to the core from here on.
const recordUpdates = (page: Page): Promise<void> =>
  page.evaluate((command) => {
    const internals = (globalThis as unknown as { __TAURI_INTERNALS__: { invoke: (...args: unknown[]) => unknown } })
      .__TAURI_INTERNALS__;
    const invoke = internals.invoke.bind(internals);
    const counter = { sent: 0 };

    (globalThis as unknown as { __updates: typeof counter }).__updates = counter;
    internals.invoke = (name: unknown, ...rest: unknown[]): unknown => {
      counter.sent += name === command ? 1 : 0;

      return invoke(name, ...rest);
    };
  }, COMMANDS.updateSetting);

const sentUpdates = (page: Page): Promise<number> =>
  page.evaluate(() => (globalThis as unknown as { __updates: { sent: number } }).__updates.sent);

// Scans the temporary files script.
const openTemp = (page: Page): Promise<void> => scanned(page, SCRIPTS.temp);

// Scans the logs script.
const openLogs = (page: Page): Promise<void> => scanned(page, SCRIPTS.logs);

// The developer caches module's own texts, as its locale file has them.
const dev = (key: string): string => DEV_CACHES_DICTIONARY[`dev-caches.${key}`] ?? '';

const NPM = dev('npm.name');
const CARGO = dev('cargo.name');
const GRADLE = dev('gradle.name');
const XCODE_DATA = dev('xcode-derived-data.name');

// Scans a developer caches script (Xcode closed unless said).
const openDevCaches = (
  page: Page,
  script: typeof SCRIPTS.devCaches | typeof SCRIPTS.devCachesXcodeOpen = SCRIPTS.devCaches,
): Promise<void> => scanned(page, script);

// The removed apps module's own texts, as its locale file has them.
const left = (key: string): string => APP_LEFTOVERS_DICTIONARY[`app-leftovers.${key}`] ?? '';

const LEFTOVERS_CACHES = left('caches.name');
const LEFTOVERS_SETTINGS = left('settings.name');
const LEFTOVERS_DATA = left('data.name');

// Scans a removed apps script.
const openLeftovers = (
  page: Page,
  script: typeof SCRIPTS.appLeftovers | typeof SCRIPTS.appLeftoversWithDevCaches = SCRIPTS.appLeftovers,
): Promise<void> => scanned(page, script);

// The downloads module's own texts, as its locale file has them.
const dl = (key: string): string => DOWNLOADS_DICTIONARY[`downloads.${key}`] ?? '';

const DL_UNFINISHED = dl('unfinished.name');
const DL_SAFARI = dl('safari-resumable.name');
const DL_INSTALLERS = dl('installers.name');
const DL_MACOS = dl('macos-installers.name');
const DL_LARGE = dl('large-untouched.name');

// Scans a downloads script.
const openDownloads = (
  page: Page,
  script:
    | typeof SCRIPTS.downloads
    | typeof SCRIPTS.downloadsUnavailable
    | typeof SCRIPTS.downloadsUnfinished = SCRIPTS.downloads,
): Promise<void> => scanned(page, script);

// The old-projects module's own texts, as its locale file has them.
const sp = (key: string): string => {
  const value = STALE_PROJECTS_DICTIONARY[`stale-projects.${key}`];
  if (value === undefined) {
    throw new Error(`no text stale-projects.${key}`);
  }

  return value;
};

const SP_DEPENDENCIES = sp('dependencies.name');
const SP_BUILD = sp('build-output.name');
const SP_PYTHON = sp('python-environments.name');
const SP_RUST = sp('rust-build-cache.name');

// Scans the old-projects script.
const openStaleProjects = (page: Page): Promise<void> => scanned(page, SCRIPTS.staleProjects);

// The local AI models module's own texts, as its locale file has them.
const ai = (key: string): string => {
  const value = LOCAL_AI_MODELS_DICTIONARY[`local-ai-models.${key}`];
  if (value === undefined) {
    throw new Error(`no text local-ai-models.${key}`);
  }

  return value;
};

const AI_HF_MODELS = ai('hugging-face-models.name');
const AI_HF_DATASETS = ai('hugging-face-datasets.name');
const AI_LM_STUDIO = ai('lm-studio.name');
const AI_OLLAMA = ai('ollama.name');

// Scans the local AI models script.
const openLocalAiModels = (
  page: Page,
  script: typeof SCRIPTS.localAiModels | typeof SCRIPTS.localAiModelsLmStudioOpen = SCRIPTS.localAiModels,
): Promise<void> => scanned(page, script);

// The installed app caches module's own texts, as its locale file has them.
const ac = (key: string): string => {
  const value = APP_CACHES_DICTIONARY[`app-caches.${key}`];
  if (value === undefined) {
    throw new Error(`no text app-caches.${key}`);
  }

  return value;
};

const AC_NAME = ac('library-caches.name');
const AC_BROWSERS = ac('browsers.name');
const AC_WEB_ENGINE = ac('web-engine.name');

// The installed app caches script, scanned.
const openAppCaches = (page: Page): Promise<void> => scanned(page, SCRIPTS.appCaches);

// The browser and web-engine caches script, scanned.
const openBrowserCaches = (page: Page): Promise<void> => scanned(page, SCRIPTS.appCachesBrowsers);

// The Xcode module's own texts, as its locale file has them.
const xc = (key: string): string => {
  const value = XCODE_DICTIONARY[`xcode.${key}`];
  if (value === undefined) {
    throw new Error(`no text xcode.${key}`);
  }

  return value;
};

const XC_DEVICE_SUPPORT = xc('device-support.name');
const XC_PREVIEWS = xc('previews.name');
const XC_SIMULATOR = xc('simulator-caches.name');
const XC_ARCHIVES = xc('archives.name');

// The docker module's own texts, as its locale file has them.
const dk = (key: string): string => DOCKER_DICTIONARY[`docker.${key}`] ?? '';

const DESCRIBED = 'Described, not cleaned';

const described = (page: Page): Locator => page.getByRole('region', { name: DESCRIBED });

// Scans the Xcode script with the docker module describing OrbStack.
const openDocker = (page: Page): Promise<void> => scanned(page, SCRIPTS.docker);

// Opens the insight screen of the one card of the section, the screen's title taking the focus.
const openInsight = async (page: Page, title: string): Promise<void> => {
  await described(page).getByRole('button', { name: 'Details' }).click();
  await expect(page.getByRole('heading', { level: 1, name: title })).toBeFocused();
};

const COPIED_LINE = 'docker --context orbstack builder prune --all';

const insightScreen = (page: Page): Locator => page.locator('[data-screen^="insight:"]:not([aria-hidden])');

// The Xcode script, scanned.
const openXcode = (page: Page): Promise<void> => scanned(page, SCRIPTS.xcode);

// The unused toolchains module's own texts, as its locale file has them.
const ut = (key: string): string => {
  const value = UNUSED_TOOLCHAINS_DICTIONARY[`unused-toolchains.${key}`];
  if (value === undefined) {
    throw new Error(`no text unused-toolchains.${key}`);
  }

  return value;
};

const UT_RUST = ut('rust.name');
const UT_NODE = ut('node.name');

// Scans the unused toolchains script.
const openUnusedToolchains = (
  page: Page,
  script: typeof SCRIPTS.unusedToolchains | typeof SCRIPTS.unusedToolchainsNoDocuments = SCRIPTS.unusedToolchains,
): Promise<void> => scanned(page, script);

const choice = (page: Page, name: string): Locator => page.getByRole('button', { name: left(`choice.${name}.label`) });

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

const trashLine = (page: Page): Locator => page.getByRole('region', { name: 'The Trash' });

const openedTrashes = (page: Page): Promise<readonly number[]> =>
  page.evaluate(() => (globalThis as unknown as { __openedTrash: number[] }).__openedTrash);

test.describe('negative cases', () => {
  test('an empty Trash shows no Trash line', async ({ page }) => {
    await scanned(page, SCRIPTS.demo);

    await expect(trashLine(page)).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Open Trash' })).toHaveCount(0);
  });

  test('the Trash line offers no way to empty the Trash', async ({ page }) => {
    await scanned(page, SCRIPTS.trashTwo);

    await expect(trashLine(page).getByRole('button')).toHaveText(['Open Trash', 'Open Trash']);
    await expect(page.getByRole('button', { name: /empty/i })).toHaveCount(0);
    await expect(page.getByRole('link', { name: /empty/i })).toHaveCount(0);
  });

  test('the result offers no way to empty the Trash', async ({ page }) => {
    await mockCore(page, SCRIPTS.trashCleaning);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();

    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();
    await expect(page.getByRole('button', { name: /empty/i })).toHaveCount(0);
  });

  test('a Trash that was not measured says so and shows only the share', async ({ page }) => {
    await scanned(page, SCRIPTS.trashNotMeasured);

    await expect(trashLine(page)).toContainText(
      'Waiting in the Trash from cleanups here: 1.2 GB. The rest of the Trash was not measured.',
    );
    await expect(trashLine(page)).not.toContainText('of it from cleanups here');
  });

  test('nothing ignored shows no ignored line on the overview', async ({ page }) => {
    await openIgnoring(page);

    await expect(page.getByText(/you ignored is left out/)).toHaveCount(0);
    await expect(cardNames(page)).toHaveText(['Example caches', 'Example downloads']);
  });

  test("Back from a category's details returns to the overview with the selection kept", async ({ page }) => {
    await scanned(page, SCRIPTS.demo);
    await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
    await openDetails(page, 'Example caches');
    await details(page).getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');

    await page.getByRole('button', { name: 'Back to Scan' }).click();

    await expect(detailsOf(page, 'Example caches')).toBeFocused();
    await expect(page.getByRole('checkbox', { name: 'Select Example downloads' })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked({ indeterminate: true });
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('a second Scan while the core runs one shows its refusal', async ({ page }) => {
    await mockCore(page, SCRIPTS.unannounced);
    await page.goto('/');
    const scan = scanButton(page);

    await scan.click();
    await scan.click();

    await expect(notifications(page)).toContainText(BUSY_TEXT);
    await releaseScan(page);
    await expect(cards(page)).toHaveCount(2);
  });

  for (const [name, refusal] of [
    ['an expired plan', PLAN_GONE],
    ['a refused echo', PLAN_MISMATCH],
  ] as const) {
    test(`${name} shows the core’s text and the way to review again`, async ({ page }) => {
      await mockCore(page, { ...SCRIPTS.cleaning, refusals: { confirmCleanup: refusal } });
      await reviewAll(page);

      await removeButton(page).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();

      const notice = page.getByRole('alert');
      await expect(notice).toContainText(CLEANUP_DICTIONARY[refusal.text.key] ?? '');
      await notice.getByRole('button', { name: 'Review it again' }).click();
      await expect(notice).toHaveCount(0);
      await expect(removeButton(page)).toBeEnabled();
      await expect(page.getByRole('list', { name: 'The items of Example caches' })).toBeVisible();
    });
  }

  test('the folders of an open IDE are not offered as items, only counted on a line', async ({ page }) => {
    await openJetbrains(page);
    await openDetails(page, JB.settings);

    const settings = details(page);
    await expect(settings).toContainText('1 item');
    await expect(settings).toContainText(`2 items left out (600 MB): ${jb('left-out.running')}`);
    await expect(settings.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
  });

  test('caches and logs alone are confirmed as Safe, with no Medium and no settings warning', async ({ page }) => {
    await openJetbrains(page);
    await page.getByRole('checkbox', { name: `Select ${JB.caches}` }).check();
    await page.getByRole('checkbox', { name: `Select ${JB.logs}` }).check();
    await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();

    await removeButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog).toContainText('Safe');
    await expect(dialog).not.toContainText('Medium');
    await expect(dialog).not.toContainText('custom keymaps');
  });

  test('closing the window during a cleanup asks, and Keep cleaning changes nothing', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaningHeld);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();
    await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();

    await requestQuit(page);
    const question = page.getByRole('dialog', { name: QUIT });
    await expect(question.getByRole('button', { name: 'Keep cleaning' })).toBeFocused();
    await question.getByRole('button', { name: 'Keep cleaning' }).click();

    await expect(question).toHaveCount(0);
    await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();
    await releaseCleanup(page);
    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();
  });

  test('the left-out lines of the temporary files are not items: nothing on them can be selected', async ({ page }) => {
    await openTemp(page);
    await openDetails(page, TEMP_CATEGORY);

    const category = details(page);
    await expect(category).toContainText(`4,712 items left out (790 MB): ${tmp('left-out.recent')}`);
    await expect(category.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await category.getByRole('checkbox', { name: `Select all ${TEMP_CATEGORY}` }).check();
    await expect(selectionBar(page)).toContainText('5 items selected');
    await expect(selectionBar(page)).not.toContainText('4,712');
  });

  test('the left-out line of the logs is not an item: nothing on it can be selected', async ({ page }) => {
    await openLogs(page);
    await openDetails(page, LOGS_CATEGORY);

    const category = details(page);
    await expect(category).toContainText(`1 item left out (3 MB): ${lg('left-out.recent')}`);
    await expect(category.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await category.getByRole('checkbox', { name: `Select all ${LOGS_CATEGORY}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('Select all of the Safe zone ticks nothing in the other zones', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await scanButton(page).click();
    await expect(cards(page)).toHaveCount(2);

    await zone(page, 'Safe to remove').getByRole('checkbox', { name: 'Select all' }).check();

    await expect(page.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: 'Select Example downloads' })).not.toBeChecked();
    await expect(zone(page, 'Low risk').getByRole('checkbox', { name: 'Select all' })).not.toBeChecked();
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('a value of 6 days is refused with a message and the saved value stays 7', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openSettings(page);
    const days = page.getByRole('spinbutton', { name: tmp('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await recordUpdates(page);

    await days.fill('6');

    await expect(page.getByText('Enter a whole number from 7 to 30.')).toBeVisible();
    expect(await sentUpdates(page)).toBe(0);
  });

  test('the left-out lines of developer caches cannot be selected', async ({ page }) => {
    await openDevCaches(page, SCRIPTS.devCachesXcodeOpen);
    // Xcode's build data, all left out while Xcode is open, has no card: it is listed as found but left out.
    await expect(cards(page).filter({ hasText: XCODE_DATA })).toHaveCount(0);
    const leftOut = page.getByRole('region', { name: 'Found, but left out' });
    await expect(leftOut).toContainText(`2 items left out (4.3 GB): ${dev('left-out.app-open')}`);
    await expect(leftOut.getByRole('checkbox')).toHaveCount(0);
    await openDetails(page, XCODE_DATA);
    await expect(details(page)).toContainText(`2 items left out (4.3 GB): ${dev('left-out.app-open')}`);
    await expect(details(page)).toContainText('Everything found here was left out: the lines above say why.');
    await expect(details(page).getByRole('checkbox')).toHaveCount(0);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, GRADLE);

    await expect(details(page)).toContainText(`1 item left out (35 MB): ${dev('left-out.not-yours')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${GRADLE}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await expect(selectionBar(page)).not.toContainText('4.3 GB');
  });

  test('the left-out lines of removed apps cannot be selected', async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_SETTINGS);
    await expect(details(page)).toContainText(`1 item left out (3 MB): ${left('left-out.used-recently')}`);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, LEFTOVERS_CACHES);

    await expect(details(page)).toContainText(`1 item left out (220 MB): ${left('left-out.running')}`);
    await expect(details(page)).not.toContainText(left('left-out.used-recently'));
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${LEFTOVERS_CACHES}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('220 MB');
  });

  test('the downloads module is unavailable until Downloads is allowed, and says how to allow it', async ({ page }) => {
    await openDownloads(page, SCRIPTS.downloadsUnavailable);

    for (const name of [DL_UNFINISHED, DL_SAFARI, DL_INSTALLERS, DL_MACOS, DL_LARGE]) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category.getByRole('region', { name: 'Not available' })).toContainText(
        'Your Downloads folder was not looked at.',
      );
      await expect(category.getByRole('checkbox')).toHaveCount(0);
      await expect(category.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('the left-out lines of downloads cannot be selected', async ({ page }) => {
    await openDownloads(page);
    await openDetails(page, DL_UNFINISHED);
    await expect(details(page)).toContainText(`1 item left out (48 MB): ${dl('left-out.browser-open')}`);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, DL_LARGE);

    await expect(details(page)).toContainText(`1 item left out (1.2 GB): ${dl('left-out.recent')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${DL_LARGE}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await expect(selectionBar(page)).not.toContainText('1.2 GB');
  });

  test('the left-out lines of old projects cannot be selected', async ({ page }) => {
    await openStaleProjects(page);
    for (const [name, line] of [
      [SP_BUILD, `1 item left out (900 MB): ${sp('left-out.recent')}`],
      [SP_RUST, `1 item left out (2.1 GB): ${sp('left-out.cache-recent')}`],
    ] as const) {
      await openDetails(page, name);
      await expect(details(page)).toContainText(line);
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
    await openDetails(page, SP_DEPENDENCIES);

    await expect(details(page)).toContainText(`2 items left out (1.3 GB): ${sp('left-out.recent')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${SP_DEPENDENCIES}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('1.3 GB');
  });

  test("an open app's cache cannot be selected", async ({ page }) => {
    await openAppCaches(page);
    await openDetails(page, AC_NAME);

    await expect(details(page)).toContainText(`1 item left out (90 MB): ${ac('left-out.app-open')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${AC_NAME}` }).check();
    await expect(selectionBar(page)).toContainText('3 items selected');
    await expect(selectionBar(page)).not.toContainText('90 MB');
  });

  test('a local AI model kept while LM Studio is open cannot be selected', async ({ page }) => {
    await openLocalAiModels(page, SCRIPTS.localAiModelsLmStudioOpen);
    // LM Studio's model, kept while LM Studio is open, has no card: it is listed as found but left out.
    await expect(cards(page).filter({ hasText: AI_LM_STUDIO })).toHaveCount(0);
    const leftOut = page.getByRole('region', { name: 'Found, but left out' });
    await expect(leftOut).toContainText(`1 item left out (4.7 GB): ${ai('left-out.app-open')}`);
    await expect(leftOut.getByRole('checkbox')).toHaveCount(0);
    await openDetails(page, AI_LM_STUDIO);
    await expect(details(page)).toContainText(`1 item left out (4.7 GB): ${ai('left-out.app-open')}`);
    await expect(details(page)).toContainText('Everything found here was left out: the lines above say why.');
    await expect(details(page).getByRole('checkbox')).toHaveCount(0);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, AI_HF_MODELS);

    await expect(details(page)).toContainText(`1 item left out (300 MB): ${ai('left-out.not-yours')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${AI_HF_MODELS}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('300 MB');
  });

  test('while Ollama downloads, its card offers nothing and says why', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllamaDownload);

    await expect(cards(page).filter({ hasText: AI_OLLAMA })).toHaveCount(0);
    const leftOut = page.getByRole('region', { name: 'Found, but left out' });
    await expect(leftOut).toContainText(`1 item left out (6.3 GB): ${ai('left-out.ollama-download')}`);
    await expect(leftOut.getByRole('checkbox')).toHaveCount(0);
    await openDetails(page, AI_OLLAMA);
    await expect(details(page)).toContainText(`1 item left out (6.3 GB): ${ai('left-out.ollama-download')}`);
    await expect(details(page).getByRole('checkbox')).toHaveCount(0);
  });

  test('a Node version not owned by you is kept and cannot be selected', async ({ page }) => {
    await openUnusedToolchains(page);

    await openDetails(page, UT_NODE);

    await expect(details(page)).toContainText(`1 item left out (110 MB): ${ut('left-out.not-yours')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${UT_NODE}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await expect(selectionBar(page)).not.toContainText('110 MB');
  });

  test('without access to Documents the unused toolchains categories say what they need', async ({ page }) => {
    await openUnusedToolchains(page, SCRIPTS.unusedToolchainsNoDocuments);

    for (const name of [UT_RUST, UT_NODE]) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category.getByRole('region', { name: 'Not available' })).toContainText(
        'Your Documents folder was not looked at.',
      );
      await expect(category.getByRole('checkbox')).toHaveCount(0);
      await expect(category.getByRole('button', { name: 'Let Fresher look there' })).toBeVisible();
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('a choice selects nothing outside its module', async ({ page }) => {
    await openLeftovers(page, SCRIPTS.appLeftoversWithDevCaches);
    await page.getByRole('checkbox', { name: `Select ${NPM}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await openDetails(page, LEFTOVERS_CACHES);

    await choice(page, 'keep-settings').click();

    await expect(selectionBar(page)).toContainText('3 items selected');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await expect(page.getByRole('checkbox', { name: `Select ${NPM}` })).toBeChecked();
  });

  test('an ignored cache category is in no total', async ({ page }) => {
    await openDevCaches(page);

    await fromCardMenu(page, NPM, 'Ignore this category');

    await expect(cards(page).filter({ hasText: NPM })).toHaveCount(0);
    await expect(page.getByText('8.3 GB').first()).toBeVisible();
    await expect(page.getByText('17.2 GB')).toHaveCount(0);
  });

  test('ignoring the temporary files module puts its findings in no total', async ({ page }) => {
    await openTemp(page);
    await page.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();

    await fromCardMenu(page, TEMP_CATEGORY, `Ignore ${tmp('enabled.label')}`);

    await expect(cards(page)).toHaveCount(0);
    await expect(page.getByRole('button', { name: /selected/ })).toHaveCount(0);
  });

  test("the card's menu closes on Escape and ignores nothing", async ({ page }) => {
    await openIgnoring(page);
    const options = page.getByRole('button', { name: 'Options for Example caches' });

    await options.click();
    await expect(page.getByRole('menu', { name: 'Options for Example caches' })).toBeVisible();
    await page.keyboard.press('Escape');

    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(options).toBeFocused();
    await expect(cards(page)).toHaveCount(2);
    await expect(page.getByText(/you ignored/u)).toHaveCount(0);
  });

  test("the row's menu closes on Escape and ignores nothing", async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');
    const options = page.getByRole('button', { name: 'Options for Example: editor-cache' });

    await options.click();
    await expect(page.getByRole('menuitem', { name: 'Ignore this item' })).toBeVisible();
    await page.keyboard.press('Escape');

    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(options).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example caches' }).getByRole('listitem')).toHaveCount(2);
    const recorded = await page.evaluate(() => sessionStorage.getItem('fresher.mock.ignore'));
    expect(JSON.parse(recorded ?? 'null')).toBeNull();
  });

  test("Review's rows have no menu", async ({ page }) => {
    await openTemp(page);
    await page.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();

    await selectionBar(page).getByRole('button', { name: 'Review' }).click();

    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: `The items of ${TEMP_CATEGORY}` })).toBeVisible();
    await expect(page.getByRole('button', { name: /^Options for / })).toHaveCount(0);
  });

  test("nothing of Xcode's is selected before the user picks it", async ({ page }) => {
    await openXcode(page);

    for (const name of [XC_DEVICE_SUPPORT, XC_PREVIEWS, XC_SIMULATOR, XC_ARCHIVES]) {
      await expect(page.getByRole('checkbox', { name: `Select ${name}` })).not.toBeChecked();
    }
    await expect(selectionBar(page)).toHaveCount(0);
    await zone(page, 'Safe to remove').getByRole('checkbox', { name: 'Select all' }).check();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_PREVIEWS}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_SIMULATOR}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_DEVICE_SUPPORT}` })).not.toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_ARCHIVES}` })).not.toBeChecked();
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('a left-out device support folder cannot be selected', async ({ page }) => {
    await openXcode(page);

    await openDetails(page, XC_DEVICE_SUPPORT);

    await expect(details(page)).toContainText(`1 item left out (300 MB): ${xc('left-out.special')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select all ${XC_DEVICE_SUPPORT}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('300 MB');
  });

  test('the described section has no checkbox and adds nothing to the selection', async ({ page }) => {
    await openDocker(page);

    await expect(described(page).getByRole('checkbox')).toHaveCount(0);
    for (const name of ['Safe to remove', 'Low risk', 'High risk']) {
      await zone(page, name).getByRole('checkbox', { name: 'Select all' }).check();
    }

    // Xcode's five findings are the whole selection; the section adds none.
    await expect(selectionBar(page)).toContainText('5 items selected');
  });

  test('the insight screen has no selection bar and Review never lists Docker', async ({ page }) => {
    await openDocker(page);
    for (const name of ['Safe to remove', 'Low risk', 'High risk']) {
      await zone(page, name).getByRole('checkbox', { name: 'Select all' }).check();
    }
    await openInsight(page, 'OrbStack');

    await expect(insightScreen(page).getByRole('checkbox')).toHaveCount(0);
    await expect(selectionBar(page)).toHaveCount(0);
    await page
      .getByRole('button', { name: /^Review/ })
      .first()
      .click();
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('main')).not.toContainText(/orbstack|docker/i);
  });

  test('a part without a command has no Copy button', async ({ page }) => {
    await openDocker(page);
    await openInsight(page, 'OrbStack');

    const containers = insightScreen(page)
      .getByRole('listitem')
      .filter({ hasText: dk('part.containers.label') });
    await expect(containers).toContainText(dk('part.containers.none'));
    await expect(containers.getByRole('button', { name: 'Copy' })).toHaveCount(0);
    await expect(containers.locator('code')).toHaveCount(0);
    await expect(insightScreen(page).getByRole('button', { name: 'Copy' })).toHaveCount(3);
  });

  test('the section is absent before a scan and while scanning', async ({ page }) => {
    await mockCore(page, SCRIPTS.dockerHeld);
    await page.goto('/');
    await expect(page.getByRole('heading', { level: 2, name: DESCRIBED })).toHaveCount(0);

    await scanButton(page).click();
    // The docker module has finished, the scan has not.
    await expect(cards(page)).toHaveCount(4);
    await expect(page.getByRole('heading', { level: 2, name: DESCRIBED })).toHaveCount(0);

    await releaseScan(page);
    await scanFinished(page);
    await expect(described(page)).toBeVisible();
  });
});

test.describe('positive cases', () => {
  test('the scan screen shows what waits in the Trash and how much is from cleanups here', async ({ page }) => {
    await scanned(page, SCRIPTS.trash);

    await expect(trashLine(page)).toContainText('Waiting in the Trash: 23 GB, 20 GB of it from cleanups here.');
    await expect(trashLine(page)).not.toContainText('Finder removes them');
  });

  test('Open Trash asks the core to open that Trash', async ({ page }) => {
    await scanned(page, SCRIPTS.trashTwo);

    await trashLine(page).getByRole('button', { name: 'Open Trash' }).nth(1).click();

    await expect.poll(() => openedTrashes(page)).toEqual([1]);
  });

  test('after a cleanup to the Trash the line shows the moved items', async ({ page }) => {
    await mockCore(page, SCRIPTS.trashCleaning);
    await reviewAll(page);
    await expect(trashLine(page)).toHaveCount(0);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();
    await page.getByRole('region', { name: 'Moved to the Trash' }).getByRole('button', { name: 'Done' }).click();
    await navScan(page).click();

    await expect(trashLine(page)).toContainText('Waiting in the Trash: 4.1 GB, all of it from cleanups here.');
  });

  test('the result invites the user to look at the Trash and empty it themselves', async ({ page }) => {
    await mockCore(page, SCRIPTS.trashCleaning);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Look at what is there and empty it yourself if you no longer need it.');
    await expect(result.getByRole('button', { name: 'Show in the Trash' })).toHaveAccessibleDescription(
      'Look at what is there and empty it yourself if you no longer need it.',
    );
  });

  test('two Trashes show a line each with their volume’s name', async ({ page }) => {
    await scanned(page, SCRIPTS.trashTwo);

    const lines = trashLine(page);
    await expect(lines).toContainText('Macintosh HD');
    await expect(lines).toContainText('Backup');
    await expect(lines.getByRole('button', { name: 'Open Trash' })).toHaveCount(2);
  });

  test('Finder’s 30-day setting shows the date the share leaves by itself', async ({ page }) => {
    await scanned(page, SCRIPTS.trashDated);

    await expect(trashLine(page)).toContainText('Finder removes them by itself by Jan 1, 2026.');
  });

  test("Ignore this category from a card's menu takes it out of every total", async ({ page }) => {
    await openIgnoring(page);

    await page.getByRole('button', { name: 'Options for Example caches' }).click();
    await page.getByRole('menuitem', { name: 'Ignore this category' }).click();

    await expect(cards(page)).toHaveCount(1);
    await expect(cardNames(page)).toHaveText(['Example downloads']);
    await expect(page.getByText('1 thing you ignored is left out of these results.')).toBeVisible();
    const recorded = await page.evaluate(() => sessionStorage.getItem('fresher.mock.ignore'));
    expect(JSON.parse(recorded ?? 'null')).toMatchObject({ categories: ['demo.caches'] });
  });

  test('Ignore this item takes the row out of the details and the focus to the next row', async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');
    const list = page.getByRole('list', { name: 'The items of Example caches' });

    await page.getByRole('button', { name: 'Options for Example: editor-cache' }).click();
    await page.getByRole('menuitem', { name: 'Ignore this item' }).click();

    await expect(list.getByRole('listitem')).toHaveCount(1);
    await expect(list).not.toContainText('Example: editor-cache');
    await expect(list.getByRole('listitem')).toBeFocused();
    const recorded = await page.evaluate(() => sessionStorage.getItem('fresher.mock.ignore'));
    expect(JSON.parse(recorded ?? 'null')).toMatchObject({ paths: ['/demo/editor-cache'] });
  });

  test('Settings lists the ignored item by its path and Restore brings its row back', async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');
    await page.getByRole('button', { name: 'Options for Example: editor-cache' }).click();
    await page.getByRole('menuitem', { name: 'Ignore this item' }).click();
    const list = page.getByRole('list', { name: 'The items of Example caches' });
    await expect(list.getByRole('listitem')).toHaveCount(1);

    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored items/ }).click();
    const restore = page.getByRole('button', { name: 'Restore /demo/editor-cache' });
    await expect(restore).toBeVisible();
    await restore.click();
    await expect(page.getByText('Nothing is ignored.')).toBeVisible();
    await navScan(page).click();
    await openDetails(page, 'Example caches');

    await expect(list.getByRole('listitem')).toHaveCount(2);
    await expect(list).toContainText('Example: editor-cache');
  });

  test("ignoring one item by keyboard alone: Tab to the row's menu, ArrowDown, Enter", async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');
    const list = page.getByRole('list', { name: 'The items of Example caches' });

    await tabTo(page, page.getByRole('button', { name: 'Options for Example: editor-cache' }));
    await page.keyboard.press('ArrowDown');
    await expect(page.getByRole('menuitem', { name: 'Ignore this item' })).toBeFocused();
    await page.keyboard.press('Enter');

    await expect(list.getByRole('listitem')).toHaveCount(1);
    await expect(list).not.toContainText('Example: editor-cache');
    await expect(list.getByRole('listitem')).toBeFocused();
  });

  test("the card's menu works by keyboard alone", async ({ page }) => {
    await openIgnoring(page);
    const options = page.getByRole('button', { name: 'Options for Example caches' });

    await tabTo(page, options);
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menuitem', { name: 'Ignore this category' })).toBeFocused();
    await page.keyboard.press('ArrowDown');
    await expect(page.getByRole('menuitem', { name: 'Ignore Demo module' })).toBeFocused();
    await page.keyboard.press('ArrowUp');
    await page.keyboard.press('Enter');

    await expect(cards(page)).toHaveCount(1);
    await expect(page.getByText('1 thing you ignored is left out of these results.')).toBeVisible();
  });

  test("Space on a card's menu item chooses it and leaves the menu closed", async ({ page }) => {
    await openIgnoring(page);
    const options = page.getByRole('button', { name: 'Options for Example caches' });

    await tabTo(page, options);
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menuitem', { name: 'Ignore this category' })).toBeFocused();
    await page.keyboard.press('Space');

    await expect(cards(page)).toHaveCount(1);
    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(page.getByRole('menuitem')).toHaveCount(0);
  });

  test("a card's Details opens the category's details with every finding", async ({ page }) => {
    await scanned(page, SCRIPTS.demo);
    await expect(cardNames(page)).toHaveText(['Example caches', 'Example downloads']);

    await openDetails(page, 'Example caches');

    await expect(navScan(page)).toHaveAttribute('aria-current', 'page');
    const list = page.getByRole('list', { name: 'The items of Example caches' });
    await expect(list.getByRole('listitem')).toHaveCount(2);
    await expect(list).toContainText('Example: editor-cache');
    await expect(list).toContainText('Example: browser-cache');
    await expect(details(page)).toContainText('2 items');
    await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toHaveCount(0);
  });

  test('ticking a finding on the details shows the selection bar, and its Review lists it', async ({ page }) => {
    await scanned(page, SCRIPTS.demo);
    await openDetails(page, 'Example downloads');
    await expect(selectionBar(page)).toHaveCount(0);

    await details(page).getByRole('checkbox', { name: 'Select Example: installer.dmg' }).check();

    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toContainText(
      'Example: installer.dmg',
    );
    await expect(page.getByRole('list', { name: 'The items of Example caches' })).toHaveCount(0);
    await expect(removeButton(page)).toBeEnabled();
  });

  test("a choice on the details of a removed app's category selects only its own categories", async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_CACHES);

    await choice(page, 'keep-settings').click();

    await expect(selectionBar(page)).toContainText('2 items selected');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_SETTINGS}` })).not.toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_DATA}` })).not.toBeChecked();
  });

  test("Select all of a zone, then the selection bar's Review lists those categories", async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await scanButton(page).click();
    await expect(cards(page)).toHaveCount(2);

    await zone(page, 'Low risk').getByRole('checkbox', { name: 'Select all' }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();

    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toContainText(
      'Example: installer.dmg',
    );
    await expect(page.getByRole('list', { name: 'The items of Example caches' })).toHaveCount(0);
  });

  test('from a scan to the review of what was ticked, in the zones of the Scan screen', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await reviewAll(page);

    await expect(page.getByRole('heading', { level: 2 })).toHaveText(['Safe to remove', 'Low risk']);
    await expect(zone(page, 'Safe to remove').getByRole('heading', { level: 3 })).toHaveText('Example caches');
    await expect(zone(page, 'Low risk').getByRole('heading', { level: 3 })).toHaveText('Example downloads');
    await expect(page.getByRole('checkbox', { name: 'Select all' })).toHaveCount(0);
  });

  test('unticking the last row of a zone removes the zone', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await reviewAll(page);
    await expect(zone(page, 'Low risk')).toBeVisible();

    await page.getByRole('checkbox', { name: 'Select Example: installer.dmg' }).click();

    await expect(zone(page, 'Low risk')).toHaveCount(0);
    await expect(page.getByRole('heading', { level: 2 })).toHaveText(['Safe to remove']);
  });

  test("the selection bar's Clear empties the selection on the overview", async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await scanButton(page).click();
    await zone(page, 'Safe to remove').getByRole('checkbox', { name: 'Select all' }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');

    await selectionBar(page).getByRole('button', { name: 'Clear' }).click();

    await expect(selectionBar(page)).toHaveCount(0);
    await expect(zone(page, 'Safe to remove').getByRole('checkbox', { name: 'Select all' })).not.toBeChecked();
  });

  test('Stop ends the scan as partial, with what it found so far', async ({ page }) => {
    await mockCore(page, SCRIPTS.live);
    await page.goto('/');

    await scanButton(page).click();
    await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');

    await page.getByRole('button', { name: 'Stop' }).click();

    await expect(page.getByText('This scan is incomplete')).toBeVisible();
    await expect(cards(page)).toHaveText([/1 item/, /1 item/]);
    await expect(page.getByRole('button', { name: 'Scan again' })).toBeVisible();
  });

  test('a long list loads its pages as the keyboard moves to its end', async ({ page }) => {
    await mockCore(page, SCRIPTS.large);
    await page.goto('/');
    await scanButton(page).click();
    await expect(cards(page)).toHaveCount(3);

    await cards(page).first().getByRole('button', { name: 'Details' }).click();
    const list = page.getByRole('list', { name: /^The items of / }).first();
    await list.getByRole('listitem').first().focus();
    await page.keyboard.press('End');

    const focused = page.locator(':focus');
    await expect(focused).toHaveAccessibleName(/^Example: item-\d+/);
    const total = await focused.getAttribute('aria-setsize');
    expect(Number(total)).toBeGreaterThan(300);
    await expect(focused).toHaveAttribute('aria-posinset', total ?? '');
  });

  test('switching the demo module off in the settings', async ({ page }) => {
    await mockCore(page, SCRIPTS.settings);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Settings' }).click();
    const enabled = page.getByRole('switch', { name: 'Demo module' });
    await expect(enabled).toBeChecked();

    await enabled.click();

    await expect(enabled).not.toBeChecked();
  });

  test('adding a scan folder with the picker, then resetting the list', async ({ page }) => {
    await mockCore(page, SCRIPTS.settings);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Settings' }).click();
    const folders = page.getByRole('group', { name: 'Folders to scan' });

    await folders.getByRole('button', { name: 'Add folder…' }).click();
    await expect(folders.getByRole('list', { name: 'Folders to scan' })).toContainText('~/Music');
    await folders.getByRole('button', { name: 'Reset' }).click();

    await expect(folders.getByText('No folders')).toBeVisible();
  });

  test('Reset all settings asks first and shows the size of Fresher’s own data', async ({ page }) => {
    await mockCore(page, SCRIPTS.settings);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Settings' }).click();

    await expect(page.getByText(/take 2 MB of the 50 MB/u)).toBeVisible();
    await page.getByRole('button', { name: 'Reset all settings…' }).click();
    const dialog = page.getByRole('dialog', { name: 'Reset all settings?' });
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: 'Keep my settings' }).click();

    await expect(dialog).toBeHidden();
  });

  test('the whole path without a pointer', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await expect(heading(page, 'Scan')).toBeVisible();

    await tabTo(page, scanButton(page));
    await page.keyboard.press('Enter');
    const open = detailsOf(page, 'Example caches');
    await expect(open).toBeVisible();

    await tabTo(page, open);
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Example caches')).toBeFocused();

    const list = page.getByRole('list', { name: 'The items of Example caches' });
    await tabTo(page, list.getByRole('listitem').first());
    await page.keyboard.press('ArrowDown');
    await expect(list.getByRole('listitem').nth(1)).toBeFocused();
    await page.keyboard.press('Space');
    await expect(selectionBar(page)).toContainText('1 item selected');

    await tabTo(page, selectionBar(page).getByRole('button', { name: 'Review' }));
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example caches' })).toContainText(
      'Example: browser-cache',
    );

    await page.keyboard.press('Escape');
    await expect(heading(page, 'Review')).toBeVisible();
    await expect(removeButton(page)).toBeEnabled();
  });

  test('select, review, confirm to the Trash, the progress, the result, the Cleanups and put back', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.cleaningHeld);
    await reviewAll(page);

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
    await expect(dialog).toContainText('You get the space back when you empty the Trash.');
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();
    await expect(page.getByRole('progressbar', { name: 'Cleanup progress' })).toHaveAttribute('aria-valuenow', '33');
    await releaseCleanup(page);

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await expect(result).toContainText('You get the space back when you empty the Trash.');
    await expect(page.getByRole('button', { name: /^Review/ })).not.toContainText('selected');
    await result.getByRole('button', { name: 'See this cleanup' }).click();

    await expect(heading(page, 'Cleanup of Jan 2, 2025')).toBeFocused();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const editor = items.getByRole('listitem').filter({ hasText: 'editor-cache' });
    await editor.getByRole('button', { name: 'Put back' }).click();
    await expect(editor).toContainText('Put back where it was.');
    await expect(editor.getByRole('button', { name: 'Put back' })).toHaveCount(0);

    await page.getByRole('button', { name: 'All cleanups' }).click();
    await expect(heading(page, 'Cleanups')).toBeFocused();
    await expect(page.getByRole('list', { name: 'Cleanups, newest first' })).toContainText('In the Trash: 2 of 3');
  });

  test('the permanent path: its own tick, the red button and Freed', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaning);
    await reviewAll(page);
    await removeButton(page).click();
    const dialog = page.getByRole('dialog');

    await dialog.getByRole('checkbox', { name: /^Delete permanently instead/ }).check();
    await expect(dialog).toHaveAccessibleName('Delete 3 items permanently?');
    await expect(dialog).not.toContainText('You get the space back when you empty the Trash.');
    const confirm = dialog.getByRole('button', { name: 'Delete permanently' });
    await expect(confirm).toHaveAttribute('data-variant', 'danger');
    await confirm.click();

    const result = page.getByRole('region', { name: 'Freed' });
    await expect(result).toContainText('Deleted 3 items permanently');
    await expect(result).toContainText('Measured on the disk: about 2.6 GB.');
    await expect(result.getByRole('button', { name: 'Show in the Trash' })).toHaveCount(0);
  });

  test('Stop mid-cleanup ends it stopped, the rest untouched', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaningHeld);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();

    await page.getByRole('button', { name: 'Stop' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 1 item to the Trash');
    await expect(result).toContainText('Stopped. 2 items were not touched.');
  });

  test('the items left out are listed in the dialog and not counted', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaningLeftOut);
    await reviewAll(page);

    await removeButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    const leftOut = dialog.getByRole('region', { name: 'These will not be removed' });
    await expect(leftOut).toContainText('Example: browser-cache');
    await expect(leftOut).toContainText('Quit Safari and try again.');
    await dialog.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByRole('dialog')).toHaveCount(0);
    await expect(removeButton(page)).toBeFocused();
  });

  test('ignoring the group Debug from Settings hides its totals and turning it off brings them back', async ({
    page,
  }) => {
    await openIgnoring(page);
    await openSettings(page);

    await page.getByRole('switch', { name: 'Ignore Debug' }).click();

    await navScan(page).click();
    await expect(page.getByText('1 thing you ignored is left out of these results.')).toBeVisible();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
    await expect(page.getByText(/you ignored is left out/)).toHaveCount(0);
  });

  test('ignoring the module from Settings hides its cards and turning it off brings them back', async ({ page }) => {
    await openIgnoring(page);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();

    await page.getByRole('switch', { name: 'Ignore Demo module' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    await page.getByRole('switch', { name: 'Ignore Demo module' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
  });

  test('a module of an ignored group is locked with its note and turning the group off frees it', async ({ page }) => {
    await openIgnoring(page);
    await openSettings(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();

    const locked = page.getByRole('switch', { name: 'Ignore Demo module' });
    await expect(locked).toBeDisabled();
    await expect(page.getByText('Ignored with its group: Debug')).toBeVisible();
    await page.getByRole('tab', { name: /^Ignored groups/ }).click();
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeEnabled();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
  });

  test('ignored modules and groups stay ignored after a reload', async ({ page }) => {
    await openIgnoring(page);
    await fromCardMenu(page, 'Example caches', 'Ignore Demo module');
    await expect(cards(page)).toHaveCount(0);

    await page.reload();

    await openSettings(page);
    await expect(page.getByRole('tab', { name: /^Ignored modules/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('the ignored line on the overview opens Settings on the tab with an entry', async ({ page }) => {
    await openIgnoring(page);
    await fromCardMenu(page, 'Example caches', 'Ignore Demo module');

    await page.getByRole('button', { name: 'Show in Settings' }).click();

    await expect(page.getByRole('tab', { name: /^Ignored modules/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test("Ignore Demo module from a card's menu moves it to the Ignored modules tab", async ({ page }) => {
    await openIgnoring(page);

    await fromCardMenu(page, 'Example downloads', 'Ignore Demo module');

    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await expect(page.getByRole('tab', { name: 'Ignored modules · 1' })).toBeVisible();
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('ignoring a module drops what was ticked from the selection', async ({ page }) => {
    await openIgnoring(page);
    await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
    await expect(page.getByRole('button', { name: /^Review \d+ items? selected/ })).toBeVisible();

    await fromCardMenu(page, 'Example caches', 'Ignore Demo module');

    await expect(cards(page)).toHaveCount(0);
    await expect(page.getByRole('button', { name: /selected/ })).toHaveCount(0);
  });

  test('the details of a JetBrains category name its level', async ({ page }) => {
    await openJetbrains(page);

    for (const [name, level] of [
      [JB.caches, 'Safe'],
      [JB.logs, 'Safe'],
      [JB.settings, 'Medium'],
      [JB.uninstalled, 'Medium'],
    ] as const) {
      await openDetails(page, name);
      await expect(details(page).getByRole('region', { name: 'Danger level' })).toContainText(level);
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
    await openDetails(page, JB.caches);
    const about = details(page).getByRole('region', { name: 'About it' });
    await expect(about).toContainText(jb('caches.what'));
    await expect(about).toContainText(jb('caches.if-removed'));
  });

  test('the items left out are lines under their category, with their reason', async ({ page }) => {
    await openJetbrains(page);

    await openDetails(page, JB.caches);
    await expect(details(page)).toContainText(`3 items left out (1.4 GB): ${jb('left-out.current')}`);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, JB.settings);
    await expect(details(page)).toContainText(`2 items left out (600 MB): ${jb('left-out.running')}`);
  });

  test('the data of an IDE that is no longer installed is its own Medium category with the findings named', async ({
    page,
  }) => {
    await openJetbrains(page);

    await openDetails(page, JB.uninstalled);

    const removed = details(page);
    await expect(removed).toContainText('Medium');
    await expect(removed).toContainText('2 items');
    await expect(removed).toContainText(jb('uninstalled.what'));
    await expect(removed).toContainText(jb('uninstalled.if-removed'));
    await expect(removed).toContainText('DataGrip 2024.3 caches');
    await expect(removed).toContainText('DataGrip 2024.3 settings and plugins');
  });

  test('WebStorm’s findings go from the review to the Trash, and the cleanup is put back from the Cleanups', async ({
    page,
  }) => {
    await reviewWebStorm(page);
    await expect(page.getByRole('list', { name: `The items of ${JB.settings}` })).toContainText('WebStorm 2024.3');

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('Medium');
    await expect(dialog).toContainText(jb('settings.if-removed'));
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const caches = items.getByRole('listitem').filter({ hasText: 'WebStorm2024.3' }).first();
    await caches.getByRole('button', { name: 'Put back' }).click();
    await expect(caches).toContainText('Put back where it was.');
  });

  test("ignoring the module from a card's menu hides its totals, Settings lists it and turning it off restores it", async ({
    page,
  }) => {
    await openJetbrains(page);

    await fromCardMenu(page, JB.caches, 'Ignore JetBrains IDEs');

    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    const ignored = page.getByRole('switch', { name: 'Ignore JetBrains IDEs' });
    await expect(ignored).toBeChecked();
    await ignored.click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(4);
  });

  test('ignoring the group Development hides JetBrains IDEs and turning the group off brings it back', async ({
    page,
  }) => {
    await openJetbrains(page);
    await expect(cards(page)).toHaveCount(4);
    await openSettings(page);

    await page.getByRole('switch', { name: 'Ignore Development' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('switch', { name: 'Ignore Development' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(4);
  });

  test('the module’s days setting is in Settings with its text', async ({ page }) => {
    await mockCore(page, SCRIPTS.jetbrains);
    await page.goto('/');
    await openSettings(page);

    const days = page.getByRole('spinbutton', { name: jb('uninstalled-after-days.label') });
    await expect(days).toHaveValue('30');
    await expect(page.getByText(jb('uninstalled-after-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: jb('enabled.label') })).toBeChecked();
  });

  test('the details of Temporary files show the category, Safe and its texts', async ({ page }) => {
    await openTemp(page);

    await openDetails(page, TEMP_CATEGORY);

    const category = details(page);
    await expect(category).toContainText('Safe');
    await expect(category).toContainText('5 items');
    for (const key of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(tmp(`old-files.${key}`));
    }
  });

  test('the items left out are lines under the category: the recent ones, the system’s and the pipe', async ({
    page,
  }) => {
    await openTemp(page);

    await openDetails(page, TEMP_CATEGORY);

    const category = details(page);
    await expect(category).toContainText(`4,712 items left out (790 MB): ${tmp('left-out.recent')}`);
    await expect(category).toContainText(`3 items left out (12 MB): ${tmp('left-out.system')}`);
    await expect(category).toContainText(`1 item left out (0 bytes): ${tmp('left-out.special')}`);
  });

  test('the temporary findings go from the review to the Trash, and the cleanup is put back from the Cleanups', async ({
    page,
  }) => {
    await openTemp(page);
    await page.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: `The items of ${TEMP_CATEGORY}` })).toContainText('pip-unpack-ak3j2');

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 5 items to the Trash?' });
    await expect(dialog).toContainText('Safe');
    await expect(dialog).toContainText(tmp('old-files.if-removed'));
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 5 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: 'pip-unpack-ak3j2' }).first();
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('old logs and old crash reports show their cards with their sizes and item counts', async ({ page }) => {
    await openLogs(page);

    await expect(cards(page)).toHaveCount(2);
    const logs = page.getByRole('checkbox', { name: `Select ${LOGS_CATEGORY}` });
    await expect(logs).toBeVisible();
    await expect(page.getByRole('checkbox', { name: `Select ${lg('crash-reports.name')}` })).toBeVisible();
    await expect(cards(page).filter({ hasText: LOGS_CATEGORY })).toContainText('44 MB');
    await expect(cards(page).filter({ hasText: LOGS_CATEGORY })).toContainText('2 items');
    await expect(cards(page).filter({ hasText: lg('crash-reports.name') })).toContainText('1 item');
  });

  test('the details of Old logs of your apps show the category, Safe and its texts', async ({ page }) => {
    await openLogs(page);

    await openDetails(page, LOGS_CATEGORY);

    const category = details(page);
    await expect(category).toContainText('Safe');
    await expect(category).toContainText('2 items');
    for (const key of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(lg(`app-logs.${key}`));
    }
  });

  test("an old log's finding names its file, its folder and its age", async ({ page }) => {
    await openLogs(page);

    await openDetails(page, LOGS_CATEGORY);

    const category = details(page);
    await expect(category).toContainText('writer.1.log');
    await expect(category).toContainText('In Writer in your Logs folder, unchanged for 30 days.');
    await expect(category).toContainText('old-tool.log');
    await expect(category).toContainText('In your Logs folder, unchanged for 30 days.');
  });

  test('selecting the logs makes the review name Safe', async ({ page }) => {
    await openLogs(page);
    await page.getByRole('checkbox', { name: `Select ${LOGS_CATEGORY}` }).check();

    await selectionBar(page).getByRole('button', { name: 'Review' }).click();

    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: `The items of ${LOGS_CATEGORY}` })).toContainText('old-tool.log');
    await removeButton(page).click();
    await expect(page.getByRole('dialog', { name: 'Move 2 items to the Trash?' })).toContainText('Safe');
  });

  test('the old logs go from the review to the Trash, and the cleanup is put back from the Cleanups', async ({
    page,
  }) => {
    await openLogs(page);
    await page.getByRole('checkbox', { name: `Select ${LOGS_CATEGORY}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog).toContainText(lg('app-logs.if-removed'));
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: 'old-tool.log' }).first();
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the logs switch and the days field, which cannot go below 7', async ({ page }) => {
    await mockCore(page, SCRIPTS.logs);
    await page.goto('/');
    await openSettings(page);

    const days = page.getByRole('spinbutton', { name: lg('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await expect(page.getByText(lg('older-than-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: lg('enabled.label') })).toBeChecked();
    await recordUpdates(page);

    await days.fill('6');

    await expect(page.getByText('Enter a whole number from 7 to 90.')).toBeVisible();
    expect(await sentUpdates(page)).toBe(0);
  });

  test('Settings has the number field at 7 days with its text, and 14 is accepted', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openSettings(page);

    const days = page.getByRole('spinbutton', { name: tmp('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await expect(page.getByText(tmp('older-than-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: tmp('enabled.label') })).toBeChecked();

    await recordUpdates(page);

    await days.fill('14');

    await expect(days).toHaveValue('14');
    expect(await sentUpdates(page)).toBe(1);
    await expect(page.getByText('Enter a whole number from 7 to 30.')).toHaveCount(0);
  });

  test("ignoring the module from a card's menu and the group System from Settings, then restoring both", async ({
    page,
  }) => {
    await openTemp(page);

    await fromCardMenu(page, TEMP_CATEGORY, `Ignore ${tmp('enabled.label')}`);

    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    const ignored = page.getByRole('switch', { name: 'Ignore Temporary files' });
    await expect(ignored).toBeChecked();
    await ignored.click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(1);

    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored groups/ }).click();
    await page.getByRole('switch', { name: 'Ignore System' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored groups/ }).click();
    await page.getByRole('switch', { name: 'Ignore System' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(1);
  });

  test('developer caches show their cards with their sizes', async ({ page }) => {
    await openDevCaches(page);

    for (const name of [NPM, CARGO, GRADLE, XCODE_DATA]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: NPM })).toContainText('8.9 GB');
  });

  test('a developer cache category shows its danger and its five texts', async ({ page }) => {
    await openDevCaches(page);

    await openDetails(page, XCODE_DATA);
    await expect(details(page)).toContainText('Safe');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, NPM);

    const npm = details(page);
    await expect(npm).toContainText('Low');
    for (const key of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(npm).toContainText(dev(`npm.${key}`));
    }
  });

  test('npm’s cache and a project go to the Trash and come back, the confirmation naming Low', async ({ page }) => {
    await openDevCaches(page);
    await page.getByRole('checkbox', { name: `Select ${NPM}` }).check();
    await page.getByRole('checkbox', { name: `Select ${XCODE_DATA}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: `The items of ${NPM}` })).toContainText(dev('title.npm'));
    await expect(page.getByRole('list', { name: `The items of ${XCODE_DATA}` })).toContainText('Notes');

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('Low');
    await expect(dialog).toContainText(dev('npm.if-removed'));
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: '_cacache' }).first();
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('ignoring npm’s cache keeps the other caches, and restoring brings it back', async ({ page }) => {
    await openDevCaches(page);

    await fromCardMenu(page, NPM, 'Ignore this category');

    await expect(cards(page).filter({ hasText: NPM })).toHaveCount(0);
    for (const name of [CARGO, GRADLE, XCODE_DATA]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored items/ }).click();
    await page.getByRole('button', { name: `Restore ${NPM}` }).click();
    await navScan(page).click();
    await expect(cards(page).filter({ hasText: NPM })).toContainText('8.9 GB');
  });

  test('ignoring the developer caches module hides them all', async ({ page }) => {
    await openDevCaches(page);

    await fromCardMenu(page, NPM, `Ignore ${dev('enabled.label')}`);

    await expect(cards(page)).toHaveCount(0);
  });

  test('Settings shows the developer caches switch alone', async ({ page }) => {
    await mockCore(page, SCRIPTS.devCaches);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: dev('enabled.label') })).toBeChecked();
    await expect(page.getByText(dev('enabled.description'))).toBeVisible();
    await expect(page.getByRole('spinbutton')).toHaveCount(0);
  });

  test('removed apps’ leftovers show their cards with their sizes', async ({ page }) => {
    await openLeftovers(page);

    for (const name of [LEFTOVERS_CACHES, LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: LEFTOVERS_DATA })).toContainText('1.8 GB');
  });

  test('each removed-app category shows its danger and its five texts', async ({ page }) => {
    await openLeftovers(page);

    for (const [name, key, danger] of [
      [LEFTOVERS_CACHES, 'caches', 'Safe'],
      [LEFTOVERS_SETTINGS, 'settings', 'Medium'],
      [LEFTOVERS_DATA, 'data', 'High'],
    ] as const) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category).toContainText(danger);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(left(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('Remove everything selects the three categories and the review names High', async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_DATA);

    await choice(page, 'everything').click();

    await page.getByRole('button', { name: 'Back to Scan' }).click();
    for (const name of [LEFTOVERS_CACHES, LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(page.getByRole('checkbox', { name: `Select ${name}` })).toBeChecked();
    }
    await expect(selectionBar(page)).toContainText('5 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 5 items to the Trash?' });
    await expect(dialog).toContainText('High');
    await expect(dialog).toContainText(left('data.if-removed'));
  });

  test('Keep settings selects only the caches and the review names Safe', async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_CACHES);

    await choice(page, 'keep-settings').click();

    await page.getByRole('button', { name: 'Back to Scan' }).click();

    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_SETTINGS}` })).not.toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_DATA}` })).not.toBeChecked();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog).toContainText('Safe');
    await expect(dialog).not.toContainText('High');
    await expect(dialog).not.toContainText('Medium');
  });

  test('the caches of a removed app go to the Trash and come back', async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_CACHES);
    await choice(page, 'keep-settings').click();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: '640 MB' }).first();
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the switch and the days for removed apps', async ({ page }) => {
    await mockCore(page, SCRIPTS.appLeftovers);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: left('enabled.label') })).toBeChecked();
    await expect(page.getByRole('spinbutton', { name: left('uninstalled-after-days.label') })).toHaveValue('30');
    await expect(page.getByText(left('uninstalled-after-days.description'))).toBeVisible();
  });

  test('downloads and installers show their cards with their sizes', async ({ page }) => {
    await openDownloads(page);

    for (const name of [DL_UNFINISHED, DL_SAFARI, DL_INSTALLERS, DL_MACOS, DL_LARGE]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: DL_MACOS })).toContainText('12.8 GB');
  });

  test('each downloads category shows its danger and its five texts', async ({ page }) => {
    await openDownloads(page);

    for (const [name, key, danger] of [
      [DL_UNFINISHED, 'unfinished', 'Low'],
      [DL_SAFARI, 'safari-resumable', 'Medium'],
      [DL_INSTALLERS, 'installers', 'Low'],
      [DL_MACOS, 'macos-installers', 'Low'],
      [DL_LARGE, 'large-untouched', 'High'],
    ] as const) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category).toContainText(danger);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(dl(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('selecting the installers and the big files makes the review name High', async ({ page }) => {
    await openDownloads(page);

    await page.getByRole('checkbox', { name: `Select ${DL_INSTALLERS}` }).check();
    await page.getByRole('checkbox', { name: `Select ${DL_LARGE}` }).check();

    await expect(selectionBar(page)).toContainText('3 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('High');
    await expect(dialog).toContainText(dl('large-untouched.if-removed'));
  });

  test('the installers and the big file go to the Trash and come back from the Cleanups', async ({ page }) => {
    await openDownloads(page);
    await page.getByRole('checkbox', { name: `Select ${DL_INSTALLERS}` }).check();
    await page.getByRole('checkbox', { name: `Select ${DL_LARGE}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: 'video.mov' });
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('an unfinished download goes to the Trash with its placeholder and comes back', async ({ page }) => {
    await openDownloads(page, SCRIPTS.downloadsUnfinished);
    await page.getByRole('checkbox', { name: `Select ${DL_UNFINISHED}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    await expect(items.getByRole('listitem')).toHaveCount(2);
    const item = items.getByRole('listitem').filter({ hasText: 'movie.mp4.crdownload' });
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the switch and the days for downloads', async ({ page }) => {
    await mockCore(page, SCRIPTS.downloads);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: dl('enabled.label') })).toBeChecked();
    await expect(page.getByRole('spinbutton', { name: dl('older-than-days.label') })).toHaveValue('30');
    await expect(page.getByText(dl('older-than-days.description'))).toBeVisible();
  });

  test('old projects show their cards with their sizes', async ({ page }) => {
    await openStaleProjects(page);

    for (const name of [SP_DEPENDENCIES, SP_BUILD, SP_PYTHON, SP_RUST]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: SP_BUILD })).toContainText('3.2 GB');
  });

  test('each old-projects category shows its danger and its five texts', async ({ page }) => {
    await openStaleProjects(page);

    for (const [name, key, danger] of [
      [SP_DEPENDENCIES, 'dependencies', 'Low'],
      [SP_BUILD, 'build-output', 'Low'],
      [SP_PYTHON, 'python-environments', 'Medium'],
      [SP_RUST, 'rust-build-cache', 'Low'],
    ] as const) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category).toContainText(danger);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(sp(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('an old project’s finding names the project, the folder, the date and the command', async ({ page }) => {
    await openStaleProjects(page);

    await openDetails(page, SP_DEPENDENCIES);

    const items = details(page);
    await expect(items).toContainText('shop · node_modules');
    await expect(items).toContainText('shop · packages/web/node_modules');
    const [beforeDate = '', afterDate = ''] = sp('reason.dependencies').split('{date}');
    await expect(items).toContainText(beforeDate.trim());
    await expect(items).toContainText('Nov 15, 2023');
    await expect(items).toContainText(afterDate.replace('{command}', 'npm install').trim());
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, SP_RUST);
    await expect(details(page)).toContainText('daily · target/debug/incremental');
  });

  test('selecting the Python environments and the build output makes the review name Medium', async ({ page }) => {
    await openStaleProjects(page);

    await page.getByRole('checkbox', { name: `Select ${SP_PYTHON}` }).check();
    await page.getByRole('checkbox', { name: `Select ${SP_BUILD}` }).check();

    await expect(selectionBar(page)).toContainText('2 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog).toContainText('Medium');
    await expect(dialog).toContainText(sp('python-environments.if-removed'));
  });

  test('a build output goes to the Trash and comes back from the Cleanups', async ({ page }) => {
    await openStaleProjects(page);
    await page.getByRole('checkbox', { name: `Select ${SP_BUILD}` }).check();
    await page.getByRole('checkbox', { name: `Select ${SP_PYTHON}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: 'target' });
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the switch and the days for old projects', async ({ page }) => {
    await mockCore(page, SCRIPTS.staleProjects);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: sp('enabled.label') })).toBeChecked();
    await expect(page.getByRole('spinbutton', { name: sp('unused-for-days.label') })).toHaveValue('365');
    await expect(page.getByText(sp('unused-for-days.description'))).toBeVisible();
  });

  test('local AI models show their cards with their sizes', async ({ page }) => {
    await openLocalAiModels(page);

    for (const name of [AI_HF_MODELS, AI_HF_DATASETS, AI_LM_STUDIO]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: AI_LM_STUDIO })).toContainText('4.7 GB');
  });

  test('each local AI models category shows its danger and its five texts', async ({ page }) => {
    await openLocalAiModels(page);

    for (const [name, key, danger] of [
      [AI_HF_MODELS, 'hugging-face-models', 'Low'],
      [AI_HF_DATASETS, 'hugging-face-datasets', 'Low'],
      [AI_LM_STUDIO, 'lm-studio', 'Medium'],
    ] as const) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category).toContainText(danger);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(ai(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('a model’s finding names the model, the date and the download size', async ({ page }) => {
    await openLocalAiModels(page);

    await openDetails(page, AI_HF_MODELS);

    const items = details(page);
    await expect(items).toContainText('meta/tiny');
    await expect(items).toContainText('gpt2');
    const [beforeDate = '', afterDate = ''] = ai('reason.hugging-face').split('{date}');
    await expect(items).toContainText(beforeDate.trim());
    await expect(items).toContainText('Mar 10, 2024');
    await expect(items).toContainText(afterDate.replace('{size}', '2 GB').trim());
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, AI_LM_STUDIO);
    await expect(details(page)).toContainText('pub/model-a');
  });

  test('selecting an LM Studio model makes the review name Medium', async ({ page }) => {
    await openLocalAiModels(page);

    await page.getByRole('checkbox', { name: `Select ${AI_LM_STUDIO}` }).check();

    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('Medium');
    await expect(dialog).toContainText(ai('lm-studio.if-removed'));
  });

  test('a Hugging Face model goes to the Trash and comes back from the Cleanups', async ({ page }) => {
    await openLocalAiModels(page);
    await openDetails(page, AI_HF_MODELS);
    await page.getByRole('checkbox', { name: /Select meta\/tiny/ }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 1 item to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: 'models--meta--tiny' });
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Ollama models show on Scan with their freed size', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);

    await expect(cards(page).filter({ hasText: AI_OLLAMA })).toBeVisible();
    await expect(cards(page).filter({ hasText: AI_OLLAMA })).toContainText('10.9 GB');
    await expect(cards(page).filter({ hasText: AI_OLLAMA })).toContainText('3 items');
  });

  test("a shared Ollama model's reason names what stays", async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);

    await openDetails(page, AI_OLLAMA);

    const items = details(page);
    await expect(items).toContainText('It is 4.9 GB in all, of which 4.6 GB is freed;');
    await expect(items).toContainText('the rest is shared with llama3:8b and stays.');
    await expect(items).toContainText('the rest is shared with llama3-custom:latest and stays.');
  });

  test("an unshared Ollama model's reason gives the command that downloads it again", async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);

    await openDetails(page, AI_OLLAMA);

    await expect(details(page)).toContainText('`ollama pull gemma2:2b` downloads its 1.6 GB again');
  });

  test('the Ollama category shows Medium and its five texts', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);

    await openDetails(page, AI_OLLAMA);

    const category = details(page);
    await expect(category.getByRole('heading', { name: AI_OLLAMA })).toBeVisible();
    await expect(category.getByRole('region', { name: 'Danger level' })).toContainText('Medium');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(ai(`ollama.${text}`));
    }
    await expect(category).toContainText('llama3:8b, llama3:latest');
  });

  test('the confirm dialog lists the files that go with a model on demand', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);
    await page.getByRole('checkbox', { name: `Select ${AI_OLLAMA}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('Medium');
    const toggle = dialog.getByRole('button', { name: 'and 3 more files that go with it: llama3:8b, llama3:latest' });
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(dialog.getByRole('list', { name: /^The other files that go with/ })).toHaveCount(0);

    await toggle.click();

    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const files = dialog.getByRole('list', { name: 'The other files that go with llama3:8b, llama3:latest' });
    await expect(files.getByRole('listitem')).toHaveCount(3);
    await expect(files).toContainText('/.ollama/models/manifests/registry.ollama.ai/library/llama3/latest');
    await expect(files).toContainText('/.ollama/models/blobs/sha256-a1f0c2');
  });

  test('an Ollama model goes to the Trash and comes back', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);
    await openDetails(page, AI_OLLAMA);
    await page.getByRole('checkbox', { name: /Select llama3:8b, llama3:latest/ }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 1 item to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem');
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the switch for local AI models', async ({ page }) => {
    await mockCore(page, SCRIPTS.localAiModels);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: ai('enabled.label') })).toBeChecked();
    await expect(page.getByText(ai('enabled.description'))).toBeVisible();
  });

  test('caches of your apps show their card with its size and item count', async ({ page }) => {
    await openAppCaches(page);

    await expect(cards(page).filter({ hasText: AC_NAME })).toBeVisible();
    await expect(cards(page).filter({ hasText: AC_NAME })).toContainText('1.1 GB');
    await expect(cards(page).filter({ hasText: AC_NAME })).toContainText('3 items');
  });

  test('the caches of your apps show their danger and their four texts', async ({ page }) => {
    await openAppCaches(page);

    await openDetails(page, AC_NAME);

    const category = details(page);
    await expect(category.getByRole('region', { name: 'Danger level' })).toContainText('Low');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(ac(`library-caches.${text}`));
    }
  });

  test("a cache's finding names its app and its size", async ({ page }) => {
    await openAppCaches(page);

    await openDetails(page, AC_NAME);

    const items = details(page);
    await expect(items).toContainText('Editor: com.example.Editor.Helper');
    await expect(items).toContainText('Player: com.example.Player.ShipIt');
    const reason = ac('reason');
    await expect(items).toContainText(reason.replaceAll('{app}', 'Editor').replace('{size}', '920 MB'));
    await expect(items).toContainText(reason.replaceAll('{app}', 'Player').replace('{size}', '64 MB'));
  });

  test('selecting app caches makes the review name Low', async ({ page }) => {
    await openAppCaches(page);

    await page.getByRole('checkbox', { name: `Select ${AC_NAME}` }).check();

    await expect(selectionBar(page)).toContainText('3 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('Low');
    await expect(dialog).toContainText(ac('library-caches.if-removed'));
  });

  test("an app's cache is emptied and comes back by put back", async ({ page }) => {
    await openAppCaches(page);
    await openDetails(page, AC_NAME);
    await page.getByRole('checkbox', { exact: true, name: 'Select Editor' }).check();
    await page.getByRole('checkbox', { name: 'Select Editor: com.example.Editor.Helper' }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const emptied = items.getByRole('listitem').filter({ hasText: 'com.example.Editor.Helper' });
    await expect(emptied).toContainText('Emptied. The folder itself stays.');
    const item = items.getByRole('listitem').filter({ hasText: /^com\.example\.Editor(?!\.)/ });
    await expect(item).toContainText('In the Trash.');
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test("an open browser's cache cannot be selected", async ({ page }) => {
    await openBrowserCaches(page);
    await openDetails(page, AC_BROWSERS);

    const category = details(page);
    await expect(category).toContainText('1 item left out (30 MB)');
    await expect(category).toContainText(ac('left-out.app-open'));
    await expect(category.getByRole('checkbox', { name: /Firefox/ })).toHaveCount(0);
    await expect(category.getByRole('checkbox', { name: /^Select Google Chrome/ })).toHaveCount(3);
  });

  test('browser caches show their card with its size and item count', async ({ page }) => {
    await openBrowserCaches(page);

    const card = cards(page).filter({ hasText: AC_BROWSERS });
    await expect(card).toContainText('640 MB');
    await expect(card).toContainText('3 items');
  });

  test('browser caches show their danger and their four texts', async ({ page }) => {
    await openBrowserCaches(page);

    await openDetails(page, AC_BROWSERS);

    const category = details(page);
    await expect(category.getByRole('region', { name: 'Danger level' })).toContainText('Low');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(ac(`browsers.${text}`));
    }
  });

  test("a browser cache's finding names its browser, profile and folder", async ({ page }) => {
    await openBrowserCaches(page);

    await openDetails(page, AC_BROWSERS);

    const items = details(page);
    await expect(items).toContainText('Google Chrome, Default: Cache');
    await expect(items).toContainText('Google Chrome, Default: Code Cache');
    await expect(items).toContainText('Google Chrome, Profile 1: Cache');
    const reason = ac('reason');
    await expect(items).toContainText(reason.replaceAll('{app}', 'Google Chrome').replace('{size}', '400 MB'));
  });

  test('caches of web-based apps show their card and their findings', async ({ page }) => {
    await openBrowserCaches(page);

    const card = cards(page).filter({ hasText: AC_WEB_ENGINE });
    await expect(card).toContainText('25 MB');
    await expect(card).toContainText('2 items');
    await openDetails(page, AC_WEB_ENGINE);

    const items = details(page);
    await expect(items).toContainText('Redis Insight: Cache');
    await expect(items).toContainText('Redis Insight: GPUCache');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(items).toContainText(ac(`web-engine.${text}`));
    }
  });

  test('selecting browser caches makes the review name Low', async ({ page }) => {
    await openBrowserCaches(page);

    await page.getByRole('checkbox', { name: `Select ${AC_BROWSERS}` }).check();

    await expect(selectionBar(page)).toContainText('3 items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog).toContainText('Low');
    await expect(dialog).toContainText(ac('browsers.if-removed'));
  });

  test('a browser cache is emptied and comes back by put back', async ({ page }) => {
    await openBrowserCaches(page);
    await openDetails(page, AC_BROWSERS);
    await page.getByRole('checkbox', { name: 'Select Google Chrome, Default: Cache' }).check();
    await page.getByRole('checkbox', { name: 'Select Google Chrome, Default: Code Cache' }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 2 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: /^Cache/ });
    const empty = items.getByRole('listitem').filter({ hasText: /^Code Cache/ });
    await expect(empty).toContainText('Emptied. The folder itself stays.');
    await expect(empty).not.toContainText('In the Trash.');
    await expect(item).toContainText('In the Trash.');
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the switch for caches of your apps', async ({ page }) => {
    await mockCore(page, SCRIPTS.appCaches);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: ac('enabled.label') })).toBeChecked();
    await expect(page.getByText(ac('enabled.description'))).toBeVisible();
  });

  test("Xcode's categories show their cards with their sizes", async ({ page }) => {
    await openXcode(page);

    for (const [name, size] of [
      [XC_DEVICE_SUPPORT, '7.3 GB'],
      [XC_PREVIEWS, '640 MB'],
      [XC_SIMULATOR, '1.2 GB'],
      [XC_ARCHIVES, '860 MB'],
    ] as const) {
      await expect(cards(page).filter({ hasText: name })).toContainText(size);
    }
  });

  test('the archives show High and their four texts', async ({ page }) => {
    await openXcode(page);

    await openDetails(page, XC_ARCHIVES);

    const category = details(page);
    await expect(category.getByRole('region', { name: 'Danger level' })).toContainText('High');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(xc(`archives.${text}`));
    }
  });

  test('a device support finding names its platform, version and build', async ({ page }) => {
    await openXcode(page);

    await openDetails(page, XC_DEVICE_SUPPORT);

    const items = details(page);
    await expect(items).toContainText('iOS 15.0 (19A346)');
    await expect(items).toContainText('iOS 17.2 (21C62)');
    const plain = xc('reason.device-support');
    await expect(items).toContainText(
      plain.replaceAll('{platform}', 'iOS').replace('{version}', '15.0').replace('{size}', '3.2 GB'),
    );
    const recent = xc('reason.device-support-recent');
    await expect(items).toContainText(
      recent.replaceAll('{platform}', 'iOS').replace('{version}', '17.2').replace('{size}', '4.1 GB'),
    );
  });

  test('choosing an archive makes the review name High', async ({ page }) => {
    await openXcode(page);

    await page.getByRole('checkbox', { name: `Select ${XC_ARCHIVES}` }).check();

    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('High');
    await expect(dialog).toContainText(xc('archives.if-removed'));
  });

  test('a cleanup moves the archive and empties the caches, and put back brings the archive back', async ({ page }) => {
    await openXcode(page);
    for (const name of [XC_PREVIEWS, XC_SIMULATOR, XC_ARCHIVES]) {
      await page.getByRole('checkbox', { name: `Select ${name}` }).check();
    }
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    await expect(items.getByRole('listitem').filter({ hasText: 'Xcode previews' })).toContainText('In the Trash.');
    await expect(items.getByRole('listitem').filter({ hasText: 'Simulator caches' })).toContainText(
      'Emptied. The folder itself stays.',
    );
    const archive = items.getByRole('listitem').filter({ hasText: '.xcarchive' });
    await expect(archive).toContainText('In the Trash.');
    await archive.getByRole('button', { name: 'Put back' }).click();
    await expect(archive).toContainText('Put back where it was.');
  });

  test("Settings shows the switch for Xcode's files", async ({ page }) => {
    await mockCore(page, SCRIPTS.xcode);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: xc('enabled.label') })).toBeChecked();
    await expect(page.getByText(xc('enabled.description'))).toBeVisible();
  });

  test('unused toolchain versions show on Scan with their size', async ({ page }) => {
    await openUnusedToolchains(page);

    for (const name of [UT_RUST, UT_NODE]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await expect(cards(page).filter({ hasText: UT_RUST })).toContainText('2.7 GB');
    await expect(cards(page).filter({ hasText: UT_NODE })).toContainText('90 MB');
  });

  test('each unused toolchains category shows its danger and its four texts', async ({ page }) => {
    await openUnusedToolchains(page);

    for (const [name, key, danger] of [
      [UT_RUST, 'rust', 'Low'],
      [UT_NODE, 'node', 'Medium'],
    ] as const) {
      await openDetails(page, name);
      const category = details(page);
      await expect(category).toContainText(danger);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(ut(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('a toolchain’s finding names the version, the date and the command', async ({ page }) => {
    await openUnusedToolchains(page);

    await openDetails(page, UT_RUST);

    const items = details(page);
    await expect(items).toContainText('Rust 1.70.0-aarch64-apple-darwin');
    await expect(items).toContainText('Mar 10, 2024');
    await expect(items).toContainText('rustup toolchain install nightly-2023-06-01-aarch64-apple-darwin');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, UT_NODE);
    await expect(details(page)).toContainText('Node v12.22.12');
    await expect(details(page)).toContainText('nvm install 12.22.12');
  });

  test('selecting a Node version makes the review name Medium', async ({ page }) => {
    await openUnusedToolchains(page);

    await page.getByRole('checkbox', { name: `Select ${UT_NODE}` }).check();

    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('Medium');
    await expect(dialog).toContainText(ut('node.if-removed'));
  });

  test('a Rust toolchain goes to the Trash and comes back from the Cleanups', async ({ page }) => {
    await openUnusedToolchains(page);
    await openDetails(page, UT_RUST);
    await page.getByRole('checkbox', { name: /Select Rust 1\.70\.0/ }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await dialog.getByRole('button', { name: 'Move to Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 1 item to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: '1.70.0-aarch64-apple-darwin' });
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('Settings shows the Node versions switch off by default', async ({ page }) => {
    await mockCore(page, SCRIPTS.unusedToolchainsNoDocuments);
    await page.goto('/');
    await openSettings(page);

    await expect(page.getByRole('switch', { name: ut('enabled.label') })).toBeChecked();
    await expect(page.getByRole('switch', { name: ut('node-versions.label') })).not.toBeChecked();
    await expect(page.getByText(ut('node-versions.description'))).toBeVisible();
  });

  test('the scan screen shows the described section with the card’s figures', async ({ page }) => {
    await openDocker(page);

    const section = described(page);
    await expect(section).toContainText('Fresher shows what these take');
    const card = section.getByRole('listitem').first();
    await expect(card.getByRole('heading', { level: 3, name: 'OrbStack' })).toBeVisible();
    await expect(card).toContainText('OrbStack uses 3.6 GB inside its disk file; 1.6 GB of it is unused.');
    await expect(card.getByText(dk('figure.on-disk'))).toBeVisible();
    await expect(card).toContainText('1 MB');
    await expect(card.getByText(dk('figure.inside'))).toBeVisible();
    await expect(card).toContainText('3.6 GB');
  });

  test('Details opens the insight screen with its parts, levels and commands', async ({ page }) => {
    await openDocker(page);

    await openInsight(page, 'OrbStack');

    const screen = insightScreen(page);
    const parts = screen.getByRole('listitem');
    for (const [label, level, lines] of [
      ['build-cache', 'Safe', ['docker --context orbstack builder prune --all']],
      ['images', 'Low', ['docker --context orbstack image prune --all']],
      ['containers', 'Medium', []],
      ['volumes', 'High', ['docker --context orbstack volume prune']],
    ] as const) {
      const part = parts.filter({ hasText: dk(`part.${label}.label`) });
      await expect(part).toContainText(level);
      await expect(part).toContainText(dk(`part.${label}.effect`));
      await expect(part.locator('code')).toHaveText(lines);
    }
    await expect(screen).toContainText(dk('runtime.orbstack.gives-back'));
    await expect(screen).toContainText(dk('note.never-runs'));
  });

  test("Back returns the focus to the insight card's Details button", async ({ page }) => {
    await openDocker(page);
    await openInsight(page, 'OrbStack');

    await page.getByRole('button', { name: 'Back' }).click();

    await expect(described(page).getByRole('button', { name: 'Details' })).toBeFocused();
  });

  test('Copy puts the command line on the clipboard and says Copied', async ({ browserName, context, page }) => {
    // eslint-disable-next-line playwright/no-skipped-test -- WebKit does not grant reading the clipboard; the next test covers it
    test.skip(browserName !== 'chromium');
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    await openDocker(page);
    await openInsight(page, 'OrbStack');
    const part = insightScreen(page)
      .getByRole('listitem')
      .filter({ hasText: dk('part.build-cache.label') });

    await part.getByRole('button', { name: 'Copy' }).click();

    await expect(part).toContainText('Copied.');
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(COPIED_LINE);
  });

  test('Copy says Copied or that it could not copy, and the line stays selectable', async ({ page }) => {
    await openDocker(page);
    await openInsight(page, 'OrbStack');
    const part = insightScreen(page)
      .getByRole('listitem')
      .filter({ hasText: dk('part.build-cache.label') });

    await part.getByRole('button', { name: 'Copy' }).click();

    await expect(part).toContainText(/Copied\.|Could not copy/);
    await expect(part.locator('code')).toHaveText(COPIED_LINE);
  });

  test('Back returns to the scan screen', async ({ page }) => {
    await openDocker(page);
    await openInsight(page, 'OrbStack');

    await page.getByRole('button', { name: 'Back' }).click();

    await expect(heading(page, 'Scan')).toBeVisible();
    await expect(described(page)).toBeVisible();
  });

  test('a stopped runtime’s card says to start it and its screen lists no part', async ({ page }) => {
    await scanned(page, SCRIPTS.dockerStopped);

    const card = described(page).getByRole('listitem').first();
    await expect(card).toContainText('Docker Desktop is not running');
    await expect(card).toContainText('Start it and scan again');
    await openInsight(page, 'Docker Desktop');

    const screen = insightScreen(page);
    await expect(screen.getByRole('listitem').filter({ has: page.getByRole('heading', { level: 3 }) })).toHaveCount(0);
    await expect(screen.getByRole('button', { name: 'Copy' })).toHaveCount(0);
    await expect(screen).toContainText(dk('runtime.docker-desktop.gives-back'));
  });
});
