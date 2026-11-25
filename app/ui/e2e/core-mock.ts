import type { Page } from '@playwright/test';

import type {
  Coverage,
  Dictionary,
  Event,
  Finding,
  FindingsQuery,
  HighLevel,
  MeaningGroup,
  ModuleInfo,
  Overview,
  Refusal,
  SettingUpdate,
  StorageMap,
} from '../src/ipc/bindings.generated';

import coreTexts from '../../../crates/core/locales/en.json' with { type: 'json' };
import { type CommandKey, COMMANDS } from '../src/ipc';
import { AGREEMENT_DICTIONARY } from '../src/test/agreement';
import {
  APP_CACHES_BROWSERS_DICTIONARY,
  APP_CACHES_BROWSERS_FINDINGS,
  APP_CACHES_BROWSERS_OVERVIEW,
  APP_CACHES_BROWSERS_SCAN,
  APP_CACHES_BROWSERS_TOTALS,
} from '../src/test/app-caches-browsers-scan';
import {
  APP_CACHES_DICTIONARY,
  APP_CACHES_FINDINGS,
  APP_CACHES_MODULES,
  APP_CACHES_OVERVIEW,
  APP_CACHES_SCAN,
  APP_CACHES_SETTINGS,
  APP_CACHES_TOTALS,
} from '../src/test/app-caches-scan';
import { uninstallPlanOf, VIDEO_STUDIO_DETAIL } from '../src/test/app-detail';
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
import { APPS_DICTIONARY } from '../src/test/apps-list';
import {
  CLEANUP_DICTIONARY,
  DELETED_DETAIL,
  emptiedDetailOf,
  HISTORY,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
  movedDetailOf,
  planOf,
} from '../src/test/cleanup-plan';
import {
  ACCESS_DICTIONARY,
  accessStatus,
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
  DOCKER_DICTIONARY,
  DOCKER_MODULES,
  DOCKER_OVERVIEW,
  DOCKER_SCAN,
  DOCKER_SETTINGS,
  DOCKER_STOPPED_SCAN,
} from '../src/test/docker-scan';
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
import { type LargeScan, largeScan, scanningOverviewOf } from '../src/test/large-scan';
import { LARGEST_DICTIONARY, LARGEST_ROWS, ownFindingOf } from '../src/test/largest-scan';
import {
  LOCAL_AI_MODELS_DICTIONARY,
  LOCAL_AI_MODELS_FINDINGS,
  LOCAL_AI_MODELS_LM_STUDIO_OPEN_OVERVIEW,
  LOCAL_AI_MODELS_LM_STUDIO_OPEN_SCAN,
  LOCAL_AI_MODELS_MODULES,
  LOCAL_AI_MODELS_OVERVIEW,
  LOCAL_AI_MODELS_SCAN,
  LOCAL_AI_MODELS_SETTINGS,
  LOCAL_AI_MODELS_TOTALS,
} from '../src/test/local-ai-models-scan';
import {
  LOGS_DICTIONARY,
  LOGS_FINDINGS,
  LOGS_MODULES,
  LOGS_OVERVIEW,
  LOGS_SCAN,
  LOGS_SETTINGS,
} from '../src/test/logs-scan';
import {
  OLLAMA_DOWNLOAD_OVERVIEW,
  OLLAMA_DOWNLOAD_SCAN,
  OLLAMA_FINDINGS,
  OLLAMA_OVERVIEW,
  OLLAMA_SCAN,
  OLLAMA_TOTALS,
} from '../src/test/ollama-scan';
import { pageOf } from '../src/test/page-of';
import { BUILD_DICTIONARY, SANDBOX_HOME, sandboxBuild } from '../src/test/sandbox-build';
import {
  STALE_PROJECTS_DICTIONARY,
  STALE_PROJECTS_FINDINGS,
  STALE_PROJECTS_MODULES,
  STALE_PROJECTS_OVERVIEW,
  STALE_PROJECTS_SCAN,
  STALE_PROJECTS_SETTINGS,
  STALE_PROJECTS_TOTALS,
} from '../src/test/stale-projects-scan';
import {
  DEMO_MAP_COMPLETE,
  DEMO_MAP_NO_SCAN,
  DEMO_MAP_SCANNING,
  DEMO_MEANING_SCANNING,
  meaningFor,
  STORAGE_DICTIONARY,
} from '../src/test/storage-map';
import {
  TEMP_DICTIONARY,
  TEMP_FINDINGS,
  TEMP_MAP,
  TEMP_MODULES,
  TEMP_OVERVIEW,
  TEMP_SCAN,
  TEMP_SETTINGS,
} from '../src/test/temp-scan';
import {
  TRASH_AFTER_CLEANUP,
  TRASH_NOT_MEASURED,
  TRASH_ONE,
  TRASH_ONE_DATED,
  TRASH_TWO,
  UNKNOWN_TRASH,
} from '../src/test/trash-view';
import {
  UNUSED_TOOLCHAINS_DEFAULT_SETTINGS,
  UNUSED_TOOLCHAINS_DICTIONARY,
  UNUSED_TOOLCHAINS_FINDINGS,
  UNUSED_TOOLCHAINS_MODULES,
  UNUSED_TOOLCHAINS_NO_DOCUMENTS_MODULES,
  UNUSED_TOOLCHAINS_NO_DOCUMENTS_OVERVIEW,
  UNUSED_TOOLCHAINS_NO_DOCUMENTS_SCAN,
  UNUSED_TOOLCHAINS_NOT_ASKED,
  UNUSED_TOOLCHAINS_OVERVIEW,
  UNUSED_TOOLCHAINS_SCAN,
  UNUSED_TOOLCHAINS_SETTINGS,
  UNUSED_TOOLCHAINS_TOTALS,
} from '../src/test/unused-toolchains-scan';
import {
  XCODE_DICTIONARY,
  XCODE_FINDINGS,
  XCODE_MODULES,
  XCODE_OVERVIEW,
  XCODE_SCAN,
  XCODE_SETTINGS,
  XCODE_TOTALS,
} from '../src/test/xcode-scan';
import { AGREEMENT_HOOK, type AgreementScript, mockAgreement } from './support/agreement-mock';
import { APPS_HOOK, mockApps } from './support/apps-mock';
import { CLEANUP_HOOKS, type CleanupScript, mockCleanup } from './support/cleanup-mock';
import { IGNORE_HOOK, IGNORED_PATHS_HOOK, mockIgnore } from './support/ignore-mock';
import { LARGEST_HOOK, mockLargest, ownIdOf } from './support/largest-mock';

