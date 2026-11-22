import type { Page } from '@playwright/test';

import type {
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
} from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { cleanupEventsOf, leftOutOf, PLAN_GONE, PLAN_MISMATCH, planOf, reportOf } from '../../src/test/cleanup-plan';

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

const PLAN_OF = '__fresherPlanOf';
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
  readonly history: CleanupsPage;
  readonly hold: null | number;
  readonly hooks: typeof CLEANUP_HOOKS;
  readonly mismatch: Refusal;
  readonly names: { readonly cleanupOf: string; readonly planOf: string; readonly release: string };
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

// Adds the cleanup commands to the mocked core of `core-mock.ts`, which calls `invoke` first for every command.
export const mockCleanup = async (page: Page, findings: readonly Finding[], script: CleanupScript): Promise<void> => {
  await page.exposeFunction(PLAN_OF, planFor(findings, script));
  await page.exposeFunction(CLEANUP_OF, (plan: CleanupPlanView, method: Method) =>
    JSON.stringify(cleanupOf(script, plan, method, script.hold ?? plan.summary.count)),
  );
  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;
      const send = (events: readonly Event[]): void => {
        (scope[setup.hooks.send] as (events: readonly Event[]) => void)(events);
      };
      const details = new Map(setup.details.map((detail) => [detail.summary.cleanup, detail]));
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
            plans += 1;
            plan = await (scope[setup.names.planOf] as (selection: unknown, id: number) => Promise<CleanupPlanView>)(
              args.selection,
              plans,
            );

            return plan;
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
      history: script.history ?? NO_HISTORY,
      hold: script.hold ?? null,
      hooks: CLEANUP_HOOKS,
      mismatch: PLAN_MISMATCH,
      names: { cleanupOf: CLEANUP_OF, planOf: PLAN_OF, release: RELEASE },
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
