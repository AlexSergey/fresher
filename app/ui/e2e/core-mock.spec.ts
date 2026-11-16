import { expect, type Page, test } from '@playwright/test';

import type {
  CleanupPlanView,
  CleanupsPage,
  FindingsPage,
  FindingsQuery,
  Overview,
} from '../src/ipc/bindings.generated';

import { COMMANDS } from '../src/ipc';
import { GRANTED, IDLE } from '../src/test/demo-scan';
import { largeScan } from '../src/test/large-scan';
import { pageOf } from '../src/test/page-of';
import { mockCore, SCRIPTS } from './core-mock';

// The mocked core pages through a scan's findings as the engine does, so the e2e tests of the lists page for real.

const SCAN = largeScan({ categories: 3, count: 1000 });

const answers = {
  appVersion: '0.0.1',
  getAccessStatus: GRANTED,
  getDictionary: SCAN.dictionary,
  getModules: [],
  getOverview: IDLE,
  startScan: 1,
} as const;

const QUERY: FindingsQuery = { category: 'demo.large-1', limit: 100, module: 'demo', offset: 100, order: 'largest' };

// `get_findings` called as the app calls it, through the Tauri internals the mock replaces.
const getFindings = (page: Page, query: FindingsQuery): Promise<FindingsPage> =>
  page.evaluate(
    async ({ command, query: asked }) => {
      const internals = (globalThis as unknown as Record<string, { invoke: (...args: unknown[]) => Promise<unknown> }>)
        .__TAURI_INTERNALS__;

      return (await internals?.invoke(command, { query: asked })) as FindingsPage;
    },
    { command: COMMANDS.getFindings, query },
  );

// A command called as the app calls it, through the Tauri internals the mock replaces.
const call = <T>(page: Page, command: string, args: Record<string, unknown>): Promise<T> =>
  page.evaluate(
    async ({ asked, name }) => {
      const internals = (globalThis as unknown as Record<string, { invoke: (...args: unknown[]) => Promise<unknown> }>)
        .__TAURI_INTERNALS__;

      return (await internals?.invoke(name, asked)) as T;
    },
    { asked: args, name: command },
  );

// `get_overview` called as the app calls it.
const getOverview = (page: Page): Promise<Overview> =>
  page.evaluate(async (command) => {
    const internals = (globalThis as unknown as Record<string, { invoke: (...args: unknown[]) => Promise<unknown> }>)
      .__TAURI_INTERNALS__;

    return (await internals?.invoke(command, {})) as Overview;
  }, COMMANDS.getOverview);

test.describe('negative cases', () => {
  test('has no findings before a scan', async ({ page }) => {
    await mockCore(page, { answers, scan: SCAN.events });
    await page.goto('/');

    expect(await getFindings(page, QUERY)).toEqual({ findings: [], offset: 100, total: 0 });
  });
});

test.describe('positive cases', () => {
  test('pages through the findings of the scan by category and order', async ({ page }) => {
    await mockCore(page, { answers, scan: SCAN.events, scanned: SCAN.overview });
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();
    const answered = await getFindings(page, QUERY);

    expect(answered).toEqual(pageOf(SCAN.findings, QUERY));
    expect(answered.total).toBe(333);
    expect(answered.findings).toHaveLength(100);
  });

  test('plans a selection over the scan’s findings, leaves out what the script says, and answers the history', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.cleaningLeftOut);
    await page.goto('/');
    await page.getByRole('button', { exact: true, name: 'Scan' }).click();
    await expect(page.getByRole('button', { name: /^Open / })).toHaveCount(2);

    const plan = await call<CleanupPlanView>(page, COMMANDS.prepareCleanup, {
      selection: { categories: [{ category: 'demo.caches', except: [] }], findings: [2] },
    });
    const history = await call<CleanupsPage>(page, COMMANDS.getCleanups, { query: { limit: 50, offset: 0 } });

    expect(plan.summary.count).toBe(2);
    expect(plan.groups.map((group) => group.category)).toStrictEqual(['demo.caches', 'demo.downloads']);
    expect(plan.leftOut.map((item) => item.finding)).toStrictEqual([3]);
    expect(history.cleanups.map((cleanup) => cleanup.cleanup)).toStrictEqual([3, 2, 1]);
  });

  test('a paced scan answers the overview of what it streamed so far, then the scanned one', async ({ page }) => {
    await mockCore(page, { answers, interval: 200, scan: SCAN.events, scanned: SCAN.overview });
    await page.goto('/');

    await page.getByRole('button', { name: 'Scan' }).click();
    await expect.poll(async () => (await getOverview(page)).status).toEqual({ scan: 1, state: 'scanning' });
    const streaming = await getOverview(page);

    expect(streaming.found.allocated).toBeLessThan(SCAN.overview.found.allocated);
    await expect.poll(() => getOverview(page)).toEqual(SCAN.overview);
  });
});
