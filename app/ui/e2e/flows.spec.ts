import { expect, type Locator, type Page, test } from '@playwright/test';

import { COMMANDS } from '../src/ipc';
import { APP_CACHES_DICTIONARY } from '../src/test/app-caches-scan';
import { APP_LEFTOVERS_DICTIONARY } from '../src/test/app-leftovers-scan';
import { CLEANUP_DICTIONARY, PLAN_GONE, PLAN_MISMATCH } from '../src/test/cleanup-plan';
import { DEV_CACHES_DICTIONARY } from '../src/test/dev-caches-scan';
import { DOCKER_DICTIONARY } from '../src/test/docker-scan';
import { DOWNLOADS_DICTIONARY } from '../src/test/downloads-scan';
import { JETBRAINS_DICTIONARY } from '../src/test/jetbrains-scan';
import { LARGEST_DICTIONARY } from '../src/test/largest-scan';
import { LOCAL_AI_MODELS_DICTIONARY } from '../src/test/local-ai-models-scan';
import { LOGS_DICTIONARY } from '../src/test/logs-scan';
import { STALE_PROJECTS_DICTIONARY } from '../src/test/stale-projects-scan';
import { TEMP_DICTIONARY } from '../src/test/temp-scan';
import { UNUSED_TOOLCHAINS_DICTIONARY } from '../src/test/unused-toolchains-scan';
import { XCODE_DICTIONARY } from '../src/test/xcode-scan';
import { mockCore, releaseScan, SCRIPTS, withheld } from './core-mock';
import { acceptedVersions } from './support/agreement-mock';
import {
  cardNames,
  details,
  detailsOf,
  fromCardMenu,
  navScan,
  openDetails,
  scanButton,
  scanFinished,
  tickItemsOf,
} from './support/cards';
import { preparedCommands, preparedSelections, releaseCleanup, requestQuit } from './support/cleanup-mock';
import { highTick, tickHigh } from './support/confirm';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { closedGroups, settingsPage } from './support/settings';

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

// A category's tile on Scan, by its name.
const tileOf = (page: Page, name: string): Locator =>
  cards(page).filter({ has: page.getByRole('heading', { exact: true, level: 3, name }) });

const zone = (page: Page, name: string): Locator => page.getByRole('region', { exact: true, name });

// The group a category's details name under its title, by the level the tests below speak of.
const GROUP_OF = {
  High: 'Only if you are sure',
  Low: 'Usually safe',
  Medium: 'Look before removing',
  Safe: 'Safe to remove',
} as const;

// Ticks a Scan group's Select all: only Safe to remove and Usually safe have one.
const selectZone = async (page: Page, name: string): Promise<void> => {
  await zone(page, name).getByRole('checkbox', { name: 'Select all' }).click();
  await expect(zone(page, name).getByRole('checkbox', { name: 'Select all' })).toBeChecked();
};

const selectionBar = (page: Page): Locator => page.getByRole('region', { name: 'Selection' });

const removeButton = (page: Page): Locator => page.getByRole('button', { exact: true, name: 'Move to the Trash' });

const notifications = (page: Page): Locator => page.getByRole('region', { name: 'Notifications' });

const QUIT = 'Stop the cleanup and quit?';

// Scans, ticks both categories and opens the review: the demo plan's three findings.
const reviewAll = async (page: Page): Promise<void> => {
  await page.goto('/');
  await scanButton(page).click();
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await selectionBar(page).getByRole('button', { name: 'Review' }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

// The review of the demo plan's findings in a world where the agreement was never accepted.
const reviewUnaccepted = async (page: Page): Promise<void> => {
  await mockCore(page, { ...SCRIPTS.cleaning, agreement: 'not-accepted' });
  await reviewAll(page);
};

const agreementSheet = (page: Page): Locator => page.getByRole('dialog', { name: 'Before your first cleanup' });

// Scans the demo, then opens Settings (the Ignored tabs).
const openIgnoring = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.ignoring);
  await page.goto('/');
  await scanButton(page).click();
  await expect(cards(page)).toHaveCount(2);
};

const openSettings = (page: Page): Promise<void> => page.getByRole('button', { exact: true, name: 'Settings' }).click();

// Settings on the page of the module `name`.
const openModulePage = async (page: Page, name: string): Promise<void> => {
  await openSettings(page);
  await settingsPage(page, name, ...(await closedGroups(page)), name);
};

// Settings on its Ignored page.
const openIgnored = async (page: Page): Promise<void> => {
  await openSettings(page);
  await settingsPage(page, 'Ignored', 'Ignored');
};

// Turns a module off, as the user does: Settings, Ignored, "Not looked for", then back to Scan.
const stopLookingFor = async (page: Page, module: string): Promise<void> => {
  await openIgnored(page);
  await page.getByRole('tab', { name: /^Not looked for/ }).click();
  await page.getByRole('switch', { name: `Ignore ${module}` }).click();
  await navScan(page).click();
};

