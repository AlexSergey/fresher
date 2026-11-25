import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CategoryId, ModuleInfo } from '../../ipc/bindings.generated';

import { mockCore, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import {
  APP_LEFTOVERS_DICTIONARY,
  APP_LEFTOVERS_MODULES,
  APP_LEFTOVERS_OVERVIEW,
  APP_LEFTOVERS_TOTALS,
} from '../../test/app-leftovers-scan';
import { DEMO_MODULES } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { ModuleChoices } from './module-choices';

const [APP_LEFTOVERS] = APP_LEFTOVERS_MODULES as readonly [ModuleInfo];
const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const text = (key: string): string => APP_LEFTOVERS_DICTIONARY[`app-leftovers.${key}`] ?? '';

const EVERYTHING = text('choice.everything.label');
const KEEP_SETTINGS = text('choice.keep-settings.label');

const showChoices = async (
  module: ModuleInfo,
  totals: typeof APP_LEFTOVERS_TOTALS,
  category: CategoryId = 'app-leftovers.caches',
): Promise<CoreStore> => {
  const store = createCoreStore();
  // The totals tell each category's level: Settings (Medium) and Data (High) are "Look before removing".
  store.dispatch({ overview: { ...APP_LEFTOVERS_OVERVIEW, totals }, type: 'overview-loaded' });
  const navigation = createNavigation();
  const choices = (): React.JSX.Element => (
    <AppProviders navigation={navigation} store={store}>
      <ModuleChoices category={category} module={module} totals={totals} />
    </AppProviders>
  );

  mockCore({
    ...sampleAgreementAnswers,
    getDictionary: () => APP_LEFTOVERS_DICTIONARY,
    getModules: () => [module],
    ...sampleStorageAnswers,
  });
  const { rerender } = render(choices());
  // The screens render once the core's dictionary is read, as the app's frame does.
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });
  rerender(choices());

  return store;
};

const nothingFound = APP_LEFTOVERS_TOTALS.map((total) => ({ ...total, count: 0 }));

describe('ModuleChoices', () => {
  describe('negative cases', () => {
    it('shows no choice that does not touch the category', async () => {
      await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS, 'app-leftovers.data');

      expect(await screen.findByRole('button', { name: EVERYTHING })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: KEEP_SETTINGS })).toBeNull();
    });

    it('renders nothing when no choice touches the category', async () => {
      await showChoices(
        { ...APP_LEFTOVERS, choices: APP_LEFTOVERS.choices.filter(({ name }) => name === 'keep-settings') },
        APP_LEFTOVERS_TOTALS,
        'app-leftovers.data',
      );

      expect(screen.queryByRole('group')).toBeNull();
    });

    it('renders nothing for a module without choices', async () => {
      await showChoices(DEMO, [], 'demo.caches');

      expect(screen.queryByRole('button')).toBeNull();
      expect(screen.queryByRole('group')).toBeNull();
    });

    it('disables a choice whose categories hold no finding', async () => {
      await showChoices(APP_LEFTOVERS, nothingFound);

      expect(await screen.findByRole('button', { name: EVERYTHING })).toBeDisabled();
      expect(screen.getByRole('button', { name: KEEP_SETTINGS })).toBeDisabled();
    });

    it('disables the choice of categories without a finding and keeps the other enabled', async () => {
      await showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.map((total) =>
          total.category === 'app-leftovers.caches' ? { ...total, count: 0 } : total,
        ),
      );

      expect(await screen.findByRole('button', { name: KEEP_SETTINGS })).toBeDisabled();
      expect(screen.getByRole('button', { name: EVERYTHING })).toBeEnabled();
    });

    it('disables the choice of an ignored category, whose findings leave the totals, and selects nothing', async () => {
      const store = await showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.filter((total) => total.category !== 'app-leftovers.caches'),
      );

      const keepSettings = await screen.findByRole('button', { name: KEEP_SETTINGS });
      fireEvent.click(keepSettings);

      expect(keepSettings).toBeDisabled();
      expect(store.getState().selection.categories.size).toBe(0);
    });

    it('a module’s choice never selects a Look before removing category whole', async () => {
      const store = await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      fireEvent.click(await screen.findByRole('button', { name: EVERYTHING }));

      expect(store.getState().selection.categories.has('app-leftovers.settings')).toBe(false);
      expect(store.getState().selection.categories.has('app-leftovers.data')).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('shows every choice that touches the category', async () => {
      await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS, 'app-leftovers.caches');

      expect(await screen.findByRole('button', { name: EVERYTHING })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: KEEP_SETTINGS })).toBeInTheDocument();
    });

    it('shows each choice’s description beside its label', async () => {
      await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      await screen.findByRole('button', { name: EVERYTHING });

      expect(screen.getByText(text('choice.everything.description'))).toBeVisible();
      expect(screen.getByText(text('choice.keep-settings.description'))).toBeVisible();
    });

    it('selects exactly the categories of the clicked choice', async () => {
      const store = await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      fireEvent.click(await screen.findByRole('button', { name: KEEP_SETTINGS }));

      expect([...store.getState().selection.categories.keys()]).toEqual(['app-leftovers.caches']);
    });

    it('Remove everything selects the Safe category and brings the two others to Review to check', async () => {
      const store = await showChoices(APP_LEFTOVERS, APP_LEFTOVERS_TOTALS);

      fireEvent.click(await screen.findByRole('button', { name: EVERYTHING }));

      expect([...store.getState().selection.categories.keys()]).toEqual(['app-leftovers.caches']);
      expect([...store.getState().selection.toCheck]).toEqual(['app-leftovers.settings', 'app-leftovers.data']);
    });

    it('selects only the categories of the choice that hold a finding', async () => {
      const store = await showChoices(
        APP_LEFTOVERS,
        APP_LEFTOVERS_TOTALS.map((total) => (total.category === 'app-leftovers.data' ? { ...total, count: 0 } : total)),
      );

      fireEvent.click(await screen.findByRole('button', { name: EVERYTHING }));

      expect([...store.getState().selection.categories.keys()]).toEqual(['app-leftovers.caches']);
      expect([...store.getState().selection.toCheck]).toEqual(['app-leftovers.settings']);
    });
  });
});
