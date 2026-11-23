import { expect, type Locator, type Page, test } from '@playwright/test';

import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { expectNoCspViolation, failOnCspViolation } from './support/csp';
import { nextFrames } from './support/frames';
import { motionOff } from './support/motion';

/* eslint-disable playwright/no-conditional-in-test, playwright/no-conditional-expect -- the sweep walks whatever focus stops each story holds */

// The keyboard alone over every story of the gallery and the screens: each focus stop is reached by Tab and shows a
// visible focus ring, a control answers its key, a dialog keeps the focus and gives it back, and a screen change never
// drops the focus on the page.

type Stop = { readonly index: number; readonly role: string };

// The mark of a story's focus stops, numbered in document order.
const STOP = 'data-keyboard-stop';

// Roles whose keyboard activation is a click: Enter for a button or a link, Space for the others.
const ACTIVATED_BY: Readonly<Record<string, 'Enter' | 'Space'>> = {
  button: 'Enter',
  checkbox: 'Space',
  link: 'Enter',
  radio: 'Space',
  switch: 'Space',
};

const openStory = async (page: Page, id: string): Promise<Locator> => {
  const [component = '', state = ''] = id.split('/');
  await page.goto('about:blank');
  await page.goto(`/gallery.html#/${encodeURIComponent(component)}/${encodeURIComponent(state)}`);
  const story = page.locator(`[data-story="${id}"]`);
  await expect(story).toBeVisible();
  await nextFrames(page);

  return story;
};

// Marks every element the Tab key can reach in the story (inside its open modal dialog, when it has one: the rest of
// the page is inert), and returns them in document order.
const markStops = (page: Page, id: string): Promise<readonly Stop[]> =>
  page.evaluate(
    ({ id: storyId, mark }) => {
      const story = document.querySelector(`[data-story="${storyId}"]`);
      const scope = story?.querySelector('dialog[open]') ?? story;
      const candidates = scope?.querySelectorAll<HTMLElement>(
        'a[href], button, input, select, textarea, summary, [tabindex]',
      );

      return [...(candidates ?? [])]
        .filter(
          (element) =>
            element.tabIndex >= 0 &&
            !element.matches(':disabled') &&
            element.closest('[inert]') === null &&
            element.checkVisibility({ visibilityProperty: true }),
        )
        .map((element, index) => {
          element.setAttribute(mark, String(index));
          const role = element.getAttribute('role') ?? '';
          const native =
            element instanceof HTMLButtonElement || element.tagName === 'SUMMARY'
              ? 'button'
              : element instanceof HTMLAnchorElement
                ? 'link'
                : element instanceof HTMLInputElement
                  ? element.type
                  : element.tagName.toLowerCase();

          return { index, role: role === '' ? native : role };
        });
    },
    { id, mark: STOP },
  );

// The focus stop that has the focus, `null` when the focus is not on one of the story's stops.
const focusedStop = (page: Page): Promise<null | number> =>
  page.evaluate((mark) => {
    const index = document.activeElement?.getAttribute(mark);

    return index === undefined || index === null ? null : Number(index);
  }, STOP);

// The focused stop and a margin around it, for its picture: the ring is drawn on or around it.
const aroundFocus = async (stop: Locator): Promise<{ height: number; width: number; x: number; y: number }> => {
  const box = await stop.evaluate((element) => {
    // A visually hidden input draws its look on its label.
    const shown = element.getBoundingClientRect().width < 8 ? (element.parentElement ?? element) : element;
    const { height, width, x, y } = shown.getBoundingClientRect();

    return { height, width, x, y };
  });
  const margin = 12;

  return {
    height: box.height + margin * 2,
    width: box.width + margin * 2,
    x: Math.max(box.x - margin, 0),
    y: Math.max(box.y - margin, 0),
  };
};

