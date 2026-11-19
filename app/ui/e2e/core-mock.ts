import type { Page } from '@playwright/test';

import type {
  Event,
  Finding,
  FindingsQuery,
  MeaningGroup,
  ModuleInfo,
  Overview,
  PlaceView,
  Refusal,
  SettingUpdate,
  StorageMap,
} from '../src/ipc/bindings.generated';

import { type CommandKey, COMMANDS } from '../src/ipc';
import {
  APP_LEFTOVERS_DICTIONARY,
  APP_LEFTOVERS_FINDINGS,
  APP_LEFTOVERS_MEANING,
  APP_LEFTOVERS_MODULES,
  APP_LEFTOVERS_OVERVIEW,
  APP_LEFTOVERS_SCAN,
  APP_LEFTOVERS_SETTINGS,
  APP_LEFTOVERS_TOTALS,
  scanOf,
} from '../src/test/app-leftovers-scan';
import {
  CLEANUP_DICTIONARY,
  DELETED_DETAIL,
  HISTORY,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
  movedDetailOf,
  planOf,
} from '../src/test/cleanup-plan';
import {
  ACCESS_DICTIONARY,
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
  LIMITED_COVERAGE,
  LIVE_HOLD,
  readyCategory,
  SCRIPTS_DICTIONARY,
  SETTINGS_OF_EVERY_KIND,
  SETTINGS_READ_ONLY,
} from '../src/test/demo-scan';
import {
  DEV_CACHES_DICTIONARY,
  DEV_CACHES_FINDINGS,
  DEV_CACHES_MEANING,
  DEV_CACHES_MODULES,
  DEV_CACHES_OVERVIEW,
  DEV_CACHES_SCAN,
  DEV_CACHES_SETTINGS,
  DEV_CACHES_XCODE_OPEN_OVERVIEW,
  DEV_CACHES_XCODE_OPEN_SCAN,
} from '../src/test/dev-caches-scan';
import {
  DOWNLOADS_DICTIONARY,
  DOWNLOADS_FINDINGS,
  DOWNLOADS_GRANTED,
  DOWNLOADS_MEANING,
  DOWNLOADS_MODULES,
  DOWNLOADS_NOT_ASKED,
  DOWNLOADS_OVERVIEW,
  DOWNLOADS_SCAN,
  DOWNLOADS_SETTINGS,
  DOWNLOADS_TOTALS,
  DOWNLOADS_UNAVAILABLE_MODULES,
  DOWNLOADS_UNAVAILABLE_OVERVIEW,
  DOWNLOADS_UNAVAILABLE_SCAN,
} from '../src/test/downloads-scan';
import {
  JETBRAINS_DICTIONARY,
  JETBRAINS_FINDINGS,
  JETBRAINS_MEANING,
  JETBRAINS_MODULES,
  JETBRAINS_OVERVIEW,
  JETBRAINS_SCAN,
  JETBRAINS_SETTINGS,
} from '../src/test/jetbrains-scan';
import { largeScan, scanningOverviewOf } from '../src/test/large-scan';
import { pageOf } from '../src/test/page-of';
import {
  DEMO_DESCENT,
  DEMO_MAP_COMPLETE,
  DEMO_MAP_NO_SCAN,
  DEMO_MAP_SCANNING,
  DEMO_MEANING_SCANNING,
  DEMO_PLACE,
  EMPTY_PLACE,
  meaningFor,
  STORAGE_DICTIONARY,
} from '../src/test/storage-map';
import {
  TEMP_DICTIONARY,
  TEMP_FINDINGS,
  TEMP_MAP,
  TEMP_MODULES,
  TEMP_OVERVIEW,
  TEMP_PLACES,
  TEMP_SCAN,
  TEMP_SETTINGS,
} from '../src/test/temp-scan';
import { CLEANUP_HOOKS, type CleanupScript, mockCleanup } from './support/cleanup-mock';
import { IGNORE_HOOK, mockIgnore } from './support/ignore-mock';

