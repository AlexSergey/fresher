import type { Page } from '@playwright/test';

import type { Event, FindingsQuery, ModuleInfo, Overview, Refusal, SettingUpdate } from '../src/ipc/bindings.generated';

import { type CommandKey, COMMANDS } from '../src/ipc';
import {
  CLEANUP_DICTIONARY,
  DELETED_DETAIL,
  HISTORY,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
} from '../src/test/cleanup-plan';
import {
  BUSY,
  DEMO_FAILED,
  DEMO_FAILED_SCAN,
  DEMO_LIVE_SCAN,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_PARTIAL,
  DEMO_PARTIAL_SCAN,
  DEMO_SCAN,
  DEMO_SCANNING,
  DEMO_SETTINGS,
  DEMO_STOP,
  DEMO_STOPPED,
  DENIED,
  GRANTED,
  IDLE,
  LIVE_HOLD,
  readyCategory,
  SCRIPTS_DICTIONARY,
  SETTINGS_OF_EVERY_KIND,
  SETTINGS_READ_ONLY,
} from '../src/test/demo-scan';
import { largeScan, scanningOverviewOf } from '../src/test/large-scan';
import { pageOf } from '../src/test/page-of';
import { CLEANUP_HOOKS, type CleanupScript, mockCleanup } from './support/cleanup-mock';

// What the mocked core does: an answer per command key, a refusal per command key, the events a scan streams, and
// the overview `get_overview` answers once the scan is asked for (until then, its answer in `answers`).
export type CoreScript = {
  readonly answers?: Partial<Readonly<Record<CommandKey, unknown>>>;
  // The cleanup commands over the scan's findings (support/cleanup-mock.ts); without it they are not answered.
  readonly cleanup?: CleanupScript;
  // The scan streams this many events, then waits for `releaseScan` (or Stop): the live states can be looked at.
  readonly hold?: number;
  // The scan streams one event every `interval` ms, as the engine does, each asked of Node as JSON and parsed in the
  // page as the channel does (the page never holds the whole scan); meanwhile `get_overview` answers the totals of the findings streamed so far, and `scanned` from the last event on.
  readonly interval?: number;
  readonly refusals?: Partial<Readonly<Record<CommandKey, Refusal>>>;
  readonly scan?: readonly Event[];
  readonly scanned?: Overview;
  // What `get_overview` answers while a held scan waits.
  readonly scanning?: Overview;
  // What `cancel_scan` streams in place of the held events, and the overview after it.
  readonly stop?: { readonly events: readonly Event[]; readonly overview: Overview };
};

type ByCommand = {
  readonly answers: Readonly<Record<string, unknown>>;
  readonly busy: Refusal;
  readonly cancelScan: string;
  readonly eventAt: string;
  readonly getFindings: string;
  readonly getOverview: string;
  readonly hold: null | number;
  readonly hooks: typeof CLEANUP_HOOKS;
  readonly interval: null | number;
  readonly overviewOf: string;
  // How many events a scan paced by `interval` streams.
  readonly paced: number;
  readonly pageOf: string;
  readonly refusals: Readonly<Record<string, Refusal>>;
  readonly release: string;
  readonly scan: readonly Event[];
  readonly scanned: null | Overview;
  readonly scanning: null | Overview;
  readonly startScan: string;
  readonly stop: NonNullable<CoreScript['stop']> | null;
  readonly subscribe: string;
  readonly updateSetting: string;
};

// The page's name of the function that answers `get_findings` from the script's findings, in Node.
const PAGE_OF = '__fresherPageOf';

// The page's name of the function that answers `get_overview` from the findings streamed so far, in Node.
const OVERVIEW_OF = '__fresherOverviewOf';

// The page's name of the function that answers a paced scan's event at an index as JSON, in Node.
const EVENT_AT = '__fresherEventAt';

// The page's name of the function that streams the rest of a held scan.
const RELEASE = '__fresherRelease';

const byCommand = <T>(table: Partial<Readonly<Record<CommandKey, T>>> = {}): Readonly<Record<string, T>> =>
  Object.fromEntries((Object.entries(table) as [CommandKey, T][]).map(([key, value]) => [COMMANDS[key], value]));

