import { expect, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// Page zoom at 125 % and 150 % in the default window (1100 × 720): what the system's zoom does, emulated as a window
// that many times smaller in CSS pixels, drawn at that many device pixels each. Sizes are in rem, so the UI scales.

const WINDOW = { height: 720, width: 1100 };

// The elements whose text is cut: it overflows a box that hides the overflow without an ellipsis to say so.
const cutText = (page: Page): Promise<readonly string[]> =>
  page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('body *')]
      .filter((element) => {
        const style = getComputedStyle(element);
        const hasText = [...element.childNodes].some(
          (node) => node.nodeType === Node.TEXT_NODE && (node.textContent ?? '').trim() !== '',
        );
        const hides = style.overflowX !== 'visible' || style.overflowY !== 'visible';
        const overflows =
          element.scrollWidth > element.clientWidth + 1 || element.scrollHeight > element.clientHeight + 1;
        // A visually hidden text (1 px) is cut on purpose: assistive technology reads it.
        const shown = element.clientWidth > 1;

        return hasText && hides && shown && overflows && style.textOverflow !== 'ellipsis';
      })
      .map((element) => `${element.tagName.toLowerCase()}: ${element.textContent.slice(0, 40)}`),
  );

// The numbers (sizes, counts) broken over two lines: "750" on one, "MB" on the next.
const brokenNumbers = (page: Page): Promise<readonly string[]> =>
  page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[data-numeric]')]
      .filter((element) => {
        const lineHeight = Number.parseFloat(getComputedStyle(element).lineHeight);

        return element.getBoundingClientRect().height > lineHeight * 1.5;
      })
      .map((element) => element.textContent),
  );

// Whether the page scrolls sideways: the window is wider than nothing should need.
const scrollsSideways = (page: Page): Promise<boolean> =>
  page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);

const moduleNav = (page: Page): ReturnType<Page['getByRole']> => page.getByRole('button', { name: /^Demo module/ });

const scanned = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.demo);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(moduleNav(page)).toContainText('2.6 GB');
  await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

for (const zoom of [125, 150]) {
  test.describe(`at ${String(zoom)} % page zoom`, () => {
    const scale = zoom / 100;
    test.use({
      deviceScaleFactor: scale,
      viewport: { height: Math.round(WINDOW.height / scale), width: Math.round(WINDOW.width / scale) },
    });

    // Only the reference engine is photographed; both engines are checked. The policy is checked first: a screenshot
    // adds a style of its own.
    const photograph = async (page: Page, browserName: string, name: string): Promise<void> => {
      await expectNoCspViolation(page);
      if (browserName === 'webkit') {
        await expect(page).toHaveScreenshot(`${name}-${String(zoom)}.png`);
      }
    };

    test.describe('negative cases', () => {
      test('the overview cuts no text, breaks no number and does not scroll sideways', async ({
        browserName,
        page,
      }) => {
        await scanned(page);

        expect(await cutText(page)).toStrictEqual([]);
        expect(await brokenNumbers(page)).toStrictEqual([]);
        expect(await scrollsSideways(page)).toBe(false);
        await photograph(page, browserName, 'overview');
      });

      test('the module page cuts no text, breaks no number and does not scroll sideways', async ({
        browserName,
        page,
      }) => {
        await scanned(page);
        await page.getByRole('button', { name: 'Open Example caches' }).click();
        const caches = page.getByRole('region', { name: 'Example caches' });
        await caches.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
        await expect(page.getByRole('region', { name: 'Selection' })).toContainText('1 item selected');

        expect(await cutText(page)).toStrictEqual([]);
        expect(await brokenNumbers(page)).toStrictEqual([]);
        expect(await scrollsSideways(page)).toBe(false);
        await photograph(page, browserName, 'module-page');
      });

      test('the cleanups cut no text, break no number and do not scroll sideways', async ({ browserName, page }) => {
        await mockCore(page, SCRIPTS.cleaning);
        await page.goto('/');
        await page.getByRole('button', { name: 'Cleanups' }).click();
        await expect(page.getByText('Waiting in the Trash from cleanups: 4.1 GB')).toBeVisible();

        expect(await cutText(page)).toStrictEqual([]);
        expect(await brokenNumbers(page)).toStrictEqual([]);
        expect(await scrollsSideways(page)).toBe(false);
        await photograph(page, browserName, 'cleanups');
      });

      test('the confirmation with an item left out cuts no text', async ({ browserName, page }) => {
        await page.goto('/gallery.html#/ConfirmDialog/left-out');
        const dialog = page.getByRole('dialog');
        await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

        expect(await cutText(page)).toStrictEqual([]);
        expect(await dialog.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true);
        await photograph(page, browserName, 'confirmation-left-out');
      });

      test('the confirmation cuts no text and does not scroll sideways', async ({ browserName, page }) => {
        await page.goto('/gallery.html#/ConfirmDialog/trash');
        const dialog = page.getByRole('dialog');
        await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

        expect(await cutText(page)).toStrictEqual([]);
        expect(await dialog.evaluate((element) => element.scrollWidth <= element.clientWidth)).toBe(true);
        await photograph(page, browserName, 'confirmation');
      });
    });
  });
}
