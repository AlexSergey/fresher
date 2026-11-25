import type { Page } from '@playwright/test';

import type {
  AppId,
  CleanupDetail,
  CleanupPlanView,
  CleanupSelection,
  CleanupsPage,
  Confirmation,
  Event,
  Finding,
  ItemOutcome,
  Method,
  Overview,
  PutBackRequest,
  Refusal,
  SkipReason,
  TrashView,
  UninstallChoice,
} from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { APP_DETAILS, uninstallPlanOf } from '../../src/test/app-detail';
import {
  cleanupEventsOf,
  HIGH_RISK_NOT_UNDERSTOOD,
  leftOutOf,
  PLAN_GONE,
  PLAN_MISMATCH,
  planOf,
  reportOf,
} from '../../src/test/cleanup-plan';

// The cleanup's part of the mocked core: a plan per selection over the scan's findings, the echo compared with it, a
// cleanup streamed item by item (held after `hold` items until `releaseCleanup` or Stop), a history and put back.
export type CleanupScript = {
  // What `get_overview` answers once a cleanup finished.
  readonly after?: Overview;
  // The id the core gives the cleanup.
  readonly cleanup?: number;
  // One cleanup of the history each, by its id.
  readonly details?: readonly CleanupDetail[];
  // What `get_cleanups` answers.
  readonly history?: CleanupsPage;
  // The cleanup streams this many items, then waits for `releaseCleanup` (or Stop).
  readonly hold?: number;
  // Findings every plan leaves out, with their reason.
  readonly leftOut?: Readonly<Record<number, SkipReason>>;
  // How each item of the plan ends, by its index; done the method's way when not given.
  readonly outcomes?: readonly ItemOutcome[];
  // What `get_trash` answers once a cleanup finished (the Trash then holds what it moved).
  readonly trashAfterCleanup?: TrashView;
};

// The names the core mock's own script shares with this one, on `globalThis`.
export const CLEANUP_HOOKS = {
  answer: '__fresherSetAnswer',
  invoke: '__fresherCleanupInvoke',
  send: '__fresherSend',
} as const;

// The names of the commands that asked for a plan, in the page, in order.
const PREPARED = '__fresherPrepared';

const PLAN_OF = '__fresherPlanOf';
const UNINSTALL_PLAN_OF = '__fresherUninstallPlanOf';
const CLEANUP_OF = '__fresherCleanupOf';
const RELEASE = '__fresherReleaseCleanup';

type Cleanup = {
  // What streams after the held items: the rest of the items, the end and the findings removed.
  readonly end: readonly Event[];
  readonly start: readonly Event[];
  // What streams when the held cleanup is stopped.
  readonly stopped: readonly Event[];
};

type Setup = {
  readonly after: null | Overview;
  readonly cleanup: number;
  readonly commands: typeof COMMANDS;
  readonly details: readonly CleanupDetail[];
  readonly gone: Refusal;
  // A plan whose highest level is High confirmed without the user's tick, as the core refuses it.
  readonly highRisk: Refusal;
  readonly history: CleanupsPage;
  readonly hold: null | number;
  readonly hooks: typeof CLEANUP_HOOKS;
  readonly mismatch: Refusal;
  readonly names: {
    readonly cleanupOf: string;
    readonly planOf: string;
    readonly prepared: string;
    readonly release: string;
    readonly uninstallPlanOf: string;
  };
  readonly trashAfter: null | TrashView;
};

const NO_HISTORY: CleanupsPage = { blocked: null, cleanups: [], offset: 0, total: 0, waiting: 0 };

const removedOf = (plan: CleanupPlanView, outcomes: readonly ItemOutcome[]): Event => ({
  findings: plan.groups
    .flatMap((group) => group.items)
    .filter((_, index) => ['deleted', 'emptied', 'moved', 'tool-done'].includes(outcomes[index]?.kind ?? ''))
    .map((item) => item.finding),
  scan: 1,
  type: 'findings-removed',
});

// The events of a cleanup of `plan` by `method`: whole, and stopped after `hold` items.
const cleanupOf = (script: CleanupScript, plan: CleanupPlanView, method: Method, hold: number): Cleanup => {
  const count = plan.summary.count;
  const done: ItemOutcome = method === 'trash' ? { kind: 'moved' } : { kind: 'deleted' };
  const outcomes = Array.from({ length: count }, (_, at) => script.outcomes?.[at] ?? done);
  const cleanup = script.cleanup ?? 3;
  const report = reportOf(plan, {
    measured: method === 'trash' ? 120_000_000 : plan.summary.sizes.allocated,
    method,
    outcomes,
  });
  const events = cleanupEventsOf(plan, report, { cleanup, outcomes });
  const stoppedOutcomes = outcomes.map((outcome, at): ItemOutcome => (at < hold ? outcome : { kind: 'not-reached' }));
  const stopped = reportOf(plan, { ended: 'cancelled', measured: 40_000_000, method, outcomes: stoppedOutcomes });

  return {
    end: [...events.items.slice(hold), events.finished, removedOf(plan, outcomes)],
    start: [events.started, ...events.items.slice(0, hold)],
    stopped: [{ cleanup, report: stopped, type: 'cleanup-finished' }, removedOf(plan, stoppedOutcomes)],
  };
};

