import { expect, type Locator, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The scan screen's hero: the ring and its legend after a scan of the demo's home volume.

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
  await expect(page.getByRole('list', { name: 'Parts of the disk' })).toBeVisible();
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

  test('before a scan the hero has no legend', async ({ page }) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');

    await expect(page.getByText('Space on Macintosh HD, internal disk')).toBeVisible();
    await expect(legend(page)).toHaveCount(0);
  });
});

test.describe('positive cases', () => {
  test('hovering a ring segment highlights its legend row, and the reverse', async ({ page }) => {
    await scan(page);
    const row = rowOf(page, 'Your folders');

    await expect(row).not.toHaveAttribute('data-highlighted');

    await pointAtRing(page, 60);

    await expect(row).toHaveAttribute('data-highlighted', '');

    await page.mouse.move(0, 0);

    await expect(row).not.toHaveAttribute('data-highlighted');

    await rowOf(page, 'macOS').hover();

    await expect(page.locator('[data-ring="arcs"] [data-tone="safe"]')).toHaveAttribute('data-dimmed', '');
    await expect(page.locator('[data-ring="arcs"] [data-tone="mac-os"]')).not.toHaveAttribute('data-dimmed');
  });

  test('focusing the i of a row highlights it, for the keyboard', async ({ page }) => {
    await scan(page);

    await rowOf(page, 'Low risk').getByRole('button', { name: 'What is Low risk?' }).focus();

    await expect(rowOf(page, 'Low risk')).toHaveAttribute('data-highlighted', '');
    await expect(page.locator('[data-ring="arcs"] [data-tone="safe"]')).toHaveAttribute('data-dimmed', '');
  });

  test("the i of the remainder lists the core's reasons", async ({ page }) => {
    await scan(page);
    const row = rowOf(page, 'Outside the scanned folders');

    await row.getByRole('button', { name: 'What is Outside the scanned folders?' }).click();

    await expect(row.getByRole('list')).toContainText('Folders Fresher could not read.');
    await expect(row.getByRole('list')).toContainText("Local snapshots and the file system's own records");
  });
});
