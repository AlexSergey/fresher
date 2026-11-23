import { describe, expect, it, vi } from 'vitest';

import type { AgreementView, Refusal } from '../ipc/bindings.generated';

import { mockCore, refuse } from '../ipc/mock';
import { AGREEMENT_ACCEPTED, AGREEMENT_CHANGED, AGREEMENT_NOT_ACCEPTED, AGREEMENT_VERSION } from '../test/agreement';
import { uninstallPlanOf, VIDEO_STUDIO_DETAIL } from '../test/app-detail';
import { DEMO_PLAN } from '../test/cleanup-plan';
import { acceptAgreementAndGoOn, declineAgreement, prepare, prepareUninstallPlan } from './cleanup-actions';
import { type CoreStore, createCoreStore } from './store';

const APP = VIDEO_STUDIO_DETAIL.summary.app;

const AGREEMENT_CHANGED_REFUSAL: Refusal = {
  error: { code: 'agreement-changed' },
  text: { args: {}, key: 'core.error.agreement-changed' },
};

const settle = (): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, 0);
  });

const storeWith = (view: AgreementView | null): CoreStore => {
  const store = createCoreStore();

  if (view !== null) {
    store.dispatch({ type: 'agreement-loaded', view });
  }

  return store;
};

// An older version was accepted, and the text of the current one is not yet.
const OLDER_ACCEPTED: AgreementView = { ...AGREEMENT_CHANGED, changes: null };

