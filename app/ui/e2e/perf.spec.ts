import { type BrowserContext, expect, type Page, test, type TestInfo } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';

import type { CategoryTotal, ModuleInfo, Overview } from '../src/ipc/bindings.generated';

import { readyCategory } from '../src/test/demo-scan';
import { largeScan } from '../src/test/large-scan';
import { type CoreScript, mockCore, releaseScan, SCRIPTS } from './core-mock';
import { beginSample, endSample, type Sample, sampleFrames, settle } from './support/frames';

// The performance budget of plan 003 (docs/plans/003-base-ui/00-design.md, "Performance budget"), measured on the
// reference Mac by `npm run test:perf`: headed, full motion, one test at a time. Each scenario runs three times on a
// new page; the median of each number is reported and held to the budget in the reference engine (WebKit). Chromium
// adds the long tasks and the JS heap, which only it reports.

const BUDGET = { cssKb: 40, jsKb: 200, longestMs: 50, p95Ms: 17, rows: 60 } as const;

// A frame that missed at least one 60 Hz refresh (1.5 × 16.7 ms).
const DROPPED_MS = 25;

const IDLE = 'idle (calibration)';

const RUNS = 3;

// The reference engine: the budget is held there; the other perf project only reports.
const REFERENCE = 'perf';

type Row = {
  readonly dropped: number;
  readonly frames: number;
  readonly heapAfterMb: null | number;
  readonly heapBeforeMb: null | number;
  readonly longestMs: number;
  readonly longTaskMs: null | number;
  readonly longTasks: null | number;
  readonly nodes: number;
  readonly over50: number;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly rows: number;
  readonly scenario: string;
};

// The scan of scenario 1: 50,000 findings in 12 categories, batches of 256 every 50 ms.
const LARGE = largeScan({ categories: 12, count: 50_000 });

const modulesOf = (totals: readonly CategoryTotal[]): readonly ModuleInfo[] => [
  {
    categories: totals.map(({ category }) => readyCategory(category, 'safe')),
    choices: [],
    group: 'debug',
    id: 'demo',
    settings: ['demo.enabled'],
  },
];

const STREAMED: CoreScript = {
  answers: { ...SCRIPTS.large.answers, getDictionary: LARGE.dictionary, getModules: modulesOf(LARGE.overview.totals) },
  interval: 50,
  scan: LARGE.events,
  scanned: LARGE.overview,
};

// The re-sort: twelve categories held mid-scan smallest last, then finished in the reverse order, so every card moves.
const TWELVE = largeScan({ categories: 12, count: 48 });

const sized = (order: 'ascending' | 'descending'): Overview => {
  const totals = TWELVE.overview.totals.map((total, at) => {
    const gigabytes = order === 'ascending' ? at + 1 : TWELVE.overview.totals.length - at;
    const bytes = gigabytes * 1_000_000_000;

    return {
      ...total,
      sizes: { allocated: bytes, completeness: 'complete', logical: bytes, reclaimable: bytes },
    } as const;
  });
  const found = totals.reduce((sum, total) => sum + total.sizes.allocated, 0);

  return {
    ...TWELVE.overview,
    found: { allocated: found, completeness: 'complete', logical: found, reclaimable: found },
    totals,
  };
};

const RESORTED: CoreScript = {
  answers: {
    ...SCRIPTS.large.answers,
    getDictionary: TWELVE.dictionary,
    getModules: modulesOf(TWELVE.overview.totals),
  },
  // Held after `scan-started` and the findings: the cards stand in the scanning order.
  hold: 2,
  scan: TWELVE.events,
  scanned: sized('descending'),
  scanning: { ...sized('ascending'), lastScan: null, status: { scan: 1, state: 'scanning' } },
};

const median = (values: readonly number[]): number => {
  const sorted = values.toSorted((a, b) => a - b);

  return sorted[Math.floor(sorted.length / 2)] ?? 0;
};

const medianOrNull = (values: readonly (null | number)[]): null | number =>
  values.includes(null) ? null : median(values as readonly number[]);

// Nearest rank.
const percentile = (values: readonly number[], share: number): number => {
  const sorted = values.toSorted((a, b) => a - b);

  return sorted[Math.max(0, Math.ceil(share * sorted.length) - 1)] ?? 0;
};

const megabytes = (bytes: null | number): null | number => (bytes === null ? null : bytes / 1_000_000);