export { withheld } from './support/withheld';

// What the mocked core does: an answer per command key, a refusal per command key, the events a scan streams, and
// the overview `get_overview` answers once the scan is asked for (until then, its answer in `answers`).
export type CoreScript = {
  // The agreement's commands (support/agreement-mock.ts): accepted by default; `not-accepted` is the world before the
  // first cleanup.
  readonly agreement?: AgreementScript;
  readonly answers?: Partial<Readonly<Record<CommandKey, unknown>>>;
  // The Apps screen's commands: the list, an app's page and Show in Finder (support/apps-mock.ts); `denied` is the world
  // after macOS refused to let Fresher move an app.
  readonly apps?: { readonly denied?: boolean; readonly measuring?: boolean; readonly withheld?: boolean };
  // The cleanup commands over the scan's findings (support/cleanup-mock.ts); without it they are not answered.
  readonly cleanup?: CleanupScript;
  // Whether the app info says the core offers High findings: `offered` by default, as the core's tests run it.
  readonly high?: HighLevel;
  // The scan streams this many events, then waits for `releaseScan` (or Stop): the live states can be looked at.
  readonly hold?: number;
  // The ignore commands of modules and groups, kept in the tab's session storage (support/ignore-mock.ts).
  readonly ignoring?: boolean;
  // The scan streams one event every `interval` ms, as the engine does, each asked of Node as JSON and parsed in the
  // page as the channel does (the page never holds the whole scan); meanwhile `get_overview` answers the totals of the findings streamed so far, and `scanned` from the last event on.
  readonly interval?: number;
  // The Largest screen's commands: the Files list, the home's folders and the pick of a row (support/largest-mock.ts).
  readonly largest?: boolean;
  // The by-meaning groups once the scan ended, in place of the demo volume's; the volume's own lines follow them.
  readonly meaning?: readonly MeaningGroup[];
  readonly refusals?: Partial<Readonly<Record<CommandKey, Refusal>>>;
  readonly scan?: readonly Event[];
  readonly scanned?: Overview;
  // What `get_overview` answers while a held scan waits.
  readonly scanning?: Overview;
  readonly stop?: { readonly events: readonly Event[]; readonly overview: Overview };
  // What `cancel_scan` streams in place of the held events, and the overview after it.
  // The storage map after the scan, in place of the demo volume's.
  readonly storage?: { readonly map: StorageMap };
};

// What the storage map's commands answer in one phase of the scan.
type StorageAnswers = {
  readonly map: StorageMap;
  readonly meaning: readonly MeaningGroup[];
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
  },
  scanning: { map: DEMO_MAP_SCANNING, meaning: DEMO_MEANING_SCANNING },
});

type ByCommand = {
  readonly agreementHook: string;
  readonly answers: Readonly<Record<string, unknown>>;
  readonly appInfo: string;
  readonly appsHook: string;
  readonly busy: Refusal;
  readonly cancelScan: string;
  readonly eventAt: string;
  readonly getFindings: string;
  readonly getOverview: string;
  readonly getStorageMap: string;
  readonly getStorageMeaning: string;
  readonly getTrash: string;
  readonly hold: null | number;
  readonly hooks: typeof CLEANUP_HOOKS;
  readonly ignoredPathsHook: string;
  readonly ignoreHook: string;
  readonly interval: null | number;
  readonly largestHook: string;
  readonly openTrash: string;
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
  readonly unknownTrash: Refusal;
  readonly updateSetting: string;
};

// The page's name of the function that answers `get_findings` from the script's findings, in Node.
const PAGE_OF = '__fresherPageOf';

// A finding is ignored when its path is an ignored path or lies inside one.
const isIgnored = ({ subject }: Finding, ignored: readonly string[]): boolean => {
  if (subject.kind !== 'path') {
    return false;
  }
  const { value } = subject;

  return ignored.some((path) => value === path || value.startsWith(`${path}/`));
};

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

// The danger levels' names, meanings and group names, as the core's dictionary has them: Review's tiles and summary
// read them.
const DANGER_TEXTS: Dictionary = Object.fromEntries(
  Object.entries(coreTexts as Dictionary).filter(([key]) => key.startsWith('core.danger.')),
);