// The Rust core replaced in the browser, by the command keys of src/ipc/index.ts, injected before the page loads.
// `@tauri-apps/api` calls `window.__TAURI_INTERNALS__`: `invoke` for a command, `transformCallback` for a channel.
// `subscribe` keeps the channel; `start_scan`, when answered, then streams `scan` through it (up to `hold` events), and
// `get_overview` answers `scanning` while the scan is held and `scanned` from its end on. A second `start_scan` while a
// held scan runs is refused as busy, as the engine does; `cancel_scan` ends a held scan with `stop`. An answered
// `update_setting` sends its `setting-changed` event. `get_findings`, unless the script answers or refuses it, pages
// through the findings streamed so far by module, category, order, offset and limit, as the engine does. A scan paced
// by `interval` streams its events one by one and is busy until its last.
export const mockCore = async (page: Page, script: CoreScript = {}): Promise<void> => {
  const findings = (script.scan ?? []).flatMap((event) => (event.type === 'findings-added' ? event.findings : []));
  const setup: ByCommand = {
    answers: byCommand(script.answers),
    busy: BUSY,
    cancelScan: COMMANDS.cancelScan,
    eventAt: EVENT_AT,
    getFindings: COMMANDS.getFindings,
    getOverview: COMMANDS.getOverview,
    hold: script.hold ?? null,
    hooks: CLEANUP_HOOKS,
    interval: script.interval ?? null,
    overviewOf: OVERVIEW_OF,
    paced: script.interval === undefined ? 0 : (script.scan ?? []).length,
    pageOf: PAGE_OF,
    refusals: byCommand(script.refusals),
    release: RELEASE,
    scan: script.interval === undefined ? (script.scan ?? []) : [],
    scanned: script.scanned ?? null,
    scanning: script.scanning ?? null,
    startScan: COMMANDS.startScan,
    stop: script.stop ?? null,
    subscribe: COMMANDS.subscribe,
    updateSetting: COMMANDS.updateSetting,
  };

  await page.exposeFunction(PAGE_OF, (query: FindingsQuery, streamed: number) =>
    pageOf(findings.slice(0, streamed), query),
  );
  await page.exposeFunction(EVENT_AT, (at: number) => JSON.stringify(script.scan?.[at] ?? null));
  await page.exposeFunction(OVERVIEW_OF, (streamed: number) => scanningOverviewOf(findings.slice(0, streamed)));
  await page.addInitScript((core: ByCommand) => {
    const callbacks = new Map<number, (message: unknown) => void>();
    const answers: Record<string, unknown> = { ...core.answers };
    let channel: number | undefined;
    // Every message of the channel has the next index: the channel delivers them in that order.
    let sent = 0;
    // How many findings the events streamed so far carried.
    let streamed = 0;
    let held: readonly Event[] = [];
    let running = false;

    const send = (events: readonly Event[]): void => {
      const deliver = channel === undefined ? undefined : callbacks.get(channel);

      for (const message of events) {
        const index = sent;

        sent += 1;
        streamed += message.type === 'findings-added' ? message.findings.length : 0;
        setTimeout(() => deliver?.({ index, message }), 0);
      }
    };

    const finish = (events: readonly Event[], overview: null | Overview): void => {
      held = [];
      running = false;
      if (overview !== null) {
        answers[core.getOverview] = overview;
      }
      send(events);
    };

    // One event every `interval` ms from now on, each asked of Node and parsed in turn; the last one ends the scan.
    const pace = (interval: number): void => {
      const eventAt = (globalThis as unknown as Record<string, (at: number) => Promise<string>>)[core.eventAt];
      let delivered = Promise.resolve();

      running = true;
      for (let at = 0; at < core.paced; at += 1) {
        setTimeout(() => {
          delivered = delivered
            .then(() => eventAt?.(at))
            .then((json) => {
              const event = JSON.parse(json ?? 'null') as Event;

              if (at === core.paced - 1) {
                finish([event], core.scanned);
              } else {
                send([event]);
              }
            });
        }, at * interval);
      }
    };

    const startScan = (): void => {
      if (core.interval !== null) {
        pace(core.interval);

        return;
      }
      if (core.hold === null) {
        finish(core.scan, core.scanned);

        return;
      }
      running = true;
      held = core.scan.slice(core.hold);
      if (core.scanning !== null) {
        answers[core.getOverview] = core.scanning;
      }
      send(core.scan.slice(0, core.hold));
    };

    Object.assign(globalThis, {
      [core.hooks.answer]: (command: string, value: unknown): void => {
        answers[command] = value;
      },
      // For the cleanup's part of the mock (support/cleanup-mock.ts): it streams through the same channel and may
      // change what a command answers.
      [core.hooks.send]: send,
      [core.release]: (): void => {
        if (running) {
          finish(held, core.scanned);
        }
      },
    });

    const invoke = (command: string, args: Readonly<Record<string, unknown>>): Promise<unknown> => {
      const cleanup = (
        globalThis as unknown as Record<
          string,
          ((command: string, args: unknown) => Promise<unknown> | undefined) | undefined
        >
      )[core.hooks.invoke];
      const answered = command in core.refusals ? undefined : cleanup?.(command, args);

      if (answered !== undefined) {
        return answered;
      }
      if (command === core.subscribe) {
        channel = (args.channel as { readonly id: number }).id;

        return Promise.resolve(null);
      }
      if (command in core.refusals || (command === core.startScan && running)) {
        // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
        return Promise.reject(core.refusals[command] ?? core.busy);
      }
      if (command === core.cancelScan && running && core.stop !== null) {
        finish(core.stop.events, core.stop.overview);

        return Promise.resolve(null);
      }
      if (command === core.getOverview && running && core.interval !== null) {
        const overviewOf = (globalThis as unknown as Record<string, (streamed: number) => Promise<unknown>>)[
          core.overviewOf
        ];

        return overviewOf === undefined ? Promise.resolve(answers[command]) : overviewOf(streamed);
      }
      if (command === core.getFindings && !(command in answers)) {
        const query = args.query as FindingsQuery;
        // Exposed by Playwright: it calls `pageOf` in Node and resolves with its answer.
        const findingsOf = (
          globalThis as unknown as Record<string, (query: FindingsQuery, streamed: number) => Promise<unknown>>
        )[core.pageOf];

        return findingsOf === undefined
          ? Promise.resolve({ findings: [], offset: query.offset, total: 0 })
          : findingsOf(query, streamed);
      }
      if (!(command in answers)) {
        return Promise.reject(new Error(`The core has no answer for "${command}"`));
      }
      if (command === core.startScan) {
        startScan();
      }
      if (command === core.updateSetting) {
        const { key, value } = args.update as SettingUpdate;

        send([{ key, type: 'setting-changed', value }]);
      }

      return Promise.resolve(answers[command]);
    };

    let nextCallback = 1;
    Object.defineProperty(globalThis, '__TAURI_INTERNALS__', {
      value: {
        invoke,
        transformCallback: (callback: (message: unknown) => void): number => {
          const id = nextCallback;
          nextCallback += 1;
          callbacks.set(id, callback);

          return id;
        },
        unregisterCallback: (id: number): void => {
          callbacks.delete(id);
        },
      },
    });
  }, setup);
  if (script.cleanup !== undefined) {
    await mockCleanup(page, findings, script.cleanup);
  }
};