// Whether the focus shows: the stop and its surroundings look different focused than blurred. A story that forces
// its focused look (`data-focus`) shows it either way.
const ringShows = async (page: Page, stop: Locator): Promise<boolean> => {
  if ((await stop.evaluate((element) => element.closest('[data-focus]'))) !== null) {
    return true;
  }
  const clip = await aroundFocus(stop);
  const focused = await page.screenshot({ clip });
  await stop.evaluate((element) => {
    element.blur();
  });
  const blurred = await page.screenshot({ clip });
  await stop.evaluate((element) => {
    element.focus();
  });

  return !focused.equals(blurred);
};

// Presses the stop's key and counts the clicks it gets; the click does nothing else, so the story stays as it is.
const answersItsKey = async (page: Page, stop: Locator, key: 'Enter' | 'Space'): Promise<boolean> => {
  await stop.evaluate((element) => {
    const swallow = (event: Event): void => {
      if (event.target instanceof Node && element.contains(event.target)) {
        element.dataset.keyboardClicks = String(Number(element.dataset.keyboardClicks ?? '0') + 1);
        event.preventDefault();
        event.stopPropagation();
      }
    };

    window.addEventListener('click', swallow, { capture: true, once: true });
  });
  await page.keyboard.press(key);

  return (await stop.getAttribute('data-keyboard-clicks')) === '1';
};

const heading = (page: Page, name: string): Locator => page.getByRole('heading', { level: 1, name });

// Whether the focus is on an element of the page, not dropped on its body.
const focusKept = (page: Page): Promise<boolean> =>
  page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body);

test.beforeEach(async ({ page }) => {
  await motionOff(page);
  await failOnCspViolation(page);
});

test.afterEach(async ({ page }) => {
  await expectNoCspViolation(page);
});

test.describe('negative cases', () => {
  test('the focus never leaves an open dialog for the page behind', async ({ page }) => {
    await openStory(page, 'Dialog/from-a-button');
    await page.getByRole('button', { name: 'Review…' }).focus();
    await page.keyboard.press('Enter');
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog.getByRole('button', { name: 'Close' })).toBeFocused();

    for (let press = 0; press < 8; press += 1) {
      await page.keyboard.press('Tab');
      expect(
        await page.evaluate(() => {
          const active = document.activeElement;

          return active === null || active === document.body || active.closest('dialog') !== null;
        }),
      ).toBe(true);
    }
  });

  test('a screen change never drops the focus', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaning);
    await page.goto('/');
    await expect(heading(page, 'Scan')).toBeVisible();

    const scan = scanButton(page);
    await scan.focus();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('heading', { level: 3, name: 'Example caches' })).toBeVisible();
    expect(await focusKept(page)).toBe(true);

    await page.getByRole('button', { name: 'Details' }).first().focus();
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Example caches')).toBeFocused();
    await page.getByRole('button', { name: 'Back to Scan' }).focus();
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Scan')).toBeFocused();

    for (const [nav, name] of [
      ['Largest', 'Largest files and folders'],
      [/^Review/, 'Review'],
      ['Cleanups', 'Cleanups'],
      ['Settings', 'Settings'],
      ['Scan', 'Scan'],
    ] as const) {
      await page.getByRole('navigation').getByRole('button', { name: nav }).focus();
      await page.keyboard.press('Enter');
      await expect(heading(page, name)).toBeFocused();
    }
  });

  test('stopping a scan by keyboard does not drop the focus', async ({ page }) => {
    await mockCore(page, SCRIPTS.live);
    await page.goto('/');
    await scanButton(page).focus();
    await page.keyboard.press('Enter');
    const stop = page.getByRole('button', { name: 'Stop' });
    await expect(stop).toBeVisible();
    expect(await focusKept(page)).toBe(true);

    await stop.focus();
    await page.keyboard.press('Enter');

    await expect(page.getByText('This scan is incomplete')).toBeVisible();
    expect(await focusKept(page)).toBe(true);
  });
});