const rowOf = (scenario: string, samples: readonly Measured[]): Row => ({
  dropped: median(samples.map(({ intervals }) => intervals.filter((interval) => interval > DROPPED_MS).length)),
  frames: median(samples.map(({ intervals }) => intervals.length)),
  heapAfterMb: medianOrNull(samples.map(({ heapAfter }) => megabytes(heapAfter))),
  heapBeforeMb: medianOrNull(samples.map(({ heapBefore }) => megabytes(heapBefore))),
  longestMs: median(samples.map(({ intervals }) => Math.max(0, ...intervals))),
  longTaskMs: medianOrNull(samples.map(({ longTasks }) => (longTasks === null ? null : Math.max(0, ...longTasks)))),
  longTasks: medianOrNull(samples.map(({ longTasks }) => longTasks?.length ?? null)),
  nodes: median(samples.map(({ nodes }) => nodes)),
  over50: median(samples.map(({ intervals }) => intervals.filter((interval) => interval > BUDGET.longestMs).length)),
  p50Ms: median(samples.map(({ intervals }) => percentile(intervals, 0.5))),
  p95Ms: median(samples.map(({ intervals }) => percentile(intervals, 0.95))),
  rows: median(samples.map(({ rows }) => rows)),
  scenario,
});

const cell = (value: null | number | string): string =>
  value === null ? 'n/a' : typeof value === 'number' ? value.toFixed(Number.isInteger(value) ? 0 : 1) : value;

const COLUMNS = [
  'scenario',
  'frames',
  'p50Ms',
  'p95Ms',
  'longestMs',
  'dropped',
  'over50',
  'rows',
  'nodes',
  'longTasks',
  'longTaskMs',
  'heapBeforeMb',
  'heapAfterMb',
] as const satisfies readonly (keyof Row)[];

// The rows as a table, attached to the report and printed.
const report = async (
  testInfo: TestInfo,
  { rows, title, version }: { readonly rows: readonly Row[]; readonly title: string; readonly version: string },
): Promise<void> => {
  const lines = [COLUMNS.join(' | '), ...rows.map((row) => COLUMNS.map((column) => cell(row[column])).join(' | '))];
  const table = `${title} (${testInfo.project.name} ${version}, median of ${String(RUNS)} runs)\n${lines.join('\n')}\n`;

  await testInfo.attach(`${title}.json`, { body: JSON.stringify(rows, null, 2), contentType: 'application/json' });
  await testInfo.attach(`${title}.txt`, { body: table, contentType: 'text/plain' });
  process.stdout.write(`\n${table}`);
};

// The budget, held only in the reference engine. WebKit's frame clock counts whole milliseconds and jitters: an idle
// page there reads a p95 of 17–18 ms at 60 Hz. The p95 is held to 17 ms, or to the idle page's p95 of the same run
// when the clock itself reads more; no frame may exceed 50 ms, whatever the clock.
const expectWithinBudget = (testInfo: TestInfo, rows: readonly Row[]): void => {
  const held = testInfo.project.name === REFERENCE;
  const idle = rows.find(({ scenario }) => scenario === IDLE)?.p95Ms ?? BUDGET.p95Ms;
  const p95 = Math.max(BUDGET.p95Ms, idle);

  for (const row of rows) {
    expect.soft(!held || row.p95Ms <= p95, `${row.scenario}: p95 ${cell(row.p95Ms)} ms, idle ${cell(idle)}`).toBe(true);
    expect.soft(!held || row.over50 === 0, `${row.scenario}: ${cell(row.over50)} frames over 50 ms`).toBe(true);
    expect.soft(!held || row.rows <= BUDGET.rows, `${row.scenario}: ${cell(row.rows)} rows`).toBe(true);
  }
};

// Runs `measure` RUNS times, each on a new page of the test's context, one sample per named part each time.
const measureRuns = async (
  context: BrowserContext,
  measure: (page: Page) => Promise<Readonly<Record<string, Measured>>>,
): Promise<readonly Row[]> => {
  const runs: Readonly<Record<string, Measured>>[] = [];

  for (let run = 0; run < RUNS; run += 1) {
    const page = await context.newPage();

    await sampleFrames(page);
    runs.push(await measure(page));
    await page.close();
  }

  return Object.keys(runs[0] ?? {}).map((scenario) =>
    rowOf(
      scenario,
      runs.flatMap((samples) => samples[scenario] ?? []),
    ),
  );
};

// The overview standing still for three seconds: what the engine's own frame clock reads.
const calibrate = (context: BrowserContext): Promise<readonly Row[]> =>
  measureRuns(context, async (page) => {
    await mockCore(page, SCRIPTS.demo);
    await page.goto('/');
    await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
    await settle(page);

    return {
      [IDLE]: await sampleOf(page, () =>
        page.evaluate(() => new Promise<void>((resolve) => setTimeout(resolve, 3000))),
      ),
    };
  });