// What the mocked core does: an answer per command key, a refusal per command key, the events a scan streams, and
// the overview `get_overview` answers once the scan is asked for (until then, its answer in `answers`).
export type CoreScript = {
  readonly answers?: Partial<Readonly<Record<CommandKey, unknown>>>;
  // The cleanup commands over the scan's findings (support/cleanup-mock.ts); without it they are not answered.
  readonly cleanup?: CleanupScript;
  // The scan streams this many events, then waits for `releaseScan` (or Stop): the live states can be looked at.
  readonly hold?: number;
  // The ignore commands of modules and groups, kept in the tab's session storage (support/ignore-mock.ts).
  readonly ignoring?: boolean;
  // The scan streams one event every `interval` ms, as the engine does, each asked of Node as JSON and parsed in the
  // page as the channel does (the page never holds the whole scan); meanwhile `get_overview` answers the totals of the findings streamed so far, and `scanned` from the last event on.
  readonly interval?: number;
  // The by-meaning groups once the scan ended, in place of the demo volume's; the volume's own lines follow them.
  readonly meaning?: readonly MeaningGroup[];
  readonly refusals?: Partial<Readonly<Record<CommandKey, Refusal>>>;
  readonly scan?: readonly Event[];
  readonly scanned?: Overview;
  // What `get_overview` answers while a held scan waits.
  readonly scanning?: Overview;
  readonly stop?: { readonly events: readonly Event[]; readonly overview: Overview };
  // What `cancel_scan` streams in place of the held events, and the overview after it.
  // The storage map and the places after the scan, in place of the demo volume's.
  readonly storage?: { readonly map: StorageMap; readonly places: Readonly<Record<string, PlaceView>> };
};

// What the storage map's commands answer in one phase of the scan: the places by their node's index, any other
// empty, unless the script answers them; a place is refused as busy while a scan runs (`null`), as the engine does.
type StorageAnswers = {
  readonly map: StorageMap;
  readonly meaning: readonly MeaningGroup[];
  readonly places: null | Readonly<Record<string, PlaceView>>;
};

// The demo volume's map while the scan runs and once it ended with `scanned` (its categories, as the core would list
// them); before a scan the default answers hold.
const storageOfScan = (
  scanned: null | Overview,
  meaning?: readonly MeaningGroup[],
  storage?: CoreScript['storage'],
): ByCommand['storage'] => ({
  scanned: {
    map: storage?.map ?? DEMO_MAP_COMPLETE,
    meaning:
      meaning === undefined
        ? meaningFor(scanned?.totals ?? [])
        : [...meaning, ...meaningFor([]).filter(({ kind }) => kind !== 'app')],
    places: { 0: DEMO_PLACE, 1: DEMO_DESCENT, ...storage?.places },
  },
  scanning: { map: DEMO_MAP_SCANNING, meaning: DEMO_MEANING_SCANNING, places: null },
});

