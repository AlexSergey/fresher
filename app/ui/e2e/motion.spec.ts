import { expect, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The system's "Reduce motion" (Playwright's emulation, motion not forced off) in the real engines: the scanning
// card's fill jumps and a screen change only fades; with full motion the fill glides and a new screen rises.

type Arrival = { readonly opacity: number; readonly rise: number };

type Watched = { __arrival__?: Arrival };

// How long the scanning card's fill takes to move to a new value: 0 when it jumps.
const fillGlide = (page: Page): Promise<number> =>
  page
    .locator('[role="progressbar"] path[stroke-dasharray]')
    .last()
    .evaluate((fill) => Number.parseFloat(getComputedStyle(fill).transitionDuration));

// Catches the next screen's arrival at its first frame: the animation is paused at 0 and the screen's opacity and
// rise (its vertical offset, in pixels) read there.
const watchArrival = (page: Page): Promise<void> =>
  page.evaluate(() => {
    const onStart = (event: AnimationEvent): void => {
      const screen = event.target;

      if (!event.animationName.includes('screen-in') || !(screen instanceof HTMLElement)) {
        return;
      }
      document.removeEventListener('animationstart', onStart);
      for (const animation of screen.getAnimations()) {
        animation.pause();
        animation.currentTime = 0;
      }
      const style = getComputedStyle(screen);

      (window as Watched).__arrival__ = {
        opacity: Number(style.opacity),
        rise: style.transform === 'none' ? 0 : new DOMMatrixReadOnly(style.transform).f,
      };
    };

    document.addEventListener('animationstart', onStart);
  });

const arrival = async (page: Page): Promise<Arrival> => {
  await page.waitForFunction(() => (window as Watched).__arrival__ !== undefined);

  return page.evaluate(() => (window as Watched).__arrival__ ?? { opacity: 1, rise: 0 });
};

// Opens the app on the held demo scan and starts it: the scan stays running.
const scanning = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.live);
  await page.goto('/');
  await scanButton(page).click();
  await expect(page.getByRole('progressbar', { name: /^Scanning, \d+ percent$/u })).toBeVisible();
};

const changeScreen = async (page: Page): Promise<Arrival> => {
  await watchArrival(page);
  await page.getByRole('button', { name: 'Settings' }).click();

  return arrival(page);
};

test.beforeEach(async ({ page }) => {
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test.use({ reducedMotion: 'reduce' });

  test("a scan under reduced motion jumps the card's fill to its value", async ({ page }) => {
    await scanning(page);

    expect(await fillGlide(page)).toBe(0);
  });

  test('a screen change under reduced motion only fades', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 0 });
  });
});

test.describe('positive cases', () => {
  test.use({ reducedMotion: 'no-preference' });

  test("a scan under full motion glides the card's fill to its value", async ({ page }) => {
    await scanning(page);

    expect(await fillGlide(page)).toBeGreaterThan(0);
  });

  test('a screen change under full motion fades and rises', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 4 });
  });
});
