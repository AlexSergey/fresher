import { expect, type Locator, type Page, test } from '@playwright/test';

import { COMMANDS } from '../src/ipc';
import { APP_LEFTOVERS_DICTIONARY } from '../src/test/app-leftovers-scan';
import { CLEANUP_DICTIONARY, PLAN_GONE, PLAN_MISMATCH } from '../src/test/cleanup-plan';
import { DEV_CACHES_DICTIONARY } from '../src/test/dev-caches-scan';
import { JETBRAINS_DICTIONARY } from '../src/test/jetbrains-scan';
import { TEMP_DICTIONARY } from '../src/test/temp-scan';
import { mockCore, releaseScan, SCRIPTS } from './core-mock';
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

const cards = (page: Page): Locator => page.getByRole('button', { name: /^Open / });

const selectionBar = (page: Page): Locator => page.getByRole('region', { name: 'Selection' });

const removeButton = (page: Page): Locator => page.getByRole('button', { name: 'Move to Trash…' });

const notifications = (page: Page): Locator => page.getByRole('region', { name: 'Notifications' });

const QUIT = 'A cleanup is running. Stop after the current item and quit?';

// Scans, ticks both categories and opens the review: the demo plan's three findings.
const reviewAll = async (page: Page): Promise<void> => {
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await page.getByRole('button', { name: /^Review/ }).click();
  await expect(heading(page, 'Review')).toBeFocused();
};

const demoModule = (page: Page): Locator => page.getByRole('button', { name: /^Demo module/ });

// Scans the demo, then opens Settings (the Ignored tabs).
const openIgnoring = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.ignoring);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
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

const jetbrainsModule = (page: Page): Locator => page.getByRole('button', { name: /^JetBrains IDEs/ });

// Scans the JetBrains script and opens the module's page.
const openJetbrains = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.jetbrains);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(jetbrainsModule(page)).toContainText('1.7 GB');
  await jetbrainsModule(page).click();
  await expect(heading(page, 'JetBrains IDEs')).toBeFocused();
};

// Ticks WebStorm's three categories on the module's page and opens the review.
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

const tempModule = (page: Page): Locator => page.getByRole('button', { name: /^Temporary files/ });

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

// Scans the temporary files script and opens the module's page.
const openTemp = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(tempModule(page)).toContainText('837 MB');
  await tempModule(page).click();
  await expect(heading(page, 'Temporary files')).toBeFocused();
};

// The developer caches module's own texts, as its locale file has them.
const dev = (key: string): string => DEV_CACHES_DICTIONARY[`dev-caches.${key}`] ?? '';

const NPM = dev('npm.name');
const CARGO = dev('cargo.name');
const GRADLE = dev('gradle.name');
const XCODE_DATA = dev('xcode-derived-data.name');

const devCachesModule = (page: Page): Locator => page.getByRole('button', { name: /^Developer caches/ });

// Scans a developer caches script (Xcode closed unless said) and opens the module's page.
const openDevCaches = async (
  page: Page,
  script: typeof SCRIPTS.devCaches | typeof SCRIPTS.devCachesXcodeOpen = SCRIPTS.devCaches,
  total = '17.2 GB',
): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(devCachesModule(page)).toContainText(total);
  await devCachesModule(page).click();
  await expect(heading(page, 'Developer caches')).toBeFocused();
};

// The removed apps module's own texts, as its locale file has them.
const left = (key: string): string => APP_LEFTOVERS_DICTIONARY[`app-leftovers.${key}`] ?? '';

const LEFTOVERS_CACHES = left('caches.name');
const LEFTOVERS_SETTINGS = left('settings.name');
const LEFTOVERS_DATA = left('data.name');