test.describe('positive cases', () => {
  test('a dialog opened by keyboard gives the focus back to its opener', async ({ page }) => {
    await openStory(page, 'Dialog/from-a-button');
    const opener = page.getByRole('button', { name: 'Review…' });
    await opener.focus();

    await page.keyboard.press('Enter');
    const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
    await expect(dialog.getByRole('button', { name: 'Close' })).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
    await page.keyboard.press('Escape');

    await expect(dialog).toHaveCount(0);
    await expect(opener).toBeFocused();
  });

  test('the confirmation opened by keyboard starts on Cancel and gives the focus back', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaning);
    await page.goto('/');
    await scanButton(page).click();
    await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
    await page
      .getByRole('navigation')
      .getByRole('button', { name: /^Review/ })
      .click();
    const remove = page.getByRole('button', { name: 'Move to Trash…' });
    await remove.focus();

    await page.keyboard.press('Enter');
    const dialog = page.getByRole('dialog', { name: 'Move 2 items to the Trash?' });
    await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
    await page.keyboard.press('Enter');

    await expect(dialog).toHaveCount(0);
    await expect(remove).toBeFocused();
  });

  test('a hint shows on keyboard focus: an icon button, a level badge and a number', async ({ page }) => {
    await mockCore(page, SCRIPTS.cleaning);
    await page.goto('/');
    await scanButton(page).click();
    await page.getByRole('checkbox', { name: 'Select Example caches' }).check();

    await page.getByRole('button', { name: 'Options for Example caches' }).first().focus();
    await expect(page.getByRole('tooltip')).toHaveText('Options for Example caches');

    await page.getByText('Safe', { exact: true }).first().locator('xpath=ancestor::*[@tabindex="0"][1]').focus();
    await expect(page.getByRole('tooltip')).toContainText('Comes back by itself');

    await page.getByRole('region', { name: 'Selection' }).locator('[data-numeric]').focus();
    await expect(page.getByRole('tooltip')).toHaveText('The space on the disk that everything you selected takes.');

    await page
      .getByText('Found', { exact: true })
      .locator('xpath=following-sibling::*[1]//*[@tabindex="0"]|following-sibling::*[@tabindex="0"]')
      .first()
      .focus();
    await expect(page.getByRole('tooltip')).toContainText('The space the scan found in all categories.');
  });

  test('a hint shows on keyboard focus: a filter of Largest', async ({ page }) => {
    await mockCore(page, SCRIPTS.largest);
    await page.goto('/');
    await scanButton(page).click();
    await page.getByRole('navigation').getByRole('button', { name: 'Largest' }).click();
    await page.getByRole('combobox', { name: 'Kind' }).focus();
    await expect(page.getByRole('tooltip')).toHaveText('Lists only items of one kind, such as videos or archives.');
  });

  test('a hint shows on keyboard focus: the filter of Apps', async ({ page }) => {
    await mockCore(page, SCRIPTS.apps);
    await page.goto('/');
    await scanButton(page).click();
    await page.getByRole('navigation').getByRole('button', { name: 'Apps' }).click();
    await page.getByRole('radio', { checked: true }).focus();
    await expect(page.getByRole('tooltip')).toHaveText('Lists only apps you have not opened for at least this long.');
  });

  test('the Ignored tabs are one tab stop, the arrow keys move between them and Space turns a switch', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.ignoring);
    await page.goto('/');
    await page.getByRole('navigation').getByRole('button', { name: 'Settings' }).focus();
    await page.keyboard.press('Enter');
    await expect(heading(page, 'Settings')).toBeFocused();
    const groups = page.getByRole('tab', { name: /^Ignored groups/ });
    const modules = page.getByRole('tab', { name: /^Ignored modules/ });
    await groups.focus();

    await page.keyboard.press('ArrowRight');
    await expect(modules).toBeFocused();
    await expect(modules).toHaveAttribute('aria-selected', 'true');
    await page.keyboard.press('ArrowLeft');
    await expect(groups).toBeFocused();
    await expect(groups).toHaveAttribute('aria-selected', 'true');
    await expect(modules).toHaveAttribute('tabindex', '-1');

    const ignoreGroup = page.getByRole('switch', { name: 'Ignore Debug' });
    await ignoreGroup.focus();
    await page.keyboard.press('Space');
    await expect(ignoreGroup).toBeChecked();
    await expect(page.getByRole('tab', { name: 'Ignored groups · 1' })).toBeVisible();
    await page.keyboard.press('Space');
    await expect(ignoreGroup).not.toBeChecked();
  });

  // The stories are shared among this many tests, which run in parallel.
  const PARTS = 8;

  for (let part = 0; part < PARTS; part += 1) {
    test(`every story, part ${String(part + 1)}: Tab reaches each focus stop, its ring shows and a control answers its key`, async ({
      browserName,
      page,
    }) => {
      test.setTimeout(240_000);
      await page.goto('/gallery.html');
      const all = await page.evaluate(() => (window as { __stories__?: readonly string[] }).__stories__ ?? []);
      // Each part takes whole components, so each component's focused look is photographed once: at the first focus
      // stop of its stories.
      const components = [...new Set(all.map((id) => id.split('/')[0] ?? ''))];
      const mine = new Set(components.filter((_, index) => index % PARTS === part));
      const photographed = new Set<string>();

      expect(all.length).toBeGreaterThan(0);

      for (const id of all.filter((story) => mine.has(story.split('/')[0] ?? ''))) {
        await test.step(id, async () => {
          await openStory(page, id);
          // Checked before the pictures: a screenshot adds a style of its own.
          await expectNoCspViolation(page);
          const stops = await markStops(page, id);
          const reached: number[] = [];
          const noRing: number[] = [];
          const noAnswer: number[] = [];
          // Stops gone from the page after their visit: the focus could not come back to them.
          const lost: number[] = [];
          const [component = ''] = id.split('/');
          const refocus = async (index: number): Promise<void> => {
            const back = await page.evaluate(
              ({ at, mark }) => {
                const element = document.querySelector<HTMLElement>(`[${mark}="${at}"]`);
                element?.focus();

                return element !== null;
              },
              { at: String(index), mark: STOP },
            );
            if (!back) {
              lost.push(index);
            }
          };

          const visit = async (index: number): Promise<void> => {
            reached.push(index);
            const stop = page.locator(`[${STOP}="${String(index)}"]`);

            if (!(await ringShows(page, stop))) {
              noRing.push(index);
            }
            if (browserName === 'webkit' && !photographed.has(component)) {
              photographed.add(component);
              await expect.soft(page).toHaveScreenshot([component, 'focus.png'], { clip: await aroundFocus(stop) });
            }
            const key = ACTIVATED_BY[stops[index]?.role ?? ''];
            if (key !== undefined && !(await answersItsKey(page, stop, key))) {
              noAnswer.push(index);
            }
            await refocus(index);
          };

          // Forwards from where the story puts the focus (a dialog focuses a control when it opens), then backwards
          // from the first stop reached: a modal dialog does not loop the focus.
          const initial = await focusedStop(page);
          if (initial !== null) {
            await visit(initial);
          }
          for (const key of ['Tab', 'Shift+Tab']) {
            const first = reached[0];
            if (key === 'Shift+Tab' && first !== undefined) {
              await refocus(first);
            }
            for (let press = 0; press <= stops.length; press += 1) {
              await page.keyboard.press(key);
              const current = await focusedStop(page);
              if (current === null || reached.includes(current)) {
                break;
              }
              await visit(current);
            }
          }

          const unreached = stops
            .filter(({ index }) => !reached.includes(index))
            .map(({ index, role }) => `${String(index)}: ${role}`);
          expect
            .soft({ lost, noAnswer, noRing, unreached }, id)
            .toStrictEqual({ lost: [], noAnswer: [], noRing: [], unreached: [] });
        });
      }
      // A new document: the last story's pictures added styles the policy check after each test would count.
      await page.goto('about:blank');
    });
  }
});
