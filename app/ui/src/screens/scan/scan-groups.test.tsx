import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CategoryTotal, DangerLevel } from '../../ipc/bindings.generated';

import { mockCore, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_MODULES, DEMO_OVERVIEW } from '../../test/demo-scan';
import { STORY_DICTIONARY } from '../../test/story-data';
import { AppProviders } from '../app-providers';
import { ScanGroups } from './scan-groups';

const total = (category: string, highestDanger: DangerLevel): CategoryTotal => ({
  category,
  count: 2,
  highestDanger,
  sizes: { allocated: 1_000_000, completeness: 'complete', logical: 1_000_000, reclaimable: 1_000_000 },
});

// One category per level but High: caches (Safe), downloads (Low), images (Medium).
const TOTALS: readonly CategoryTotal[] = [
  total('demo.images', 'medium'),
  total('demo.downloads', 'low'),
  total('demo.caches', 'safe'),
];

// The groups of `totals`, once the core's texts have arrived.
const renderGroups = async (totals: readonly CategoryTotal[] = TOTALS): Promise<{ readonly store: CoreStore }> => {
  const store = createCoreStore();

  mockCore({
    ...sampleAgreementAnswers,
    getDictionary: () => STORY_DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => DEMO_OVERVIEW,
    ...sampleStorageAnswers,
  });
  render(
    <AppProviders store={store}>
      <ScanGroups access={new Map()} totals={totals} unavailable={[]} />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });

  return { store };
};

const group = (name: string): HTMLElement => screen.getByRole('region', { name });

describe('ScanGroups', () => {
  describe('negative cases', () => {
    it('an empty level has no group', async () => {
      await renderGroups([total('demo.caches', 'safe')]);

      expect(screen.getAllByRole('heading', { level: 2 }).map(({ textContent }) => textContent)).toStrictEqual([
        'Safe to remove',
      ]);
      expect(screen.queryByRole('region', { name: 'Usually safe' })).toBeNull();
    });

    it('Look before removing has no Select all, and ticking its tile selects none of its items', async () => {
      const { store } = await renderGroups();
      const look = group('Look before removing');

      expect(within(look).queryByRole('checkbox', { name: 'Select all' })).toBeNull();

      fireEvent.click(within(look).getByRole('checkbox', { name: 'Select Example images' }));

      expect(store.getState().selection.categories.size).toBe(0);
      expect(store.getState().selection.findings.size).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('groups come Safe, Usually safe, Look before removing, each with its core name and line', async () => {
      await renderGroups();

      expect(screen.getAllByRole('heading', { level: 2 }).map(({ textContent }) => textContent)).toStrictEqual([
        'Safe to remove',
        'Usually safe',
        'Look before removing',
      ]);
      expect(within(group('Safe to remove')).getByText('· Apps make these again on their own')).toBeInTheDocument();
      expect(within(group('Usually safe')).getByText('· You may need to download them again')).toBeInTheDocument();
      expect(
        within(group('Look before removing')).getByText('· May hold something you still want'),
      ).toBeInTheDocument();
    });

    it('a Look tile ticked reads checked and brought to Review, and unticks again', async () => {
      const { store } = await renderGroups();
      const tile = (): HTMLElement =>
        within(group('Look before removing')).getByRole('checkbox', { name: 'Select Example images' });

      fireEvent.click(tile());

      expect(tile()).toBeChecked();
      expect([...store.getState().selection.toCheck]).toStrictEqual(['demo.images']);

      fireEvent.click(tile());

      expect(tile()).not.toBeChecked();
      expect(store.getState().selection.toCheck.size).toBe(0);
    });

    it('Select all of a group selects its tiles', async () => {
      const { store } = await renderGroups();
      const safe = group('Safe to remove');

      fireEvent.click(within(safe).getByRole('checkbox', { name: 'Select all' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(within(safe).getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
    });
  });
});
