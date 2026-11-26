import { describe, expect, it, vi } from 'vitest';

import type { CleanupPlanView, Confirmation, Event, Finding } from '../ipc/bindings.generated';

import { mockCore, refuse, sampleAgreementAnswers } from '../ipc/mock';
import { AGREEMENT_ACCEPTED } from '../test/agreement';
import { uninstallPlanOf, VIDEO_STUDIO_DETAIL } from '../test/app-detail';
import {
  cleanupEventsOf,
  DEMO_PLAN,
  HIGH_PLAN,
  LEFT_OUT_PLAN,
  LONG_PLAN,
  MOVED_REPORT,
  ONE_ITEM_PLAN,
  PLAN_GONE,
  SKIPPED_OUTCOMES,
  SKIPPED_REPORT,
  TOO_MANY_ITEMS,
  TOOL_ONLY_PLAN,
  TOOL_PLAN,
} from '../test/cleanup-plan';
import { DEMO_FINDINGS, DEMO_OVERVIEW } from '../test/demo-scan';
import { confirmationOf } from './cleanup';
import { confirm, discard, prepare, prepareUninstallPlan, quitAfter, showInTrash, stop } from './cleanup-actions';
import { type CoreAction, type CoreState, INITIAL_STATE, reduce } from './reduce';
import { createCoreStore } from './store';

// The totals tell each category's level: a Safe category is selected whole.
const TOTALS_LOADED: CoreAction = { overview: DEMO_OVERVIEW, type: 'overview-loaded' };

const run = (actions: readonly CoreAction[], from: CoreState = INITIAL_STATE): CoreState =>
  actions.reduce(reduce, from);

const [EDITOR_CACHE, INSTALLER] = DEMO_FINDINGS as readonly [Finding, Finding, Finding];

const EVENTS = cleanupEventsOf(DEMO_PLAN, MOVED_REPORT);

// The demo plan confirmed for the Trash and named cleanup 1 by the core.
const RUNNING: readonly CoreAction[] = [
  { origin: 'selection', type: 'cleanup-preparing' },
  { plan: DEMO_PLAN, type: 'cleanup-prepared' },
  { method: 'trash', type: 'cleanup-confirming' },
  { cleanup: 1, type: 'cleanup-confirmed' },
];

const PICK_CACHES: CoreAction = { category: 'demo.caches', type: 'toggle-category' };

const APP = VIDEO_STUDIO_DETAIL.summary.app;

// The uninstall of the demo plan's cleanup, confirmed for the Trash and named cleanup 1 by the core.
const UNINSTALLING: readonly CoreAction[] = [
  { origin: 'uninstall', type: 'cleanup-preparing' },
  { plan: DEMO_PLAN, type: 'cleanup-prepared' },
  { method: 'trash', type: 'cleanup-confirming' },
  { cleanup: 1, type: 'cleanup-confirmed' },
];

const settle = (): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, 0);
  });

