import { expect, type Locator, type Page, test } from '@playwright/test';

import { CLEANUP_DICTIONARY, PLAN_GONE, PLAN_MISMATCH } from '../src/test/cleanup-plan';
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

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
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
    await expect(page.getByRole('list', { name: 'Parts of the scan' })).toContainText('scanning');

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
    await page.getByRole('button', { name: 'Settings' }).click();
    const enabled = page.getByRole('switch', { name: 'Demo module' });
    await expect(enabled).toBeChecked();

    await enabled.click();

    await expect(enabled).not.toBeChecked();
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
});
