import type {
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
  | { readonly cleanup: CleanupId; readonly type: 'cleanup-confirmed' }
  | { readonly method: Method; readonly type: 'cleanup-confirming' }
  | { readonly plan: CleanupPlanView; readonly type: 'cleanup-prepared' }
  | { readonly refusal: Refusal; readonly type: 'cleanup-refused' }
  // Cancel, Done, Review it again: nothing is shown any more.
  | { readonly type: 'cleanup-dismissed' }
  | { readonly type: 'cleanup-preparing' }
  | { readonly type: 'cleanup-stopping' };

// Where the cleanup is, from the review screen's button to its result. `running.cleanup` is `null` from the confirm
// until the core names the cleanup; `plan` is the one the user confirmed (`null` for a cleanup this UI did not start).
export type CleanupState =
  | {
      readonly cleanup: CleanupId;
      // The index of the first item moved to the Trash, which "Show in the Trash" reveals.
      readonly firstMoved: null | number;
      readonly kind: 'finished';
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
      readonly plan: CleanupPlanView | null;
      readonly problems: readonly ItemProblem[];
      // Stop was asked for: the cleanup ends after the item in hand.
      readonly stopping: boolean;
      readonly total: number;
    }
  | { readonly kind: 'confirming'; readonly plan: CleanupPlanView }
  | { readonly kind: 'idle' }
  | { readonly kind: 'preparing' }
  | { readonly kind: 'refused'; readonly refusal: Refusal };

type CleanupEvent = Extract<Event, { readonly type: 'cleanup-finished' | 'cleanup-items' | 'cleanup-started' }>;

// An item of a cleanup that was not removed, with the title the plan showed it under (`null` for a cleanup this UI
// did not start) and why: what the result lists.
type ItemProblem = {
  readonly finding: FindingId;
  readonly outcome: Extract<ItemOutcome, { readonly kind: 'failed' | 'skipped' }>;
  readonly title: null | Text;
};

export const NO_CLEANUP: CleanupState = { kind: 'idle' };

// What the dialog showed, sent back as it is: the plan's own count, allocated bytes and highest level, and the method
// the user chose. Nothing is summed here.
export const confirmationOf = (plan: CleanupPlanView, method: Method): Confirmation => ({
  allocated: plan.summary.sizes.allocated,
  count: plan.summary.count,
  highest: plan.summary.highest,
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
      return state.kind === 'preparing' ? { kind: 'confirming', plan: action.plan } : state;
    case 'cleanup-preparing':
      return state.kind === 'running' ? state : { kind: 'preparing' };
    case 'cleanup-refused':
      return { kind: 'refused', refusal: action.refusal };
    case 'cleanup-stopping':
      return state.kind === 'running' && !state.stopping ? { ...state, stopping: true } : state;
  }
};