type ByCommand = {
  readonly answers: Readonly<Record<string, unknown>>;
  readonly busy: Refusal;
  readonly cancelScan: string;
  // What a place the scan has no answer for shows.
  readonly emptyPlace: PlaceView;
  readonly eventAt: string;
  readonly getFindings: string;
  readonly getOverview: string;
  readonly getStorageMap: string;
  readonly getStorageMeaning: string;
  readonly getStoragePlace: string;
  readonly hold: null | number;
  readonly hooks: typeof CLEANUP_HOOKS;
  readonly ignoreHook: string;
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
  readonly storage: { readonly scanned: StorageAnswers; readonly scanning: StorageAnswers };
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

// `refresh_access` answers what `get_access_status` does, unless the script answers it.
const withRefreshAccess = <T>(
  answers: Partial<Readonly<Record<CommandKey, T>>> = {},
): Partial<Readonly<Record<CommandKey, T>>> =>
  answers.getAccessStatus === undefined || 'refreshAccess' in answers
    ? answers
    : { ...answers, refreshAccess: answers.getAccessStatus };

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
    answers: byCommand(withRefreshAccess(script.answers)),
    busy: BUSY,
    cancelScan: COMMANDS.cancelScan,
    emptyPlace: EMPTY_PLACE,
    eventAt: EVENT_AT,
    getFindings: COMMANDS.getFindings,
    getOverview: COMMANDS.getOverview,
    getStorageMap: COMMANDS.getStorageMap,
    getStorageMeaning: COMMANDS.getStorageMeaning,
    getStoragePlace: COMMANDS.getStoragePlace,
    hold: script.hold ?? null,
    hooks: CLEANUP_HOOKS,
    ignoreHook: IGNORE_HOOK,
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
    storage: storageOfScan(script.scanned ?? null, script.meaning, script.storage),
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
    // The places of the last scan by their node's index; `null` while a scan runs, so they are refused as busy.
    let places: StorageAnswers['places'] = {};

    const send = (events: readonly Event[]): void => {
      const deliver = channel === undefined ? undefined : callbacks.get(channel);

      for (const message of events) {
        const index = sent;

        sent += 1;
        streamed += message.type === 'findings-added' ? message.findings.length : 0;
        setTimeout(() => deliver?.({ index, message }), 0);
      }
    };

    // The storage map's answers of a phase of the scan, as the engine's would change.
    const storageOf = (storage: StorageAnswers): void => {
      answers[core.getStorageMap] = storage.map;
      answers[core.getStorageMeaning] = storage.meaning;
      places = storage.places;
    };

    const finish = (events: readonly Event[], overview: null | Overview): void => {
      held = [];
      running = false;
      if (overview !== null) {
        answers[core.getOverview] = overview;
      }
      storageOf(core.storage.scanned);
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
      storageOf(core.storage.scanning);
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

    // A place of the last scan by its node's index, unless the script answers it; refused as busy while a scan runs.
    const placeOf = (args: Readonly<Record<string, unknown>>): Promise<unknown> => {
      const { index } = args.node as { readonly index: number };

      if (core.getStoragePlace in core.answers) {
        return Promise.resolve(answers[core.getStoragePlace]);
      }
      if (places === null) {
        // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
        return Promise.reject(core.busy);
      }

      return Promise.resolve(places[String(index)] ?? core.emptyPlace);
    };

    const invoke = (command: string, args: Readonly<Record<string, unknown>>): Promise<unknown> => {
      const cleanup = (
        globalThis as unknown as Record<
          string,
          ((command: string, args: unknown) => Promise<unknown> | undefined) | undefined
        >
      )[core.hooks.invoke];
      const ignoring = (
        globalThis as unknown as Record<
          string,
          | ((command: string, args: unknown, answers: Record<string, unknown>) => Promise<unknown> | undefined)
          | undefined
        >
      )[core.ignoreHook];
      const answered =
        command in core.refusals ? undefined : (cleanup?.(command, args) ?? ignoring?.(command, args, answers));

      if (answered !== undefined) {
        return answered;
      }
      if (command === core.getStoragePlace && !(command in core.refusals)) {
        return placeOf(args);
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
  if (script.ignoring === true) {
    await mockIgnore(page);
  }
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
  appInfo: { build: { kind: 'release' }, version: '0.0.1' },
  cancelScan: null,
  getAccessStatus: GRANTED,
  getDictionary: { ...SCRIPTS_DICTIONARY, ...STORAGE_DICTIONARY },
  getModules: DEMO_MODULES,
  getOverview: IDLE,
  getSettings: DEMO_SETTINGS,
  getStorageMap: DEMO_MAP_NO_SCAN,
  getStorageMeaning: DEMO_MEANING_SCANNING,
  startScan: 1,
  updateSetting: null,
} as const;

const LARGE = largeScan({ categories: 3, count: 1000 });

// The demo scan's texts with the cleanup's.
const CLEANING_ANSWERS = { ...ANSWERS, getDictionary: { ...ANSWERS.getDictionary, ...CLEANUP_DICTIONARY } };

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
    choices: [],
    group: 'debug',
    id: 'demo',
    settings: ['demo.enabled'],
  },
];

// The texts of the Ignored tabs the demo's own dictionary does not have.
const IGNORING_DICTIONARY = {
  'core.group.debug.description': 'The demo module and whatever else exists only to try the app.',
  'core.ignore.groups.description': 'Hides everything the modules of a group find and never cleans it.',
} as const;

// The cleanup of WebStorm's three findings (the mock makes the core's plan): what is left is DataGrip's data, and the
// history holds that one cleanup.
const WEBSTORM_CLEANUP = movedDetailOf(planOf(JETBRAINS_FINDINGS.slice(0, 3)), 3, 1_735_819_200);

const JETBRAINS_CLEANUP: CleanupScript = {
  after: {
    ...JETBRAINS_OVERVIEW,
    found: { allocated: 420_000_000, completeness: 'complete', logical: 420_000_000, reclaimable: 420_000_000 },
    totals: JETBRAINS_OVERVIEW.totals.filter(({ category }) => category === 'jetbrains.uninstalled'),
  },
  details: [WEBSTORM_CLEANUP],
  history: { blocked: null, cleanups: [WEBSTORM_CLEANUP.summary], offset: 0, total: 1, waiting: 1_325_000_000 },
};

// What the JetBrains scripts answer before a scan: the module, its texts and settings, the ignore tabs' texts.
const JETBRAINS_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...JETBRAINS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY, ...IGNORING_DICTIONARY },
  getModules: JETBRAINS_MODULES,
  getSettings: JETBRAINS_SETTINGS,
  updateSetting: null,
} as const;