describe('the cleanup state', () => {
  describe('negative cases', () => {
    it('a refused prepare shows its text and returns to idle', async () => {
      mockCore({ ...sampleAgreementAnswers, prepareCleanup: refuse(TOO_MANY_ITEMS) });
      const store = createCoreStore();
      store.dispatch({ type: 'agreement-loaded', view: AGREEMENT_ACCEPTED });
      store.dispatch(TOTALS_LOADED);
      store.dispatch({ category: 'demo.caches', type: 'toggle-category' });

      prepare(store);
      expect(store.getState().cleanup).toStrictEqual({ app: null, kind: 'preparing', origin: 'selection' });
      await settle();

      expect(store.getState().cleanup).toStrictEqual({
        app: null,
        kind: 'refused',
        origin: 'selection',
        refusal: TOO_MANY_ITEMS,
      });
      expect(store.getState().failure).toBeNull();
      // Nothing was removed: the selection is kept, and a new prepare may start.
      expect(store.getState().selection.categories.has('demo.caches')).toBe(true);
      expect(reduce(store.getState(), { origin: 'selection', type: 'cleanup-preparing' }).cleanup).toStrictEqual({
        app: null,
        kind: 'preparing',
        origin: 'selection',
      });
    });

    it('an unreachable core on prepare is the frame’s failure and shows nothing of the cleanup', async () => {
      mockCore({ ...sampleAgreementAnswers });
      const store = createCoreStore();

      prepare(store);
      await settle();

      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(store.getState().failure).toStrictEqual({ kind: 'unreachable' });
    });

    it('an event of another cleanup is ignored', () => {
      const running = run([...RUNNING, EVENTS.started]);
      const other: readonly Event[] = [
        { cleanup: 7, done: 1, items: [{ finding: 1, index: 0, outcome: { kind: 'moved' } }], type: 'cleanup-items' },
        { cleanup: 7, report: MOVED_REPORT, type: 'cleanup-finished' },
      ];

      expect(run(other, running)).toBe(running);
    });

    it('items before started are ignored', () => {
      const state = run([...(EVENTS.items as CoreAction[]), EVENTS.finished]);

      expect(state).toBe(INITIAL_STATE);
    });

    it('the echo never differs from the plan view', () => {
      // Groups that do not add up: the echo still carries the plan's own summary, never a sum of its own.
      const uneven: CleanupPlanView = { ...DEMO_PLAN, groups: DEMO_PLAN.groups.slice(0, 1), plan: 9 };

      for (const plan of [DEMO_PLAN, ONE_ITEM_PLAN, HIGH_PLAN, LEFT_OUT_PLAN, LONG_PLAN, TOOL_PLAN, uneven]) {
        for (const method of ['trash', 'permanent'] as const) {
          expect(confirmationOf(plan, method, false)).toStrictEqual({
            allocated: plan.summary.sizes.allocated,
            count: plan.summary.count,
            highest: plan.summary.highest,
            highRiskUnderstood: false,
            method,
            plan: plan.plan,
          } satisfies Confirmation);
        }
      }
    });

    it('a refused confirm shows the core’s text', async () => {
      mockCore({ ...sampleAgreementAnswers, confirmCleanup: refuse(PLAN_GONE), prepareCleanup: () => TOOL_ONLY_PLAN });
      const store = createCoreStore();
      prepare(store);
      await settle();

      confirm(store, 'trash', false);
      await settle();

      expect(store.getState().cleanup).toStrictEqual({
        app: null,
        kind: 'refused',
        origin: 'selection',
        refusal: PLAN_GONE,
      });
    });

    it('confirm, stop and a second stop do nothing out of their state', async () => {
      const cancelCleanup = vi.fn();
      const confirmCleanup = vi.fn();
      mockCore({ ...sampleAgreementAnswers, cancelCleanup, confirmCleanup });
      const store = createCoreStore();

      confirm(store, 'trash', false);
      stop(store);
      await settle();

      expect(confirmCleanup).not.toHaveBeenCalled();
      expect(cancelCleanup).not.toHaveBeenCalled();
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
    });

    it('stop is sent once', async () => {
      const cancelCleanup = vi.fn(() => null);
      mockCore({ ...sampleAgreementAnswers, cancelCleanup });
      const store = createCoreStore();
      for (const action of [...RUNNING, EVENTS.started]) {
        store.dispatch(action);
      }

      stop(store);
      stop(store);
      await settle();

      expect(cancelCleanup).toHaveBeenCalledOnce();
      expect(store.getState().cleanup).toMatchObject({ kind: 'running', stopping: true });
    });

    it('a discarded plan already gone shows nothing', async () => {
      mockCore({ ...sampleAgreementAnswers, discardCleanup: refuse(PLAN_GONE), prepareCleanup: () => DEMO_PLAN });
      const store = createCoreStore();
      prepare(store);
      await settle();

      discard(store);
      await settle();

      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(store.getState().failure).toBeNull();
    });

    it('a finished uninstall does not empty the selection', () => {
      const selected = run([
        TOTALS_LOADED,
        { category: 'demo.caches', type: 'toggle-category' },
        ...UNINSTALLING,
        EVENTS.started,
      ]);

      const state = run([EVENTS.finished], selected);

      expect(state.cleanup).toMatchObject({ kind: 'finished', origin: 'uninstall' });
      expect(state.selection.categories.size).toBe(1);
    });

    it('dismissing does not hide a running cleanup', () => {
      const running = run([...RUNNING, EVENTS.started]);

      expect(run([{ type: 'cleanup-dismissed' }, { origin: 'selection', type: 'cleanup-preparing' }], running)).toBe(
        running,
      );
    });

    it('a new scan or a new pick does not hide a running cleanup', () => {
      const running = run([...RUNNING, EVENTS.started]);

      const state = run([{ modules: ['demo'], scan: 2, type: 'scan-started' }, TOTALS_LOADED, PICK_CACHES], running);

      expect(state.cleanup).toBe(running.cleanup);
    });

    it('a pick that changes nothing keeps the result of a finished cleanup', () => {
      const finished = run([...RUNNING, EVENTS.started, EVENTS.finished]);

      expect(run([{ type: 'clear' }], finished)).toBe(finished);
    });
  });

  describe('positive cases', () => {
    it('a new scan clears the result of a finished cleanup, so Review shows the new pick', () => {
      const finished = run([...RUNNING, EVENTS.started, EVENTS.finished]);

      const state = run([{ modules: ['demo'], scan: 2, type: 'scan-started' }], finished);

      expect(state.cleanup).toStrictEqual({ kind: 'idle' });
    });

    it('a new pick clears the result of a finished cleanup, so Review shows the new pick', () => {
      const finished = run([TOTALS_LOADED, ...RUNNING, EVENTS.started, EVENTS.finished]);

      const state = run([PICK_CACHES], finished);

      expect(state.cleanup).toStrictEqual({ kind: 'idle' });
      expect(state.selection.categories.size).toBe(1);
    });

    it('the confirmation sends the risk as given', () => {
      for (const highRiskUnderstood of [true, false]) {
        expect(confirmationOf(HIGH_PLAN, 'trash', highRiskUnderstood)).toMatchObject({
          highest: 'high',
          highRiskUnderstood,
        } satisfies Partial<Confirmation>);
        expect(confirmationOf(DEMO_PLAN, 'trash', highRiskUnderstood).highRiskUnderstood).toBe(highRiskUnderstood);
      }
    });

    it('an uninstall asks for its own plan and keeps its origin through the confirmation', async () => {
      const prepareUninstall = vi.fn(() => uninstallPlanOf(VIDEO_STUDIO_DETAIL, 'keep-data'));
      mockCore({ ...sampleAgreementAnswers, prepareUninstall });
      const store = createCoreStore();
      store.dispatch({ type: 'agreement-loaded', view: AGREEMENT_ACCEPTED });

      prepareUninstallPlan(store, APP, 'keep-data');
      expect(store.getState().cleanup).toStrictEqual({ app: APP, kind: 'preparing', origin: 'uninstall' });
      await settle();

      expect(prepareUninstall).toHaveBeenCalledWith({ app: APP, choice: 'keep-data' });
      expect(store.getState().cleanup).toMatchObject({ kind: 'confirming', origin: 'uninstall' });
    });

    it('a refused uninstall plan keeps its origin and its text', async () => {
      mockCore({ ...sampleAgreementAnswers, prepareUninstall: refuse(TOO_MANY_ITEMS) });
      const store = createCoreStore();

      prepareUninstallPlan(store, APP, 'everything');
      await settle();

      expect(store.getState().cleanup).toStrictEqual({
        app: APP,
        kind: 'refused',
        origin: 'uninstall',
        refusal: TOO_MANY_ITEMS,
      });
    });

    it('the origin runs through running and finished', () => {
      const running = run([...UNINSTALLING, EVENTS.started]);

      expect(running.cleanup).toMatchObject({ kind: 'running', origin: 'uninstall' });
      expect(run([EVENTS.finished], running).cleanup).toMatchObject({ kind: 'finished', origin: 'uninstall' });
    });

    it('prepare then confirm runs then finishes', async () => {
      const seen: unknown[] = [];
      mockCore({
        ...sampleAgreementAnswers,
        confirmCleanup: (args) => {
          seen.push(args);

          return 1;
        },
        prepareCleanup: (args) => {
          seen.push(args);

          return DEMO_PLAN;
        },
      });
      const store = createCoreStore();
      store.dispatch(TOTALS_LOADED);
      store.dispatch({ category: 'demo.caches', type: 'toggle-category' });
      store.dispatch({ finding: EDITOR_CACHE, type: 'toggle-finding' });
      store.dispatch({ finding: INSTALLER, type: 'toggle-finding' });

      prepare(store);
      await settle();
      expect(store.getState().cleanup).toStrictEqual({
        app: null,
        kind: 'confirming',
        origin: 'selection',
        plan: DEMO_PLAN,
      });

      confirm(store, 'permanent', false);
      expect(store.getState().cleanup).toMatchObject({ cleanup: null, done: 0, kind: 'running', total: 3 });
      await settle();
      for (const event of [EVENTS.started, ...EVENTS.items, EVENTS.finished]) {
        store.dispatch(event);
      }

      expect(seen).toStrictEqual([
        {
          selection: { categories: [{ category: 'demo.caches', except: [EDITOR_CACHE.id] }], findings: [INSTALLER.id] },
        },
        { confirmation: confirmationOf(DEMO_PLAN, 'permanent', false) },
      ]);
      expect(store.getState().cleanup).toStrictEqual({
        cleanup: 1,
        firstMoved: 0,
        kind: 'finished',
        origin: 'selection',
        problems: [],
        report: MOVED_REPORT,
      });
    });

    it('the core may name the cleanup before its answer', () => {
      const state = run([...RUNNING.slice(0, 3), EVENTS.started, { cleanup: 1, type: 'cleanup-confirmed' }]);

      expect(state.cleanup).toMatchObject({ cleanup: 1, kind: 'running', method: 'trash', total: 3 });
    });

    it('items advance the done count', () => {
      const [first, second] = EVENTS.items as [CoreAction, CoreAction];
      const titles = DEMO_PLAN.groups.flatMap((group) => group.items.map((item) => item.title));

      const once = run([...RUNNING, EVENTS.started, first]);
      const twice = run([second], once);

      expect(once.cleanup).toMatchObject({ current: titles[1], done: 1 });
      expect(twice.cleanup).toMatchObject({ current: titles[2], done: 2 });
    });

    it('the skipped items are kept with their titles', () => {
      const events = cleanupEventsOf(DEMO_PLAN, SKIPPED_REPORT, { outcomes: SKIPPED_OUTCOMES });
      const titles = DEMO_PLAN.groups.flatMap((group) => group.items.map((item) => item.title));

      const state = run([...RUNNING, events.started, ...events.items, events.finished]);

      expect(state.cleanup).toMatchObject({
        firstMoved: 2,
        kind: 'finished',
        problems: [
          { outcome: SKIPPED_OUTCOMES[0], title: titles[0] },
          { outcome: SKIPPED_OUTCOMES[1], title: titles[1] },
        ],
      });
    });

    it('a cleanup this UI did not start is followed from its start', () => {
      const state = run([EVENTS.started, ...EVENTS.items.slice(0, 1)]);

      expect(state.cleanup).toMatchObject({ cleanup: 1, current: null, done: 1, kind: 'running', plan: null });
    });

    it('findings removed leave the totals: the overview is asked for again', () => {
      const state = run([{ findings: [1, 3], scan: 1, type: 'findings-removed' }]);

      expect(state.removed).toStrictEqual([1, 3]);
      expect(state.stale).toBe(true);
    });

    it('a finished cleanup empties the selection', () => {
      const selected = run([
        TOTALS_LOADED,
        { category: 'demo.caches', type: 'toggle-category' },
        ...RUNNING,
        EVENTS.started,
      ]);

      const state = run([EVENTS.finished], selected);

      expect(selected.selection.categories.size).toBe(1);
      expect(state.selection.categories.size).toBe(0);
    });

    it('quit requested opens the question', async () => {
      const quitAfterCleanup = vi.fn(() => null);
      mockCore({ ...sampleAgreementAnswers, quitAfterCleanup });
      const store = createCoreStore();

      store.dispatch({ type: 'quit-requested' });
      expect(store.getState().quitAsked).toBe(true);
      store.dispatch({ type: 'quit-answered' });
      expect(store.getState().quitAsked).toBe(false);

      store.dispatch({ type: 'quit-requested' });
      quitAfter(store);
      await settle();

      expect(store.getState().quitAsked).toBe(false);
      expect(quitAfterCleanup).toHaveBeenCalledOnce();
    });

    it('show in the trash asks the core for the first moved item', async () => {
      const revealInTrash = vi.fn(() => null);
      mockCore({ ...sampleAgreementAnswers, revealInTrash });
      const store = createCoreStore();
      showInTrash(store);
      const events = cleanupEventsOf(DEMO_PLAN, SKIPPED_REPORT, { outcomes: SKIPPED_OUTCOMES });
      for (const action of [...RUNNING, events.started, ...events.items, events.finished]) {
        store.dispatch(action);
      }

      showInTrash(store);
      await settle();

      expect(revealInTrash).toHaveBeenCalledExactlyOnceWith({ item: { cleanup: 1, item: 2 } });
    });

    it('cancel discards the plan', async () => {
      const discardCleanup = vi.fn(() => null);
      mockCore({ ...sampleAgreementAnswers, discardCleanup, prepareCleanup: () => DEMO_PLAN });
      const store = createCoreStore();
      prepare(store);
      await settle();

      discard(store);
      await settle();

      expect(discardCleanup).toHaveBeenCalledExactlyOnceWith({ plan: DEMO_PLAN.plan });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
    });
  });
});