type Measured = Sample & {
  // Bytes of the JS heap after a full garbage collection (Chromium, through its DevTools protocol); `null` elsewhere.
  readonly heapAfter: null | number;
  readonly heapBefore: null | number;
};

const heapOf = async (page: Page): Promise<null | number> => {
  if (page.context().browser()?.browserType().name() !== 'chromium') {
    return null;
  }
  const session = await page.context().newCDPSession(page);

  await session.send('HeapProfiler.collectGarbage');
  const { usedSize } = await session.send('Runtime.getHeapUsage');
  await session.detach();

  return usedSize;
};

// Samples from `act` until the page has settled.
const sampleOf = async (page: Page, act: () => Promise<void>): Promise<Measured> => {
  const heapBefore = await heapOf(page);

  await beginSample(page);
  await act();
  await settle(page);
  const sample = await endSample(page);

  return { ...sample, heapAfter: await heapOf(page), heapBefore };
};

const heading = (page: Page, name: string): ReturnType<Page['getByRole']> =>
  page.getByRole('heading', { level: 1, name });

const cards = (page: Page): ReturnType<Page['getByRole']> => page.getByRole('button', { name: /^Open / });

const scanButton = (page: Page): ReturnType<Page['getByRole']> =>
  page.getByRole('button', { exact: true, name: 'Scan' });

// The scan's 196 batches take about ten seconds.
const SCAN_TIMEOUT = 60_000;

test.describe.configure({ timeout: 300_000 });

