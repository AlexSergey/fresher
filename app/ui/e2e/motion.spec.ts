import { expect, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The system's "Reduce motion" (Playwright's emulation, motion not forced off) in the real engines: a scan stands
// still and a screen change only fades; with full motion the scan's loops run and a new screen rises.

type Arrival = { readonly opacity: number; readonly rise: number };

type Loops = { readonly running: number; readonly sweep: number };

type Watched = { __arrival__?: Arrival };

// The endless animations of the page that run, and those of the scan's sweep (running or not).
const loops = (page: Page): Promise<Loops> =>
  page.evaluate(() => {
    const endless = document
      .getAnimations()
      .filter((animation) => animation.effect?.getComputedTiming().iterations === Infinity);
    const sweep = document.querySelector('[data-sweep]');

    return {
      running: endless.filter((animation) => animation.playState === 'running').length,
      sweep: sweep === null ? 0 : sweep.getAnimations({ subtree: true }).length,
    };
  });

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
  await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');
  await expect(page.locator('[data-sweep]')).toBeAttached();
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

  test('a scan under reduced motion has no rotating sweep and no breathing', async ({ page }) => {
    await scanning(page);

    const { running, sweep } = await loops(page);
    // The comet is drawn, standing still.
    expect(sweep).toBeGreaterThan(0);
    expect(running).toBe(0);
    await expect(page.locator('[data-breathing]')).toHaveCount(0);
  });

  test('a screen change under reduced motion only fades', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 0 });
  });
});

test.describe('positive cases', () => {
  test.use({ reducedMotion: 'no-preference' });

  test('a scan under full motion runs the sweep and breathes', async ({ page }) => {
    await scanning(page);

    const { running, sweep } = await loops(page);
    expect(sweep).toBeGreaterThan(0);
    expect(running).toBeGreaterThanOrEqual(2);
    await expect(page.locator('[data-breathing]')).toHaveCount(1);
  });

  test('a screen change under full motion fades and rises', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 4 });
  });
});