// The cleanup of the six developer cache findings: nothing is left, and the history holds that one cleanup.
const DEV_CACHES_CLEANUP_DETAIL = movedDetailOf(planOf(DEV_CACHES_FINDINGS), 3, 1_735_819_200);

const DEV_CACHES_CLEANUP: CleanupScript = {
  after: {
    ...DEV_CACHES_OVERVIEW,
    found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
    totals: [],
  },
  details: [DEV_CACHES_CLEANUP_DETAIL],
  history: {
    blocked: null,
    cleanups: [DEV_CACHES_CLEANUP_DETAIL.summary],
    offset: 0,
    total: 1,
    waiting: 17_200_000_000,
  },
};

// What the developer caches scripts answer before a scan: the module, its texts and settings.
const DEV_CACHES_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...DEV_CACHES_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: DEV_CACHES_MODULES,
  getSettings: DEV_CACHES_SETTINGS,
  updateSetting: null,
} as const;

// The cleanup of the caches and logs of the removed app (what Keep settings selects): the data and settings stay.
const APP_LEFTOVERS_CLEANUP_DETAIL = movedDetailOf(planOf(APP_LEFTOVERS_FINDINGS.slice(0, 2)), 3, 1_735_819_200);

const APP_LEFTOVERS_CLEANUP: CleanupScript = {
  after: {
    ...APP_LEFTOVERS_OVERVIEW,
    found: { allocated: 2_130_000_000, completeness: 'complete', logical: 2_130_000_000, reclaimable: 2_130_000_000 },
    totals: APP_LEFTOVERS_OVERVIEW.totals.filter(({ category }) => category !== 'app-leftovers.caches'),
  },
  details: [APP_LEFTOVERS_CLEANUP_DETAIL],
  history: {
    blocked: null,
    cleanups: [APP_LEFTOVERS_CLEANUP_DETAIL.summary],
    offset: 0,
    total: 1,
    waiting: 664_000_000,
  },
};

// What the removed apps scripts answer before a scan: the module, its texts and settings.
const APP_LEFTOVERS_ANSWERS = {
  ...ANSWERS,
  getDictionary: {
    ...APP_LEFTOVERS_DICTIONARY,
    ...DEV_CACHES_DICTIONARY,
    ...STORAGE_DICTIONARY,
    ...CLEANUP_DICTIONARY,
  },
  getModules: APP_LEFTOVERS_MODULES,
  getSettings: APP_LEFTOVERS_SETTINGS,
  updateSetting: null,
} as const;

const sizedOf = (bytes: number): Overview['found'] => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// The cleanup of the findings `ids`: the categories in `gone` leave the totals, `waiting` bytes wait in the Trash.
const downloadsCleanupOf = (ids: readonly number[], waiting: number, gone: readonly string[]): CleanupScript => {
  const detail = movedDetailOf(planOf(DOWNLOADS_FINDINGS.filter(({ id }) => ids.includes(id))), 3, 1_735_819_200);

  return {
    after: {
      ...DOWNLOADS_OVERVIEW,
      found: sizedOf(15_795_000_000 - waiting),
      totals: DOWNLOADS_TOTALS.filter(({ category }) => !gone.includes(category)),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting },
  };
};

