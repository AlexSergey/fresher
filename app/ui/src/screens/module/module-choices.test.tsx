import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { ModuleInfo } from '../../ipc/bindings.generated';

import { mockCore, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { APP_LEFTOVERS_DICTIONARY, APP_LEFTOVERS_MODULES, APP_LEFTOVERS_TOTALS } from '../../test/app-leftovers-scan';
import { DEMO_MODULES } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { ModuleChoices } from './module-choices';

const [APP_LEFTOVERS] = APP_LEFTOVERS_MODULES as readonly [ModuleInfo];
const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const text = (key: string): string => APP_LEFTOVERS_DICTIONARY[`app-leftovers.${key}`] ?? '';

const EVERYTHING = text('choice.everything.label');
const KEEP_SETTINGS = text('choice.keep-settings.label');

const showChoices = (module: ModuleInfo, totals: typeof APP_LEFTOVERS_TOTALS): CoreStore => {
  const store = createCoreStore();

  mockCore({ getDictionary: () => APP_LEFTOVERS_DICTIONARY, getModules: () => [module], ...sampleStorageAnswers });
  render(
    <AppProviders navigation={createNavigation()} store={store}>
      <ModuleChoices module={module} totals={totals} />
    </AppProviders>,
  );

  return store;
};

const nothingFound = APP_LEFTOVERS_TOTALS.map((total) => ({ ...total, count: 0 }));

describe('ModuleChoices', () => {
  describe('negative cases', () => {
    it('renders nothing for a module without choices', () => {
      showChoices(DEMO, []);

      expect(screen.queryByRole('button')).toBeNull();
      expect(screen.queryByRole('group')).toBeNull();
    });

    it('disables a choice whose categories hold no finding', async () => {
      showChoices(APP_LEFTOVERS, nothingFound);

      expect(await screen.findByRole('button', { name: EVERYTHING })).toBeDisabled();
      expect(screen.getByRole('button', { name: KEEP_SETTINGS })).toBeDisabled();
    });

    it('disables the choice of categories without a finding and keeps the other enabled', async () => {
      showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.map((total) =>
          total.category === 'app-leftovers.caches' ? { ...total, count: 0 } : total,
        ),
      );

      expect(await screen.findByRole('button', { name: KEEP_SETTINGS })).toBeDisabled();
      expect(screen.getByRole('button', { name: EVERYTHING })).toBeEnabled();
    });

    it('disables the choice of an ignored category, whose findings leave the totals, and selects nothing', async () => {
      const store = showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.filter((total) => total.category !== 'app-leftovers.caches'),
      );

      const keepSettings = await screen.findByRole('button', { name: KEEP_SETTINGS });
      fireEvent.click(keepSettings);

      expect(keepSettings).toBeDisabled();
      expect(store.getState().selection.categories.size).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('shows each choice’s description beside its label', async () => {
      showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      await screen.findByRole('button', { name: EVERYTHING });

      expect(screen.getByText(text('choice.everything.description'))).toBeVisible();
      expect(screen.getByText(text('choice.keep-settings.description'))).toBeVisible();
    });

    it('selects exactly the categories of the clicked choice', async () => {
      const store = showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      fireEvent.click(await screen.findByRole('button', { name: KEEP_SETTINGS }));

      expect([...store.getState().selection.categories.keys()]).toEqual(['app-leftovers.caches']);
    });

    it('selects the three categories for Remove everything', async () => {
      const store = showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      fireEvent.click(await screen.findByRole('button', { name: EVERYTHING }));

      expect([...store.getState().selection.categories.keys()]).toEqual([
        'app-leftovers.caches',
        'app-leftovers.settings',
        'app-leftovers.data',
      ]);
    });

    it('selects only the categories of the choice that hold a finding', async () => {
      const store = showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.map((total) => (total.category === 'app-leftovers.data' ? { ...total, count: 0 } : total)),
      );

      fireEvent.click(await screen.findByRole('button', { name: EVERYTHING }));

      expect([...store.getState().selection.categories.keys()]).toEqual([
        'app-leftovers.caches',
        'app-leftovers.settings',
      ]);
    });
  });
});
