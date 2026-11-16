import AxeBuilder from '@axe-core/playwright';
import { expect, type Locator, type Page, test } from '@playwright/test';

import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { motionOff } from './support/motion';

// The automated checker over every screen in each of its states and every story of the gallery: no violation of
// WCAG 2.1 A and AA. Motion is off, so nothing is checked mid-animation.

const WCAG_21_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

type Violation = { readonly id: string; readonly nodes: readonly string[] };

// Whether the element is the text of an inactive control: a greyed-out row whose own control is disabled. WCAG 1.4.3
// sets no contrast for it; the checker cannot tell, since a row has no disabled state of its own in ARIA.
const inactive = (page: Page, selector: string): Promise<boolean> =>
  page.evaluate(
    (target) => document.querySelector(target)?.closest('[data-disabled]')?.querySelector(':disabled') != null,
    selector,
  );

// The violations of the page (or of `include`), each with the selectors of the elements it concerns.
const violations = async (page: Page, include?: string): Promise<readonly Violation[]> => {
  const builder = new AxeBuilder({ page }).withTags(WCAG_21_AA);
  const results = await (include === undefined ? builder : builder.include(include)).analyze();
  const found: Violation[] = [];

  for (const { id, nodes } of results.violations) {
    const targets: string[] = [];

    for (const { target } of nodes) {
      const selector = target.join(' ');

      if (id !== 'color-contrast' || !(await inactive(page, selector))) {
        targets.push(selector);
      }
    }
    if (targets.length > 0) {
      found.push({ id, nodes: targets });
    }
  }

  return found;
};

const moduleNav = (page: Page): Locator => page.getByRole('button', { name: /^Demo module/ });

const open = async (page: Page, script: CoreScript): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await expect(moduleNav(page)).toBeVisible();
};

const scan = async (page: Page, found: string): Promise<void> => {
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  await expect(moduleNav(page)).toContainText(found);
};

const go = async (page: Page, nav: RegExp | string, heading: string): Promise<void> => {
  await page.getByRole('button', { name: nav }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: heading })).toBeFocused();
};

// The demo scan with one finding and one category ticked, on the module page.
const selectSome = async (page: Page): Promise<void> => {
  await open(page, SCRIPTS.demo);
  await scan(page, '2.6 GB');
  await page.getByRole('button', { name: 'Open Example caches' }).click();
  const caches = page.getByRole('region', { name: 'Example caches' });
  await caches.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await expect(page.getByRole('region', { name: 'Selection' })).toContainText('2 items selected');
};

test.beforeEach(async ({ page }) => {
  await motionOff(page);
});

test.describe('positive cases', () => {
  test.describe('screens', () => {
    test('the overview before a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview during a scan', async ({ page }) => {
      await open(page, SCRIPTS.live);
      await scan(page, '2.3 GB');
      await expect(page.getByRole('list', { name: 'Parts of the scan' })).toContainText('scanning');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview after a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview after a partial scan', async ({ page }) => {
      await open(page, SCRIPTS.partial);
      await scan(page, '2.6 GB');
      await expect(page.getByText('This scan is incomplete')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview after a failed scan', async ({ page }) => {
      await open(page, SCRIPTS.failed);
      await scan(page, '2.3 GB');
      await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview with Full Disk Access denied', async ({ page }) => {
      await open(page, SCRIPTS.denied);
      await expect(page.getByText('Fresher cannot see every folder')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page with a selection', async ({ page }) => {
      await selectSome(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the review of a selection', async ({ page }) => {
      await selectSome(page);
      await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the review with nothing selected', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await go(page, /^Review/, 'Review');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings of every kind', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByText('Version 0.0.1')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings a newer version saved', async ({ page }) => {
      await open(page, SCRIPTS.readOnly);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByRole('switch', { name: 'Demo module' })).toBeDisabled();

      expect(await violations(page)).toStrictEqual([]);
    });
  });

  // The stories are shared among this many tests, which run in parallel.
  const PARTS = 4;

  for (let part = 0; part < PARTS; part += 1) {
    test(`every story of the gallery, part ${String(part + 1)}`, async ({ page }) => {
      test.setTimeout(240_000);
      await page.goto('/gallery.html');
      const ids = await page.evaluate(() => (window as { __stories__?: readonly string[] }).__stories__ ?? []);

      expect(ids.length).toBeGreaterThan(0);

      for (const id of ids.filter((_, index) => index % PARTS === part)) {
        await test.step(id, async () => {
          const [component = '', state = ''] = id.split('/');
          await page.goto('about:blank');
          await page.goto(`/gallery.html#/${encodeURIComponent(component)}/${encodeURIComponent(state)}`);
          const selector = `[data-story="${id}"]`;
          await expect(page.locator(selector)).toBeVisible();

          expect.soft(await violations(page, selector), id).toStrictEqual([]);
        });
      }
    });
  }
});