// The core's dictionary also holds the agreement's texts and the danger levels', whatever the script's own dictionary
// is.
const withCoreTexts = <T>(
  answers: Partial<Readonly<Record<CommandKey, T>>> = {},
): Partial<Readonly<Record<CommandKey, T>>> =>
  typeof answers.getDictionary === 'object' && answers.getDictionary !== null
    ? { ...answers, getDictionary: { ...AGREEMENT_DICTIONARY, ...DANGER_TEXTS, ...answers.getDictionary } }
    : answers;

// The app info answers the script's `high`, when it states one.
const withHigh = <T>(
  answers: Partial<Readonly<Record<CommandKey, T>>> = {},
  high: HighLevel | undefined,
): Partial<Readonly<Record<CommandKey, T>>> =>
  high === undefined || typeof answers.appInfo !== 'object' || answers.appInfo === null
    ? answers
    : { ...answers, appInfo: { ...answers.appInfo, high } };

const byCommand = <T>(table: Partial<Readonly<Record<CommandKey, T>>> = {}): Readonly<Record<string, T>> =>
  Object.fromEntries((Object.entries(table) as [CommandKey, T][]).map(([key, value]) => [COMMANDS[key], value]));

// The Rust core replaced in the browser, by the command keys of src/ipc/index.ts, injected before the page loads.
// `@tauri-apps/api` calls `window.__TAURI_INTERNALS__`: `invoke` for a command, `transformCallback` for a channel.
// `subscribe` keeps the channel; `start_scan`, when answered, then streams `scan` through it (up to `hold` events), and
// `get_overview` answers `scanning` while the scan is held and `scanned` from its end on. A second `start_scan` while a
// held scan runs is refused as busy, as the engine does; `cancel_scan` ends a held scan with `stop`. An answered
// `update_setting` sends its `setting-changed` event. `get_findings`, unless the script answers or refuses it, pages
// through the findings streamed so far by module, category, order, offset and limit, as the engine does. A scan paced
// by `interval` streams its events one by one and is busy until its last. With `ignoring`, `get_findings` leaves out a
// finding whose path is ignored or inside an ignored path, and its total with it; the overview's category totals are not
// recomputed for an ignored item (a simplification: the core's own e2e proves them).
export const mockCore = async (page: Page, script: CoreScript = {}): Promise<void> => {
  const findings: Finding[] = (script.scan ?? []).flatMap((event) =>
    event.type === 'findings-added' ? event.findings : [],
  );
  const setup: ByCommand = {
    agreementHook: AGREEMENT_HOOK,
    answers: byCommand(withHigh(withCoreTexts(withRefreshAccess(script.answers)), script.high)),
    appInfo: COMMANDS.appInfo,
    appsHook: APPS_HOOK,
    busy: BUSY,
    cancelScan: COMMANDS.cancelScan,
    eventAt: EVENT_AT,
    getFindings: COMMANDS.getFindings,
    getOverview: COMMANDS.getOverview,
    getStorageMap: COMMANDS.getStorageMap,
    getStorageMeaning: COMMANDS.getStorageMeaning,
    getTrash: COMMANDS.getTrash,
    hold: script.hold ?? null,
    hooks: CLEANUP_HOOKS,
    ignoredPathsHook: IGNORED_PATHS_HOOK,
    ignoreHook: IGNORE_HOOK,
    interval: script.interval ?? null,
    largestHook: LARGEST_HOOK,
    openTrash: COMMANDS.openTrash,
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
    unknownTrash: UNKNOWN_TRASH,
    updateSetting: COMMANDS.updateSetting,
  };

  await page.exposeFunction(PAGE_OF, (query: FindingsQuery, streamed: number, ignored: readonly string[]) =>
    pageOf(
      findings.slice(0, streamed).filter((finding) => !isIgnored(finding, ignored)),
      query,
    ),
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
    // The Trash lines the last `get_trash` answered, and the Trashes `open_trash` was asked to open.
    let listed = 0;
    const opened: number[] = [];

    Object.assign(globalThis, { __openedTrash: opened });

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

    // `get_trash` answers the script's view (none by default); `open_trash` records the index, or refuses one the last
    // answer did not have.
    const trashOf = (command: string, args: Readonly<Record<string, unknown>>): Promise<unknown> | undefined => {
      if (command !== core.getTrash && command !== core.openTrash) {
        return undefined;
      }
      if (command === core.getTrash) {
        const view = (answers[command] ?? { trashes: [] }) as { readonly trashes: readonly unknown[] };

        listed = view.trashes.length;

        return Promise.resolve(view);
      }
      const index = args.index as number;

      if (!Number.isInteger(index) || index < 0 || index >= listed) {
        // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects a plain object
        return Promise.reject({ ...core.unknownTrash, error: { code: 'unknown-trash', index } });
      }
      opened.push(index);

      return Promise.resolve(null);
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
      const largest = (
        globalThis as unknown as Record<
          string,
          ((command: string, args: unknown) => Promise<unknown> | undefined) | undefined
        >
      )[core.largestHook];
      const apps = (
        globalThis as unknown as Record<
          string,
          ((command: string, args: unknown) => Promise<unknown> | undefined) | undefined
        >
      )[core.appsHook];
      const agreement = (
        globalThis as unknown as Record<
          string,
          ((command: string, args: unknown) => Promise<unknown> | undefined) | undefined
        >
      )[core.agreementHook];
      const answered =
        command in core.refusals
          ? undefined
          : (agreement?.(command, args) ??
            cleanup?.(command, args) ??
            ignoring?.(command, args, answers) ??
            largest?.(command, args) ??
            apps?.(command, args) ??
            trashOf(command, args));

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
          globalThis as unknown as Record<
            string,
            (query: FindingsQuery, streamed: number, ignored: readonly string[]) => Promise<unknown>
          >
        )[core.pageOf];
        const ignoredPaths = (globalThis as unknown as Record<string, (() => readonly string[]) | undefined>)[
          core.ignoredPathsHook
        ];

        return findingsOf === undefined
          ? Promise.resolve({ findings: [], offset: query.offset, total: 0 })
          : findingsOf(query, streamed, ignoredPaths?.() ?? []);
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

    // The last scan finished two minutes before the page's clock reads it, so Scan says "2 minutes ago" on any day.
    const recently = (answer: unknown): unknown => {
      const lastScan = (answer as null | undefined | { readonly lastScan?: unknown })?.lastScan;

      return typeof lastScan === 'object' && lastScan !== null
        ? { ...(answer as object), lastScan: { ...lastScan, finished: Math.floor(Date.now() / 1000) - 120 } }
        : answer;
    };

    // The core's clock is the page's: the app info's moment is read when the page asks.
    const onThePageClock = (answer: unknown): unknown =>
      typeof answer === 'object' && answer !== null ? { ...answer, now: Math.floor(Date.now() / 1000) } : answer;

    const timed = (command: string, answer: Promise<unknown>): Promise<unknown> => {
      if (command === core.getOverview) {
        return answer.then(recently);
      }

      return command === core.appInfo ? answer.then(onThePageClock) : answer;
    };

    let nextCallback = 1;
    Object.defineProperty(globalThis, '__TAURI_INTERNALS__', {
      value: {
        invoke: (command: string, args: Readonly<Record<string, unknown>>): Promise<unknown> =>
          timed(command, invoke(command, args)),
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
    await mockIgnore(page, findings);
  }
  const moved = script.largest === true ? await mockLargest(page, findings) : undefined;

  await mockAgreement(page, script.agreement);

  if (script.apps !== undefined) {
    await mockApps(page, script.apps);
  }

  if (script.cleanup !== undefined) {
    await mockCleanup(page, findings, script.cleanup, moved);
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
  appInfo: { build: { kind: 'release' }, high: 'offered', home: '/Users/someone', version: '0.0.1' },
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

const DESKTOP = { kind: 'folder', value: 'desktop' } as const;

const LARGE = largeScan({ categories: 3, count: 1000 });

// The core's texts of the Largest screen, of its refusal and of the Desktop the scan did not enter, with the cleanup's.
const LARGEST_SCREEN_DICTIONARY: Dictionary = {
  ...CLEANUP_DICTIONARY,
  ...LARGEST_DICTIONARY,
  ...Object.fromEntries(
    Object.entries(coreTexts as Dictionary).filter(
      ([key]) =>
        key.startsWith('core.access.desktop.') ||
        key.startsWith('core.access.all-files.') ||
        key === 'core.error.not-own-item',
    ),
  ),
};

// The demo scan whose coverage did not enter the Desktop (not asked yet).
const LARGEST_COVERAGE: Coverage = { notEntered: [{ access: 'not-asked', capability: DESKTOP }], unreadable: [] };

const LARGEST_SCAN: readonly Event[] = DEMO_SCAN.map((event) =>
  event.type === 'scan-coverage' ? { ...event, coverage: LARGEST_COVERAGE } : event,
);

const LARGEST_SCANNED: NonNullable<Overview['lastScan']> = {
  ...(DEMO_OVERVIEW.lastScan as NonNullable<Overview['lastScan']>),
  coverage: LARGEST_COVERAGE,
};

const LARGEST_OVERVIEW: Overview = { ...DEMO_OVERVIEW, lastScan: LARGEST_SCANNED };

const LARGEST_ANSWERS = {
  ...ANSWERS,
  getAccessStatus: accessStatus([
    [{ kind: 'all-files' }, 'granted'],
    [DESKTOP, 'not-asked'],
  ]),
  getDictionary: { ...ANSWERS.getDictionary, ...LARGEST_SCREEN_DICTIONARY },
};

// The cleanup of the first row of the Largest list when the user picks it: the disk image goes to the Trash (a cleanup
// of own files does not touch the scan's totals, but it changed the disk, so the list says to scan again).
const largestCleanup = (): CleanupScript => {
  const [row] = LARGEST_ROWS;
  const plan = planOf([
    ownFindingOf(row as (typeof LARGEST_ROWS)[number], ownIdOf((row as (typeof LARGEST_ROWS)[number]).node.id)),
  ]);
  const detail = movedDetailOf(plan, 3, 1_735_819_200);

  return {
    after: { ...LARGEST_OVERVIEW, lastScan: { ...LARGEST_SCANNED, changedSince: 3 } },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 12_000_000_000 },
  };
};

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

// The demo module with the generated categories of a large scan.
const modulesOf = (scan: LargeScan): readonly ModuleInfo[] => [
  {
    categories: scan.overview.totals.map(({ category }) => readyCategory(category, 'safe')),
    choices: [],
    group: 'debug',
    id: 'demo',
    settings: ['demo.enabled'],
  },
];

// Fourteen categories of three findings: more finding lists at once than the ten pages the UI keeps.
const WIDE = largeScan({ categories: 14, count: 42 });

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

// The cleanup of the old-projects findings `ids`: the categories in `gone` leave the totals, `waiting` bytes wait.
const staleProjectsCleanupOf = (ids: readonly number[], waiting: number, gone: readonly string[]): CleanupScript => {
  const detail = movedDetailOf(planOf(STALE_PROJECTS_FINDINGS.filter(({ id }) => ids.includes(id))), 3, 1_735_819_200);

  return {
    after: {
      ...STALE_PROJECTS_OVERVIEW,
      found: sizedOf(6_570_000_000 - waiting),
      totals: STALE_PROJECTS_TOTALS.filter(({ category }) => !gone.includes(category)),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting },
  };
};

// What the old-projects script answers before a scan: the module, its texts and settings.
const STALE_PROJECTS_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...STALE_PROJECTS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: STALE_PROJECTS_MODULES,
  getSettings: STALE_PROJECTS_SETTINGS,
  updateSetting: null,
} as const;

// The engine removes every emptied folder from the results, even one that kept an entry, so none of Editor's two stays.
// The cleanup of Editor's two folders: the first is emptied into the Trash (two entries went), the second held nothing
// that could go, so nothing of it is in the Trash. One item stays, 64 MB.
const appCachesCleanup = (): CleanupScript => {
  const plan = planOf(APP_CACHES_FINDINGS.filter(({ id }) => id === 701 || id === 702));
  const detail = emptiedDetailOf(plan, 3, 1_735_819_200, [
    { moved: 2, skipped: 0 },
    { moved: 0, skipped: 1 },
  ]);

  return {
    after: {
      ...APP_CACHES_OVERVIEW,
      found: sizedOf(64_000_000),
      totals: APP_CACHES_TOTALS.map((total) => ({ ...total, count: 1, sizes: sizedOf(64_000_000) })),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 920_000_000 },
    outcomes: [
      { kind: 'emptied', moved: 2, skipped: 0 },
      { kind: 'emptied', moved: 0, skipped: 1 },
    ],
  };
};

// The cleanup of the browser caches' first two folders (Chrome's Default Cache and Code Cache): each is emptied into
// the Trash, but Code Cache held nothing that could go, so nothing of it is in the Trash. Three items stay, 115 MB.
const appCachesBrowsersCleanup = (): CleanupScript => {
  const plan = planOf(APP_CACHES_BROWSERS_FINDINGS.filter(({ id }) => id === 801 || id === 802));
  const detail = emptiedDetailOf(plan, 3, 1_735_819_200, [
    { moved: 4, skipped: 0 },
    { moved: 0, skipped: 0 },
  ]);

  return {
    after: {
      ...APP_CACHES_BROWSERS_OVERVIEW,
      found: sizedOf(115_000_000),
      totals: APP_CACHES_BROWSERS_TOTALS.map((total) =>
        total.category === 'app-caches.browsers' ? { ...total, count: 1, sizes: sizedOf(90_000_000) } : total,
      ),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 400_000_000 },
    outcomes: [
      { kind: 'emptied', moved: 4, skipped: 0 },
      { kind: 'emptied', moved: 0, skipped: 0 },
    ],
  };
};

// The cleanup of the previews, the simulator caches and the archive: the previews are emptied into the Trash (the
// folder stays), the simulator caches had nothing left to move (the folder emptied itself between the scan and the
// cleanup; a file held open would skip the item as in use instead), the archive is moved. Only the device support is
// left, 7.3 GB; 1.5 GB wait in the Trash. The mock states the Trash line alone, without a matching cleanup history.
const xcodeCleanup = (): CleanupScript => {
  const plan = planOf(XCODE_FINDINGS.filter(({ id }) => id >= 803));
  const detail = emptiedDetailOf(plan, 3, 1_735_819_200, [
    { moved: 4, skipped: 0 },
    { moved: 0, skipped: 0 },
  ]);

  return {
    after: {
      ...XCODE_OVERVIEW,
      found: sizedOf(7_300_000_000),
      totals: XCODE_TOTALS.filter(({ category }) => category === 'xcode.device-support'),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 1_500_000_000 },
    outcomes: [{ kind: 'emptied', moved: 4, skipped: 0 }, { kind: 'emptied', moved: 0, skipped: 0 }, { kind: 'moved' }],
  };
};

// What the Xcode script answers before a scan: the module, its texts and settings.
const XCODE_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...XCODE_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: XCODE_MODULES,
  getSettings: XCODE_SETTINGS,
  updateSetting: null,
} as const;

// What the docker scripts answer before a scan: Xcode's module, the docker module and their texts and settings.
const DOCKER_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...DOCKER_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: DOCKER_MODULES,
  getSettings: DOCKER_SETTINGS,
  updateSetting: null,
} as const;

// What the app caches script answers before a scan: the module, its texts and settings.
const APP_CACHES_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...APP_CACHES_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: APP_CACHES_MODULES,
  getSettings: APP_CACHES_SETTINGS,
  updateSetting: null,
} as const;