const leftoversModule = (page: Page): Locator => page.getByRole('button', { name: /^Removed apps' leftovers/ });

// Scans a removed apps script and opens the module's page.
const openLeftovers = async (
  page: Page,
  script: typeof SCRIPTS.appLeftovers | typeof SCRIPTS.appLeftoversWithDevCaches = SCRIPTS.appLeftovers,
  total = '2.5 GB',
): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(leftoversModule(page)).toContainText(total);
  await leftoversModule(page).click();
  await expect(heading(page, left('enabled.label'))).toBeFocused();
};

const choice = (page: Page, name: string): Locator => page.getByRole('button', { name: left(`choice.${name}.label`) });

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('nothing ignored shows no ignored line on the overview and no note on the module page', async ({ page }) => {
    await openIgnoring(page);

    await expect(page.getByText(/you ignored is left out/)).toHaveCount(0);
    await demoModule(page).click();
    await expect(page.getByText('You ignore this module.')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Ignore this module' })).toBeVisible();
  });

  test('a second Scan while the core runs one shows its refusal', async ({ page }) => {
    await mockCore(page, SCRIPTS.unannounced);
    await page.goto('/');
    const scan = page.getByRole('button', { exact: true, name: 'Scan' });

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

    const settings = page.getByRole('region', { name: JB.settings });
    await expect(settings).toContainText('1 item');
    await expect(settings).toContainText(`2 items left out (600 MB): ${jb('left-out.running')}`);
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

    const category = page.getByRole('region', { name: TEMP_CATEGORY });
    await expect(category).toContainText(`4,712 items left out (790 MB): ${tmp('left-out.recent')}`);
    await expect(category.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await category.getByRole('checkbox', { name: `Select ${TEMP_CATEGORY}` }).check();
    await expect(selectionBar(page)).toContainText('5 items selected');
    await expect(selectionBar(page)).not.toContainText('4,712');
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
    await openDevCaches(page, SCRIPTS.devCachesXcodeOpen, '12.9 GB');

    const data = page.getByRole('region', { name: XCODE_DATA });
    await expect(data).toContainText(`2 items left out (4.3 GB): ${dev('left-out.app-open')}`);
    await expect(page.getByRole('region', { name: GRADLE })).toContainText(
      `1 item left out (35 MB): ${dev('left-out.not-yours')}`,
    );
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await expect(data.getByRole('checkbox')).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select ${GRADLE}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await expect(selectionBar(page)).not.toContainText('4.3 GB');
  });

  test('the left-out lines of removed apps cannot be selected', async ({ page }) => {
    await openLeftovers(page);

    await expect(page.getByRole('region', { name: LEFTOVERS_CACHES })).toContainText(
      `1 item left out (220 MB): ${left('left-out.running')}`,
    );
    await expect(page.getByRole('region', { name: LEFTOVERS_SETTINGS })).toContainText(
      `1 item left out (3 MB): ${left('left-out.used-recently')}`,
    );
    await expect(page.getByRole('checkbox', { name: /left out/ })).toHaveCount(0);
    await page.getByRole('checkbox', { name: `Select ${LEFTOVERS_CACHES}` }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');
    await expect(selectionBar(page)).not.toContainText('220 MB');
  });

  test('a choice selects nothing outside its module', async ({ page }) => {
    await openLeftovers(page, SCRIPTS.appLeftoversWithDevCaches);
    await devCachesModule(page).click();
    await page.getByRole('checkbox', { name: `Select ${NPM}` }).check();
    await expect(selectionBar(page)).toContainText('1 item selected');
    await leftoversModule(page).click();

    await choice(page, 'keep-settings').click();

    await expect(selectionBar(page)).toContainText('3 items selected');
    await devCachesModule(page).click();
    await expect(page.getByRole('checkbox', { name: `Select ${NPM}` })).toBeChecked();
  });

  test('an ignored cache category is in no total', async ({ page }) => {
    await openDevCaches(page);

    await page.getByRole('button', { name: `Ignore ${NPM}` }).click();

    await expect(devCachesModule(page)).toContainText('8.3 GB');
    await expect(devCachesModule(page)).not.toContainText('17.2 GB');
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page).filter({ hasText: NPM })).toHaveCount(0);
    await expect(page.getByText('8.3 GB').first()).toBeVisible();
  });

  test('ignoring the temporary files module puts its findings in no total', async ({ page }) => {
    await openTemp(page);

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await expect(page.getByRole('button', { name: /selected/ })).toHaveCount(0);
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(0);
  });
});

