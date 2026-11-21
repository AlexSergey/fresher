import { expect, type Locator, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { motionOff } from './support/motion';

// The system's "Increase contrast" (Playwright's `prefers-contrast: more`): no glow and no noise, stronger borders.

// The elements (and their ::before and ::after) that cast coloured light: a blurred or spread box shadow, or a radial
// gradient, in a colour that is not grey. Elevation shadows are black and an edge mark is neither blurred nor spread,
// so only a glow counts.
const colouredLight = (page: Page): Promise<readonly string[]> =>
  page.evaluate(() => {
    const coloured = (value: string): boolean =>
      [...value.matchAll(/rgba?\(([^)]+)\)/gu)].some(([, channels = '']) => {
        const [red, green, blue, alpha = 1] = channels
          .split(/[\s,/]+/u)
          .filter((part) => part !== '')
          .map(Number);

        return alpha > 0 && !(red === green && green === blue);
      });
    // Each shadow of a list: its colour, then x, y, blur and spread.
    const glows = (shadows: string): boolean =>
      shadows.split(/,(?![^(]*\))/u).some((shadow) => {
        const [, , blur = 0, spread = 0] = [...shadow.replace(/rgba?\([^)]*\)/u, '').matchAll(/-?[\d.]+px/gu)].map(
          ([length]) => Number.parseFloat(length),
        );

        return coloured(shadow) && (blur > 0 || spread > 0);
      });
    const found: string[] = [];

    for (const element of document.querySelectorAll('*')) {
      for (const pseudo of [null, '::before', '::after']) {
        const style = getComputedStyle(element, pseudo);
        const radial = style.backgroundImage.includes('radial-gradient') ? style.backgroundImage : '';

        if (style.display !== 'none' && (glows(style.boxShadow) || coloured(radial))) {
          found.push(`${element.tagName.toLowerCase()}${pseudo ?? ''}.${element.className}`);
        }
      }
    }

    return found;
  });

const noiseShown = (page: Page): Promise<boolean> =>
  page.evaluate(() => getComputedStyle(document.documentElement, '::after').display !== 'none');

// The alpha of a colour as computed (`rgba(…, a)`; an `rgb(…)` is opaque).
const alphaOf = (colour: string): number => {
  const channels = /rgba?\(([^)]+)\)/u.exec(colour)?.[1]?.split(/[\s,/]+/u) ?? [];

  return channels.length === 4 ? Number(channels[3]) : 1;
};

const firstCard = (page: Page): Locator => page.locator('[data-card]').first();

// The overview after the demo scan, one card selected, nothing focused.
const scanned = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.demo);
  await page.goto('/');
  await scanButton(page).click();
  await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.evaluate(() => {
    (document.activeElement as HTMLElement | null)?.blur();
  });
  await page.mouse.move(0, 0);
};

// The border of a card that is not selected (a selected card's border is the accent).
const cardBorder = (page: Page): Promise<string> =>
  page
    .locator('[data-card]')
    .nth(1)
    .locator(':scope > *')
    .first()
    .evaluate((surface) => getComputedStyle(surface).borderTopColor);

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.describe('negative cases', () => {
  test.use({ contrast: 'more' });

  test('under increased contrast nothing glows and there is no noise', async ({ page }) => {
    await scanned(page);

    expect(await colouredLight(page)).toStrictEqual([]);
    expect(await noiseShown(page)).toBe(false);
    await expectNoCspViolation(page);
  });
});

test.describe('positive cases', () => {
  test.describe('without increased contrast', () => {
    test.use({ contrast: 'no-preference' });

    test('the glows and the noise are drawn', async ({ page }) => {
      await scanned(page);

      expect(await colouredLight(page)).not.toHaveLength(0);
      expect(await noiseShown(page)).toBe(true);
      await expectNoCspViolation(page);
    });
  });

  test.describe('with increased contrast', () => {
    test.use({ contrast: 'more' });

    test('the borders are stronger', async ({ browser }) => {
      const borders: number[] = [];

      for (const contrast of ['no-preference', 'more'] as const) {
        const page = await browser.newPage({ contrast });
        await motionOff(page);
        await scanned(page);
        borders.push(alphaOf(await cardBorder(page)));
        await page.close();
      }

      expect(borders[1]).toBeGreaterThan(borders[0] ?? 1);
    });

    test('the Scan screen and a card', async ({ page }) => {
      await scanned(page);
      // Checked before the pictures: a screenshot adds a style of its own.
      await expectNoCspViolation(page);

      await expect(page).toHaveScreenshot('contrast-scan.png');
      // Scrolled to the top of the window, clear of the selection bar that floats at its bottom.
      await firstCard(page).evaluate((card) => {
        card.scrollIntoView({ block: 'start' });
      });
      await expect(firstCard(page)).toHaveScreenshot('contrast-card.png');
    });
  });
});
