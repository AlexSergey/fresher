import { expect, type Page, test } from '@playwright/test';

import type {
  CleanupPlanView,
  CleanupsPage,
  Event,
  Finding,
  FindingsPage,
  FindingsQuery,
  LargestPage,
  LargestQuery,
  Overview,
} from '../src/ipc/bindings.generated';

import { COMMANDS } from '../src/ipc';
import { DEMO_FINDINGS, DEMO_SCAN, GRANTED, IDLE } from '../src/test/demo-scan';
import { largeScan } from '../src/test/large-scan';
import { pageOf } from '../src/test/page-of';
import { DEMO_MAP_NO_SCAN } from '../src/test/storage-map';
import { mockCore, SCRIPTS } from './core-mock';
import { scanButton } from './support/cards';
import { largestAnswerOf, pickAnswerOf, placeAnswerOf } from './support/largest-mock';

// The mocked core pages through a scan's findings as the engine does, so the e2e tests of the lists page for real.

const SCAN = largeScan({ categories: 3, count: 1000 });

const answers = {
  appInfo: { build: { kind: 'release' }, version: '0.0.1' },
  getAccessStatus: GRANTED,
  getDictionary: SCAN.dictionary,
  getModules: [],
  getOverview: IDLE,
  getStorageMap: DEMO_MAP_NO_SCAN,
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

const ANY: LargestQuery = { kind: null, limit: 100, offset: 0, place: null, unchanged: 'any' };

const DEMO_QUERY: FindingsQuery = { category: 'demo.caches', limit: 10, module: 'demo', offset: 0, order: 'largest' };

const idsOf = (answered: FindingsPage): readonly number[] => answered.findings.map((finding) => finding.id);

// The demo scan with a finding inside the editor cache and a sibling whose name only starts the same.
const [FIRST_FINDING] = DEMO_FINDINGS as readonly [(typeof DEMO_FINDINGS)[number]];
const INSIDE: Finding = { ...FIRST_FINDING, id: 9, subject: { kind: 'path', value: '/demo/editor-cache/inner' } };
const SIBLING: Finding = { ...FIRST_FINDING, id: 10, subject: { kind: 'path', value: '/demo/editor-cache-old' } };
const NESTED_SCAN: readonly Event[] = [
  ...DEMO_SCAN.slice(0, 3),
  { findings: [INSIDE, SIBLING], scan: 1, type: 'findings-added' },
  ...DEMO_SCAN.slice(3),
];

test.describe('negative cases', () => {
  test('a finding outside the ignored paths stays', async ({ page }) => {
    await mockCore(page, SCRIPTS.ignoring);
    await page.goto('/');
    await scanButton(page).click();

    await call(page, COMMANDS.ignore, { target: { finding: 1, kind: 'finding' } });
    const answered = await getFindings(page, DEMO_QUERY);

    expect(idsOf(answered)).toStrictEqual([3]);
    expect(answered.total).toBe(1);
  });

  test('open_trash refuses an index the last get_trash answer did not have', async ({ page }) => {
    await mockCore(page, SCRIPTS.trash);
    await page.goto('/');
    await expect(page.getByRole('button', { name: 'Open Trash' })).toBeVisible();

    const refusal = await page.evaluate(async (command) => {
      const internals = (globalThis as unknown as Record<string, { invoke: (...args: unknown[]) => Promise<unknown> }>)
        .__TAURI_INTERNALS__;

      return internals?.invoke(command, { index: 1 }).then(
        () => null,
        (error: unknown) => error,
      );
    }, COMMANDS.openTrash);

    expect(refusal).toMatchObject({ error: { code: 'unknown-trash', index: 1 } });
    expect(await page.evaluate(() => (globalThis as unknown as { __openedTrash: number[] }).__openedTrash)).toEqual([]);
  });

  test('has no findings before a scan', async ({ page }) => {
    await mockCore(page, { answers, scan: SCAN.events });
    await page.goto('/');

    expect(await getFindings(page, QUERY)).toEqual({ findings: [], offset: 100, total: 0 });
  });

  test('a withheld row is never picked, and an unknown node either', async ({ page }) => {
    await mockCore(page, SCRIPTS.largest);
    await page.goto('/');

    const refusals = await page.evaluate(
      async ({ command, nodes }) => {
        const internals = (
          globalThis as unknown as Record<string, { invoke: (...args: unknown[]) => Promise<unknown> }>
        ).__TAURI_INTERNALS__;

        const refused: unknown[] = [];

        for (const node of nodes) {
          refused.push(
            await internals?.invoke(command, { node }).then(
              () => null,
              (refusal: unknown) => refusal,
            ),
          );
        }

        return refused;
      },
      {
        command: COMMANDS.pickOwnItem,
        nodes: [
          { index: 103, state: 1 },
          { index: 11, state: 1 },
          { index: 999, state: 1 },
        ],
      },
    );

    expect(refusals).toMatchObject([
      { error: { code: 'not-own-item' } },
      { error: { code: 'not-own-item' } },
      { error: { code: 'not-own-item' } },
    ]);
  });

  test('a folder that does not open is refused as the core refuses it', () => {
    expect(placeAnswerOf({ index: 11, state: 1 }, 0, 100)).toBe('not-own-item');
    expect(placeAnswerOf({ index: 999, state: 1 }, 0, 100)).toBe('stale-node');
  });

  test('a file that is not offered has no pick', () => {
    expect(pickAnswerOf({ index: 103, state: 1 })).toBeNull();
    expect(pickAnswerOf({ index: 101, state: 2 })).toBeNull();
  });

  test('a row a cleanup moved leaves the list and cannot be picked again', () => {
    const picks = { gone: new Set([101]), ids: new Map<number, number>() };

    expect(largestAnswerOf(ANY, picks).rows.map((row) => row.node.id.index)).not.toContain(101);
    expect(pickAnswerOf({ index: 101, state: 1 }, picks)).toBeNull();
  });
});

test.describe('positive cases', () => {
  test('an ignored path takes a finding inside it, not a sibling that shares its start', async ({ page }) => {
    await mockCore(page, { ...SCRIPTS.ignoring, scan: NESTED_SCAN });
    await page.goto('/');
    await scanButton(page).click();

    await call(page, COMMANDS.ignore, { target: { finding: 1, kind: 'finding' } });
    const answered = await getFindings(page, DEMO_QUERY);

    expect(idsOf(answered)).toStrictEqual([10, 3]);
    expect(answered.total).toBe(2);
  });

  test('an ignored path leaves get_findings with what is inside it, and a restore brings it back', async ({ page }) => {
    await mockCore(page, SCRIPTS.ignoring);
    await page.goto('/');
    await scanButton(page).click();

    await call(page, COMMANDS.ignore, { target: { finding: 1, kind: 'finding' } });
    const hidden = await getFindings(page, DEMO_QUERY);
    await call(page, COMMANDS.restoreIgnored, { entry: { kind: 'path', path: '/demo/editor-cache' } });
    const restored = await getFindings(page, DEMO_QUERY);

    expect(idsOf(hidden)).toStrictEqual([3]);
    expect(hidden.total).toBe(1);
    expect(restored).toEqual(pageOf(DEMO_FINDINGS, DEMO_QUERY));
  });

  test('pages through the findings of the scan by category and order', async ({ page }) => {
    await mockCore(page, { answers, scan: SCAN.events, scanned: SCAN.overview });
    await page.goto('/');

    await scanButton(page).click();
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
    await scanButton(page).click();
    await expect(page.getByRole('button', { name: 'Details' })).toHaveCount(2);

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

    await scanButton(page).click();
    await expect.poll(async () => (await getOverview(page)).status).toEqual({ scan: 1, state: 'scanning' });
    const streaming = await getOverview(page);

    expect(streaming.found.allocated).toBeLessThan(SCAN.overview.found.allocated);
    await expect.poll(() => getOverview(page)).toEqual(SCAN.overview);
  });

  test('the Files list keeps the rows under the kind, the place and the unchanged filters, largest first', () => {
    const names = (query: Partial<LargestQuery>): string[] =>
      largestAnswerOf({ ...ANY, ...query }).rows.map((row) => row.node.name);

    expect(names({})).toStrictEqual([
      'ubuntu-24.04.dmg',
      'Windows 11.pvm',
      'Photos Library.photoslibrary',
      'holiday-2022.mov',
      'project-backup.zip',
    ]);
    expect(names({ kind: 'video' })).toStrictEqual(['holiday-2022.mov']);
    expect(names({ place: { index: 12, state: 1 } })).toStrictEqual(['ubuntu-24.04.dmg']);
    expect(names({ unchanged: 'two-years' })).toStrictEqual(['ubuntu-24.04.dmg', 'holiday-2022.mov']);
  });

  test('the kinds and the folders are counted under the other filters', () => {
    const answer = largestAnswerOf({ ...ANY, unchanged: 'two-years' });

    expect(answer.kinds).toStrictEqual([
      { count: 1, kind: 'disk-image' },
      { count: 1, kind: 'video' },
    ]);
    expect(answer.places.map(({ count, folder }) => [folder.name, count])).toStrictEqual([
      ['Downloads', 1],
      ['Movies', 1],
    ]);
    expect(largestAnswerOf({ ...ANY, kind: 'video' }).kinds).toHaveLength(5);
  });

  test('the home lists its folders largest first and a folder that opens lists what it holds', () => {
    const home = placeAnswerOf(null, 0, 100);
    const movies = placeAnswerOf({ index: 15, state: 1 }, 0, 100);

    expect(home).toMatchObject({ total: 6 });
    expect(typeof home === 'string' ? [] : home.rows.map((row) => row.node.name)).toStrictEqual([
      'Library',
      'Downloads',
      'Parallels',
      'Pictures',
      'Movies',
      'Documents',
    ]);
    expect(typeof movies === 'string' ? [] : movies.rows.map((row) => row.node.name)).toStrictEqual([
      'holiday-2022.mov',
    ]);
    expect(typeof movies === 'string' ? [] : movies.path.map((node) => node.name)).toStrictEqual(['someone', 'Movies']);
  });

  test('a pick answers the core’s High finding, the same one twice, and the page tells it was picked', async ({
    page,
  }) => {
    await mockCore(page, SCRIPTS.largest);
    await page.goto('/');

    const node = { index: 101, state: 1 };
    const first = await call<Finding>(page, COMMANDS.pickOwnItem, { node });
    const second = await call<Finding>(page, COMMANDS.pickOwnItem, { node });
    const listed = await call<LargestPage>(page, COMMANDS.getLargest, { query: ANY });

    expect(first).toMatchObject({ category: 'core.own-file', danger: 'high' });
    expect(second).toStrictEqual(first);
    expect(listed.rows.map((row) => row.picked)).toStrictEqual([first.id, null, null, null, null]);
  });
});