test.describe('positive cases', () => {
  test('the scan of 50,000 findings streams within the budget', async ({ context }, testInfo) => {
    const overview = await measureRuns(context, async (page) => {
      await mockCore(page, STREAMED);
      await page.goto('/');
      await expect(heading(page, 'Overview')).toBeVisible();
      await settle(page);

      return {
        'scan, overview': await sampleOf(page, async () => {
          await scanButton(page).click();
          await expect(page.getByRole('button', { name: 'Scan again' })).toBeVisible({ timeout: SCAN_TIMEOUT });
          await expect(cards(page)).toHaveCount(12);
        }),
      };
    });
    const modulePage = await measureRuns(context, async (page) => {
      await mockCore(page, STREAMED);
      await page.goto('/');
      await page.getByRole('button', { name: /^Demo module/ }).click();
      await expect(heading(page, 'Demo module')).toBeFocused();
      await settle(page);

      return {
        'scan, module page': await sampleOf(page, async () => {
          await scanButton(page).click();
          await expect(page.getByRole('button', { name: 'Stop' })).toBeVisible();
          await expect(scanButton(page)).toBeVisible({ timeout: SCAN_TIMEOUT });
          // The largest category is expanded and its rows have loaded.
          await expect(
            page
              .getByRole('list', { name: /^The items of / })
              .getByRole('listitem')
              .first(),
          ).toContainText('Example: item-');
        }),
      };
    });
    const rows = [...(await calibrate(context)), ...overview, ...modulePage];

    await report(testInfo, { rows, title: 'scan', version: context.browser()?.version() ?? '' });
    expectWithinBudget(testInfo, rows);
  });

  test('the transitions stay within the budget', async ({ context }, testInfo) => {
    const screens = await measureRuns(context, async (page) => {
      await mockCore(page, SCRIPTS.demo);
      await page.goto('/');
      await scanButton(page).click();
      await expect(cards(page)).toHaveCount(2);
      await settle(page);

      const toModule = await sampleOf(page, async () => {
        await page.getByRole('button', { name: /^Demo module/ }).click();
        await expect(heading(page, 'Demo module')).toBeFocused();
      });
      const selectionBar = await sampleOf(page, async () => {
        await page.getByRole('checkbox', { name: 'Select Example: browser-cache' }).check();
        await expect(page.getByRole('region', { name: 'Selection' })).toContainText('1 item selected');
      });
      const toReview = await sampleOf(page, async () => {
        await page
          .getByRole('button', { name: /^Review/ })
          .first()
          .click();
        await expect(heading(page, 'Review')).toBeFocused();
      });
      const toSettings = await sampleOf(page, async () => {
        await page.getByRole('button', { name: 'Settings' }).click();
        await expect(heading(page, 'Settings')).toBeFocused();
      });
      const toOverview = await sampleOf(page, async () => {
        await page.getByRole('button', { name: 'Overview' }).click();
        await expect(heading(page, 'Overview')).toBeFocused();
      });

      return {
        'screen, module → review': toReview,
        'screen, overview → module': toModule,
        'screen, review → settings': toSettings,
        'screen, settings → overview': toOverview,
        'selection bar appears': selectionBar,
      };
    });
    const resort = await measureRuns(context, async (page) => {
      await mockCore(page, RESORTED);
      await page.goto('/');
      await scanButton(page).click();
      await expect(cards(page)).toHaveCount(12);
      await expect(cards(page).first()).toHaveAccessibleName('Open Example group 12');
      await settle(page);

      return {
        're-sort of 12 cards': await sampleOf(page, async () => {
          await releaseScan(page);
          await expect(cards(page).first()).toHaveAccessibleName('Open Example group 1');
        }),
      };
    });
    const dialog = await measureRuns(context, async (page) => {
      await page.goto('/gallery.html#/');
      await expect(page.getByText('Pick a component to see its states.')).toBeVisible();
      await settle(page);
      const open = await sampleOf(page, async () => {
        await page.evaluate(() => {
          window.location.hash = '#/ConfirmDialog/trash';
        });
        await expect(page.getByRole('dialog')).toBeVisible();
      });
      // The story's owner never sets `open` to false: leaving the story unmounts the open dialog.
      const close = await sampleOf(page, async () => {
        await page.evaluate(() => {
          window.location.hash = '#/';
        });
        await expect(page.getByRole('dialog')).toHaveCount(0);
      });

      return { 'dialog closes (gallery)': close, 'dialog opens (gallery)': open };
    });
    const rows = [...(await calibrate(context)), ...screens, ...resort, ...dialog];

    await report(testInfo, { rows, title: 'transitions', version: context.browser()?.version() ?? '' });
    expectWithinBudget(testInfo, rows);
  });

  test('the 100,000-row list scrolls within the budget', async ({ context }, testInfo) => {
    const scroll = await measureRuns(context, async (page) => {
      await page.goto('/gallery.html#/VirtualList/hundred-thousand');
      const list = page.getByRole('list', { name: 'Caches' });
      await expect(list.getByRole('listitem').first()).toBeVisible();
      await settle(page);

      return {
        'scroll, 100,000 rows': await sampleOf(page, async () => {
          // From the top to the bottom in ten seconds, the same distance every millisecond.
          await list.evaluate(
            (viewport, duration) =>
              new Promise<void>((resolve) => {
                const end = viewport.scrollHeight - viewport.clientHeight;
                const start = performance.now();
                const step = (now: number): void => {
                  const share = Math.min(1, (now - start) / duration);

                  viewport.scrollTop = end * share;
                  if (share < 1) {
                    requestAnimationFrame(step);
                  } else {
                    resolve();
                  }
                };

                requestAnimationFrame(step);
              }),
            10_000,
          );
          await expect(list.getByRole('listitem').last()).toHaveAttribute('aria-posinset', '100000');
        }),
      };
    });

    const rows = [...(await calibrate(context)), ...scroll];

    await report(testInfo, { rows, title: 'scroll', version: context.browser()?.version() ?? '' });
    expectWithinBudget(testInfo, rows);
  });

  test('the main entry is within its gzipped budget', async ({}, testInfo) => {
    const ui = fileURLToPath(new URL('..', import.meta.url));
    const out = testInfo.outputPath('bundle');

    execFileSync('npx', ['vite', 'build', '--outDir', out, '--emptyOutDir', '--logLevel', 'error'], { cwd: ui });
    const html = readFileSync(path.join(out, 'index.html'), 'utf8');
    const gzipped = (pattern: RegExp): number =>
      [...html.matchAll(pattern)].reduce(
        (sum, [, href = '']) => sum + gzipSync(readFileSync(path.join(out, href))).length,
        0,
      ) / 1000;
    const js = gzipped(/<(?:script[^>]*src|link rel="modulepreload"[^>]*href)="\/?([^"]+\.js)"/gu);
    const css = gzipped(/<link rel="stylesheet"[^>]*href="\/?([^"]+\.css)"/gu);

    process.stdout.write(
      `\nbundle (gzipped): JS ${js.toFixed(1)} kB of ${String(BUDGET.jsKb)}, CSS ${css.toFixed(1)} kB of ${String(BUDGET.cssKb)}\n`,
    );
    await testInfo.attach('bundle.json', { body: JSON.stringify({ css, js }), contentType: 'application/json' });
    expect(js).toBeGreaterThan(0);
    expect(css).toBeGreaterThan(0);
    expect(js).toBeLessThanOrEqual(BUDGET.jsKb);
    expect(css).toBeLessThanOrEqual(BUDGET.cssKb);
  });
});