// Streams the rest of a held scan: it finishes as the script's scan.
export const releaseScan = async (page: Page): Promise<void> => {
  await page.evaluate((release) => {
    (globalThis as unknown as Record<string, () => void>)[release]?.();
  }, RELEASE);
};

// What the core answers before any scan: the demo module, its texts and settings, Full Disk Access granted.
const ANSWERS = {
  appVersion: '0.0.1',
  cancelScan: null,
  getAccessStatus: GRANTED,
  getDictionary: SCRIPTS_DICTIONARY,
  getModules: DEMO_MODULES,
  getOverview: IDLE,
  getSettings: DEMO_SETTINGS,
  startScan: 1,
  updateSetting: null,
} as const;

const LARGE = largeScan({ categories: 3, count: 1000 });

// The demo scan's texts with the cleanup's.
const CLEANING_ANSWERS = { ...ANSWERS, getDictionary: { ...SCRIPTS_DICTIONARY, ...CLEANUP_DICTIONARY } };

// What the core answers once every demo finding was removed: nothing left, the scan's tree out of date.
const CLEANED: Overview = {
  ...DEMO_OVERVIEW,
  found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
  lastScan: DEMO_OVERVIEW.lastScan === null ? null : { ...DEMO_OVERVIEW.lastScan, changedSince: 3 },
  totals: [],
};