const JB = {
  caches: 'Old JetBrains IDE caches',
  logs: 'Old JetBrains IDE logs',
  settings: 'Old JetBrains IDE settings',
  uninstalled: 'Removed JetBrains IDE data',
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

// Ticks WebStorm's caches and logs on their tiles, its settings (Look before removing) one by one, and opens the
// review.
const reviewWebStorm = async (page: Page): Promise<void> => {
  await openJetbrains(page);
  for (const name of [JB.caches, JB.logs]) {
    await page.getByRole('checkbox', { name: `Select ${name}` }).check();
  }
  await tickItemsOf(page, JB.settings);
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

const DESCRIBED = 'Shown, never removed';

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

// The Largest screen's own texts, as the core's locale file has them.
const lgt = (key: string): string => LARGEST_DICTIONARY[`core.largest.${key}`] ?? '';

const LARGEST = {
  archive: 'project-backup.zip',
  disk: 'ubuntu-24.04.dmg',
  movie: 'holiday-2022.mov',
  photos: 'Photos Library.photoslibrary',
  vm: 'Windows 11.pvm',
} as const;

const largestFiles = (page: Page): Locator => page.getByRole('list', { name: 'Largest files, largest first' });

const largestFolders = (page: Page): Locator => page.getByRole('list', { name: 'Folders, largest first' });

const largestRow = (list: Locator, name: string): Locator => list.getByRole('listitem').filter({ hasText: name });

const tick = (row: Locator, name: string): Locator =>
  row.getByRole('checkbox', { name: new RegExp(`^Choose ${name}`) });

// Ticks a row: the core picks it first, so the box is checked when its answer comes, not when it is clicked.
const pickRow = async (row: Locator, name: string): Promise<void> => {
  await tick(row, name).click();
  await expect(tick(row, name)).toBeChecked();
};

// Scans the Largest script and opens the screen: the Files list.
const openLargest = async (page: Page): Promise<void> => {
  await scanned(page, SCRIPTS.largest);
  await page.getByRole('navigation').getByRole('button', { name: 'Largest' }).click();
  await expect(heading(page, 'Largest files and folders')).toBeFocused();
  await expect(largestFiles(page)).toBeVisible();
};

const showFolders = async (page: Page): Promise<void> => {
  await page.getByRole('tab', { name: 'Folders' }).click();
  await expect(largestFolders(page)).toBeVisible();
};

// Picks the disk image on Largest and opens the review.
const reviewDisk = async (page: Page): Promise<void> => {
  await openLargest(page);
  await pickRow(largestRow(largestFiles(page), LARGEST.disk), LARGEST.disk);
  await selectionBar(page).getByRole('button', { name: 'Review' }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

// Scans the downloads, ticks the installers (Low) on their tile and the big untouched file (High) in its details, and
// opens the review.
const reviewDownloads = async (page: Page): Promise<void> => {
  await openDownloads(page);
  await page.getByRole('checkbox', { name: `Select ${DL_INSTALLERS}` }).check();
  await tickItemsOf(page, DL_LARGE);
  await selectionBar(page).getByRole('button', { name: 'Review' }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

test.describe('negative cases', () => {
  test('the sidebar has no Largest while High is withheld', async ({ page }) => {
    await mockCore(page, withheld(SCRIPTS.largest));
    await page.goto('/');
    await scanButton(page).click();
    await scanFinished(page);

    await expect(page.getByRole('navigation').getByRole('button', { name: 'Apps' })).toBeVisible();
    await expect(page.getByRole('navigation').getByRole('button', { name: 'Largest' })).toHaveCount(0);
  });

  test('no High tile is on Scan while High is withheld', async ({ page }) => {
    await mockCore(page, withheld(SCRIPTS.appLeftovers));
    await page.goto('/');
    await scanButton(page).click();
    await scanFinished(page);

    await expect(zone(page, 'Safe to remove')).toBeVisible();
    await expect(zone(page, 'Only if you are sure')).toHaveCount(0);
  });

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
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
      await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
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
    await category.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
    await expect(selectionBar(page)).toContainText('5 items selected');
    await expect(selectionBar(page)).not.toContainText('4,712');
  });

  test('the left-out line of the logs is not an item: nothing on it can be selected', async ({ page }) => {
    await openLogs(page);
    await openDetails(page, LOGS_CATEGORY);

    const category = details(page);
    await expect(category).toContainText(`1 item left out (3 MB): ${lg('left-out.recent')}`);
    await expect(category.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await category.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
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
    await expect(zone(page, 'Usually safe').getByRole('checkbox', { name: 'Select all' })).not.toBeChecked();
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('a value of 6 days is refused with a message and the saved value stays 7', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openModulePage(page, tmp('enabled.label'));
    const days = page.getByRole('spinbutton', { name: tmp('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await recordUpdates(page);

    await days.fill('6');

    await expect(page.getByText('Enter a whole number from 7 to 30.')).toBeVisible();
    expect(await sentUpdates(page)).toBe(0);
  });

  test('a number typed outside its limits is refused and the slider does not move', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openModulePage(page, tmp('enabled.label'));
    const slider = page.getByRole('slider', { name: `${tmp('older-than-days.label')}, days` });
    await recordUpdates(page);

    await page.getByRole('spinbutton', { name: tmp('older-than-days.label') }).fill('99');

    await expect(page.getByText('Enter a whole number from 7 to 30.')).toBeVisible();
    await expect(slider).toHaveValue('7');
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
    await expect(details(page)).toContainText('Everything found here was left out. The lines above say why.');
    await expect(details(page).getByRole('checkbox')).toHaveCount(0);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, GRADLE);

    await expect(details(page)).toContainText(`1 item left out (35 MB): ${dev('left-out.not-yours')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
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
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
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
    // A "Look before removing" category has no Select all: its items are ticked one by one.
    await expect(page.getByRole('checkbox', { name: /^Select all \d+$/ })).toHaveCount(0);
    const items = details(page).getByRole('checkbox', { name: /^Select /u });
    await expect(items.first()).toBeVisible();
    for (const item of await items.all()) {
      await item.check();
    }
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
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('1.3 GB');
  });

  test("an open app's cache cannot be selected", async ({ page }) => {
    await openAppCaches(page);
    await openDetails(page, AC_NAME);

    await expect(details(page)).toContainText(`1 item left out (90 MB): ${ac('left-out.app-open')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
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
    await expect(details(page)).toContainText('Everything found here was left out. The lines above say why.');
    await expect(details(page).getByRole('checkbox')).toHaveCount(0);
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, AI_HF_MODELS);

    await expect(details(page)).toContainText(`1 item left out (300 MB): ${ai('left-out.not-yours')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
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
    // A "Look before removing" category has no Select all: its items are ticked one by one.
    await expect(page.getByRole('checkbox', { name: /^Select all \d+$/ })).toHaveCount(0);
    const items = details(page).getByRole('checkbox', { name: /^Select /u });
    await expect(items.first()).toBeVisible();
    for (const item of await items.all()) {
      await item.check();
    }
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

    await fromCardMenu(page, NPM, `Ignore all ${NPM}`);

    await expect(cards(page).filter({ hasText: NPM })).toHaveCount(0);
    await expect(cards(page).first()).toBeVisible();
    await expect(page.getByText('17.2 GB')).toHaveCount(0);
  });

  test('ignoring the temporary files module puts its findings in no total', async ({ page }) => {
    await openTemp(page);
    await page.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();

    await stopLookingFor(page, tmp('enabled.label'));

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

  test("Review's rows have the same menu as the details", async ({ page }) => {
    await openTemp(page);
    await page.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();

    await selectionBar(page).getByRole('button', { name: 'Review' }).click();

    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: `The items of ${TEMP_CATEGORY}` })).toBeVisible();
    await page
      .getByRole('button', { name: /^Options for / })
      .first()
      .click();
    await expect(page.getByRole('menuitem')).toHaveText([/^Show in Finder/u, /^Ignore this item/u, /^Ignore all /u]);
    await expect(page.getByRole('separator')).toHaveCount(1);
  });

  test("nothing of Xcode's is selected before the user picks it", async ({ page }) => {
    await openXcode(page);

    for (const name of [XC_DEVICE_SUPPORT, XC_PREVIEWS, XC_SIMULATOR]) {
      await expect(page.getByRole('checkbox', { name: `Select ${name}` })).not.toBeChecked();
    }
    await expect(tileOf(page, XC_ARCHIVES).getByRole('checkbox')).not.toBeChecked();
    await expect(selectionBar(page)).toHaveCount(0);
    await zone(page, 'Safe to remove').getByRole('checkbox', { name: 'Select all' }).check();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_PREVIEWS}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_SIMULATOR}` })).toBeChecked();
    await expect(page.getByRole('checkbox', { name: `Select ${XC_DEVICE_SUPPORT}` })).not.toBeChecked();
    await expect(tileOf(page, XC_ARCHIVES).getByRole('checkbox')).not.toBeChecked();
    await expect(tileOf(page, XC_ARCHIVES)).not.toHaveAttribute('data-selected');
    await expect(selectionBar(page)).toContainText('2 items selected');
  });

  test('a left-out device support folder cannot be selected', async ({ page }) => {
    await openXcode(page);

    await openDetails(page, XC_DEVICE_SUPPORT);

    await expect(details(page)).toContainText(`1 item left out (300 MB): ${xc('left-out.special')}`);
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: /^Select all \d+$/ }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('300 MB');
  });

  test('the described section has no checkbox and adds nothing to the selection', async ({ page }) => {
    await openDocker(page);

    await expect(described(page).getByRole('checkbox')).toHaveCount(0);
    await selectZone(page, 'Safe to remove');
    await selectZone(page, 'Usually safe');
    for (const name of await zone(page, 'Only if you are sure').getByRole('heading', { level: 3 }).allTextContents()) {
      await tickItemsOf(page, name);
    }

    // Xcode's five findings are the whole selection; the section adds none.
    await expect(selectionBar(page)).toContainText('5 items selected');
  });

  test('the insight screen has no selection bar and Review never lists Docker', async ({ page }) => {
    await openDocker(page);
    await selectZone(page, 'Safe to remove');
    await selectZone(page, 'Usually safe');
    for (const name of await zone(page, 'Only if you are sure').getByRole('heading', { level: 3 }).allTextContents()) {
      await tickItemsOf(page, name);
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

  test('nothing on Largest is ticked when it opens', async ({ page }) => {
    await openLargest(page);

    await expect(page.getByRole('checkbox', { checked: true })).toHaveCount(0);
    await expect(page.getByRole('checkbox', { name: /^Choose / })).toHaveCount(4);
    await expect(page.getByRole('button', { name: /select all/i })).toHaveCount(0);
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('a withheld row cannot be ticked and says why', async ({ page }) => {
    await openLargest(page);

    const row = largestRow(largestFiles(page), LARGEST.photos);
    await expect(row.getByRole('checkbox')).toHaveCount(0);
    await expect(row).toContainText(lgt('withheld.read-in-part'));
  });

  test('a standard folder opens but cannot be chosen', async ({ page }) => {
    await openLargest(page);
    await showFolders(page);

    const movies = largestRow(largestFolders(page), 'Movies');
    await expect(movies.getByRole('checkbox')).toHaveCount(0);
    await expect(movies).toContainText(lgt('withheld.standard-folder'));
    await movies.getByRole('button', { name: 'Open' }).click();
    await expect(largestRow(largestFolders(page), LARGEST.movie)).toBeVisible();
    await expect(largestRow(largestFolders(page), LARGEST.movie).getByRole('checkbox')).toHaveCount(1);

    await page.getByRole('navigation', { name: 'Where you are' }).getByRole('button', { name: 'Home' }).click();
    const library = largestRow(largestFolders(page), 'Library');
    await expect(library.getByRole('checkbox')).toHaveCount(0);
    await expect(library.getByRole('button', { name: 'Open' })).toHaveCount(0);
    await expect(library).toContainText(lgt('withheld.library'));
  });

  test('the folders Fresher did not see are named above the list', async ({ page }) => {
    await openLargest(page);

    const notice = page.getByRole('region', { name: 'Desktop folder' });
    await expect(notice).toContainText('Your Desktop was not looked at.');
    const [noticeBox, listBox] = await Promise.all([notice.boundingBox(), largestFiles(page).boundingBox()]);
    expect((noticeBox?.y ?? 0) < (listBox?.y ?? 0)).toBe(true);
  });

  test('Not now starts no cleanup', async ({ page }) => {
    await reviewUnaccepted(page);
    await removeButton(page).click();

    await agreementSheet(page).getByRole('button', { name: 'Not now' }).click();

    await expect(agreementSheet(page)).toHaveCount(0);
    await expect(page.getByRole('dialog')).toHaveCount(0);
    expect(await preparedCommands(page)).toStrictEqual([]);
    expect(await acceptedVersions(page)).toStrictEqual([]);
    await expect(heading(page, 'Review')).toBeVisible();
  });

  test('Escape on the agreement starts no cleanup', async ({ page }) => {
    await reviewUnaccepted(page);
    await removeButton(page).click();
    await expect(agreementSheet(page)).toBeVisible();

    await page.keyboard.press('Escape');

    await expect(agreementSheet(page)).toHaveCount(0);
    await expect(page.getByRole('dialog')).toHaveCount(0);
    expect(await preparedCommands(page)).toStrictEqual([]);
    expect(await acceptedVersions(page)).toStrictEqual([]);
  });

  test('a High plan cannot be confirmed without its tick', async ({ page }) => {
    await reviewDownloads(page);
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    const confirm = dialog.getByRole('button', { name: 'Yes, move to the Trash' });

    await expect(highTick(dialog)).not.toBeChecked();
    await expect(confirm).toBeDisabled();

    await expect(dialog).toBeVisible();
    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toHaveCount(0);
    await expect(page.getByRole('alert')).toHaveCount(0);
  });

  test('the High tick is cleared when the confirmation opens again', async ({ page }) => {
    await reviewDownloads(page);
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await tickHigh(dialog);
    await expect(dialog.getByRole('button', { name: 'Yes, move to the Trash' })).toBeEnabled();
    await dialog.getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog).toHaveCount(0);

    await removeButton(page).click();

    await expect(highTick(dialog)).not.toBeChecked();
    await expect(dialog.getByRole('button', { name: 'Yes, move to the Trash' })).toBeDisabled();
  });

  test('a Look before removing group has no Select all', async ({ page }) => {
    await openDownloads(page);
    const high = zone(page, 'Only if you are sure');

    await expect(high.getByRole('heading', { level: 3, name: DL_LARGE })).toBeVisible();
    await expect(high.getByRole('checkbox', { name: 'Select all' })).toHaveCount(0);
    await expect(high.getByRole('checkbox', { exact: true, name: `Select ${DL_LARGE}` })).not.toBeChecked();
    await expect(page.getByRole('dialog')).toHaveCount(0);
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('nothing in the danger zone is selected before you pick it', async ({ page }) => {
    await openDownloads(page);

    for (const name of ['Look before removing', 'Only if you are sure']) {
      await expect(zone(page, name).getByRole('heading', { level: 3 }).first()).toBeVisible();
      await expect(zone(page, name).getByRole('checkbox', { name: 'Select all' })).toHaveCount(0);
      await expect(zone(page, name).getByRole('checkbox').first()).toBeVisible();
      for (const box of await zone(page, name).getByRole('checkbox').all()) {
        await expect(box).not.toBeChecked();
      }
    }
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('a Look before removing category ticked on Scan sends nothing until its items are ticked', async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_CACHES}` }).check();
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_SETTINGS}` }).check();

    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).toContainText('· 1 to check in Review');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    const settings = page.getByRole('article', { exact: true, name: LEFTOVERS_SETTINGS });
    await expect(settings.getByRole('checkbox', { name: /^Select /u }).first()).toBeVisible();
    for (const item of await settings.getByRole('checkbox', { name: /^Select /u }).all()) {
      await expect(item).not.toBeChecked();
    }
    await removeButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog).not.toContainText('Medium');
    expect(preparedSelections(page)).toStrictEqual([
      { categories: [{ category: 'app-leftovers.caches', except: [] }], findings: [] },
    ]);
  });

  test('a Look tile shows how many of its items are not checked', async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_SETTINGS}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    const settings = page.getByRole('article', { exact: true, name: LEFTOVERS_SETTINGS });
    const summary = page.getByRole('region', { name: 'What you are about to remove' });

    await expect(settings).toContainText(/0 of \d+ checked/u);
    await expect(settings).toContainText('· tick each one you have checked');
    await expect(summary).toContainText(/· \d+ not checked/u);
    await settings
      .getByRole('checkbox', { name: /^Select /u })
      .first()
      .check();
    await expect(settings).toContainText(/1 of \d+ checked/u);
  });

  test("the bar's button is inactive while nothing is ticked", async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_SETTINGS}` }).check();
    await expect(selectionBar(page)).toContainText('1 to check in Review');
    await expect(selectionBar(page)).not.toContainText('items selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();

    await expect(page.getByRole('article', { exact: true, name: LEFTOVERS_SETTINGS })).toBeVisible();
    await expect(page.getByRole('region', { name: 'Ready to remove' })).toContainText('Tick at least one item first.');
    await expect(removeButton(page)).toBeDisabled();
    await expect(removeButton(page)).toHaveAccessibleDescription('Tick at least one item first.');
    await expect(page.getByRole('dialog')).toHaveCount(0);
    expect(await preparedCommands(page)).toStrictEqual([]);
  });
});

test.describe('positive cases', () => {
  test('the sidebar has Largest when High is offered', async ({ page }) => {
    await scanned(page, SCRIPTS.largest);

    await expect(page.getByRole('navigation').getByRole('button', { name: 'Largest' })).toBeVisible();
  });

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
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
    await page.getByRole('region', { name: 'Moved to the Trash' }).getByRole('button', { name: 'Done' }).click();
    await navScan(page).click();

    await expect(trashLine(page)).toContainText('Waiting in the Trash: 4.1 GB, all of it from cleanups here.');
  });

  test('the result invites the user to look at the Trash and empty it themselves', async ({ page }) => {
    await mockCore(page, SCRIPTS.trashCleaning);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Look at what is there and empty it yourself if you no longer need it.');
    await expect(result.getByRole('button', { name: 'Show in the Trash' })).toHaveAccessibleDescription(
      'Look at what is there and empty it yourself if you no longer need it.',
    );
  });

  test('the result offers help to get an item back', async ({ page }) => {
    await mockCore(page, SCRIPTS.trashCleaning);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();
    const button = page.getByRole('region', { name: 'Moved to the Trash' }).getByRole('button', {
      name: 'Removed something you need?',
    });

    // WebKit does not focus a clicked button: open it from the keyboard, as the focus's return is the point.
    await button.focus();
    await page.keyboard.press('Enter');

    const help = page.getByRole('dialog', { name: 'Removed something you need?' });
    await expect(help.getByRole('heading', { level: 3 })).toHaveText([
      'Put it back',
      'Find it in the Trash',
      'If you deleted it permanently',
      'Tell us what went wrong',
    ]);
    await expect(help.getByRole('link')).toHaveCount(0);
    await expect(help).toContainText('template=wrong-finding.yml');
    await help.getByRole('button', { name: 'Close' }).click();
    await expect(help).toHaveCount(0);
    await expect(button).toBeFocused();
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

  test("Ignore all from a card's menu takes the category out of every total", async ({ page }) => {
    await openIgnoring(page);

    await page.getByRole('button', { name: 'Options for Example caches' }).click();
    await page.getByRole('menuitem', { name: 'Ignore all Example caches' }).click();

    await expect(cards(page)).toHaveCount(1);
    await expect(cardNames(page)).toHaveText(['Example downloads']);
    await expect(page.getByText('1 thing you ignored is not shown here.')).toBeVisible();
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

  test("Ignore all from a row's menu ignores its category", async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');

    await page.getByRole('button', { name: 'Options for Example: editor-cache' }).click();
    await page.getByRole('menuitem', { name: 'Ignore all Example caches' }).click();

    const recorded = async (): Promise<unknown> =>
      JSON.parse((await page.evaluate(() => sessionStorage.getItem('fresher.mock.ignore'))) ?? 'null') as unknown;
    await expect.poll(recorded).toMatchObject({ categories: ['demo.caches'] });
  });

  test('Settings lists the ignored item by its path and Restore brings its row back', async ({ page }) => {
    await openIgnoring(page);
    await openDetails(page, 'Example caches');
    await page.getByRole('button', { name: 'Options for Example: editor-cache' }).click();
    await page.getByRole('menuitem', { name: 'Ignore this item' }).click();
    const list = page.getByRole('list', { name: 'The items of Example caches' });
    await expect(list.getByRole('listitem')).toHaveCount(1);

    await openIgnored(page);
    await page.getByRole('tab', { name: /^Items/ }).click();
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
    await expect(page.getByRole('menuitem', { name: 'Show in Finder' })).toBeFocused();
    // The arrow keys skip the separator after Show in Finder.
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
    await expect(page.getByRole('menuitem', { name: 'Ignore all Example caches' })).toBeFocused();
    await page.keyboard.press('Enter');

    await expect(cards(page)).toHaveCount(1);
    await expect(page.getByText('1 thing you ignored is not shown here.')).toBeVisible();
  });

  test("Space on a card's menu item chooses it and leaves the menu closed", async ({ page }) => {
    await openIgnoring(page);
    const options = page.getByRole('button', { name: 'Options for Example caches' });

    await tabTo(page, options);
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menuitem', { name: 'Ignore all Example caches' })).toBeFocused();
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
    await expect(selectionBar(page)).not.toContainText('to check in Review');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` })).toBeChecked();
    for (const name of [LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(tileOf(page, name)).not.toHaveAttribute('data-selected');
      await expect(tileOf(page, name).getByRole('checkbox')).not.toBeChecked();
    }
  });

  test("Select all of a zone, then the selection bar's Review lists those categories", async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await scanButton(page).click();
    await expect(cards(page)).toHaveCount(2);

    await zone(page, 'Usually safe').getByRole('checkbox', { name: 'Select all' }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();

    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toContainText(
      'Example: installer.dmg',
    );
    await expect(page.getByRole('list', { name: 'The items of Example caches' })).toHaveCount(0);
  });

  test('from a scan to the review of what was ticked, one tile per category in the groups of the Scan screen', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.demo);
    await reviewAll(page);

    await expect(page.getByRole('heading', { level: 2 })).toHaveText(['Example caches', 'Example downloads']);
    await expect(page.getByRole('article', { name: 'Example caches' })).toContainText(
      'Safe to remove · Apps make these again on their own',
    );
    await expect(page.getByRole('article', { name: 'Example downloads' })).toContainText('Usually safe · ');
    await expect(page.getByRole('checkbox', { name: 'Select all' })).toHaveCount(0);
    const summary = page.getByRole('region', { name: 'What you are about to remove' });
    await expect(summary).toContainText('goes to the Trash · 3 items');
    await expect(
      summary.getByRole('img', { name: /^Size of each group: Safe to remove, .+; Usually safe, .+$/u }),
    ).toBeVisible();
    await expect(summary).not.toContainText('not checked');
  });

  test('unticking the last row of a tile removes the tile', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await reviewAll(page);
    await expect(page.getByRole('article', { name: 'Example downloads' })).toBeVisible();

    await page.getByRole('checkbox', { name: 'Select Example: installer.dmg' }).click();

    await expect(page.getByRole('article', { name: 'Example downloads' })).toHaveCount(0);
    await expect(page.getByRole('heading', { level: 2 })).toHaveText(['Example caches']);
  });

  test('unticking an item takes it out of the summary', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await reviewAll(page);
    const summary = page.getByRole('region', { name: 'What you are about to remove' });
    const caches = page.getByRole('article', { name: 'Example caches' });
    await expect(summary).toContainText('goes to the Trash · 3 items');

    await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).click();

    await expect(summary).toContainText('goes to the Trash · 2 items');
    await expect(page.getByRole('region', { name: 'Ready to remove' })).toContainText('2 items');
    await expect(caches).toContainText('1 of 2 checked');
    await expect(caches.getByRole('checkbox', { name: 'All Example caches' })).toBeChecked({ indeterminate: true });
  });

  test("a tile's pill turns green when every item is checked", async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_SETTINGS}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    const settings = page.getByRole('article', { exact: true, name: LEFTOVERS_SETTINGS });
    const pill = settings.getByText(/^\d+ of \d+ checked$/u);
    await expect(settings.getByRole('checkbox', { name: /^Select /u }).first()).toBeVisible();
    await expect(pill).not.toHaveAttribute('data-all');

    for (const item of await settings.getByRole('checkbox', { name: /^Select /u }).all()) {
      await item.check();
    }

    await expect(pill).toHaveAttribute('data-all', 'true');
    await expect(page.getByRole('region', { name: 'What you are about to remove' })).not.toContainText('not checked');
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
    await expect(page.getByRole('progressbar', { name: /^Scanning, \d+ percent$/u })).toBeVisible();

    await page.getByRole('button', { name: 'Stop' }).click();

    await expect(page.getByText('This scan is not complete')).toBeVisible();
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
    await settingsPage(page, 'Demo module', 'Debug', 'Demo module');
    const enabled = page.getByRole('switch', { name: 'Look for Demo module' });
    await expect(enabled).toBeChecked();

    await enabled.click();

    await expect(enabled).not.toBeChecked();
  });

  test.describe('the settings search', () => {
    const open = async (page: Page): Promise<Locator> => {
      await mockCore(page, SCRIPTS.settings);
      await page.goto('/');
      await openSettings(page);
      await expect(heading(page, 'General')).toBeVisible();

      return page.getByRole('searchbox', { name: 'Search settings' });
    };

    test('a search with no match says so and leaves the settings unchanged', async ({ page }) => {
      const field = await open(page);

      await field.fill('zebra');

      await expect(heading(page, 'Nothing matches “zebra”')).toBeVisible();
      await field.fill('');
      await expect(heading(page, 'General')).toBeVisible();
      await expect(page.getByRole('radio', { name: 'Follow your Mac' })).toBeChecked();
    });

    test('a search lists every setting whose label or description matches, the match marked', async ({ page }) => {
      const field = await open(page);

      await field.fill('days');

      await expect(heading(page, '1 setting matches “days”')).toBeVisible();
      const hit = page.getByRole('button', { name: /Stale after/u });
      await expect(hit).toContainText('Scan');
      await expect(hit).toContainText('90 days. A project untouched for this long counts as stale.');
      await expect(hit.locator('mark')).toHaveText('days');
    });

    test('a click opens the page with the setting focused', async ({ page }) => {
      const field = await open(page);
      await field.fill('days');

      await page.getByRole('button', { name: /Stale after/u }).click();

      await expect(heading(page, 'Scan')).toBeVisible();
      await expect(field).toHaveValue('');
      await expect(page.getByRole('slider', { name: 'Stale after, days' })).toBeFocused();
    });

    test('Esc clears the search', async ({ page }) => {
      const field = await open(page);
      await field.fill('days');
      await expect(heading(page, '1 setting matches “days”')).toBeVisible();

      await page.keyboard.press('Escape');

      await expect(field).toHaveValue('');
      await expect(heading(page, 'General')).toBeVisible();
    });
  });

  test('adding a scan folder with the picker, then resetting the list', async ({ page }) => {
    await mockCore(page, SCRIPTS.settings);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Settings' }).click();
    await settingsPage(page, 'Scan', 'Scan');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await expect(result).toContainText('Freed on the disk: about 2.6 GB.');
    await expect(result.getByRole('button', { name: 'Show in the Trash' })).toHaveCount(0);
  });

  test('Stop mid-cleanup ends it stopped, the rest untouched', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaningHeld);
    await reviewAll(page);
    await removeButton(page).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openIgnored(page);

    await page.getByRole('switch', { name: 'Ignore Debug' }).click();

    await navScan(page).click();
    await expect(page.getByText('1 thing you ignored is not shown here.')).toBeVisible();
    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
    await expect(page.getByText(/you ignored is left out/)).toHaveCount(0);
  });

  test('ignoring the module from Settings hides its cards and turning it off brings them back', async ({ page }) => {
    await openIgnoring(page);
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Not looked for/ }).click();

    await page.getByRole('switch', { name: 'Ignore Demo module' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Not looked for/ }).click();
    await page.getByRole('switch', { name: 'Ignore Demo module' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
  });

  test('a module of an ignored group is locked with its note and turning the group off frees it', async ({ page }) => {
    await openIgnoring(page);
    await openIgnored(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('tab', { name: /^Not looked for/ }).click();

    const locked = page.getByRole('switch', { name: 'Ignore Demo module' });
    await expect(locked).toBeDisabled();
    await expect(page.getByText('Ignored with its section: Debug')).toBeVisible();
    await page.getByRole('tab', { name: /^Sections/ }).click();
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('tab', { name: /^Not looked for/ }).click();
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeEnabled();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(2);
  });

  test('ignored modules and groups stay ignored after a reload', async ({ page }) => {
    await openIgnoring(page);
    await stopLookingFor(page, 'Demo module');
    await expect(cards(page)).toHaveCount(0);

    await page.reload();

    await openIgnored(page);
    await expect(page.getByRole('tab', { name: /^Not looked for/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('the ignored line on the overview opens Settings on the tab with an entry', async ({ page }) => {
    await openIgnoring(page);
    await stopLookingFor(page, 'Demo module');

    await page.getByRole('button', { name: 'Show in Settings' }).click();

    await expect(page.getByRole('tab', { name: /^Not looked for/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('ignoring a module drops what was ticked from the selection', async ({ page }) => {
    await openIgnoring(page);
    await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
    await expect(page.getByRole('button', { name: /^Review \d+ items? selected/ })).toBeVisible();

    await stopLookingFor(page, 'Demo module');

    await expect(cards(page)).toHaveCount(0);
    await expect(page.getByRole('button', { name: /selected/ })).toHaveCount(0);
  });

  test('the details of a JetBrains category name its level', async ({ page }) => {
    await openJetbrains(page);

    for (const [name, level] of [
      [JB.caches, 'Safe to remove'],
      [JB.logs, 'Safe to remove'],
      [JB.settings, 'Look before removing'],
      [JB.uninstalled, 'Look before removing'],
    ] as const) {
      await openDetails(page, name);
      await expect(details(page)).toContainText(level);
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
    await openDetails(page, JB.caches);
    const about = details(page);
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
    await expect(removed).toContainText('Look before removing');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await expect(result).toContainText('Moved 3 items to the Trash');
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const caches = items.getByRole('listitem').filter({ hasText: 'WebStorm2024.3' }).first();
    await caches.getByRole('button', { name: 'Put back' }).click();
    await expect(caches).toContainText('Put back where it was.');
  });

  test('ignoring the module in Settings hides its totals, Settings lists it and turning it off restores it', async ({
    page,
  }) => {
    await openJetbrains(page);

    await stopLookingFor(page, 'JetBrains IDEs');

    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Not looked for/ }).click();
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
    await openIgnored(page);

    await page.getByRole('switch', { name: 'Ignore Development' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('switch', { name: 'Ignore Development' }).click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(4);
  });

  test('the module’s days setting is in Settings with its text', async ({ page }) => {
    await mockCore(page, SCRIPTS.jetbrains);
    await page.goto('/');
    await openModulePage(page, jb('enabled.label'));

    const days = page.getByRole('spinbutton', { name: jb('uninstalled-after-days.label') });
    await expect(days).toHaveValue('30');
    await expect(page.getByText(jb('uninstalled-after-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: `Look for ${jb('enabled.label')}` })).toBeChecked();
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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

  test('the details of Old app logs show the category, Safe and its texts', async ({ page }) => {
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, lg('enabled.label'));

    const days = page.getByRole('spinbutton', { name: lg('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await expect(page.getByText(lg('older-than-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: `Look for ${lg('enabled.label')}` })).toBeChecked();
    await recordUpdates(page);

    await days.fill('6');

    await expect(page.getByText('Enter a whole number from 7 to 90.')).toBeVisible();
    expect(await sentUpdates(page)).toBe(0);
  });

  test('dragging the slider saves the number', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openModulePage(page, tmp('enabled.label'));
    const slider = page.getByRole('slider', { name: `${tmp('older-than-days.label')}, days` });
    const days = page.getByRole('spinbutton', { name: tmp('older-than-days.label') });
    await expect(slider).toHaveValue('7');
    await recordUpdates(page);

    await slider.focus();
    await page.keyboard.press('ArrowRight');

    await expect(slider).toHaveValue('8');
    await expect(days).toHaveValue('8');
    expect(await sentUpdates(page)).toBe(1);
  });

  test('Settings has the number field at 7 days with its text, and 14 is accepted', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await openModulePage(page, tmp('enabled.label'));

    const days = page.getByRole('spinbutton', { name: tmp('older-than-days.label') });
    await expect(days).toHaveValue('7');
    await expect(page.getByText(tmp('older-than-days.description'))).toBeVisible();
    await expect(page.getByRole('switch', { name: `Look for ${tmp('enabled.label')}` })).toBeChecked();

    await recordUpdates(page);

    await days.fill('14');

    await expect(days).toHaveValue('14');
    expect(await sentUpdates(page)).toBe(1);
    await expect(page.getByText('Enter a whole number from 7 to 30.')).toHaveCount(0);
  });

  test('ignoring the module and the group System from Settings, then restoring both', async ({ page }) => {
    await openTemp(page);

    await stopLookingFor(page, tmp('enabled.label'));

    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Not looked for/ }).click();
    const ignored = page.getByRole('switch', { name: 'Ignore Temporary files' });
    await expect(ignored).toBeChecked();
    await ignored.click();
    await navScan(page).click();
    await expect(cards(page)).toHaveCount(1);

    await openIgnored(page);
    await page.getByRole('tab', { name: /^Sections/ }).click();
    await page.getByRole('switch', { name: 'Ignore System' }).click();

    await navScan(page).click();
    await expect(cards(page)).toHaveCount(0);
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Sections/ }).click();
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
    await expect(details(page)).toContainText('Safe to remove');
    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await openDetails(page, NPM);

    const npm = details(page);
    await expect(npm).toContainText('Usually safe');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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

    await fromCardMenu(page, NPM, `Ignore all ${NPM}`);

    await expect(cards(page).filter({ hasText: NPM })).toHaveCount(0);
    for (const name of [CARGO, GRADLE, XCODE_DATA]) {
      await expect(cards(page).filter({ hasText: name })).toBeVisible();
    }
    await openIgnored(page);
    await page.getByRole('tab', { name: /^Items/ }).click();
    await page.getByRole('button', { name: `Restore ${NPM}` }).click();
    await navScan(page).click();
    await expect(cards(page).filter({ hasText: NPM })).toContainText('8.9 GB');
  });

  test('ignoring the developer caches module hides them all', async ({ page }) => {
    await openDevCaches(page);

    await stopLookingFor(page, dev('enabled.label'));

    await expect(cards(page)).toHaveCount(0);
  });

  test('Settings shows the developer caches switch alone', async ({ page }) => {
    await mockCore(page, SCRIPTS.devCaches);
    await page.goto('/');
    await openModulePage(page, dev('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${dev('enabled.label')}` })).toBeChecked();
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
      await expect(category).toContainText(GROUP_OF[danger as keyof typeof GROUP_OF]);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(left(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('Remove everything brings its Look categories to Review to check; ticked there, the review names High', async ({
    page,
  }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_DATA);

    await choice(page, 'everything').click();

    await page.getByRole('button', { name: 'Back to Scan' }).click();
    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` })).toBeChecked();
    for (const name of [LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(tileOf(page, name)).toHaveAttribute('data-selected', 'true');
      await expect(tileOf(page, name).getByRole('checkbox')).toBeChecked();
    }
    // Only the caches go: the Look categories are to check, nothing of them is ticked for the user.
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).toContainText('· 2 to check in Review');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    for (const name of [LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(page.getByRole('checkbox', { exact: true, name: `All ${name}` })).not.toBeChecked();
      await page.getByRole('checkbox', { exact: true, name: `All ${name}` }).check();
    }
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 5 items to the Trash?' });
    await expect(dialog).toContainText('High');
    await expect(dialog).toContainText(left('data.if-removed'));
    expect(preparedSelections(page)).toStrictEqual([
      {
        categories: [
          { category: 'app-leftovers.caches', except: [] },
          { category: 'app-leftovers.settings', except: [] },
          { category: 'app-leftovers.data', except: [] },
        ],
        findings: [],
      },
    ]);
  });

  test('an item ticked in Review goes', async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_SETTINGS}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    const settings = page.getByRole('article', { exact: true, name: LEFTOVERS_SETTINGS });

    await settings
      .getByRole('checkbox', { name: /^Select /u })
      .first()
      .check();
    await expect(settings.getByRole('checkbox', { exact: true, name: `All ${LEFTOVERS_SETTINGS}` })).not.toBeChecked();
    await removeButton(page).click();

    await expect(page.getByRole('dialog', { name: 'Move 1 item to the Trash?' })).toBeVisible();
    const [sent] = preparedSelections(page);
    expect(sent?.categories).toStrictEqual([]);
    expect(sent?.findings).toHaveLength(1);
  });

  test('ticking all of a Look category in Review sends the category', async ({ page }) => {
    await openLeftovers(page);
    await page.getByRole('checkbox', { exact: true, name: `Select ${LEFTOVERS_DATA}` }).check();
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();

    await page.getByRole('checkbox', { exact: true, name: `All ${LEFTOVERS_DATA}` }).check();
    await removeButton(page).click();

    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('High');
    expect(preparedSelections(page)).toStrictEqual([
      { categories: [{ category: 'app-leftovers.data', except: [] }], findings: [] },
    ]);
  });

  test('Keep settings selects only the caches and the review names Safe', async ({ page }) => {
    await openLeftovers(page);
    await openDetails(page, LEFTOVERS_CACHES);

    await choice(page, 'keep-settings').click();

    await page.getByRole('button', { name: 'Back to Scan' }).click();

    await expect(page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` })).toBeChecked();
    for (const name of [LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(tileOf(page, name)).not.toHaveAttribute('data-selected');
      await expect(tileOf(page, name).getByRole('checkbox')).not.toBeChecked();
    }
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('to check in Review');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, left('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${left('enabled.label')}` })).toBeChecked();
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
      await expect(category).toContainText(GROUP_OF[danger as keyof typeof GROUP_OF]);
      for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(dl(`${key}.${text}`));
      }
      await page.getByRole('button', { name: 'Back to Scan' }).click();
    }
  });

  test('selecting the installers and the big files makes the review name High', async ({ page }) => {
    await openDownloads(page);

    await page.getByRole('checkbox', { name: `Select ${DL_INSTALLERS}` }).check();
    await tickItemsOf(page, DL_LARGE);

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
    await tickItemsOf(page, DL_LARGE);
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await tickHigh(dialog);
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, dl('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${dl('enabled.label')}` })).toBeChecked();
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
      await expect(category).toContainText(GROUP_OF[danger as keyof typeof GROUP_OF]);
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

    await tickItemsOf(page, SP_PYTHON);
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
    await tickItemsOf(page, SP_PYTHON);
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, sp('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${sp('enabled.label')}` })).toBeChecked();
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
      await expect(category).toContainText(GROUP_OF[danger as keyof typeof GROUP_OF]);
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

    await tickItemsOf(page, AI_LM_STUDIO);

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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await expect(category).toContainText('Look before removing');
    for (const text of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(ai(`ollama.${text}`));
    }
    await expect(category).toContainText('llama3:8b, llama3:latest');
  });

  test('the confirm dialog lists the files that go with a model on demand', async ({ page }) => {
    await scanned(page, SCRIPTS.localAiModelsOllama);
    await tickItemsOf(page, AI_OLLAMA);
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, ai('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${ai('enabled.label')}` })).toBeChecked();
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
    await expect(category).toContainText('Usually safe');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await expect(category).toContainText('Usually safe');
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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, ac('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${ac('enabled.label')}` })).toBeChecked();
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
    await expect(category).toContainText('Only if you are sure');
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

    await tickItemsOf(page, XC_ARCHIVES);

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
    for (const name of [XC_PREVIEWS, XC_SIMULATOR]) {
      await page.getByRole('checkbox', { name: `Select ${name}` }).check();
    }
    await tickItemsOf(page, XC_ARCHIVES);
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await tickHigh(dialog);
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, xc('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${xc('enabled.label')}` })).toBeChecked();
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
      await expect(category).toContainText(GROUP_OF[danger as keyof typeof GROUP_OF]);
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

    await tickItemsOf(page, UT_NODE);

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
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

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
    await openModulePage(page, ut('enabled.label'));

    await expect(page.getByRole('switch', { name: `Look for ${ut('enabled.label')}` })).toBeChecked();
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

  test('Largest lists the largest files with kind, place and size', async ({ page }) => {
    await openLargest(page);

    const row = largestRow(largestFiles(page), LARGEST.disk);
    await expect(row).toContainText('Disk image');
    await expect(row).toContainText('~/Downloads');
    await expect(row).toContainText('12 GB');
    await expect(largestFiles(page).getByRole('listitem').first()).toContainText(LARGEST.disk);
    await expect(largestFiles(page).getByRole('listitem').last()).toContainText(LARGEST.archive);
  });

  test('a filter narrows the list', async ({ page }) => {
    await openLargest(page);

    await page.getByLabel('Kind').selectOption({ label: 'Video (1)' });
    await expect(largestFiles(page).getByRole('listitem')).toHaveCount(1);
    await expect(largestFiles(page)).toContainText(LARGEST.movie);

    await page.getByLabel('Kind').selectOption({ label: 'Any kind' });
    await page.getByLabel('Not changed for').selectOption({ label: lgt('unchanged.two-years') });
    await expect(largestFiles(page).getByRole('listitem')).toHaveCount(2);
    await expect(largestFiles(page)).not.toContainText(LARGEST.vm);
  });

  test('an archive names the folder beside it', async ({ page }) => {
    await openLargest(page);

    await expect(largestRow(largestFiles(page), LARGEST.archive)).toContainText(
      lgt('unpacked').replace('{folder}', 'project-backup'),
    );
  });

  test('the folders view opens a folder and goes back by its way', async ({ page }) => {
    await openLargest(page);
    await showFolders(page);

    await expect(largestFolders(page).getByRole('listitem')).toHaveCount(6);
    await largestRow(largestFolders(page), 'Parallels').getByRole('button', { name: 'Open' }).click();
    await expect(heading(page, 'Largest files and folders')).toBeFocused();
    await expect(largestFolders(page).getByRole('listitem')).toHaveCount(1);
    await expect(largestFolders(page)).toContainText(LARGEST.vm);
    await expect(page.getByRole('navigation', { name: 'Where you are' }).getByText('Parallels')).toHaveAttribute(
      'aria-current',
      'page',
    );

    await page.getByRole('navigation', { name: 'Where you are' }).getByRole('button', { name: 'Home' }).click();
    await expect(largestFolders(page).getByRole('listitem')).toHaveCount(6);
  });

  test('a ticked file is reviewed under Your own files at High', async ({ page }) => {
    await openLargest(page);

    await pickRow(largestRow(largestFiles(page), LARGEST.disk), LARGEST.disk);
    await expect(selectionBar(page)).toContainText('1 item selected');
    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    const own = page.getByRole('article', { name: 'Your own files' });
    await expect(own).toContainText(LARGEST.disk);
    await expect(own).toContainText('Only if you are sure · May hold the only copy of your data');
    await expect(own).toContainText('1 picked');

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await expect(dialog).toContainText('High');
    await expect(dialog).toContainText(LARGEST_DICTIONARY['core.own-file.if-removed'] ?? '');
    await expect(highTick(dialog)).not.toBeChecked();
    await expect(dialog.getByRole('checkbox', { name: /^Delete permanently instead/u })).not.toBeChecked();
    await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
  });

  test('a picked file goes to the Trash and leaves the list', async ({ page }) => {
    await reviewDisk(page);

    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await tickHigh(dialog);
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();
    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toContainText('Moved 1 item to the Trash');

    await page.getByRole('navigation').getByRole('button', { name: 'Largest' }).click();
    await expect(largestFiles(page)).toBeVisible();
    await expect(largestFiles(page)).not.toContainText(LARGEST.disk);
    await expect(largestFiles(page).getByRole('listitem')).toHaveCount(4);
    await expect(
      page.getByText('A cleanup changed the disk since this scan. Scan again for exact sizes.'),
    ).toBeVisible();
  });

  test('a picked file comes back by put back', async ({ page }) => {
    await reviewDisk(page);
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 1 item to the Trash?' });
    await tickHigh(dialog);
    await dialog.getByRole('button', { name: 'Yes, move to the Trash' }).click();

    const result = page.getByRole('region', { name: 'Moved to the Trash' });
    await result.getByRole('button', { name: 'See this cleanup' }).click();
    const items = page.getByRole('list', { name: 'The items of this cleanup' });
    const item = items.getByRole('listitem').filter({ hasText: LARGEST.disk });
    await expect(item).toContainText('In the Trash.');
    await item.getByRole('button', { name: 'Put back' }).click();
    await expect(item).toContainText('Put back where it was.');
  });

  test('the agreement is not asked again once accepted', async ({ page }) => {
    await reviewUnaccepted(page);
    await removeButton(page).click();
    await agreementSheet(page).getByRole('button', { name: 'I understand' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();

    await removeButton(page).click();

    await expect(agreementSheet(page)).toHaveCount(0);
    await expect(page.getByRole('dialog', { name: /^Move 3 items to the Trash/ })).toBeVisible();
    expect(await acceptedVersions(page)).toStrictEqual([1]);
  });

  test('the first cleanup asks for the agreement and goes on after I understand', async ({ page }) => {
    await reviewUnaccepted(page);

    await removeButton(page).click();

    const sheet = agreementSheet(page);
    await expect(sheet.getByRole('button', { name: 'Not now' })).toBeFocused();
    await expect(sheet.getByRole('listitem')).toHaveCount(5);
    expect(await preparedCommands(page)).toStrictEqual([]);
    await sheet.getByRole('button', { name: 'I understand' }).click();

    await expect(page.getByRole('dialog', { name: /^Move 3 items to the Trash/ })).toBeVisible();
    expect(await preparedCommands(page)).toStrictEqual(['prepare_cleanup']);
    expect(await acceptedVersions(page)).toStrictEqual([1]);
  });

  test('Settings shows the agreement and when it was accepted', async ({ page }) => {
    await scanned(page, SCRIPTS.demo);
    await openSettings(page);
    await settingsPage(page, 'About', 'About');

    const about = page.getByRole('region', { name: 'About' });

    await expect(about.getByRole('listitem')).toHaveCount(5);
    await expect(about).toContainText('You accepted it on Jan 1, 2025.');
  });

  test('Settings says the agreement is not accepted yet', async ({ page }) => {
    await mockCore(page, { ...SCRIPTS.demo, agreement: 'not-accepted' });
    await page.goto('/');
    await openSettings(page);
    await settingsPage(page, 'About', 'About');

    await expect(page.getByRole('region', { name: 'About' })).toContainText(
      'Not accepted yet. Fresher asks before your first cleanup.',
    );
  });

  test('the full licence opens from the agreement and from About', async ({ page }) => {
    await reviewUnaccepted(page);
    await removeButton(page).click();

    await agreementSheet(page).getByRole('button', { name: 'Read the full licence' }).click();

    const licence = page.getByRole('dialog', { name: 'Licence' });
    await expect(licence).toContainText('MIT licence');
    await expect(licence).toContainText('Apache License');
    await page.keyboard.press('Escape');
    await expect(licence).toHaveCount(0);
    await agreementSheet(page).getByRole('button', { name: 'Not now' }).click();

    await openSettings(page);
    await settingsPage(page, 'About', 'About');
    await page.getByRole('region', { name: 'About' }).getByRole('button', { name: 'Read the full licence' }).click();

    await expect(licence).toContainText('Permission is hereby granted');
  });

  test('the High tick lets the cleanup go on and is sent with it', async ({ page }) => {
    await reviewDownloads(page);
    await removeButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    const confirm = dialog.getByRole('button', { name: 'Yes, move to the Trash' });
    await expect(confirm).toBeDisabled();

    await tickHigh(dialog);
    await expect(confirm).toBeEnabled();
    await confirm.click();

    // The core mock refuses a High echo without the tick: the cleanup runs only because it was sent.
    await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toContainText('Moved 3 items to the Trash');
  });

  test('Select all of Safe to remove ticks its tiles', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await scanButton(page).click();
    const safe = zone(page, 'Safe to remove');

    await safe.getByRole('checkbox', { name: 'Select all' }).check();

    await expect(safe.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
    await expect(selectionBar(page)).toContainText('2 items selected');

    await safe.getByRole('checkbox', { name: 'Select all' }).uncheck();

    await expect(safe.getByRole('checkbox', { name: 'Select Example caches' })).not.toBeChecked();
    await expect(selectionBar(page)).toHaveCount(0);
  });

  test('a picked file is reviewed on a tile of the Only if you are sure group', async ({ page }) => {
    await reviewDisk(page);

    await expect(page.getByRole('heading', { level: 2 })).toHaveText(['Your own files']);
    await expect(page.getByRole('article', { name: 'Your own files' })).toContainText(LARGEST.disk);
    await expect(page.getByRole('region', { name: 'What you are about to remove' })).toContainText(
      'Only if you are sure',
    );
  });
});
