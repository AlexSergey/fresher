import type { Failure } from '../ipc';
import type {
  ByteSize,
  CategoryId,
  Dictionary,
  EngineStatus,
  Event,
  Finding,
  ModuleId,
  ModuleOutcome,
  Overview,
  ScanId,
  ScanOutcome,
} from '../ipc/bindings.generated';

// The findings of one category, in the order the core sent them.
export type CategoryGroup = { readonly category: CategoryId; readonly findings: readonly Finding[] };

// What happened to the UI itself: answers of commands, next to the core's events.
export type CoreAction =
  | Event
  | { readonly dictionary: Dictionary; readonly type: 'dictionary-loaded' }
  | { readonly failure: Failure; readonly type: 'command-failed' }
  | { readonly overview: Overview; readonly type: 'overview-loaded' };

// What the UI knows of the core: the scan whose events it keeps, and everything that scan sent.
export type CoreState = {
  readonly dictionary: Dictionary | null;
  readonly failure: Failure | null;
  readonly groups: readonly CategoryGroup[];
  readonly modules: readonly { readonly module: ModuleId; readonly outcome: ModuleOutcome }[];
  readonly outcome: null | ScanOutcome;
  readonly progress: null | { readonly bytes: ByteSize; readonly entries: number };
  readonly scan: null | ScanId;
  // The newest scan the events told of, and whether it finished: what an overview answered later must not undo.
  readonly seen: null | SeenScan;
  readonly status: EngineStatus;
};

type ScanEvent = Extract<Event, { readonly scan: ScanId }>;

type SeenScan = { readonly finished: boolean; readonly scan: ScanId };

export const INITIAL_STATE: CoreState = {
  dictionary: null,
  failure: null,
  groups: [],
  modules: [],
  outcome: null,
  progress: null,
  scan: null,
  seen: null,
  status: { state: 'idle' },
};

// A batch is grouped in one pass, then each group it touches is copied once: a scan costs no more than its batches.
const addFindings = (groups: readonly CategoryGroup[], findings: readonly Finding[]): readonly CategoryGroup[] => {
  const batch = new Map<CategoryId, Finding[]>();

  for (const finding of findings) {
    const added = batch.get(finding.category);

    if (added === undefined) {
      batch.set(finding.category, [finding]);
    } else {
      added.push(finding);
    }
  }

  const grown = groups.map((group) => {
    const added = batch.get(group.category);

    if (added === undefined) {
      return group;
    }

    batch.delete(group.category);

    return { ...group, findings: [...group.findings, ...added] };
  });

  return [...grown, ...Array.from(batch, ([category, added]) => ({ category, findings: added }))];
};

const reduceEvent = (state: CoreState, event: Event): CoreState => {
  // Every event of a scan belongs to one scan: one the UI does not follow is ignored.
  const follows = (scan: ScanId): boolean => scan === state.scan;

  switch (event.type) {
    case 'access-changed':
    case 'setting-changed':
      // The settings and the permissions have their screens in plans 005 and 006.
      return state;
    case 'findings-added':
      return follows(event.scan) ? { ...state, groups: addFindings(state.groups, event.findings) } : state;
    case 'module-finished':
      return follows(event.scan)
        ? { ...state, modules: [...state.modules, { module: event.module, outcome: event.outcome }] }
        : state;
    case 'scan-finished':
      return follows(event.scan) ? { ...state, outcome: event.outcome, status: { state: 'idle' } } : state;
    case 'scan-progress':
      return follows(event.scan) ? { ...state, progress: { bytes: event.bytes, entries: event.entries } } : state;
    case 'scan-started':
      return {
        ...state,
        failure: null,
        groups: [],
        modules: [],
        outcome: null,
        progress: null,
        scan: event.scan,
        status: { scan: event.scan, state: 'scanning' },
      };
  }
};

const see = (seen: null | SeenScan, event: ScanEvent): null | SeenScan => {
  const finished = event.type === 'scan-finished';
  const known = seen !== null && (seen.scan > event.scan || (seen.scan === event.scan && (seen.finished || !finished)));

  return known ? seen : { finished, scan: event.scan };
};

// The overview is answered after the events are subscribed, so an event that came before it may be newer than it is.
const loadOverview = (state: CoreState, overview: Overview): CoreState => {
  const { seen } = state;

  if (overview.status.state === 'scanning') {
    const { scan } = overview.status;
    const outdated = seen !== null && (seen.scan > scan || (seen.scan === scan && seen.finished));

    return outdated ? state : { ...state, scan, status: overview.status };
  }

  // A scan the events started is running until its own `scan-finished` arrives.
  return seen !== null && !seen.finished ? state : { ...state, status: overview.status };
};

// The next state: a pure function of the state and one action.
export const reduce = (state: CoreState, action: CoreAction): CoreState => {
  switch (action.type) {
    case 'access-changed':
    case 'setting-changed':
      return reduceEvent(state, action);
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
  }
};
