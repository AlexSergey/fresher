import type { Failure } from '../ipc';
import type {
  AccessStatus,
  ByteSize,
  Dictionary,
  EngineStatus,
  Event,
  FindingId,
  LeftOutCount,
  ModuleId,
  ModuleOutcome,
  Overview,
  ScanId,
  ScanOutcome,
  SettingsView,
  Sizes,
  StorageMap,
} from '../ipc/bindings.generated';

import { type CleanupAction, type CleanupState, NO_CLEANUP, reduceCleanup } from './cleanup';
import { NOTHING_SELECTED, reduceSelection, type Selection, type SelectionAction } from './selection';

// What happened to the UI itself: answers of commands and what the user picked, next to the core's events.
export type CoreAction =
  | CleanupAction
  | Event
  | SelectionAction
  | { readonly dictionary: Dictionary; readonly type: 'dictionary-loaded' }
  | { readonly failure: Failure; readonly type: 'command-failed' }
  | { readonly overview: Overview; readonly type: 'overview-loaded' }
  | { readonly settings: SettingsView; readonly type: 'settings-loaded' }
  | { readonly status: AccessStatus; readonly type: 'access-loaded' }
  | { readonly storageMap: StorageMap; readonly type: 'storage-map-loaded' }
  | { readonly type: 'overview-requested' }
  // The quit question was answered, either way.
  | { readonly type: 'quit-answered' };

// What the UI knows of the core: the scan whose events it follows, what they told, and the core's totals. No finding
// is kept: the lists ask the core for pages (state/findings.ts), so the memory does not grow with the scan.
export type CoreState = {
  // The permissions the app has, as `get_access_status` answered and `access-changed` events keep them.
  readonly access: AccessStatus | null;
  // The cleanup from the review screen's button to its result (state/cleanup.ts).
  readonly cleanup: CleanupState;
  readonly dictionary: Dictionary | null;
  readonly failure: Failure | null;
  // The modules that finished in the followed scan, with how, and what each left out (counted by the engine).
  readonly modules: readonly {
    readonly leftOut: readonly LeftOutCount[];
    readonly module: ModuleId;
    readonly outcome: ModuleOutcome;
  }[];
  readonly outcome: null | ScanOutcome;
  // The totals per category, everything found and the last scan, as `get_overview` answered them.
  readonly overview: null | Overview;
  readonly progress: null | { readonly bytes: ByteSize; readonly entries: number };
  // The window was closed during a cleanup: the frame asks whether to stop after the item in hand and quit.
  readonly quitAsked: boolean;
  // The findings the last cleanup removed: the finding pages drop them (state/findings.ts).
  readonly removed: null | readonly FindingId[];
  readonly scan: null | ScanId;
  // The modules the followed scan runs, as its `scan-started` listed them (a module switched off is not among them);
  // `null` until a `scan-started` arrived, as for a scan the UI learned of from an overview only.
  readonly scanModules: null | readonly ModuleId[];
  // The newest scan the events told of, and whether it finished: what an overview answered later must not undo.
  readonly seen: null | SeenScan;
  // What the user picked for removal in this scan (state/selection.ts); nothing is picked when a scan starts.
  readonly selection: Selection;
  // Every setting and its value, as `get_settings` answered and `setting-changed` events keep them.
  readonly settings: null | SettingsView;
  // The events told of changes the overview does not count yet: the refresher (state/refresh.ts) asks for a new one.
  readonly stale: boolean;
  readonly status: EngineStatus;
  // The storage map as `get_storage_map` last answered: totals per volume, never rows. A new scan keeps it until the
  // first answer of that scan replaces it.
  readonly storageMap: null | StorageMap;
};

type ScanEvent = Extract<Event, { readonly scan: ScanId }>;

type SeenScan = { readonly finished: boolean; readonly scan: ScanId };

export const INITIAL_STATE: CoreState = {
  access: null,
  cleanup: NO_CLEANUP,
  dictionary: null,
  failure: null,
  modules: [],
  outcome: null,
  overview: null,
  progress: null,
  quitAsked: false,
  removed: null,
  scan: null,
  scanModules: null,
  seen: null,
  selection: NOTHING_SELECTED,
  settings: null,
  stale: false,
  status: { state: 'idle' },
  storageMap: null,
};

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

// The overview no longer counts everything: unchanged when it already did not, so a batch renders nothing.
const markStale = (state: CoreState): CoreState => (state.stale ? state : { ...state, stale: true });

const withCleanup = (state: CoreState, cleanup: CleanupState): CoreState =>
  cleanup === state.cleanup ? state : { ...state, cleanup };