test.describe('positive cases', () => {
  test('from a scan to the review of what was ticked', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveText([/^Open Example caches/, /^Open Example downloads/]);

    await page.getByRole('button', { name: 'Open Example downloads' }).click();
    await expect(heading(page, 'Demo module')).toBeFocused();
    const downloads = page.getByRole('region', { name: 'Example downloads' });
    await expect(downloads.getByRole('button', { name: 'Show the items of Example downloads' })).toHaveAttribute(
      'aria-expanded',
      'true',
    );
    await expect(downloads).toBeInViewport();

    await downloads.getByRole('checkbox', { name: 'Select Example: installer.dmg' }).check();
    const caches = page.getByRole('region', { name: 'Example caches' });
    await caches.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
    await expect(selectionBar(page)).toContainText('2 items selected');

    await selectionBar(page).getByRole('button', { name: 'Review' }).click();
    await expect(heading(page, 'Review')).toBeFocused();
    await expect(page.getByRole('list', { name: 'The items of Example downloads' })).toContainText(
      'Example: installer.dmg',
    );
    await expect(page.getByRole('list', { name: 'The items of Example caches' })).toContainText(
      'Example: browser-cache',
    );
    await expect(removeButton(page)).toBeEnabled();
  });

  test('Stop ends the scan as partial, with what it found so far', async ({ page }) => {
    await mockCore(page, SCRIPTS.live);
    await page.goto('/');

    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');

    await page.getByRole('button', { name: 'Stop' }).click();

    await expect(page.getByText('This scan is incomplete')).toBeVisible();
    await expect(cards(page)).toHaveText([/1 item$/, /1 item$/]);
    await expect(page.getByRole('button', { name: /^Demo module/ })).toContainText('2.3 GB');
    await expect(page.getByRole('button', { name: 'Scan again' })).toBeVisible();
  });

  test('a long list loads its pages as the keyboard moves to its end', async ({ page }) => {
    await mockCore(page, SCRIPTS.large);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(3);

    await cards(page).first().click();
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
    await expect(heading(page, 'Overview')).toBeVisible();

    await tabTo(page, page.getByRole('button', { exact: true, name: 'Scan' }));
    await page.keyboard.press('Enter');
    const open = page.getByRole('button', { name: 'Open Example caches' });
    await expect(open).toBeVisible();

    await tabTo(page, open);
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Demo module')).toBeFocused();

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

    await expect(page.getByRole('navigation').getByText('Debug · Ignored')).toBeVisible();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(page.getByText('1 thing you ignored is left out of these results.')).toBeVisible();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(2);
    await expect(page.getByText(/you ignored is left out/)).toHaveCount(0);
  });

  test('ignoring the module from Settings dims it in the sidebar and Restore on its page brings it back', async ({
    page,
  }) => {
    await openIgnoring(page);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();

    await page.getByRole('switch', { name: 'Ignore Demo module' }).click();

    await expect(demoModule(page)).toContainText('Ignored');
    await demoModule(page).click();
    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await page.getByRole('button', { exact: true, name: 'Restore' }).click();
    await expect(page.getByText('You ignore this module.')).toHaveCount(0);
    await expect(demoModule(page)).not.toContainText('Ignored');
  });

  test('a module of an ignored group is locked with its note and the group’s Restore frees it', async ({ page }) => {
    await openIgnoring(page);
    await openSettings(page);
    await page.getByRole('switch', { name: 'Ignore Debug' }).click();
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();

    const locked = page.getByRole('switch', { name: 'Ignore Demo module' });
    await expect(locked).toBeDisabled();
    await expect(page.getByText('Ignored with its group: Debug')).toBeVisible();
    await demoModule(page).click();
    await page.getByRole('button', { name: 'Restore the group' }).click();
    await expect(page.getByText('You ignore this module.')).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeEnabled();
  });

  test('ignored modules and groups stay ignored after a reload', async ({ page }) => {
    await openIgnoring(page);
    await demoModule(page).click();
    await page.getByRole('button', { name: 'Ignore this module' }).click();
    await expect(page.getByText('You ignore this module.')).toBeVisible();

    await page.reload();

    await expect(demoModule(page)).toContainText('Ignored');
    await openSettings(page);
    await expect(page.getByRole('tab', { name: /^Ignored modules/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('the ignored line on the overview opens Settings on the tab with an entry', async ({ page }) => {
    await openIgnoring(page);
    await demoModule(page).click();
    await page.getByRole('button', { name: 'Ignore this module' }).click();
    await page.getByRole('button', { name: 'Overview' }).click();

    await page.getByRole('button', { name: 'Show in Settings' }).click();

    await expect(page.getByRole('tab', { name: /^Ignored modules/ })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('Ignore this module on the module page moves it to the Ignored modules tab', async ({ page }) => {
    await openIgnoring(page);
    await demoModule(page).click();

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await openSettings(page);
    await expect(page.getByRole('tab', { name: 'Ignored modules · 1' })).toBeVisible();
    await expect(page.getByRole('switch', { name: 'Ignore Demo module' })).toBeChecked();
  });

  test('ignoring a module drops what was ticked from the selection', async ({ page }) => {
    await openIgnoring(page);
    await demoModule(page).click();
    await page
      .getByRole('checkbox', { name: /^Select / })
      .first()
      .check();
    await expect(page.getByRole('button', { name: /^Review \d+ items? selected/ })).toBeVisible();

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await expect(page.getByRole('button', { name: /selected/ })).toHaveCount(0);
  });

  test('the sidebar lists JetBrains IDEs under Development and its page names each category with its level', async ({
    page,
  }) => {
    await openJetbrains(page);

    await expect(page.getByRole('navigation').getByText('Development', { exact: true })).toBeVisible();
    for (const [name, level] of [
      [JB.caches, 'Safe'],
      [JB.logs, 'Safe'],
      [JB.settings, 'Medium'],
      [JB.uninstalled, 'Medium'],
    ] as const) {
      await expect(page.getByRole('region', { name })).toContainText(level);
    }
    const caches = page.getByRole('region', { name: JB.caches });
    await caches.getByText('About this category').click();
    await expect(caches).toContainText(jb('caches.what'));
    await expect(caches).toContainText(jb('caches.if-removed'));
  });

  test('the items left out are lines under their category, with their reason', async ({ page }) => {
    await openJetbrains(page);

    await expect(page.getByRole('region', { name: JB.caches })).toContainText(
      `3 items left out (1.4 GB): ${jb('left-out.current')}`,
    );
    await expect(page.getByRole('region', { name: JB.settings })).toContainText(
      `2 items left out (600 MB): ${jb('left-out.running')}`,
    );
  });

  test('the data of an IDE that is no longer installed is its own Medium category with the findings named', async ({
    page,
  }) => {
    await openJetbrains(page);

    const removed = page.getByRole('region', { name: JB.uninstalled });
    await expect(removed).toContainText('Medium');
    await expect(removed).toContainText('2 items');
    await removed.getByText('About this category').click();
    await expect(removed).toContainText(jb('uninstalled.what'));
    await expect(removed).toContainText(jb('uninstalled.if-removed'));
    await removed.getByRole('button', { name: `Show the items of ${JB.uninstalled}` }).click();
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

  test('ignoring the module from its page hides its totals, Settings lists it and turning it off restores it', async ({
    page,
  }) => {
    await openJetbrains(page);

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await expect(jetbrainsModule(page)).toContainText('Ignored');
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(0);
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    const ignored = page.getByRole('switch', { name: 'Ignore JetBrains IDEs' });
    await expect(ignored).toBeChecked();
    await ignored.click();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(4);
    await expect(jetbrainsModule(page)).not.toContainText('Ignored');
  });

  test('ignoring the group Development hides JetBrains IDEs and restoring the group brings it back', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.jetbrains);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(4);
    await openSettings(page);

    await page.getByRole('switch', { name: 'Ignore Development' }).click();

    await expect(page.getByRole('navigation').getByText('Development · Ignored')).toBeVisible();
    await jetbrainsModule(page).click();
    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await page.getByRole('button', { name: 'Restore the group' }).click();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(4);
    await expect(page.getByRole('navigation').getByText('Development · Ignored')).toHaveCount(0);
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

  test('the by-meaning view puts WebStorm’s findings under its app line, not DataGrip’s', async ({ page }) => {
    await mockCore(page, SCRIPTS.jetbrains);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(4);

    await page.getByRole('radio', { name: 'By meaning' }).click();

    const app = page.getByRole('list', { name: 'WebStorm' });
    await expect(app).toContainText('1.3 GB');
    await page.getByRole('button', { name: 'Show what WebStorm keeps' }).click();
    const kept = page.getByRole('group', { name: 'What WebStorm keeps' });
    await expect(kept).toContainText(JB.caches);
    await expect(kept).not.toContainText(JB.uninstalled);
    await expect(page.getByRole('list', { name: 'DataGrip' })).toContainText('not installed');
  });

  test('the sidebar lists Temporary files under System and its page shows the category, Safe and its texts', async ({
    page,
  }) => {
    await openTemp(page);

    await expect(page.getByRole('navigation').getByText('System', { exact: true })).toBeVisible();
    const category = page.getByRole('region', { name: TEMP_CATEGORY });
    await expect(category).toContainText('Safe');
    await expect(category).toContainText('5 items');
    await category.getByText('About this category').click();
    for (const key of ['what', 'made-by', 'if-removed', 'comes-back']) {
      await expect(category).toContainText(tmp(`old-files.${key}`));
    }
  });

  test('the items left out are lines under the category: the recent ones, the system’s and the pipe', async ({
    page,
  }) => {
    await openTemp(page);

    const category = page.getByRole('region', { name: TEMP_CATEGORY });
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

  test('ignoring the module from its page and the group System from Settings, then restoring both', async ({
    page,
  }) => {
    await openTemp(page);

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(tempModule(page)).toContainText('Ignored');
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored modules/ }).click();
    const ignored = page.getByRole('switch', { name: 'Ignore Temporary files' });
    await expect(ignored).toBeChecked();
    await ignored.click();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(1);

    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored groups/ }).click();
    await page.getByRole('switch', { name: 'Ignore System' }).click();

    await expect(page.getByRole('navigation').getByText('System · Ignored')).toBeVisible();
    await tempModule(page).click();
    await expect(page.getByText('You ignore this module.')).toBeVisible();
    await page.getByRole('button', { name: 'Restore the group' }).click();
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(cards(page)).toHaveCount(1);
    await expect(page.getByRole('navigation').getByText('System · Ignored')).toHaveCount(0);
  });

  test('the overview by place lists the two temporary folders by their names', async ({ page }) => {
    await mockCore(page, SCRIPTS.temp);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(1);

    await page.getByRole('radio', { name: 'By place' }).click();

    const roots = page.getByRole('list', { name: 'Scanned folders' });
    await expect(roots).toContainText(TEMP_DICTIONARY['core.place.user-temp.name'] ?? '');
    await expect(roots).toContainText(TEMP_DICTIONARY['core.place.shared-temp.name'] ?? '');
    await expect(roots.getByText('tmp', { exact: true })).toHaveCount(0);
  });

  test('developer caches show under Development with their size', async ({ page }) => {
    await openDevCaches(page);

    await expect(page.getByRole('navigation').getByText('Development', { exact: true })).toBeVisible();
    await expect(devCachesModule(page)).toContainText('17.2 GB');
    for (const name of [NPM, CARGO, GRADLE, XCODE_DATA]) {
      await expect(page.getByRole('region', { name })).toBeVisible();
    }
    await expect(page.getByRole('region', { name: NPM })).toContainText('8.9 GB');
  });

  test('a developer cache category shows its danger and its five texts', async ({ page }) => {
    await openDevCaches(page);

    const npm = page.getByRole('region', { name: NPM });
    await expect(npm).toContainText('Low');
    await expect(page.getByRole('region', { name: XCODE_DATA })).toContainText('Safe');
    await expect(npm).toContainText(dev('npm.what'));
    await npm.getByText('About this category').click();
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

    await page.getByRole('button', { name: `Ignore ${NPM}` }).click();

    await expect(page.getByRole('region', { name: NPM })).toHaveCount(0);
    for (const name of [CARGO, GRADLE, XCODE_DATA]) {
      await expect(page.getByRole('region', { name })).toBeVisible();
    }
    await openSettings(page);
    await page.getByRole('tab', { name: /^Ignored items/ }).click();
    await page.getByRole('button', { name: `Restore ${NPM}` }).click();
    await devCachesModule(page).click();
    await expect(page.getByRole('region', { name: NPM })).toContainText('8.9 GB');
    await expect(devCachesModule(page)).toContainText('17.2 GB');
  });

  test('ignoring the developer caches module hides them all', async ({ page }) => {
    await openDevCaches(page);

    await page.getByRole('button', { name: 'Ignore this module' }).click();

    await expect(devCachesModule(page)).toContainText('Ignored');
    await expect(page.getByText('You ignore this module.')).toBeVisible();
    for (const name of [NPM, CARGO, GRADLE, XCODE_DATA]) {
      await expect(page.getByRole('region', { name })).toHaveCount(0);
    }
    await page.getByRole('button', { name: 'Overview' }).click();
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

  test('a project’s build data counts in Xcode’s line', async ({ page }) => {
    await mockCore(page, SCRIPTS.devCaches);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(1);

    await page.getByRole('radio', { name: 'By meaning' }).click();

    const app = page.getByRole('list', { name: 'Xcode' });
    await expect(app).toContainText('4.3 GB');
    await page.getByRole('button', { name: 'Show what Xcode keeps' }).click();
    await expect(page.getByRole('group', { name: 'What Xcode keeps' })).toContainText(XCODE_DATA);
  });

  test('removed apps’ leftovers show under Apps with their size', async ({ page }) => {
    await openLeftovers(page);

    await expect(page.getByRole('navigation').getByText('Apps', { exact: true })).toBeVisible();
    await expect(leftoversModule(page)).toContainText('2.5 GB');
    for (const name of [LEFTOVERS_CACHES, LEFTOVERS_SETTINGS, LEFTOVERS_DATA]) {
      await expect(page.getByRole('region', { name })).toBeVisible();
    }
    await expect(page.getByRole('region', { name: LEFTOVERS_DATA })).toContainText('1.8 GB');
  });

  test('each removed-app category shows its danger and its five texts', async ({ page }) => {
    await openLeftovers(page);

    for (const [name, key, danger] of [
      [LEFTOVERS_CACHES, 'caches', 'Safe'],
      [LEFTOVERS_SETTINGS, 'settings', 'Medium'],
      [LEFTOVERS_DATA, 'data', 'High'],
    ] as const) {
      const category = page.getByRole('region', { name });
      await expect(category).toContainText(danger);
      await expect(category).toContainText(left(`${key}.what`));
      await category.getByText('About this category').click();
      for (const text of ['made-by', 'if-removed', 'comes-back']) {
        await expect(category).toContainText(left(`${key}.${text}`));
      }
    }
  });

  test('Remove everything selects the three categories and the review names High', async ({ page }) => {
    await openLeftovers(page);

    await choice(page, 'everything').click();

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

    await choice(page, 'keep-settings').click();

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

  test('a removed app’s line in the map is named by its id', async ({ page }) => {
    await mockCore(page, SCRIPTS.appLeftovers);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(cards(page)).toHaveCount(3);

    await page.getByRole('radio', { name: 'By meaning' }).click();

    const app = page.getByRole('list', { name: 'com.example.Gone' });
    await expect(app).toContainText('not installed');
    await expect(app).toContainText('2.5 GB');
  });
});