// The caches scan with the Apps screen over it: the list, an app's page and the uninstall of Video Studio, whose
// category of cleanable caches is the module's own. The history already holds that uninstall, keep-data, in the Trash.
const APPS_ANSWERS = {
  ...APP_CACHES_ANSWERS,
  getDictionary: { ...APP_CACHES_ANSWERS.getDictionary, ...APPS_DICTIONARY },
} as const;

const appsAccess = (management: 'denied' | 'not-asked'): ReturnType<typeof accessStatus> =>
  accessStatus([
    [{ kind: 'all-files' }, 'granted'],
    [{ kind: 'app-management' }, management],
  ]);

const appsCleanup = (): CleanupScript => {
  const detail = movedDetailOf(uninstallPlanOf(VIDEO_STUDIO_DETAIL, 'keep-data'), 3, 1_735_819_200);

  return {
    after: {
      ...APP_CACHES_OVERVIEW,
      lastScan: APP_CACHES_OVERVIEW.lastScan === null ? null : { ...APP_CACHES_OVERVIEW.lastScan, changedSince: 3 },
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 3_472_000_000 },
  };
};

// What the browser caches script answers before a scan: the module with its three categories, its texts, settings.
const APP_CACHES_BROWSERS_ANSWERS = {
  ...APP_CACHES_ANSWERS,
  getDictionary: { ...APP_CACHES_BROWSERS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
} as const;

// The cleanup of the first local AI model (`meta/tiny`): the Hugging Face models stay with one model, 2 GB wait.
const localAiModelsCleanup = (): CleanupScript => {
  const detail = movedDetailOf(planOf(LOCAL_AI_MODELS_FINDINGS.filter(({ id }) => id === 501)), 3, 1_735_819_200);

  return {
    after: {
      ...LOCAL_AI_MODELS_OVERVIEW,
      found: sizedOf(8_448_000_000 - 2_000_000_000),
      totals: LOCAL_AI_MODELS_TOTALS.map((total) =>
        total.category === 'local-ai-models.hugging-face-models'
          ? { ...total, count: 1, sizes: sizedOf(548_000_000) }
          : total,
      ),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 2_000_000_000 },
  };
};

// The cleanup of the first Ollama model: one item, its parts go together, 4.7 GB wait.
const ollamaCleanup = (): CleanupScript => {
  const detail = movedDetailOf(planOf(OLLAMA_FINDINGS.filter(({ id }) => id === 511)), 3, 1_735_819_200);

  return {
    after: {
      ...OLLAMA_OVERVIEW,
      found: sizedOf(10_900_000_000 - 4_700_000_000),
      totals: OLLAMA_TOTALS.map((total) => ({ ...total, count: 2, sizes: sizedOf(6_200_000_000) })),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 4_700_000_000 },
  };
};

// What the local AI models scripts answer before a scan: the module, its texts and settings.
const LOCAL_AI_MODELS_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...LOCAL_AI_MODELS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: LOCAL_AI_MODELS_MODULES,
  getSettings: LOCAL_AI_MODELS_SETTINGS,
  updateSetting: null,
} as const;

