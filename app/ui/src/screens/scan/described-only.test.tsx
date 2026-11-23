import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Insight, Overview } from '../../ipc/bindings.generated';

import { mockCore, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_INSIGHT, DEMO_INSIGHT_DICTIONARY, DEMO_INSIGHT_STOPPED } from '../../test/demo-insight';
import { DEMO_DICTIONARY, DEMO_MODULES, DEMO_OVERVIEW, SCAN_STARTED } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { DescribedOnly } from './described-only';

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore };

const finish = (store: CoreStore, module: string, insights: readonly Insight[], scan = 1): void => {
  store.dispatch({
    insights: [...insights],
    leftOut: [],
    module,
    outcome: { kind: 'completed' },
    scan,
    type: 'module-finished',
  });
};

// The section once the dictionary has arrived; the scan runs only when asked and the modules finish as given.
const renderSection = async (
  finished: readonly (readonly [string, readonly Insight[]])[],
  overview: Overview = DEMO_OVERVIEW,
): Promise<Rendered> => {
  const navigation = createNavigation();
  const store = createCoreStore();

  mockCore({
    ...sampleAgreementAnswers,
    getDictionary: () => ({ ...DEMO_DICTIONARY, ...DEMO_INSIGHT_DICTIONARY }),
    getModules: () => DEMO_MODULES,
    getOverview: () => overview,
    ...sampleStorageAnswers,
  });
  render(
    <AppProviders navigation={navigation} store={store}>
      <DescribedOnly />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });
  act(() => {
    store.dispatch(SCAN_STARTED);
    for (const [module, insights] of finished) {
      finish(store, module, insights);
    }
    store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });
  });
  await vi.waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
  });

  return { navigation, store };
};

const section = (): HTMLElement | null => screen.queryByRole('region', { name: 'Described, not cleaned' });

describe('DescribedOnly', () => {
  describe('negative cases', () => {
    it('renders nothing before a scan', () => {
      mockCore({
        ...sampleAgreementAnswers,
        getDictionary: () => DEMO_DICTIONARY,
        getModules: () => DEMO_MODULES,
        ...sampleStorageAnswers,
      });

      render(
        <AppProviders store={createCoreStore()}>
          <DescribedOnly />
        </AppProviders>,
      );

      expect(section()).toBeNull();
    });

    it('renders nothing while scanning', async () => {
      const { store } = await renderSection([['demo', [DEMO_INSIGHT]]]);

      act(() => {
        store.dispatch({ modules: ['demo'], scan: 2, type: 'scan-started' });
        finish(store, 'demo', [DEMO_INSIGHT], 2);
      });

      expect(store.getState().modules).toHaveLength(1);
      expect(section()).toBeNull();
    });

    it('renders nothing when no module described anything', async () => {
      await renderSection([['demo', []]]);

      expect(section()).toBeNull();
    });

    it("leaves out a hidden module's insights", async () => {
      await renderSection([['demo', [DEMO_INSIGHT]]], {
        ...DEMO_OVERVIEW,
        hiddenModules: [{ by: 'module', module: 'demo' }],
      });

      expect(section()).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('shows one card per insight in module order', async () => {
      await renderSection([
        ['demo', [DEMO_INSIGHT]],
        ['other', [DEMO_INSIGHT_STOPPED]],
      ]);

      const titles = within(section() as HTMLElement)
        .getAllByRole('heading', { level: 3 })
        .map((heading) => heading.textContent);
      expect(titles).toStrictEqual(['Example tool', 'Example tool (stopped)']);
      expect(
        screen.getByText('Fresher shows what these take and how you can free it yourself. It never removes any of it.'),
      ).toBeVisible();
    });

    it("Details opens the insight's screen", async () => {
      const { navigation } = await renderSection([['demo', [DEMO_INSIGHT]]]);

      fireEvent.click(within(section() as HTMLElement).getByRole('button', { name: 'Details' }));

      expect(navigation.getScreen()).toStrictEqual({ identity: 'demo-tool', kind: 'insight', module: 'demo' });
    });
  });
});
