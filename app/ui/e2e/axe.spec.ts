import AxeBuilder from '@axe-core/playwright';
import { expect, type Locator, type Page, test } from '@playwright/test';

import { type CoreScript, mockCore, SCRIPTS } from './core-mock';
import { requestQuit } from './support/cleanup-mock';
import { nextFrames } from './support/frames';
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

// Ticks both categories of the demo scan and opens the review.
const reviewAll = async (page: Page, script: CoreScript): Promise<void> => {
  await open(page, script);
  await scan(page, '2.6 GB');
  await page.getByRole('checkbox', { name: 'Select Example caches' }).check();
  await page.getByRole('checkbox', { name: 'Select Example downloads' }).check();
  await go(page, /^Review/, 'Review');
};

// The JetBrains scan, on the module's page; with `review`, WebStorm's three categories ticked and the review open.
const jetbrainsPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.jetbrains);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^JetBrains IDEs/ });
  await expect(nav).toContainText('1.7 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'JetBrains IDEs' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Caches of older JetBrains IDE versions' })).toContainText(
    '3 items left out (1.4 GB)',
  );
};

// The temporary files scan, on the module's page.
const tempPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Temporary files/ });
  await expect(nav).toContainText('837 MB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Temporary files' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Old temporary files' })).toContainText(
    '4,712 items left out (790 MB)',
  );
};

// The removed apps' leftovers scan, on the module's page: its two choices, and an open app's and a recent one's files
// left out.
const leftoversPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.appLeftovers);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Removed apps' leftovers/ });
  await expect(nav).toContainText('2.5 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: "Removed apps' leftovers" })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Caches of removed apps' })).toContainText('1 item left out (220 MB)');
};

// The developer caches scan with Xcode open, on the module's page: the projects left out, one Gradle folder not the
// user's.
const devCachesPage = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.devCachesXcodeOpen);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Developer caches/ });
  await expect(nav).toContainText('12.9 GB');
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Developer caches' })).toBeFocused();
  await expect(page.getByRole('region', { name: 'Xcode build data' })).toContainText('2 items left out (4.3 GB)');
};

// The downloads scan, on the module's page: five categories, a browser-open download and a recent big file left out.
const downloadsPage = async (
  page: Page,
  script: typeof SCRIPTS.downloads | typeof SCRIPTS.downloadsUnavailable = SCRIPTS.downloads,
  total = '15.8 GB',
): Promise<void> => {
  await mockCore(page, script);
  await page.goto('/');
  await page.getByRole('button', { exact: true, name: 'Scan' }).click();
  const nav = page.getByRole('button', { name: /^Downloads and installers/ });
  await expect(nav).toContainText(total);
  await nav.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Downloads and installers' })).toBeFocused();
};

// Settings, with the temporary files module's number field in view.
const tempSettings = async (page: Page): Promise<void> => {
  await mockCore(page, SCRIPTS.temp);
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeFocused();
  const days = page.getByRole('spinbutton', { name: 'Offer temporary files unchanged for' });
  await expect(days).toHaveValue('7');
  await days.scrollIntoViewIfNeeded();
};