// The cleanup of the first Rust toolchain: the Rust toolchains stay with one, 1.2 GB wait.
const unusedToolchainsCleanup = (): CleanupScript => {
  const detail = movedDetailOf(planOf(UNUSED_TOOLCHAINS_FINDINGS.filter(({ id }) => id === 601)), 3, 1_735_819_200);

  return {
    after: {
      ...UNUSED_TOOLCHAINS_OVERVIEW,
      found: sizedOf(UNUSED_TOOLCHAINS_OVERVIEW.found.logical - 1_200_000_000),
      totals: UNUSED_TOOLCHAINS_TOTALS.map((total) =>
        total.category === 'unused-toolchains.rust' ? { ...total, count: 1, sizes: sizedOf(1_500_000_000) } : total,
      ),
    },
    details: [detail],
    history: { blocked: null, cleanups: [detail.summary], offset: 0, total: 1, waiting: 1_200_000_000 },
  };
};

// What the unused toolchains scripts answer before a scan: the module, its texts and settings.
const UNUSED_TOOLCHAINS_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...UNUSED_TOOLCHAINS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: UNUSED_TOOLCHAINS_MODULES,
  getSettings: UNUSED_TOOLCHAINS_SETTINGS,
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

// The cleanup of the two old logs: nothing is left, and the history holds that one cleanup.
const LOGS_CLEANUP_DETAIL = movedDetailOf(planOf(LOGS_FINDINGS.slice(0, 2)), 3, 1_735_819_200);