// The plan of a selection over the scan's findings, in the order the scan found them.
const planFor =
  (findings: readonly Finding[], script: CleanupScript) =>
  (selection: CleanupSelection, id: number): CleanupPlanView => {
    const picked = findings.filter(
      (finding) =>
        selection.findings.includes(finding.id) ||
        selection.categories.some((pick) => pick.category === finding.category && !pick.except.includes(finding.id)),
    );
    const reasons = script.leftOut ?? {};

    return planOf(
      picked.filter((finding) => reasons[finding.id] === undefined),
      {
        leftOut: picked.flatMap((finding) => {
          const reason = reasons[finding.id];

          return reason === undefined ? [] : [leftOutOf(finding, reason)];
        }),
        plan: id,
      },
    );
  };

// The selections `prepare_cleanup` received on each page, in order: what the UI sent the core.
const SELECTIONS = new WeakMap<Page, readonly CleanupSelection[]>();

// Adds the cleanup commands to the mocked core of `core-mock.ts`, which calls `invoke` first for every command.
// `confirmed` is told the plan of every cleanup the user confirmed (the Largest mock forgets what it moved).
export const mockCleanup = async (
  page: Page,
  findings: readonly Finding[],
  script: CleanupScript,
  confirmed?: (plan: CleanupPlanView) => void,
): Promise<void> => {
  const planOfSelection = planFor(findings, script);
  const selections: CleanupSelection[] = [];
  SELECTIONS.set(page, selections);
  await page.exposeFunction(PLAN_OF, (selection: CleanupSelection, id: number) => {
    selections.push(selection);

    return planOfSelection(selection, id);
  });
  // The plan of the uninstall of an app of the mock's apps (support/apps-mock.ts): never over the scan's findings.
  await page.exposeFunction(UNINSTALL_PLAN_OF, (app: AppId, choice: UninstallChoice, id: number) => {
    const detail = APP_DETAILS.find(
      ({ summary }) => summary.app.state === app.state && summary.app.index === app.index,
    );

    return detail === undefined ? null : uninstallPlanOf(detail, choice, id);
  });
  await page.exposeFunction(CLEANUP_OF, (plan: CleanupPlanView, method: Method) => {
    confirmed?.(plan);

    return JSON.stringify(cleanupOf(script, plan, method, script.hold ?? plan.summary.count));
  });
  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;
      const send = (events: readonly Event[]): void => {
        (scope[setup.hooks.send] as (events: readonly Event[]) => void)(events);
      };
      const details = new Map(setup.details.map((detail) => [detail.summary.cleanup, detail]));
      const prepared: string[] = [];
      let plans = 0;
      let plan: CleanupPlanView | null = null;
      let held: Cleanup | null = null;

      const end = (events: readonly Event[]): void => {
        held = null;
        if (setup.after !== null) {
          (scope[setup.hooks.answer] as (command: string, value: unknown) => void)(
            setup.commands.getOverview,
            setup.after,
          );
        }
        if (setup.trashAfter !== null) {
          (scope[setup.hooks.answer] as (command: string, value: unknown) => void)(
            setup.commands.getTrash,
            setup.trashAfter,
          );
        }
        send(events);
      };

      scope[setup.names.prepared] = prepared;
      scope[setup.names.release] = (): void => {
        if (held !== null) {
          end(held.end);
        }
      };

      const confirm = async (echo: Confirmation): Promise<unknown> => {
        const shown = plan;

        plan = null;
        if (shown?.plan !== echo.plan) {
          // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
          throw setup.gone;
        }
        const { summary } = shown;
        if (
          summary.count !== echo.count ||
          summary.sizes.allocated !== echo.allocated ||
          summary.highest !== echo.highest
        ) {
          // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
          throw setup.mismatch;
        }
        if (summary.highest === 'high' && !echo.highRiskUnderstood) {
          // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
          throw setup.highRisk;
        }
        const json = await (scope[setup.names.cleanupOf] as (plan: unknown, method: Method) => Promise<string>)(
          shown,
          echo.method,
        );
        const cleanup = JSON.parse(json) as Cleanup;

        held = cleanup;
        // The answer comes before the events, as the engine starts the cleanup on its own thread.
        setTimeout(() => {
          send(cleanup.start);
          if (setup.hold === null) {
            end(cleanup.end);
          }
        }, 0);

        return setup.cleanup;
      };

      const putBack = ({ cleanup, items }: PutBackRequest): unknown => {
        const detail = details.get(cleanup);
        const wanted = (detail?.items ?? []).filter((item) =>
          items === null ? item.trash === 'in-trash' : items.includes(item.index),
        );

        if (detail !== undefined) {
          details.set(cleanup, {
            ...detail,
            items: detail.items.map((item) =>
              wanted.includes(item) ? { ...item, putBack: { kind: 'put-back' }, trash: null } : item,
            ),
            summary: { ...detail.summary, inTrash: detail.summary.inTrash - wanted.length },
          });
        }

        return {
          items: wanted.map((item) => ({
            index: item.index,
            outcome: { kind: 'put-back' },
            text: { args: {}, key: 'core.cleanup.put-back.put-back' },
          })),
        };
      };

      const answer = async (command: string, args: Record<string, unknown>): Promise<unknown> => {
        const { commands } = setup;

        switch (command) {
          case commands.cancelCleanup:
            if (held !== null && setup.hold !== null) {
              end(held.stopped);
            }

            return null;
          case commands.confirmCleanup:
            return confirm(args.confirmation as Confirmation);
          case commands.discardCleanup:
            plan = null;

            return null;
          case commands.getCleanup:
            // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
            return details.get(args.cleanup as number) ?? Promise.reject(setup.gone);
          case commands.getCleanups:
            return {
              ...setup.history,
              cleanups: setup.history.cleanups.map((one) => details.get(one.cleanup)?.summary ?? one),
            };
          case commands.prepareCleanup:
            prepared.push(command);
            plans += 1;
            plan = await (scope[setup.names.planOf] as (selection: unknown, id: number) => Promise<CleanupPlanView>)(
              args.selection,
              plans,
            );

            return plan;
          case commands.prepareUninstall: {
            prepared.push(command);
            plans += 1;
            const made = await (
              scope[setup.names.uninstallPlanOf] as (
                app: unknown,
                choice: unknown,
                id: number,
              ) => Promise<CleanupPlanView | null>
            )(args.app, args.choice, plans);

            if (made === null) {
              // The core refuses an id it does not list, as `get_app` does.
              // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
              throw {
                error: { code: 'unknown-app', id: args.app },
                text: { args: {}, key: 'core.error.unknown-app' },
              };
            }
            plan = made;

            return made;
          }
          case commands.putBack:
            return putBack(args.request as PutBackRequest);
          default:
            return null;
        }
      };

      const handled = new Set<string>([
        setup.commands.cancelCleanup,
        setup.commands.confirmCleanup,
        setup.commands.discardCleanup,
        setup.commands.getCleanup,
        setup.commands.getCleanups,
        setup.commands.prepareCleanup,
        setup.commands.prepareUninstall,
        setup.commands.putBack,
        setup.commands.quitAfterCleanup,
        setup.commands.revealInTrash,
      ]);

      scope[setup.hooks.invoke] = (command: string, args: Record<string, unknown>): Promise<unknown> | undefined =>
        handled.has(command) ? answer(command, args) : undefined;
    },
    {
      after: script.after ?? null,
      cleanup: script.cleanup ?? 3,
      commands: COMMANDS,
      details: script.details ?? [],
      gone: PLAN_GONE,
      highRisk: HIGH_RISK_NOT_UNDERSTOOD,
      history: script.history ?? NO_HISTORY,
      hold: script.hold ?? null,
      hooks: CLEANUP_HOOKS,
      mismatch: PLAN_MISMATCH,
      names: {
        cleanupOf: CLEANUP_OF,
        planOf: PLAN_OF,
        prepared: PREPARED,
        release: RELEASE,
        uninstallPlanOf: UNINSTALL_PLAN_OF,
      },
      trashAfter: script.trashAfterCleanup ?? null,
    } satisfies Setup,
  );
};

// Streams the rest of a held cleanup: it finishes as the script's cleanup.
export const releaseCleanup = async (page: Page): Promise<void> => {
  await page.evaluate((release) => {
    (globalThis as unknown as Record<string, () => void>)[release]?.();
  }, RELEASE);
};

// The window was closed during a cleanup: the core asks the UI.
export const requestQuit = async (page: Page): Promise<void> => {
  await page.evaluate((send) => {
    (globalThis as unknown as Record<string, (events: readonly unknown[]) => void>)[send]?.([
      { type: 'quit-requested' },
    ]);
  }, CLEANUP_HOOKS.send);
};

// The commands that asked the core for a plan so far (`prepare_cleanup`, `prepare_uninstall`), in order.
export const preparedCommands = (page: Page): Promise<readonly string[]> =>
  page.evaluate((name) => (globalThis as unknown as Record<string, string[]>)[name] ?? [], PREPARED);

// The selections the UI sent with `prepare_cleanup` so far, in order: ids only, as the core takes them.
export const preparedSelections = (page: Page): readonly CleanupSelection[] => SELECTIONS.get(page) ?? [];
