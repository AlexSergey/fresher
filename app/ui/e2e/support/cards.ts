import { expect, type Locator, type Page } from '@playwright/test';

// The category cards of the scan screen: each has a Details button, which opens the category's details.

// The Scan button of the scan screen's toolbar, not the sidebar's Scan item, which has the same name.
export const scanButton = (page: Page): Locator =>
  page.getByRole('main').getByRole('button', { exact: true, name: 'Scan' });

// The sidebar's Scan item.
export const navScan = (page: Page): Locator =>
  page.getByRole('navigation', { name: 'Screens' }).getByRole('button', { exact: true, name: 'Scan' });

// A card's Details button, by the card's name.
export const detailsOf = (page: Page, name: string): Locator =>
  page
    .getByRole('listitem')
    .filter({ has: page.getByRole('heading', { level: 3, name }) })
    .getByRole('button', { name: 'Details' });

// The cards' names, in the order the page lists them: by danger zone, the safest first.
export const cardNames = (page: Page): Locator =>
  page.locator('section[data-level]').getByRole('heading', { level: 3 });

// Waits for the end of a scan on Scan: Scan again shows once the core's last overview has arrived.
export const scanFinished = async (page: Page): Promise<void> => {
  await expect(page.getByRole('button', { name: 'Scan again' })).toBeVisible();
};

// Opens a category's details from its card: the details' heading, the category's name, takes the focus.
export const openDetails = async (page: Page, name: string): Promise<void> => {
  await detailsOf(page, name).click();
  await expect(page.getByRole('heading', { level: 1, name })).toBeFocused();
};

// The open category's details, for what they say: the screen the app shows, not the one fading out.
export const details = (page: Page): Locator => page.locator('[data-screen^="category:"]:not([aria-hidden])');

// Picks an item of a card's menu: "Ignore all" and the category's name.
export const fromCardMenu = async (page: Page, card: string, item: string): Promise<void> => {
  await page.getByRole('button', { name: `Options for ${card}` }).click();
  await page.getByRole('menuitem', { name: item }).click();
};

// Ticks a "Look before removing" category's tile on Scan, which only brings it to Review, then ticks every one of its
// items there one by one, as a user does, and goes back to Scan. Every item is ticked: the tile's all-checkbox in
// Review reads checked.
export const tickItemsOf = async (page: Page, name: string): Promise<void> => {
  await page.getByRole('checkbox', { exact: true, name: `Select ${name}` }).check();
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
  const tile = page.getByRole('article', { exact: true, name });
  const items = tile.getByRole('checkbox', { name: /^Select /u });
  await expect(items.first()).toBeVisible();
  for (const item of await items.all()) {
    await item.check();
  }
  await expect(tile.getByRole('checkbox', { exact: true, name: `All ${name}` })).toBeChecked();
  await navScan(page).click();
  await scanFinished(page);
};