const reduceEvent = (state: CoreState, event: Event): CoreState => {
  // Every event of a scan belongs to one scan: one the UI does not follow is ignored.
  const follows = (scan: ScanId): boolean => scan === state.scan;

  switch (event.type) {
    case 'access-changed':
      return { ...state, access: event.status };
    case 'cleanup-finished': {
      const cleanup = reduceCleanup(state.cleanup, event);

      // The cleanup is over: what was picked for it is no longer picked.
      return cleanup === state.cleanup ? state : { ...state, cleanup, quitAsked: false, selection: NOTHING_SELECTED };
    }
    case 'cleanup-items':
    case 'cleanup-started':
      return withCleanup(state, reduceCleanup(state.cleanup, event));
    case 'findings-added':
      return follows(event.scan) ? markStale(state) : state;
    case 'findings-removed':
      // The removed findings leave the lists and the totals: the overview is asked for again.
      return { ...markStale(state), removed: event.findings };
    case 'module-finished':
      return follows(event.scan)
        ? markStale({
            ...state,
            modules: [...state.modules, { leftOut: event.leftOut, module: event.module, outcome: event.outcome }],
          })
        : state;
    case 'quit-requested':
      return state.quitAsked ? state : { ...state, quitAsked: true };
    case 'scan-coverage':
      // The coverage of a scan has its notices in plan 006.
      return state;
    case 'scan-finished':
      // The overview is always asked for once more at the end, so the last totals are the finished scan's.
      return follows(event.scan) ? markStale({ ...state, outcome: event.outcome, status: { state: 'idle' } }) : state;
    case 'scan-progress':
      return follows(event.scan) ? { ...state, progress: { bytes: event.bytes, entries: event.entries } } : state;
    case 'scan-started': {
      const status: EngineStatus = { scan: event.scan, state: 'scanning' };

      // The core forgets the previous scan's findings when a scan starts: nothing is counted until the next overview.
      return {
        ...state,
        failure: null,
        modules: [],
        outcome: null,
        overview: state.overview === null ? null : { ...state.overview, found: NOTHING, status, totals: [] },
        progress: null,
        scan: event.scan,
        scanModules: event.modules,
        selection: NOTHING_SELECTED,
        stale: false,
        status,
      };
    }
    case 'setting-changed': {
      // A change may hide or show findings (the ignore lists) or mark the last scan: the overview is asked for again.
      const settings = state.settings === null ? null : changeSetting(state.settings, event);

      // The core drops the pending plan when an ignore list changes, so the selection goes with it.
      const selection = event.key.startsWith('core.ignore.') ? NOTHING_SELECTED : state.selection;

      return markStale({ ...state, selection, settings });
    }
  }
};

// A stored value's problem is gone once the setting holds a new value.
const changeSetting = (
  settings: SettingsView,
  { key, value }: Extract<Event, { readonly type: 'setting-changed' }>,
): SettingsView => ({
  ...settings,
  problems: settings.problems.filter((problem) => problem.key !== key),
  values: { ...settings.values, [key]: value },
});

const see = (seen: null | SeenScan, event: ScanEvent): null | SeenScan => {
  const finished = event.type === 'scan-finished';
  const known = seen !== null && (seen.scan > event.scan || (seen.scan === event.scan && (seen.finished || !finished)));

  return known ? seen : { finished, scan: event.scan };
};

// The overview is answered after the events, so an event that came before it may be newer than it is: an older
// overview changes nothing, neither the engine's status nor the totals.
const loadOverview = (state: CoreState, overview: Overview): CoreState => {
  const { seen } = state;
  const { status } = overview;

  if (status.state === 'scanning') {
    const outdated = seen !== null && (seen.scan > status.scan || (seen.scan === status.scan && seen.finished));

    return outdated ? state : { ...state, overview, scan: status.scan, status };
  }

  // A scan the events started is running until its own `scan-finished` arrives; an earlier scan's totals are old.
  const last = overview.lastScan?.scan;
  const outdated = seen !== null && (!seen.finished || last === undefined || last < seen.scan);

  return outdated ? state : { ...state, overview, status };
};

// The next state: a pure function of the state and one action.
export const reduce = (state: CoreState, action: CoreAction): CoreState => {
  switch (action.type) {
    case 'access-changed':
    case 'cleanup-finished':
    case 'cleanup-items':
    case 'cleanup-started':
    case 'findings-removed':
    case 'quit-requested':
    case 'scan-coverage':
    case 'setting-changed':
      return reduceEvent(state, action);
    case 'access-loaded':
      // An event told of the access before the answer came: the event is newer.
      return state.access === null ? { ...state, access: action.status } : state;
    case 'choose':
    case 'clear':
    case 'toggle-category':
    case 'toggle-finding': {
      const selection = reduceSelection(state.selection, action);

      return selection === state.selection ? state : { ...state, selection };
    }
    case 'cleanup-confirmed':
    case 'cleanup-confirming':
    case 'cleanup-dismissed':
    case 'cleanup-prepared':
    case 'cleanup-preparing':
    case 'cleanup-refused':
    case 'cleanup-stopping':
      return withCleanup(state, reduceCleanup(state.cleanup, action));
    case 'command-failed':
      return { ...state, failure: action.failure };
    case 'dictionary-loaded':
      return { ...state, dictionary: action.dictionary };
    case 'findings-added':
    case 'module-finished':
    case 'scan-finished':
    case 'scan-progress':
    case 'scan-started': {
      const seen = see(state.seen, action);

      return reduceEvent(seen === state.seen ? state : { ...state, seen }, action);
    }
    case 'overview-loaded':
      return loadOverview(state, action.overview);
    case 'overview-requested':
      // The request counts every event so far; an event after it marks the overview stale again.
      return state.stale ? { ...state, stale: false } : state;
    case 'quit-answered':
      return state.quitAsked ? { ...state, quitAsked: false } : state;
    case 'settings-loaded':
      // The settings are asked for once; the events since keep them, so a later answer is older than they are.
      return state.settings === null ? { ...state, settings: action.settings } : state;
    case 'storage-map-loaded':
      // The map is polled with the overview: an answer equal to the map held changes nothing, so nothing renders.
      return JSON.stringify(state.storageMap) === JSON.stringify(action.storageMap)
        ? state
        : { ...state, storageMap: action.storageMap };
  }
};
