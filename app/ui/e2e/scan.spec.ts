import { expect, type Locator, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton, scanFinished } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The scan screen's disk card: the ring and its legend after a scan of the demo's home volume; and the scanning card
// while a scan runs.

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

const scan = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.demo);
  await page.goto('/');
  await scanButton(page).click();
  await scanFinished(page);
};

// Puts the pointer on the ring's band, `degrees` clockwise from the top.
const pointAtRing = async (page: Page, degrees: number): Promise<void> => {
  const box = await page.locator('svg[data-ring="arcs"]').boundingBox();

  if (box === null) {
    throw new Error('the ring is not on the page');
  }

  const radius = box.width / 2 - 6;
  const angle = (degrees * Math.PI) / 180;

  await page.mouse.move(
    box.x + box.width / 2 + Math.sin(angle) * radius,
    box.y + box.height / 2 - Math.cos(angle) * radius,
  );
};

const legend = (page: Page): Locator => page.getByRole('list', { name: 'Parts of the disk' });

const rowOf = (page: Page, name: string): Locator =>
  legend(page)
    .getByRole('listitem')
    .filter({ has: page.getByText(name, { exact: true }) });

test.describe('negative cases', () => {
  test('the legend shows no explanation until its i is pressed', async ({ page }) => {
    await scan(page);

    await expect(rowOf(page, 'macOS').getByRole('button', { name: 'What is macOS?' })).toHaveAttribute(
      'aria-expanded',
      'false',
    );
    await expect(page.getByText(/^The system itself/u)).toBeHidden();
  });

  test('before a scan the disk card holds Scan and does not say what can be freed', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    const disk = page.getByRole('region', { name: 'Your disk' });

    await expect(disk.getByRole('button', { name: 'Scan' })).toBeVisible();
    await expect(rowOf(page, 'Can be freed')).toContainText('Not scanned yet');
    await expect(disk.getByText('can be freed', { exact: true })).toHaveCount(0);
  });
});

test.describe('positive cases', () => {
  test('hovering a ring segment highlights its legend row, and the reverse', async ({ page }) => {
    await scan(page);
    const row = rowOf(page, 'Your files and apps');

    await expect(row).not.toHaveAttribute('data-highlighted');

    await pointAtRing(page, 60);

    await expect(row).toHaveAttribute('data-highlighted', '');

    await page.mouse.move(0, 0);

    await expect(row).not.toHaveAttribute('data-highlighted');

    await rowOf(page, 'macOS').hover();

    await expect(page.locator('[data-ring="arcs"] [data-tone="can-be-freed"]')).toHaveAttribute('data-dimmed', '');
    await expect(page.locator('[data-ring="arcs"] [data-tone="mac-os"]')).not.toHaveAttribute('data-dimmed');
  });

  test('focusing the i of a row highlights it, for the keyboard', async ({ page }) => {
    await scan(page);

    await rowOf(page, 'Can be freed').getByRole('button', { name: 'What is Can be freed?' }).focus();

    await expect(rowOf(page, 'Can be freed')).toHaveAttribute('data-highlighted', '');
    await expect(page.locator('[data-ring="arcs"] [data-tone="files-and-apps"]')).toHaveAttribute('data-dimmed', '');
  });

  test("the i of System space says what it is in the core's words", async ({ page }) => {
    await scan(page);
    const row = rowOf(page, 'System space');

    await row.getByRole('button', { name: 'What is System space?' }).click();

    await expect(row).toContainText('Memory macOS keeps on disk');
  });

  test("the scanning card reads its percent from the scan's bytes and estimate, and Stop ends the scan", async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.live);
    await page.goto('/');
    await scanButton(page).click();
    const card = page.getByRole('region', { name: 'Scanning' });

    // The live script has walked 2.25 GB of an estimated 3 GB.
    await expect(card.getByRole('progressbar', { name: 'Scanning, 75 percent' })).toBeVisible();
    await expect(card.getByText('75%')).toBeVisible();
    await expect(card.getByText('found so far')).toBeVisible();
    await expect(page.getByRole('checkbox')).toHaveCount(0);

    await page.getByRole('button', { name: 'Stop' }).click();

    await expect(card).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Scan again' })).toBeVisible();
  });
});
