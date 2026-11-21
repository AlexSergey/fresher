import type {
  ActionKind,
  CleanupDetail,
  CleanupId,
  CleanupItemView,
  CleanupPlanView,
  CleanupReport,
  CleanupsPage,
  CleanupSummary,
  DangerLevel,
  Dictionary,
  Ended,
  Event,
  Finding,
  ItemOutcome,
  ItemReport,
  LeftOut,
  Method,
  OutcomeCounts,
  PlanSummary,
  Refusal,
  Sizes,
  SkipReason,
} from '../ipc/bindings.generated';

import { DEMO_FINDINGS } from './demo-scan';
import { EXTERNAL_FINDING, findingOf, HIGH_FINDING, LONG_PATH_FINDING, MEDIUM_FINDING } from './finding-fixtures';

// Cleanup plans, cleanups and their history as the core sends them, built from findings: shared by the unit tests,
// the stories and the e2e mock. The sums here stand in for the core's; the UI never makes them.

const LEVELS: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

const highestOf = (levels: readonly DangerLevel[]): DangerLevel =>
  levels.reduce<DangerLevel>(
    (highest, level) => (LEVELS.indexOf(level) > LEVELS.indexOf(highest) ? level : highest),
    'safe',
  );

const sumOf = (all: readonly Sizes[]): Sizes => {
  const bytes = all.reduce((sum, sizes) => sum + sizes.allocated, 0);
  const completeness = all.some((sizes) => sizes.completeness !== 'complete') ? 'partial' : 'complete';

  return { allocated: bytes, completeness, logical: bytes, reclaimable: bytes };
};

const actionOf = (finding: Finding): ActionKind => {
  switch (finding.removal.kind) {
    case 'empty-folder':
      return 'empty';
    case 'move-to-trash':
    case 'move-together':
      return 'remove';
    case 'tool-command':
      return 'tool';
  }
};

const summaryOf = (findings: readonly Finding[]): PlanSummary => ({
  count: findings.length,
  highest: highestOf(findings.map((finding) => finding.danger)),
  irreversible: findings.filter((finding) => actionOf(finding) === 'tool').length,
  sizes: sumOf(findings.map((finding) => finding.sizes)),
});

// A finding of the selection the plan leaves out, with its reason.
export const leftOutOf = (finding: Finding, reason: SkipReason): LeftOut => ({
  finding: finding.id,
  reason,
  title: finding.title,
});

// The plan the core answers for `findings`: grouped by category and by how they go, in the order they come.
export const planOf = (
  findings: readonly Finding[],
  { leftOut = [], plan = 1 }: { readonly leftOut?: readonly LeftOut[]; readonly plan?: number } = {},
): CleanupPlanView => {
  const keys = [...new Set(findings.map((finding) => `${finding.category} ${actionOf(finding)}`))];

  return {
    groups: keys.map((key) => {
      const own = findings.filter((finding) => `${finding.category} ${actionOf(finding)}` === key);
      const [first] = own as [Finding];

      return {
        action: actionOf(first),
        category: first.category,
        items: own.map((finding) => ({
          danger: finding.danger,
          finding: finding.id,
          more: finding.removal.kind === 'move-together' ? finding.removal.value.slice(1) : [],
          path: finding.subject.kind === 'path' ? finding.subject.value : null,
          reason: finding.reason,
          sizes: finding.sizes,
          title: finding.title,
        })),
        summary: summaryOf(own),
      };
    }),
    leftOut: [...leftOut],
    permanentAvailable: findings.some((finding) => actionOf(finding) !== 'tool'),
    plan,
    summary: summaryOf(findings),
  };
};

// The demo scan's three findings: two caches (Safe) and an installer (Low), 2.57 GB.
export const DEMO_PLAN: CleanupPlanView = planOf(DEMO_FINDINGS);

// One cache (Safe), 1.5 GB.
export const ONE_ITEM_PLAN: CleanupPlanView = planOf(DEMO_FINDINGS.slice(0, 1));

// The demo findings with a medium, a long path and a high one.
export const HIGH_PLAN: CleanupPlanView = planOf([...DEMO_FINDINGS, MEDIUM_FINDING, LONG_PATH_FINDING, HIGH_FINDING]);

// Two of the demo findings; the browser cache is left out because an app uses it.
export const LEFT_OUT_PLAN: CleanupPlanView = planOf(DEMO_FINDINGS.slice(0, 2), {
  leftOut: [leftOutOf(DEMO_FINDINGS[2] as Finding, { app: 'Safari', kind: 'in-use' })],
});

// Every item left out: nothing to confirm.
export const ALL_LEFT_OUT_PLAN: CleanupPlanView = {
  ...planOf([], {
    leftOut: [
      leftOutOf(DEMO_FINDINGS[0] as Finding, { app: 'Xcode', kind: 'in-use' }),
      leftOutOf(DEMO_FINDINGS[2] as Finding, { kind: 'vanished' }),
    ],
  }),
  permanentAvailable: false,
};

