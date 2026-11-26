import { expect, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';

// The system's "Reduce motion" (Playwright's emulation, motion not forced off) in the real engines: the scanning
// card's fill jumps, a screen change only fades and the "Looking at" line stands still; with full motion the fill
// glides, a new screen rises and the line's colour keeps brightening and settling.

type Arrival = { readonly opacity: number; readonly rise: number };

type Watched = { __arrival__?: Arrival; __fill__?: number[] };

// Records the scanning card's fill on every frame from the page's start: each new offset of its dash.
const watchFill = async (page: Page): Promise<void> => {
  await page.addInitScript(() => {
    const offsets: number[] = [];
    const sample = (): void => {
      const fill = [...document.querySelectorAll('[role="progressbar"] path[stroke-dasharray]')].at(-1);
      const offset = fill?.getAttribute('stroke-dashoffset');

      if (offset !== null && offset !== undefined && Number(offset) !== offsets.at(-1)) {
        offsets.push(Number(offset));
      }
      requestAnimationFrame(sample);
    };

    (window as Watched).__fill__ = offsets;
    requestAnimationFrame(sample);
  });
};

// How many offsets the fill showed between its first one and the one of its value, once it rests there: 0 when it
// jumps, more when it glides.
const fillSteps = async (page: Page): Promise<number> => {
  const bar = page.getByRole('progressbar');
  const resting = (100 - Number(await bar.getAttribute('aria-valuenow'))) * 10;

  await page.waitForFunction((offset) => (window as Watched).__fill__?.at(-1) === offset, resting);

  return page.evaluate(() => Math.max(((window as Watched).__fill__ ?? []).length - 2, 0));
};

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
  await watchFill(page);
  await mockCore(page, SCRIPTS.live);
  await page.goto('/');
  await scanButton(page).click();
  await expect(page.getByRole('progressbar', { name: /^Scanning, \d+ percent$/u })).toBeVisible();
};

// The names of the animations on the scanning card's "Looking at" line.
const lookingAt = (page: Page): Promise<string> =>
  page.getByText(/^Looking at /u).evaluate((line) => getComputedStyle(line).animationName);

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

    expect(await fillSteps(page)).toBe(0);
  });

  test('a screen change under reduced motion only fades', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 0 });
  });

  test('the Looking at line under reduced motion does not shimmer', async ({ page }) => {
    await scanning(page);

    expect(await lookingAt(page)).not.toContain('looking-shimmer');
  });
});

test.describe('positive cases', () => {
  test.use({ reducedMotion: 'no-preference' });

  test("a scan under full motion glides the card's fill to its value", async ({ page }) => {
    await scanning(page);

    expect(await fillSteps(page)).toBeGreaterThan(0);
  });

  test('a screen change under full motion fades and rises', async ({ page }) => {
    await scanning(page);

    expect(await changeScreen(page)).toStrictEqual({ opacity: 0, rise: 4 });
  });

  test('the Looking at line under full motion shimmers', async ({ page }) => {
    await scanning(page);

    expect(await lookingAt(page)).toContain('looking-shimmer');
  });
});
