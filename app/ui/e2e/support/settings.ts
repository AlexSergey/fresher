import { expect, type Page } from '@playwright/test';

// A page of Settings, picked in its sidebar: the names are the buttons to click in order (a module group, then one of
// its modules); a group already open is left as it is. Waits for the page's heading.
export const settingsPage = async (page: Page, heading: string, ...path: string[]): Promise<void> => {
  for (const name of path) {
    const button = page
      .getByRole('navigation', { name: 'Settings sections' })
      .getByRole('button', { exact: true, name });

    if ((await button.getAttribute('aria-expanded')) !== 'true') {
      await button.click();
    }
  }
  await expect(page.getByRole('heading', { level: 1, name: heading })).toBeVisible();
};

// The names of the module groups in Settings' sidebar that are closed: clicking them in order opens every module.
export const closedGroups = async (page: Page): Promise<string[]> => {
  const groups = page.getByRole('navigation', { name: 'Settings sections' }).locator('button[aria-expanded="false"]');

  return (await groups.allTextContents()).map((name) => name.trim());
};