// A cache and a Docker image: the image is removed by its tool and cannot be put back.
export const TOOL_PLAN: CleanupPlanView = planOf([DEMO_FINDINGS[0] as Finding, EXTERNAL_FINDING]);

// Only a Docker image: the permanent choice does not apply.
export const TOOL_ONLY_PLAN: CleanupPlanView = planOf([EXTERNAL_FINDING]);

// 240 caches: more than the 50 a group lists without a virtual list.
export const LONG_PLAN: CleanupPlanView = planOf(
  Array.from({ length: 240 }, (_, at) =>
    findingOf({
      bytes: 4_000_000 + ((at * 7_919) % 90) * 1_000_000,
      id: 100 + at,
      level: 'safe',
      name: `cache-${String(at + 1)}`,
    }),
  ),
);

const MODEL_PATHS = [
  '/Users/someone/.ollama/models/manifests/registry.ollama.ai/library/demo/latest',
  '/Users/someone/.ollama/models/blobs/sha256-0a1b2c3d4e5f',
  '/Users/someone/.ollama/models/blobs/sha256-6f5e4d3c2b1a',
] as const;

// A model that is one item with three paths: the manifest, then its two layers.
export const TOGETHER_FINDING: Finding = {
  ...findingOf({ bytes: 4_700_000_000, id: 30, level: 'low', name: 'demo-model' }),
  removal: { kind: 'move-together', value: [...MODEL_PATHS] },
  subject: { kind: 'path', value: MODEL_PATHS[0] },
};

// A cache and a model of two layers: the model is one item that goes with two more files.
export const TOGETHER_PLAN: CleanupPlanView = planOf([DEMO_FINDINGS[0] as Finding, TOGETHER_FINDING]);

// The model among 60 caches: the group is long enough for a virtual list.
export const LONG_TOGETHER_PLAN: CleanupPlanView = planOf([
  TOGETHER_FINDING,
  ...Array.from({ length: 60 }, (_, at) => findingOf({ id: 200 + at, level: 'safe', name: `cache-${String(at)}` })),
]);

const refusal = (error: Refusal['error'], key: string, args: Refusal['text']['args'] = {}): Refusal => ({
  error,
  text: { args, key },
});

export const PLAN_GONE: Refusal = refusal({ code: 'plan-gone' }, 'core.error.plan-gone');

export const PLAN_MISMATCH: Refusal = refusal({ code: 'plan-mismatch' }, 'core.error.plan-mismatch');

export const CLEANING_BLOCKED: Refusal = refusal(
  { code: 'cleaning-blocked', reason: 'journal' },
  'core.error.cleaning-blocked',
);

export const TOO_MANY_ITEMS: Refusal = refusal({ code: 'too-many-items', limit: 10_000 }, 'core.error.too-many-items', {
  limit: { kind: 'count', value: 10_000 },
});

export const NOT_IN_TRASH: Refusal = refusal({ code: 'not-in-trash' }, 'core.error.not-in-trash');

const NO_COUNTS: OutcomeCounts = {
  deleted: 0,
  emptied: 0,
  failed: 0,
  moved: 0,
  notReached: 0,
  skipped: 0,
  toolDone: 0,
  unknown: 0,
};

const countKey = (outcome: ItemOutcome): keyof OutcomeCounts => {
  switch (outcome.kind) {
    case 'deleted':
    case 'emptied':
    case 'failed':
    case 'moved':
    case 'skipped':
      return outcome.kind;
    case 'not-reached':
      return 'notReached';
    case 'tool-done':
      return 'toolDone';
  }
};

const itemsOf = (plan: CleanupPlanView): CleanupPlanView['groups'][number]['items'] =>
  plan.groups.flatMap((group) => group.items);

// What a cleanup of `plan` did, item by item in the plan's order: `outcomes` (each item done its method's way when
// not given), how it ended and the space measured.
export const reportOf = (
  plan: CleanupPlanView,
  {
    ended = 'completed',
    measured = 0,
    method = 'trash',
    outcomes,
  }: {
    readonly ended?: Ended;
    readonly measured?: null | number;
    readonly method?: Method;
    readonly outcomes?: readonly ItemOutcome[];
  } = {},
): CleanupReport => {
  const items = itemsOf(plan);
  const done: ItemOutcome = method === 'trash' ? { kind: 'moved' } : { kind: 'deleted' };
  const all = items.map((_, at) => outcomes?.[at] ?? done);
  const counts = { ...NO_COUNTS };

  for (const outcome of all) {
    counts[countKey(outcome)] += 1;
  }

  // An emptied folder's contents count once, under the cleanup's method: moved to the Trash, or deleted.
  const sizesWhere = (kind: 'deleted' | 'moved'): Sizes =>
    sumOf(
      items
        .filter((_, at) => {
          const outcome = all[at]?.kind;

          return outcome === kind || (outcome === 'emptied' && kind === (method === 'trash' ? 'moved' : 'deleted'));
        })
        .map((item) => item.sizes),
    );

  return { counts, deleted: sizesWhere('deleted'), ended, measured, method, moved: sizesWhere('moved') };
};

