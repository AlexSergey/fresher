import type {
  AppId,
  CleanupId,
  CleanupPlanView,
  CleanupReport,
  Confirmation,
  Event,
  FindingId,
  ItemOutcome,
  Method,
  Refusal,
  Text,
} from '../ipc/bindings.generated';

// What the UI did about the cleanup, and the core's answers.
export type CleanupAction =
  // The confirmation was sent; the progress shows until the core names the cleanup.
  | { readonly app?: AppId; readonly origin: CleanupOrigin; readonly type: 'cleanup-preparing' }
  | { readonly cleanup: CleanupId; readonly type: 'cleanup-confirmed' }
  | { readonly method: Method; readonly type: 'cleanup-confirming' }
  | { readonly plan: CleanupPlanView; readonly type: 'cleanup-prepared' }
  // Cancel, Done, Review it again: nothing is shown any more.
  | { readonly refusal: Refusal; readonly type: 'cleanup-refused' }
  | { readonly type: 'cleanup-dismissed' }
  | { readonly type: 'cleanup-stopping' };

// Where the cleanup is, from the review screen's button to its result. `running.cleanup` is `null` from the confirm
// until the core names the cleanup; `plan` is the one the user confirmed (`null` for a cleanup this UI did not start).
export type CleanupState =
  | {
      readonly app: AppId | null;
      readonly kind: 'confirming';
      readonly origin: CleanupOrigin;
      readonly plan: CleanupPlanView;
    }
  | { readonly app: AppId | null; readonly kind: 'preparing'; readonly origin: CleanupOrigin }
  | { readonly app: AppId | null; readonly kind: 'refused'; readonly origin: CleanupOrigin; readonly refusal: Refusal }
  | {
      readonly cleanup: CleanupId;
      // The index of the first item moved to the Trash, which "Show in the Trash" reveals.
      readonly firstMoved: null | number;
      readonly kind: 'finished';
      readonly origin: CleanupOrigin;
      readonly problems: readonly ItemProblem[];
      readonly report: CleanupReport;
    }
  | {
      readonly cleanup: CleanupId | null;
      // The title of the item in hand: the next one of the plan.
      readonly current: null | Text;
      readonly done: number;
      readonly firstMoved: null | number;
      readonly kind: 'running';
      readonly method: Method;
      readonly origin: CleanupOrigin;
      readonly plan: CleanupPlanView | null;
      readonly problems: readonly ItemProblem[];
      // Stop was asked for: the cleanup ends after the item in hand.
      readonly stopping: boolean;
      readonly total: number;
    }
  | { readonly kind: 'idle' };

type CleanupEvent = Extract<Event, { readonly type: 'cleanup-finished' | 'cleanup-items' | 'cleanup-started' }>;

// What started the cleanup: the selection on Review, or the uninstall of one app from its page. Only a selection's
// cleanup empties the selection when it ends.
type CleanupOrigin = 'selection' | 'uninstall';

// An item of a cleanup that was not removed, with the title the plan showed it under (`null` for a cleanup this UI
// did not start) and why: what the result lists.
type ItemProblem = {
  readonly finding: FindingId;
  readonly outcome: Extract<ItemOutcome, { readonly kind: 'failed' | 'skipped' }>;
  readonly title: null | Text;
};

export const NO_CLEANUP: CleanupState = { kind: 'idle' };

// What the dialog showed, sent back as it is: the plan's own count, allocated bytes and highest level, the method
// the user chose and whether they ticked that a High item may be the only copy. Nothing is summed or judged here: the
// core checks the tick.
export const confirmationOf = (plan: CleanupPlanView, method: Method, highRiskUnderstood: boolean): Confirmation => ({
  allocated: plan.summary.sizes.allocated,
  count: plan.summary.count,
  highest: plan.summary.highest,
  highRiskUnderstood,
  method,
  plan: plan.plan,
});

const itemsOf = (plan: CleanupPlanView | null): CleanupPlanView['groups'][number]['items'] =>
  plan?.groups.flatMap((group) => group.items) ?? [];

const currentOf = (plan: CleanupPlanView | null, done: number): null | Text => itemsOf(plan)[done]?.title ?? null;

const problemsOf = (
  plan: CleanupPlanView | null,
  event: Extract<CleanupEvent, { readonly type: 'cleanup-items' }>,
): readonly ItemProblem[] =>
  event.items.flatMap(({ finding, outcome }) =>
    outcome.kind === 'failed' || outcome.kind === 'skipped'
      ? [{ finding, outcome, title: itemsOf(plan).find((item) => item.finding === finding)?.title ?? null }]
      : [],
  );

const reduceEvent = (state: CleanupState, event: CleanupEvent): CleanupState => {
  const running = state.kind === 'running' ? state : null;
  const ours = running !== null && running.cleanup === event.cleanup;

  switch (event.type) {
    case 'cleanup-finished':
      return ours
        ? {
            cleanup: event.cleanup,
            firstMoved: running.firstMoved,
            kind: 'finished',
            origin: running.origin,
            problems: running.problems,
            report: event.report,
          }
        : state;
    case 'cleanup-items':
      return ours
        ? {
            ...running,
            current: currentOf(running.plan, event.done),
            done: event.done,
            firstMoved: running.firstMoved ?? event.items.find((item) => item.outcome.kind === 'moved')?.index ?? null,
            problems: [...running.problems, ...problemsOf(running.plan, event)],
          }
        : state;
    case 'cleanup-started': {
      // The cleanup this UI confirmed, named by the core before or after its answer; else one the UI joins now.
      if (running !== null && (running.cleanup === null || ours)) {
        return { ...running, cleanup: event.cleanup, method: event.method, total: event.total };
      }

      return {
        cleanup: event.cleanup,
        current: null,
        done: 0,
        firstMoved: null,
        kind: 'running',
        method: event.method,
        origin: 'selection',
        plan: null,
        problems: [],
        stopping: false,
        total: event.total,
      };
    }
  }
};

export const reduceCleanup = (state: CleanupState, action: CleanupAction | CleanupEvent): CleanupState => {
  switch (action.type) {
    case 'cleanup-confirmed':
      return state.kind === 'running' && state.cleanup === null ? { ...state, cleanup: action.cleanup } : state;
    case 'cleanup-confirming':
      return state.kind === 'confirming'
        ? {
            cleanup: null,
            current: currentOf(state.plan, 0),
            done: 0,
            firstMoved: null,
            kind: 'running',
            method: action.method,
            origin: state.origin,
            plan: state.plan,
            problems: [],
            stopping: false,
            total: state.plan.summary.count,
          }
        : state;
    case 'cleanup-dismissed':
      return state.kind === 'running' ? state : NO_CLEANUP;
    case 'cleanup-finished':
    case 'cleanup-items':
    case 'cleanup-started':
      return reduceEvent(state, action);
    case 'cleanup-prepared':
      return state.kind === 'preparing'
        ? { app: state.app, kind: 'confirming', origin: state.origin, plan: action.plan }
        : state;
    case 'cleanup-preparing':
      return state.kind === 'running' ? state : { app: action.app ?? null, kind: 'preparing', origin: action.origin };
    case 'cleanup-refused':
      return {
        app: state.kind === 'preparing' || state.kind === 'confirming' ? state.app : null,
        kind: 'refused',
        origin: state.kind === 'idle' ? 'selection' : state.origin,
        refusal: action.refusal,
      };
    case 'cleanup-stopping':
      return state.kind === 'running' && !state.stopping ? { ...state, stopping: true } : state;
  }
};