// The history of three cleanups, the newest the demo plan moved to the Trash.
const CLEANUP = {
  after: CLEANED,
  details: [MOVED_DETAIL, DELETED_DETAIL, INTERRUPTED_DETAIL],
  history: HISTORY,
} as const;

// The large scan's module: the demo module with the generated categories.
const LARGE_MODULES: readonly ModuleInfo[] = [
  {
    categories: LARGE.overview.totals.map(({ category }) => readyCategory(category, 'safe')),
    id: 'demo',
    settings: ['demo.enabled'],
  },
];

// The scripts the flows and the screenshots play.
export const SCRIPTS = {
  // The demo scan, then a cleanup of what is picked: the core's plan, the cleanup streamed at once, a history.
  cleaning: { answers: CLEANING_ANSWERS, cleanup: CLEANUP, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // The same, the cleanup held after its first item until `releaseCleanup` or Stop.
  cleaningHeld: {
    answers: CLEANING_ANSWERS,
    cleanup: { ...CLEANUP, hold: 1 },
    scan: DEMO_SCAN,
    scanned: DEMO_OVERVIEW,
  },
  // The same, the browser cache left out of every plan: an app uses it.
  cleaningLeftOut: {
    answers: CLEANING_ANSWERS,
    cleanup: { ...CLEANUP, leftOut: { 3: { app: 'Safari', kind: 'in-use' } } },
    scan: DEMO_SCAN,
    scanned: DEMO_OVERVIEW,
  },
  // No cleanup yet.
  cleanupsEmpty: { answers: CLEANING_ANSWERS, cleanup: {} },
  // The demo scan, streamed at once.
  demo: { answers: ANSWERS, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // Full Disk Access refused: the overview shows the permission card.
  denied: { answers: { ...ANSWERS, getAccessStatus: DENIED } },
  // The demo module fails after its first batch, and the scan with it.
  failed: { answers: ANSWERS, scan: DEMO_FAILED_SCAN, scanned: DEMO_FAILED },
  // A thousand findings in three categories, paged by the lists.
  large: {
    answers: { ...ANSWERS, getDictionary: LARGE.dictionary, getModules: LARGE_MODULES },
    scan: LARGE.events,
    scanned: LARGE.overview,
  },
  // The demo scan held after its first batch and progress, until `releaseScan` or Stop (partial, the first batch).
  live: {
    answers: ANSWERS,
    hold: LIVE_HOLD,
    scan: DEMO_LIVE_SCAN,
    scanned: DEMO_OVERVIEW,
    scanning: DEMO_SCANNING,
    stop: { events: DEMO_STOP, overview: DEMO_STOPPED },
  },
  // Some folders could not be read: the scan ends partial, the caches' size a lower bound.
  partial: { answers: ANSWERS, scan: DEMO_PARTIAL_SCAN, scanned: DEMO_PARTIAL },
  // Settings of every kind, in a file a newer version of the app saved: none can be changed.
  readOnly: { answers: { ...ANSWERS, getSettings: SETTINGS_READ_ONLY } },
  // The core refuses every scan as busy.
  refused: { answers: ANSWERS, refusals: { startScan: BUSY } },
  // Settings of every kind.
  settings: { answers: { ...ANSWERS, getSettings: SETTINGS_OF_EVERY_KIND } },
  // The demo scan held before its first event: the core runs it, the app does not know yet.
  unannounced: { answers: ANSWERS, hold: 0, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
} as const satisfies Readonly<Record<string, CoreScript>>;