// The events of a cleanup of `plan`: started, one `cleanup-items` per item, finished with the report.
export const cleanupEventsOf = (
  plan: CleanupPlanView,
  report: CleanupReport,
  { cleanup = 1, outcomes }: { readonly cleanup?: CleanupId; readonly outcomes?: readonly ItemOutcome[] } = {},
): { readonly finished: Event; readonly items: readonly Event[]; readonly started: Event } => {
  const done: ItemOutcome = report.method === 'trash' ? { kind: 'moved' } : { kind: 'deleted' };
  const items = itemsOf(plan).map((item, index): Event => {
    const reported: ItemReport = { finding: item.finding, index, outcome: outcomes?.[index] ?? done };

    return { cleanup, done: index + 1, items: [reported], type: 'cleanup-items' };
  });

  return {
    finished: { cleanup, report, type: 'cleanup-finished' },
    items,
    started: { cleanup, method: report.method, total: plan.summary.count, type: 'cleanup-started' },
  };
};

// The demo plan moved to the Trash; 0.1 GB measured on the disk.
export const MOVED_REPORT: CleanupReport = reportOf(DEMO_PLAN, { measured: 100_000_000 });

// The demo plan deleted permanently.
export const DELETED_REPORT: CleanupReport = reportOf(DEMO_PLAN, { measured: 2_400_000_000, method: 'permanent' });

// The browser cache in use, the installer moved, the editor cache vanished.
export const SKIPPED_OUTCOMES: readonly ItemOutcome[] = [
  { kind: 'skipped', reason: { kind: 'vanished' } },
  { kind: 'skipped', reason: { app: 'Safari', kind: 'in-use' } },
  { kind: 'moved' },
];

export const SKIPPED_REPORT: CleanupReport = reportOf(DEMO_PLAN, { measured: 0, outcomes: SKIPPED_OUTCOMES });

// Stopped after the first item.
export const STOPPED_OUTCOMES: readonly ItemOutcome[] = [
  { kind: 'moved' },
  { kind: 'not-reached' },
  { kind: 'not-reached' },
];

export const STOPPED_REPORT: CleanupReport = reportOf(DEMO_PLAN, {
  ended: 'cancelled',
  measured: 0,
  outcomes: STOPPED_OUTCOMES,
});

// The space on the disk could not be read.
export const UNMEASURED_REPORT: CleanupReport = reportOf(DEMO_PLAN, { measured: null });

// One cleanup of the history: `inTrash` of its items still in the Trash.
export const cleanupSummaryOf = (
  cleanup: CleanupId,
  report: CleanupReport,
  { inTrash = report.counts.moved, started = 1_735_646_400 }: { readonly inTrash?: number; readonly started?: number },
): CleanupSummary => ({
  cleanup,
  count: Object.values(report.counts).reduce((sum, count) => sum + count, 0),
  counts: report.counts,
  deleted: report.deleted,
  ended: report.ended,
  inTrash,
  method: report.method,
  moved: report.moved,
  started,
});

// The demo plan's items as the history keeps them after a move to the Trash.
const movedItems = (plan: CleanupPlanView): CleanupItemView[] =>
  plan.groups
    .flatMap((group) =>
      group.items.map((item): Omit<CleanupItemView, 'index'> => ({
        action: group.action,
        category: group.category,
        danger: item.danger,
        outcome: { kind: 'moved' },
        path: item.path,
        putBack: null,
        sizes: item.sizes,
        trash: 'in-trash',
      })),
    )
    .map((item, index) => ({ ...item, index }));

// A cleanup that moved every item of `plan` to the Trash, all still there.
export const movedDetailOf = (plan: CleanupPlanView, cleanup: CleanupId, started: number): CleanupDetail => ({
  items: movedItems(plan),
  summary: cleanupSummaryOf(cleanup, reportOf(plan, { measured: 100_000_000 }), { started }),
});