// WebStorm's three categories ticked, the review open and the confirmation (it names Medium) asked for.
const jetbrainsConfirmation = async (page: Page): Promise<Locator> => {
  await jetbrainsPage(page);
  for (const name of ['Caches', 'Logs', 'Settings and plugins']) {
    await page.getByRole('checkbox', { name: new RegExp(`^Select ${name} of older`) }).check();
  }
  await page.getByRole('region', { name: 'Selection' }).getByRole('button', { name: 'Review' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Review' })).toBeFocused();
  await page.getByRole('button', { name: 'Move to Trash…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Move 3 items to the Trash?' });
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();

  return dialog;
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
      await expect(page.getByRole('group', { name: 'Parts of the scan' })).toContainText('scanning');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview after a scan', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview by place', async ({ page }) => {
      await open(page, SCRIPTS.demo);
      await scan(page, '2.6 GB');
      await page.getByRole('radio', { name: 'By place' }).click();
      await expect(page.getByRole('list', { name: 'Inside home' }).getByRole('listitem')).toHaveCount(6);

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

    test('the overview in a sandbox, with its banner', async ({ page }) => {
      await open(page, SCRIPTS.sandboxed);
      await expect(page.getByRole('note')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview of a development build, with its banner', async ({ page }) => {
      await open(page, SCRIPTS.development);
      await expect(page.getByRole('note')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the overview with Full Disk Access denied after a limited scan', async ({ page }) => {
      await open(page, SCRIPTS.denied);
      await scan(page, '2.6 GB');
      await expect(page.getByText('Fresher cannot see every folder')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page with a selection', async ({ page }) => {
      await selectSome(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of JetBrains IDEs, with its left-out lines', async ({ page }) => {
      await jetbrainsPage(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of Temporary files, with its left-out lines', async ({ page }) => {
      await tempPage(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of Developer caches, with its left-out lines', async ({ page }) => {
      await devCachesPage(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of Removed apps’ leftovers, with its choices and left-out lines', async ({ page }) => {
      await leftoversPage(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of Downloads and installers, with its left-out lines', async ({ page }) => {
      await downloadsPage(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the module page of Downloads and installers before Downloads is allowed', async ({ page }) => {
      await downloadsPage(page, SCRIPTS.downloadsUnavailable, '0 bytes');

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings with the number field of Temporary files', async ({ page }) => {
      await tempSettings(page);

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the confirmation of a cleanup that names Medium', async ({ page }) => {
      await jetbrainsConfirmation(page);

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

    test('the confirmation of a cleanup', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaningLeftOut);
      await page.getByRole('button', { name: 'Move to Trash…' }).click();
      await expect(page.getByRole('dialog').getByRole('button', { name: 'Cancel' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('a cleanup under way, with the quit question', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaningHeld);
      await page.getByRole('button', { name: 'Move to Trash…' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByText('Moving 2 of 3 to the Trash…')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);

      await requestQuit(page);
      await expect(page.getByRole('button', { name: 'Keep cleaning' })).toBeFocused();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the result of a cleanup', async ({ page }) => {
      await reviewAll(page, SCRIPTS.cleaning);
      await page.getByRole('button', { name: 'Move to Trash…' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Move to Trash' }).click();
      await expect(page.getByRole('region', { name: 'Moved to the Trash' })).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the cleanups and one cleanup', async ({ page }) => {
      await open(page, SCRIPTS.cleaning);
      await go(page, 'Cleanups', 'Cleanups');
      await expect(page.getByText('Waiting in the Trash from cleanups: 4.1 GB')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);

      await page.getByRole('button', { name: /^Cleanup of Dec 31, 2024/ }).click();
      await expect(page.getByText(/Items marked Unknown/)).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the settings of every kind', async ({ page }) => {
      await open(page, SCRIPTS.settings);
      await go(page, 'Settings', 'Settings');
      await expect(page.getByText('Version 0.0.1')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    test('the page of an ignored module', async ({ page }) => {
      await open(page, SCRIPTS.ignoring);
      await scan(page, '2.6 GB');
      await go(page, /^Demo module/, 'Demo module');
      await page.getByRole('button', { name: 'Ignore this module' }).click();
      await expect(page.getByText('You ignore this module.')).toBeVisible();

      expect(await violations(page)).toStrictEqual([]);
    });

    for (const tab of ['Ignored groups', 'Ignored modules', 'Ignored items']) {
      test(`the Ignored section of Settings, ${tab}`, async ({ page }) => {
        await open(page, SCRIPTS.ignoring);
        await go(page, 'Settings', 'Settings');
        await page.getByRole('switch', { name: 'Ignore Debug' }).click();
        await page.getByRole('tab', { name: new RegExp(`^${tab}`) }).click();
        await expect(page.getByRole('tab', { name: new RegExp(`^${tab}`) })).toHaveAttribute('aria-selected', 'true');

        expect(await violations(page)).toStrictEqual([]);
      });
    }

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
          await nextFrames(page);

          expect.soft(await violations(page, selector), id).toStrictEqual([]);
        });
      }
    });
  }
});
