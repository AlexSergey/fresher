import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { CategoryId, LeftOutCount, Overview } from '../../ipc/bindings.generated';

import { mockCore, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_DICTIONARY, DEMO_LEFT_OUT, DEMO_MODULES, DEMO_OVERVIEW, SCAN_STARTED } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { LeftOutOnly } from './left-out-only';

const DOWNLOADS_LEFT_OUT: LeftOutCount = { ...DEMO_LEFT_OUT, category: 'demo.downloads' };

// The demo scan with only the caches counted: the downloads were all left out.
const CACHES_ONLY: Overview = { ...DEMO_OVERVIEW, totals: DEMO_OVERVIEW.totals.slice(0, 1) };

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore };

// The section among the given cards, once the dictionary has arrived and the demo scan finished leaving `leftOut` out.
const renderSection = async (
  leftOut: readonly LeftOutCount[],
  cards: readonly CategoryId[] = ['demo.caches'],
  overview: Overview = CACHES_ONLY,
): Promise<Rendered> => {
  const navigation = createNavigation();
  const store = createCoreStore();
  const section = (): React.JSX.Element => (
    <AppProviders navigation={navigation} store={store}>
      <LeftOutOnly cards={cards} />
    </AppProviders>
  );

  mockCore({
    getDictionary: () => DEMO_DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => overview,
    ...sampleStorageAnswers,
  });
  const { rerender } = render(section());
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });
  act(() => {
    store.dispatch(SCAN_STARTED);
    store.dispatch({
      leftOut: [...leftOut],
      module: 'demo',
      outcome: { kind: 'completed' },
      scan: 1,
      type: 'module-finished',
    });
    store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });
    store.dispatch({ overview, type: 'overview-loaded' });
  });
  rerender(section());

  return { navigation, store };
};

const section = (): HTMLElement | null => screen.queryByRole('region', { name: 'Found, but left out' });

describe('LeftOutOnly', () => {
  describe('negative cases', () => {
    it('shows nothing before a scan', () => {
      mockCore({ getDictionary: () => DEMO_DICTIONARY, getModules: () => DEMO_MODULES, ...sampleStorageAnswers });

      render(
        <AppProviders store={createCoreStore()}>
          <LeftOutOnly cards={[]} />
        </AppProviders>,
      );

      expect(section()).toBeNull();
    });

    it('shows nothing when every category with left-outs has a card', async () => {
      await renderSection([DEMO_LEFT_OUT]);

      expect(section()).toBeNull();
    });

    it("shows nothing for a hidden module's left-outs", async () => {
      await renderSection([DOWNLOADS_LEFT_OUT], [], {
        ...CACHES_ONLY,
        hiddenModules: [{ by: 'module', module: 'demo' }],
        totals: [],
      });

      expect(section()).toBeNull();
    });
    it('shows no left-out section while a scan runs', async () => {
      const { store } = await renderSection([DOWNLOADS_LEFT_OUT]);

      act(() => {
        store.dispatch({ modules: ['demo'], scan: 2, type: 'scan-started' });
        store.dispatch({
          leftOut: [DOWNLOADS_LEFT_OUT],
          module: 'demo',
          outcome: { kind: 'completed' },
          scan: 2,
          type: 'module-finished',
        });
      });

      expect(store.getState().modules).toHaveLength(1);
      expect(section()).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('lists a category whose items were all left out, with its left-out lines', async () => {
      await renderSection([DEMO_LEFT_OUT, DOWNLOADS_LEFT_OUT]);

      const rows = within(section() as HTMLElement).getAllByRole('listitem', { name: /^Example/u });
      expect(rows).toHaveLength(1);
      expect(rows[0]).toHaveAccessibleName('Example downloads');
      expect(rows[0]).toHaveTextContent('1 item left out (157.3 MB): The app that made it is open.');
      expect(screen.getByText('The scan found these, but left every item out. Each line says why.')).toBeVisible();
    });

    it("Details opens the category's details", async () => {
      const { navigation } = await renderSection([DOWNLOADS_LEFT_OUT]);

      fireEvent.click(within(section() as HTMLElement).getByRole('button', { name: 'Details' }));

      expect(navigation.getScreen()).toStrictEqual({ category: 'demo.downloads', kind: 'category' });
    });
  });
});