// A cleanup that emptied every folder of `plan` into the Trash, each with what went and what stayed: an item whose
// entries went is still in the Trash, one with none is not.
export const emptiedDetailOf = (
  plan: CleanupPlanView,
  cleanup: CleanupId,
  started: number,
  emptied: readonly { readonly moved: number; readonly skipped: number }[],
): CleanupDetail => {
  const outcomes = emptied.map((counts) => ({ kind: 'emptied' as const, ...counts }));
  const items = movedItems(plan).map((item, at): CleanupItemView => ({
    ...item,
    outcome: outcomes[at] ?? item.outcome,
    trash: emptied[at]?.moved === 0 ? null : 'in-trash',
  }));

  return {
    items,
    summary: cleanupSummaryOf(cleanup, reportOf(plan, { measured: 100_000_000, outcomes }), {
      inTrash: items.filter((item) => item.trash === 'in-trash').length,
      started,
    }),
  };
};

// Cleanup 3: the demo plan moved to the Trash, every item still there.
export const MOVED_DETAIL: CleanupDetail = {
  items: movedItems(DEMO_PLAN),
  summary: cleanupSummaryOf(3, MOVED_REPORT, { started: 1_735_819_200 }),
};

// Cleanup 2: the demo plan deleted permanently.
export const DELETED_DETAIL: CleanupDetail = {
  items: movedItems(DEMO_PLAN).map((item) => ({ ...item, outcome: { kind: 'deleted' }, trash: null })),
  summary: cleanupSummaryOf(2, DELETED_REPORT, { inTrash: 0, started: 1_735_732_800 }),
};

// Cleanup 1: Fresher quit during it; the first item moved, the others unknown.
export const INTERRUPTED_DETAIL: CleanupDetail = {
  items: movedItems(DEMO_PLAN).map((item, index) =>
    index === 0 ? item : { ...item, outcome: { kind: 'unknown' }, trash: null },
  ),
  summary: {
    ...cleanupSummaryOf(1, reportOf(ONE_ITEM_PLAN, { ended: 'interrupted' }), { inTrash: 1 }),
    count: DEMO_PLAN.summary.count,
    counts: { ...NO_COUNTS, moved: 1, unknown: 2 },
  },
};

// The history of the three cleanups, newest first: 2.57 GB of the first waiting in the Trash, 1.5 GB of the last.
export const HISTORY: CleanupsPage = {
  blocked: null,
  cleanups: [MOVED_DETAIL.summary, DELETED_DETAIL.summary, INTERRUPTED_DETAIL.summary],
  offset: 0,
  total: 3,
  waiting: 4_070_000_000,
};

// The core's texts of the cleanup, as its dictionary has them.
export const CLEANUP_DICTIONARY: Dictionary = {
  'core.cleanup.blocked.journal':
    'The cleanup history cannot be written, and Fresher removes nothing without a record. Restart or update the ' +
    'app, then try again.',
  'core.cleanup.ended.cancelled': 'Stopped by you. The items after that point were not touched.',
  'core.cleanup.ended.completed': 'Finished.',
  'core.cleanup.ended.interrupted': 'Interrupted: Fresher quit during this cleanup.',
  'core.cleanup.ended.stopped':
    'Stopped because the cleanup history could not be saved. The items after that point were not touched.',
  'core.cleanup.outcome.deleted': 'Deleted permanently.',
  'core.cleanup.outcome.emptied': 'Emptied. The folder itself stays.',
  'core.cleanup.outcome.moved': 'Moved to the Trash.',
  'core.cleanup.outcome.not-reached': 'Not removed: the cleanup stopped before this item.',
  'core.cleanup.outcome.unknown': 'Unknown: Fresher stopped during this cleanup. Check the Trash.',
  'core.cleanup.put-back.gone': 'It is no longer in the Trash, so it cannot be put back.',
  'core.cleanup.put-back.occupied':
    'Something is already at its old place, so it stayed in the Trash. Show it in the Trash to decide.',
  'core.cleanup.put-back.put-back': 'Put back where it was.',
  'core.cleanup.skip.in-use': 'An app is using it, so Fresher left it alone.',
  'core.cleanup.skip.in-use.how': 'Quit {app} and try again.',
  'core.cleanup.skip.vanished': 'It is no longer there.',
  'core.cleanup.trash.gone': 'No longer in the Trash.',
  'core.cleanup.trash.in-trash': 'In the Trash.',
  'core.cleanup.trash.replaced': 'Something else is in its place in the Trash.',
  'core.danger.high.name': 'High',
  'core.danger.medium.name': 'Medium',
  'core.error.cleaning-blocked': 'Fresher cannot remove anything right now, so nothing was removed.',
  'core.error.not-in-trash': 'This item is no longer in the Trash.',
  'core.error.plan-gone':
    'This list is out of date: it is older than 10 minutes, or the scan results changed since. Review it again.',
  'core.error.plan-mismatch':
    'What you confirmed is not the list Fresher prepared, so nothing was removed. Review the list again.',
  'core.error.too-many-items':
    'That is more than {limit} items, too many to review at once. Pick fewer, then try again.',
  'demo.images.name': 'Example images',
};
