import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCore, sampleAgreementAnswers } from '../ipc/mock';
import { createNavigation } from '../state/navigation';
import { type CoreStore, createCoreStore } from '../state/store';
import { AGREEMENT_DICTIONARY, AGREEMENT_NOT_ACCEPTED } from '../test/agreement';
import { fakeReducedMotion, setMotion } from '../test/animation';
import { AgreementGate } from './agreement-gate';
import { AppProviders } from './app-providers';

const DICTIONARY = {
  ...AGREEMENT_DICTIONARY,
  'ui.agreement.accept': 'I understand',
  'ui.agreement.later': 'Not now',
  'ui.agreement.licence': 'Read the full licence',
  'ui.licence.title': 'Licence',
};

beforeEach(async () => {
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

const mountGate = async (): Promise<CoreStore> => {
  mockCore({ ...sampleAgreementAnswers, getDictionary: () => DICTIONARY });
  const store = createCoreStore();
  render(
    <AppProviders navigation={createNavigation()} store={store}>
      <AgreementGate />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });
  act(() => {
    store.dispatch({ type: 'agreement-loaded', view: AGREEMENT_NOT_ACCEPTED });
  });

  return store;
};

const ask = (store: CoreStore): void => {
  act(() => {
    store.dispatch({ next: { kind: 'prepare' }, type: 'agreement-asked' });
  });
};

describe('AgreementGate', () => {
  describe('negative cases', () => {
    it('a licence left open when the sheet closed is not open at the next asking', async () => {
      const store = await mountGate();
      ask(store);
      fireEvent.click(await screen.findByRole('button', { name: 'Read the full licence' }));
      expect(await screen.findByRole('dialog', { name: 'Licence' })).toBeInTheDocument();

      act(() => {
        store.dispatch({ type: 'agreement-dismissed' });
      });
      ask(store);

      expect(await screen.findByRole('button', { name: 'Not now' })).toBeInTheDocument();
      expect(screen.queryByRole('dialog', { name: 'Licence' })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('the licence opens over the sheet', async () => {
      const store = await mountGate();
      ask(store);

      fireEvent.click(await screen.findByRole('button', { name: 'Read the full licence' }));

      expect(await screen.findByRole('dialog', { name: 'Licence' })).toBeInTheDocument();
    });
  });
});