describe('the agreement gate', () => {
  describe('negative cases', () => {
    it('prepare asks for the agreement when it is not accepted, and prepares nothing', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ prepareCleanup });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);

      prepare(store);
      await settle();

      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(prepareCleanup).not.toHaveBeenCalled();
    });

    it('prepare loads the agreement when it is not loaded, then asks', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ getAgreement: () => AGREEMENT_NOT_ACCEPTED, prepareCleanup });
      const store = storeWith(null);

      prepare(store);
      await settle();

      expect(store.getState().agreement.view).toStrictEqual(AGREEMENT_NOT_ACCEPTED);
      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
      expect(prepareCleanup).not.toHaveBeenCalled();
    });

    it('prepare asks when an older version was accepted', async () => {
      mockCore({});
      const store = storeWith(OLDER_ACCEPTED);

      prepare(store);
      await settle();

      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
    });

    it('an uninstall asks before it prepares', async () => {
      const prepareUninstall = vi.fn(() => uninstallPlanOf(VIDEO_STUDIO_DETAIL, 'keep-data'));
      mockCore({ prepareUninstall });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);

      prepareUninstallPlan(store, APP, 'keep-data');
      await settle();

      expect(store.getState().agreement.asking).toStrictEqual({ app: APP, choice: 'keep-data', kind: 'uninstall' });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(prepareUninstall).not.toHaveBeenCalled();
    });

    it('an unreachable core while loading the agreement is the frame’s failure', async () => {
      mockCore({});
      const store = storeWith(null);

      prepare(store);
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'unreachable' });
      expect(store.getState().agreement.asking).toBeNull();
    });

    it('declining runs nothing', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ prepareCleanup });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      prepare(store);
      await settle();

      declineAgreement(store);
      await settle();

      expect(store.getState().agreement.asking).toBeNull();
      expect(prepareCleanup).not.toHaveBeenCalled();
    });

    it('a refused acceptance keeps the sheet open with the new view', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({
        acceptAgreement: refuse(AGREEMENT_CHANGED_REFUSAL),
        getAgreement: () => AGREEMENT_CHANGED,
        prepareCleanup,
      });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      prepare(store);
      await settle();

      acceptAgreementAndGoOn(store);
      await settle();

      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
      expect(store.getState().agreement.view).toStrictEqual(AGREEMENT_CHANGED);
      expect(prepareCleanup).not.toHaveBeenCalled();
    });

    it('accepting with no sheet open does nothing', async () => {
      const accept = vi.fn(() => AGREEMENT_ACCEPTED);
      mockCore({ acceptAgreement: accept });

      acceptAgreementAndGoOn(storeWith(AGREEMENT_NOT_ACCEPTED));
      await settle();

      expect(accept).not.toHaveBeenCalled();
    });

    it('an unreachable core on acceptance is the frame’s failure and closes the sheet', async () => {
      mockCore({});
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      store.dispatch({ next: { kind: 'prepare' }, type: 'agreement-asked' });

      acceptAgreementAndGoOn(store);
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'unreachable' });
      expect(store.getState().agreement.asking).toBeNull();
    });

    it('an acceptance refused because the settings file is not loaded is shown and keeps the sheet open', async () => {
      const blocked: Refusal = {
        error: { code: 'cleaning-blocked', reason: 'settings' },
        text: { args: {}, key: 'core.error.cleaning-blocked' },
      };
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ acceptAgreement: refuse(blocked), prepareCleanup });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      store.dispatch({ next: { kind: 'prepare' }, type: 'agreement-asked' });

      acceptAgreementAndGoOn(store);
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: blocked });
      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
      expect(store.getState().agreement.view).toStrictEqual(AGREEMENT_NOT_ACCEPTED);
      expect(prepareCleanup).not.toHaveBeenCalled();
    });

    it('a second click while the acceptance is on its way accepts once and prepares once', async () => {
      const acceptAgreement = vi.fn(() => AGREEMENT_ACCEPTED);
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ acceptAgreement, prepareCleanup });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      store.dispatch({ next: { kind: 'prepare' }, type: 'agreement-asked' });

      acceptAgreementAndGoOn(store);
      acceptAgreementAndGoOn(store);
      await settle();

      expect(acceptAgreement).toHaveBeenCalledOnce();
      expect(prepareCleanup).toHaveBeenCalledOnce();
    });

    it('a failed load of the agreement prepares nothing', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      const prepareUninstall = vi.fn(() => uninstallPlanOf(VIDEO_STUDIO_DETAIL, 'keep-data'));
      const storage: Refusal = { error: { code: 'storage' }, text: { args: {}, key: 'core.error.storage' } };
      mockCore({ getAgreement: refuse(storage), prepareCleanup, prepareUninstall });
      const store = storeWith(null);

      prepare(store);
      prepareUninstallPlan(store, APP, 'keep-data');
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: storage });
      expect(store.getState().agreement).toStrictEqual({ asking: null, view: null });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(prepareCleanup).not.toHaveBeenCalled();
      expect(prepareUninstall).not.toHaveBeenCalled();
    });

    it('another refusal of the acceptance is shown and keeps the sheet open', async () => {
      const other: Refusal = { error: { code: 'storage' }, text: { args: {}, key: 'core.error.storage' } };
      mockCore({ acceptAgreement: refuse(other) });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      store.dispatch({ next: { kind: 'prepare' }, type: 'agreement-asked' });

      acceptAgreementAndGoOn(store);
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: other });
      expect(store.getState().agreement.asking).toStrictEqual({ kind: 'prepare' });
    });
  });

  describe('positive cases', () => {
    it('prepare goes straight on when accepted', async () => {
      const prepareCleanup = vi.fn(() => DEMO_PLAN);
      mockCore({ prepareCleanup });
      const store = storeWith(AGREEMENT_ACCEPTED);

      prepare(store);
      await settle();

      expect(store.getState().agreement.asking).toBeNull();
      expect(store.getState().cleanup).toMatchObject({ kind: 'confirming' });
    });

    it('accepting runs the held prepare', async () => {
      const acceptAgreement = vi.fn(() => AGREEMENT_ACCEPTED);
      mockCore({ acceptAgreement, prepareCleanup: () => DEMO_PLAN });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      prepare(store);
      await settle();

      acceptAgreementAndGoOn(store);
      await settle();

      expect(acceptAgreement).toHaveBeenCalledExactlyOnceWith({ version: AGREEMENT_VERSION });
      expect(store.getState().agreement.asking).toBeNull();
      expect(store.getState().agreement.view).toStrictEqual(AGREEMENT_ACCEPTED);
      expect(store.getState().cleanup).toMatchObject({ kind: 'confirming' });
    });

    it('accepting runs the held uninstall with its app and choice', async () => {
      const prepareUninstall = vi.fn(() => uninstallPlanOf(VIDEO_STUDIO_DETAIL, 'everything'));
      mockCore({ acceptAgreement: () => AGREEMENT_ACCEPTED, prepareUninstall });
      const store = storeWith(AGREEMENT_NOT_ACCEPTED);
      prepareUninstallPlan(store, APP, 'everything');
      await settle();

      acceptAgreementAndGoOn(store);
      await settle();

      expect(prepareUninstall).toHaveBeenCalledExactlyOnceWith({ app: APP, choice: 'everything' });
      expect(store.getState().cleanup).toMatchObject({ kind: 'confirming', origin: 'uninstall' });
    });
  });
});
