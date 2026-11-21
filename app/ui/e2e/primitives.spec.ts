import { expect, type Locator, type Page, test } from '@playwright/test';

import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// The keyboard of the primitives, in the real engines: what a unit test's DOM cannot do (a native control's Space and
// Enter). Each control is a story of the gallery.

const openStory = async (page: Page, id: string): Promise<void> => {
  await page.goto(`/gallery.html#/${id}`);
  await expect(page.locator(`[data-story="${id}"]`)).toBeVisible();
};

// Counts the `click` events the control receives from now on: a native control's keyboard activation is a click.
const countClicks = async (control: Locator): Promise<() => Promise<number>> => {
  await control.evaluate((element) => {
    element.dataset.clicks = '0';
    element.addEventListener('click', () => {
      element.dataset.clicks = String(Number(element.dataset.clicks) + 1);
    });
  });

  return async () => Number(await control.getAttribute('data-clicks'));
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('a disabled checkbox is out of the tab order', async ({ page }) => {
    await openStory(page, 'Checkbox/disabled');

    await page.keyboard.press('Tab');

    await expect(page.getByRole('checkbox', { name: 'Xcode caches' })).not.toBeFocused();
  });
});

test.describe('positive cases', () => {
  test('the checkbox toggles with Space', async ({ page }) => {
    await openStory(page, 'Checkbox/unchecked');
    const checkbox = page.getByRole('checkbox', { name: 'Xcode caches' });
    const clicks = await countClicks(checkbox);

    await checkbox.focus();
    await page.keyboard.press('Space');

    expect(await clicks()).toBe(1);
  });

  test('the switch toggles with Space and Enter', async ({ page }) => {
    await openStory(page, 'Switch/off');
    const control = page.getByRole('switch', { name: 'Include the Trash' });
    const clicks = await countClicks(control);

    await control.focus();
    await page.keyboard.press('Space');
    await page.keyboard.press('Enter');

    expect(await clicks()).toBe(2);
  });

  test('the button fires on Enter and Space', async ({ page }) => {
    await openStory(page, 'Button/primary-rest');
    const button = page.getByRole('button', { name: 'Move to Trash' }).first();
    const clicks = await countClicks(button);

    await button.focus();
    await page.keyboard.press('Enter');
    await page.keyboard.press('Space');

    expect(await clicks()).toBe(2);
  });

  test('the icon button shows its label on keyboard focus', async ({ page }) => {
    await openStory(page, 'IconButton/rest');

    await page.keyboard.press('Tab');

    await expect(page.getByRole('button', { name: 'Close' }).first()).toBeFocused();
    await expect(page.getByRole('tooltip')).toHaveText('Close');
  });

  test('the tooltip hides on Escape and shows again on hover', async ({ page }) => {
    await openStory(page, 'Tooltip/top');
    const button = page.getByRole('button', { name: 'Clean' });
    const tooltip = page.getByRole('tooltip');

    await button.focus();
    await expect(tooltip).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(tooltip).toBeHidden();
    await expect(button).toBeFocused();

    await button.hover();
    await expect(tooltip).toHaveText('Moves the items to the Trash');
  });

  test('the segmented control takes the focus on its chosen option', async ({ page }) => {
    await openStory(page, 'SegmentedControl/three-name');

    await page.keyboard.press('Tab');

    await expect(page.getByRole('radio', { name: 'Name' })).toBeFocused();
  });

  test('Space that chooses a menu item leaves the menu closed, however the engine releases it', async ({ page }) => {
    await openStory(page, 'Menu/closed');
    const trigger = page.getByRole('button', { name: 'Options for Caches' });
    const clicks = await countClicks(trigger);

    await page.keyboard.press('Tab');
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menuitem', { name: 'Ignore this category' })).toBeFocused();
    await page.keyboard.press('Space');

    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(trigger).toBeFocused();
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(await clicks()).toBe(1);
  });
});