// What the downloads scripts answer before a scan: the module, its texts and settings, Downloads allowed.
const DOWNLOADS_ANSWERS = {
  ...ANSWERS,
  getAccessStatus: DOWNLOADS_GRANTED,
  getDictionary: { ...DOWNLOADS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: DOWNLOADS_MODULES,
  getSettings: DOWNLOADS_SETTINGS,
  updateSetting: null,
} as const;

// Two modules at once: npm's cache of the developer caches module beside the removed app's findings.
const [NPM_CACHE] = DEV_CACHES_FINDINGS as readonly [Finding];
const WITH_DEV_CACHES_FINDINGS: readonly Finding[] = [NPM_CACHE, ...APP_LEFTOVERS_FINDINGS];
const [DEV_CACHES_MODULE] = DEV_CACHES_MODULES as readonly [ModuleInfo];
const [APP_LEFTOVERS_MODULE] = APP_LEFTOVERS_MODULES as readonly [ModuleInfo];

// The cleanup of the five temporary findings: nothing is left, and the history holds that one cleanup.
const TEMP_CLEANUP_DETAIL = movedDetailOf(planOf(TEMP_FINDINGS), 3, 1_735_819_200);

const TEMP_CLEANUP: CleanupScript = {
  after: {
    ...TEMP_OVERVIEW,
    found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
    totals: [],
  },
  details: [TEMP_CLEANUP_DETAIL],
  history: { blocked: null, cleanups: [TEMP_CLEANUP_DETAIL.summary], offset: 0, total: 1, waiting: 837_000_000 },
};

// What the temporary files scripts answer before a scan: the module, its texts and settings, the ignore tabs' texts.
const TEMP_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...TEMP_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: TEMP_MODULES,
  getSettings: TEMP_SETTINGS,
  updateSetting: null,
} as const;