const LOGS_CLEANUP: CleanupScript = {
  after: {
    ...LOGS_OVERVIEW,
    found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
    totals: [],
  },
  details: [LOGS_CLEANUP_DETAIL],
  history: { blocked: null, cleanups: [LOGS_CLEANUP_DETAIL.summary], offset: 0, total: 1, waiting: 44_000_000 },
};

// What the logs scripts answer before a scan: the module, its texts and settings.
const LOGS_ANSWERS = {
  ...ANSWERS,
  getDictionary: { ...LOGS_DICTIONARY, ...STORAGE_DICTIONARY, ...CLEANUP_DICTIONARY },
  getModules: LOGS_MODULES,
  getSettings: LOGS_SETTINGS,
  updateSetting: null,
} as const;

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
  // The installed app caches scan, streamed at once; the cleanup empties Editor's two folders.
  appCaches: {
    answers: APP_CACHES_ANSWERS,
    cleanup: appCachesCleanup(),
    ignoring: true,
    scan: APP_CACHES_SCAN,
    scanned: APP_CACHES_OVERVIEW,
  },
  // The browser and web-engine caches scan, streamed at once; the cleanup empties Chrome's Default caches.
  appCachesBrowsers: {
    answers: APP_CACHES_BROWSERS_ANSWERS,
    cleanup: appCachesBrowsersCleanup(),
    ignoring: true,
    scan: APP_CACHES_BROWSERS_SCAN,
    scanned: APP_CACHES_BROWSERS_OVERVIEW,
  },
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
  // The caches scan with the Apps screen: five apps, the uninstall of Video Studio and its history entry.
  apps: {
    answers: { ...APPS_ANSWERS, getAccessStatus: appsAccess('not-asked') },
    apps: {},
    cleanup: appsCleanup(),
    ignoring: true,
    scan: APP_CACHES_SCAN,
    scanned: APP_CACHES_OVERVIEW,
  },
  // The same world after macOS refused to let Fresher move an app: Video Studio's page says so, and asks for the
  // permission.
  appsDenied: {
    answers: { ...APPS_ANSWERS, getAccessStatus: appsAccess('denied') },
    apps: { denied: true },
    cleanup: appsCleanup(),
    ignoring: true,
    scan: APP_CACHES_SCAN,
    scanned: APP_CACHES_OVERVIEW,
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
  development: {
    answers: {
      ...ANSWERS,
      appInfo: { build: { kind: 'development' }, high: 'offered', home: '/Users/someone', version: '0.1.0' },
    },
  },
  // Xcode's scan with the docker module describing OrbStack (answered, four parts, three commands) and offering nothing.
  docker: {
    answers: DOCKER_ANSWERS,
    cleanup: xcodeCleanup(),
    ignoring: true,
    scan: DOCKER_SCAN,
    scanned: DOCKER_OVERVIEW,
  },
  // The same held after the docker module finished: the scan still runs, so the section is not shown yet.
  dockerHeld: {
    answers: DOCKER_ANSWERS,
    hold: 5,
    scan: DOCKER_SCAN,
    scanned: DOCKER_OVERVIEW,
    scanning: { ...DOCKER_OVERVIEW, lastScan: null, status: { scan: 1, state: 'scanning' } },
  },
  // The same with Docker Desktop not running: its card says to start it, its screen lists no part.
  dockerStopped: {
    answers: DOCKER_ANSWERS,
    ignoring: true,
    scan: DOCKER_STOPPED_SCAN,
    scanned: DOCKER_OVERVIEW,
  },
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
    answers: {
      ...ANSWERS,
      getDictionary: { ...LARGE.dictionary, ...STORAGE_DICTIONARY },
      getModules: modulesOf(LARGE),
    },
    scan: LARGE.events,
    scanned: LARGE.overview,
  },
  // The demo scan with the Desktop not entered, and the Largest screen over it: the list, the home's folders, the pick
  // of a row. The cleanup moves the disk image, the first row, and the history holds it.
  largest: {
    answers: LARGEST_ANSWERS,
    cleanup: largestCleanup(),
    largest: true,
    scan: LARGEST_SCAN,
    scanned: LARGEST_OVERVIEW,
  },
  // The demo scan held after its first batch and progress, until `releaseScan` or Stop (stopped, the first batch).
  live: {
    answers: ANSWERS,
    hold: LIVE_HOLD,
    scan: DEMO_LIVE_SCAN,
    scanned: DEMO_OVERVIEW,
    scanning: DEMO_SCANNING,
    stop: { events: DEMO_STOP, overview: DEMO_STOPPED },
  },
  // The local AI models scan, streamed at once; the cleanup moves a Hugging Face model.
  localAiModels: {
    answers: LOCAL_AI_MODELS_ANSWERS,
    cleanup: localAiModelsCleanup(),
    ignoring: true,
    scan: LOCAL_AI_MODELS_SCAN,
    scanned: LOCAL_AI_MODELS_OVERVIEW,
  },
  // The same with LM Studio open: its model is left out, only the Hugging Face ones are offered.
  localAiModelsLmStudioOpen: {
    answers: LOCAL_AI_MODELS_ANSWERS,
    ignoring: true,
    scan: LOCAL_AI_MODELS_LM_STUDIO_OPEN_SCAN,
    scanned: LOCAL_AI_MODELS_LM_STUDIO_OPEN_OVERVIEW,
  },
  // The local AI models scan with Ollama's three models offered; the cleanup moves the first one.
  localAiModelsOllama: {
    answers: LOCAL_AI_MODELS_ANSWERS,
    cleanup: ollamaCleanup(),
    ignoring: true,
    scan: OLLAMA_SCAN,
    scanned: OLLAMA_OVERVIEW,
  },
  // Ollama is downloading a model: its store is left out as a whole, nothing of it is offered.
  localAiModelsOllamaDownload: {
    answers: LOCAL_AI_MODELS_ANSWERS,
    ignoring: true,
    scan: OLLAMA_DOWNLOAD_SCAN,
    scanned: OLLAMA_DOWNLOAD_OVERVIEW,
  },
  // The logs module's scan, streamed at once, with cleanup and ignoring.
  logs: {
    answers: LOGS_ANSWERS,
    cleanup: LOGS_CLEANUP,
    ignoring: true,
    scan: LOGS_SCAN,
    scanned: LOGS_OVERVIEW,
  },
  // Some folders could not be read: the scan ends partial, the caches' size a lower bound.
  partial: { answers: ANSWERS, scan: DEMO_PARTIAL_SCAN, scanned: DEMO_PARTIAL },
  // Settings of every kind, in a file a newer version of the app saved: none can be changed.
  readOnly: { answers: { ...ANSWERS, getSettings: SETTINGS_READ_ONLY } },
  // The core refuses every scan as busy.
  refused: { answers: ANSWERS, refusals: { startScan: BUSY } },
  // A development build in a sandbox: the banner names its folder and its Trash, and says its open apps are simulated.
  sandboxed: {
    answers: {
      ...ANSWERS,
      appInfo: { build: sandboxBuild(['running']), high: 'offered', home: SANDBOX_HOME, version: '0.1.0' },
      getDictionary: { ...ANSWERS.getDictionary, ...BUILD_DICTIONARY },
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
  // The old-projects scan, streamed at once; the cleanup moves the Python environment and the build output.
  staleProjects: {
    answers: STALE_PROJECTS_ANSWERS,
    cleanup: staleProjectsCleanupOf([403, 404], 3_720_000_000, [
      'stale-projects.build-output',
      'stale-projects.python-environments',
    ]),
    ignoring: true,
    scan: STALE_PROJECTS_SCAN,
    scanned: STALE_PROJECTS_OVERVIEW,
  },
  // The temporary files module's scan, streamed at once, with cleanup, ignoring, and the two temporary folders as
  // places of the storage map's volume.
  temp: {
    answers: TEMP_ANSWERS,
    cleanup: TEMP_CLEANUP,
    ignoring: true,
    scan: TEMP_SCAN,
    scanned: TEMP_OVERVIEW,
    storage: { map: TEMP_MAP },
  },
  // The demo scan; the Trash holds 23 GB, 20 GB of it from earlier cleanups.
  trash: { answers: { ...ANSWERS, getTrash: TRASH_ONE }, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // The same, with a cleanup of the demo plan: nothing waits before it, its share does after it.
  trashCleaning: {
    answers: { ...CLEANING_ANSWERS, getTrash: { trashes: [] } },
    cleanup: { ...CLEANUP, trashAfterCleanup: TRASH_AFTER_CLEANUP },
    scan: DEMO_SCAN,
    scanned: DEMO_OVERVIEW,
  },
  // The same with Finder's 30-day setting: the share's date is shown.
  trashDated: { answers: { ...ANSWERS, getTrash: TRASH_ONE_DATED }, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // The same, the Trash not measured: only the share of the cleanups is known.
  trashNotMeasured: {
    answers: { ...ANSWERS, getTrash: TRASH_NOT_MEASURED },
    scan: DEMO_SCAN,
    scanned: DEMO_OVERVIEW,
  },
  // Two Trashes, the second volume's not measured.
  trashTwo: { answers: { ...ANSWERS, getTrash: TRASH_TWO }, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // The demo scan held before its first event: the core runs it, the app does not know yet.
  unannounced: { answers: ANSWERS, hold: 0, scan: DEMO_SCAN, scanned: DEMO_OVERVIEW },
  // The unused toolchains scan, streamed at once, with the Node switch on; the cleanup moves a Rust toolchain.
  unusedToolchains: {
    answers: UNUSED_TOOLCHAINS_ANSWERS,
    cleanup: unusedToolchainsCleanup(),
    ignoring: true,
    scan: UNUSED_TOOLCHAINS_SCAN,
    scanned: UNUSED_TOOLCHAINS_OVERVIEW,
  },
  // Documents not allowed yet, the Node switch at its default, off: the module could not look.
  unusedToolchainsNoDocuments: {
    answers: {
      ...UNUSED_TOOLCHAINS_ANSWERS,
      getAccessStatus: UNUSED_TOOLCHAINS_NOT_ASKED,
      getModules: UNUSED_TOOLCHAINS_NO_DOCUMENTS_MODULES,
      getSettings: UNUSED_TOOLCHAINS_DEFAULT_SETTINGS,
    },
    scan: UNUSED_TOOLCHAINS_NO_DOCUMENTS_SCAN,
    scanned: UNUSED_TOOLCHAINS_NO_DOCUMENTS_OVERVIEW,
  },
  // Fourteen categories of three findings each, in the groups of their findings.
  wide: {
    answers: { ...ANSWERS, getDictionary: { ...WIDE.dictionary, ...STORAGE_DICTIONARY }, getModules: modulesOf(WIDE) },
    scan: WIDE.events,
    scanned: WIDE.overview,
  },
  // The Xcode scan, streamed at once; the cleanup empties the two caches and moves the archive.
  xcode: {
    answers: XCODE_ANSWERS,
    cleanup: xcodeCleanup(),
    ignoring: true,
    scan: XCODE_SCAN,
    scanned: XCODE_OVERVIEW,
  },
} as const satisfies Readonly<Record<string, CoreScript>>;