// The scripts the flows and the screenshots play.
export const SCRIPTS = {
  // The removed apps' leftovers scan, streamed at once, with cleanup, ignoring and its storage meaning.
  appLeftovers: {
    answers: APP_LEFTOVERS_ANSWERS,
    cleanup: APP_LEFTOVERS_CLEANUP,
    ignoring: true,
    meaning: APP_LEFTOVERS_MEANING,
    scan: APP_LEFTOVERS_SCAN,
    scanned: APP_LEFTOVERS_OVERVIEW,
  },
  // The same beside the developer caches module, which found npm's cache: a choice must leave its selection alone.
  appLeftoversWithDevCaches: {
    answers: { ...APP_LEFTOVERS_ANSWERS, getModules: [DEV_CACHES_MODULE, APP_LEFTOVERS_MODULE] },
    scan: scanOf(WITH_DEV_CACHES_FINDINGS, ['dev-caches', 'app-leftovers']),
    scanned: {
      ...APP_LEFTOVERS_OVERVIEW,
      found: sizedOf(11_384_000_000),
      totals: [
        { category: 'dev-caches.npm', count: 1, highestDanger: 'low', sizes: sizedOf(8_900_000_000) },
        ...APP_LEFTOVERS_TOTALS,
      ],
    },
  },
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
  // Full Disk Access refused and the scan limited by it: after the scan, the overview shows the coverage notices.
  denied: {
    answers: {
      ...ANSWERS,
      getAccessStatus: DENIED,
      getDictionary: { ...ANSWERS.getDictionary, ...ACCESS_DICTIONARY },
    },
    scan: DEMO_SCAN.map((event) => (event.type === 'scan-coverage' ? { ...event, coverage: LIMITED_COVERAGE } : event)),
    scanned: DEMO_OVERVIEW,
  },
  // The developer caches scan with Xcode closed, streamed at once, with cleanup, ignoring and its storage meaning.
  devCaches: {
    answers: DEV_CACHES_ANSWERS,
    cleanup: DEV_CACHES_CLEANUP,
    ignoring: true,
    meaning: DEV_CACHES_MEANING,
    scan: DEV_CACHES_SCAN,
    scanned: DEV_CACHES_OVERVIEW,
  },
  // The same with Xcode open: the projects are left out, only the downloads are offered.
  devCachesXcodeOpen: {
    answers: DEV_CACHES_ANSWERS,
    ignoring: true,
    scan: DEV_CACHES_XCODE_OPEN_SCAN,
    scanned: DEV_CACHES_XCODE_OPEN_OVERVIEW,
  },
  // A development build on the real home folder: the banner says nothing can be removed outside a sandbox.
  development: { answers: { ...ANSWERS, appInfo: { build: { kind: 'development' }, version: '0.1.0' } } },
  // The downloads and installers scan, streamed at once; the cleanup moves the installers and the big file.
  downloads: {
    answers: DOWNLOADS_ANSWERS,
    cleanup: downloadsCleanupOf([304, 305, 307], 2_565_000_000, ['downloads.installers', 'downloads.large-untouched']),
    ignoring: true,
    meaning: DOWNLOADS_MEANING,
    scan: DOWNLOADS_SCAN,
    scanned: DOWNLOADS_OVERVIEW,
  },
  // Downloads not allowed yet: the module could not look, every category says what it needs.
  downloadsUnavailable: {
    answers: { ...DOWNLOADS_ANSWERS, getAccessStatus: DOWNLOADS_NOT_ASKED, getModules: DOWNLOADS_UNAVAILABLE_MODULES },
    scan: DOWNLOADS_UNAVAILABLE_SCAN,
    scanned: DOWNLOADS_UNAVAILABLE_OVERVIEW,
  },
  // The same; the cleanup moves the unfinished download and its placeholder.
  downloadsUnfinished: {
    answers: DOWNLOADS_ANSWERS,
    cleanup: downloadsCleanupOf([301, 302], 310_000_000, ['downloads.unfinished']),
    scan: DOWNLOADS_SCAN,
    scanned: DOWNLOADS_OVERVIEW,
  },
  // The demo module fails after its first batch, and the scan with it.
  failed: { answers: ANSWERS, scan: DEMO_FAILED_SCAN, scanned: DEMO_FAILED },
  // The demo scan, streamed at once, with modules and groups that can be ignored and restored.
  ignoring: {
    answers: {
      ...ANSWERS,
      getDictionary: { ...ANSWERS.getDictionary, ...IGNORING_DICTIONARY },
      updateSetting: null,
    },
    ignoring: true,
    scan: DEMO_SCAN,
    scanned: DEMO_OVERVIEW,
  },
  // The JetBrains module's scan, streamed at once, with cleanup, ignoring modules and groups, and its storage meaning.
  jetbrains: {
    answers: JETBRAINS_ANSWERS,
    cleanup: JETBRAINS_CLEANUP,
    ignoring: true,
    meaning: JETBRAINS_MEANING,
    scan: JETBRAINS_SCAN,
    scanned: JETBRAINS_OVERVIEW,
  },
  // A thousand findings in three categories, paged by the lists.
  large: {
    answers: { ...ANSWERS, getDictionary: { ...LARGE.dictionary, ...STORAGE_DICTIONARY }, getModules: LARGE_MODULES },
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
  // A development build in a sandbox: the banner names its folder and says its open apps are simulated.
  sandboxed: {
    answers: {
      ...ANSWERS,
      appInfo: {
        build: {
          kind: 'sandbox',
          path: '/Users/dev/sandboxes/trial',
          simulated: ['running'],
        },
        version: '0.1.0',
      },
    },
  },
  // Settings of every kind.
  settings: {
    answers: {
      ...ANSWERS,
      getOwnData: { budget: 50_000_000, total: 2_000_000 },
      getSettings: SETTINGS_OF_EVERY_KIND,
      pickFolder: '~/Music',
      resetSettings: null,
    },
  },
  // The temporary files module's scan, streamed at once, with cleanup, ignoring, and the two temporary folders as
  // places of the storage map.
  temp: {
    answers: TEMP_ANSWERS,
    cleanup: TEMP_CLEANUP,
    ignoring: true,
    scan: TEMP_SCAN,
    scanned: TEMP_OVERVIEW,
    storage: { map: TEMP_MAP, places: TEMP_PLACES },
  },
  // The demo scan held before its first event: the core runs it, the app does not know yet.
  unannounced: { answers: ANSWERS, hold: 0, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
} as const satisfies Readonly<Record<string, CoreScript>>;
