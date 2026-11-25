import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Finding, FindingsQuery, Overview } from '../../ipc/bindings.generated';

import { mockCore, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore, useCore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { DEMO_FINDINGS, DEMO_MODULES, DEMO_OVERVIEW } from '../../test/demo-scan';
import { FINDING_DICTIONARY, findingOf } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { reviewGroups } from './review-groups';
import { ReviewTile } from './review-tile';

// A "Look before removing" category of two findings.
const OLD_SHOP = findingOf({ bytes: 800_000_000, category: 'demo.builds', id: 21, level: 'medium', name: 'old-shop' });
const BLOG = findingOf({ bytes: 400_000_000, category: 'demo.builds', id: 22, level: 'medium', name: 'blog-2023' });

const FINDINGS: readonly Finding[] = [...DEMO_FINDINGS, OLD_SHOP, BLOG];

const OVERVIEW: Overview = {
  ...DEMO_OVERVIEW,
  totals: [
    ...DEMO_OVERVIEW.totals,
    {
      category: 'demo.builds',
      count: 2,
      highestDanger: 'medium',
      sizes: { allocated: 1_200_000_000, completeness: 'complete', logical: 1, reclaimable: 1_200_000_000 },
    },
  ],
};

const NO_TOTALS: Overview['totals'] = [];

// The store's tiles, as Review shows them, ticked and unticked through the store.
const Tiles = ({ store }: { readonly store: CoreStore }): React.JSX.Element => {
  const selection = useCore((state) => state.selection);
  const totals = useCore((state) => state.overview?.totals ?? NO_TOTALS);

  return (
    <>
      {reviewGroups(selection, totals).map((group) => (
        <ReviewTile
          group={group}
          home="/demo"
          icon="project"
          key={group.category}
          onAll={(category, ticked) => {
            store.dispatch({ category, type: ticked ? 'tick-all' : 'untick-all' });
          }}
          onToggle={(finding) => {
            store.dispatch({ finding, type: 'toggle-finding' });
          }}
        />
      ))}
    </>
  );
};

// The tiles of what `select` picks, once the core's dictionary and overview have arrived.
const showTiles = async (select: (store: CoreStore) => void): Promise<CoreStore> => {
  mockCore({
    ...sampleAgreementAnswers,
    ...sampleStorageAnswers,
    getDictionary: () => ({
      ...FINDING_DICTIONARY,
      ...STORAGE_DICTIONARY,
      'demo.builds.name': 'Builds of old projects',
    }),
    getFindings: (args) => pageOf(FINDINGS, (args as { query: FindingsQuery }).query),
    getModules: () => DEMO_MODULES,
    getOverview: () => OVERVIEW,
    subscribe: () => null,
  });
  const store = createCoreStore();
  render(
    <AppProviders store={store}>
      <Tiles store={store} />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
  });
  act(() => {
    select(store);
  });

  return store;
};

const toggleCategory =
  (category: string) =>
  (store: CoreStore): void => {
    store.dispatch({ category, type: 'toggle-category' });
  };

const builds = (): HTMLElement => screen.getByRole('article', { name: 'Builds of old projects' });

const allBuilds = (): HTMLElement => screen.getByRole('checkbox', { name: 'All Builds of old projects' });

beforeEach(async () => {
  fakeViewport(720);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('ReviewTile', () => {
  describe('negative cases', () => {
    it("a Look tile's items start unticked", async () => {
      await showTiles(toggleCategory('demo.builds'));

      expect(await screen.findByRole('checkbox', { name: 'Select Example: old-shop' })).not.toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: blog-2023' })).not.toBeChecked();
      expect(allBuilds()).not.toBeChecked();
      expect(within(builds()).getByText('0 of 2 checked')).not.toHaveAttribute('data-all');
      expect(within(builds()).getByText('~/old-shop')).toBeInTheDocument();
    });

    it('an untick-all keeps a Look category in the list', async () => {
      const store = await showTiles(toggleCategory('demo.builds'));

      fireEvent.click(allBuilds());
      fireEvent.click(allBuilds());

      expect(builds()).toBeInTheDocument();
      expect(allBuilds()).not.toBeChecked();
      expect(store.getState().selection.toCheck.has('demo.builds')).toBe(true);
      expect(store.getState().selection.categories.has('demo.builds')).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('the pill reads checked of count and turns green when all', async () => {
      await showTiles(toggleCategory('demo.builds'));

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select Example: old-shop' }));

      expect(within(builds()).getByText('1 of 2 checked')).not.toHaveAttribute('data-all');
      expect(allBuilds()).toBePartiallyChecked();

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example: blog-2023' }));

      expect(within(builds()).getByText('2 of 2 checked')).toHaveAttribute('data-all', 'true');
    });

    it('the all-checkbox ticks the category whole', async () => {
      const store = await showTiles(toggleCategory('demo.builds'));

      fireEvent.click(allBuilds());

      expect(store.getState().selection.categories.get('demo.builds')).toStrictEqual(new Map());
      expect(store.getState().selection.toCheck.has('demo.builds')).toBe(false);
      expect(allBuilds()).toBeChecked();
      expect(await screen.findByRole('checkbox', { name: 'Select Example: old-shop' })).toBeChecked();
    });

    it("a row's menu lists its items in order with its separators", async () => {
      await showTiles(toggleCategory('demo.builds'));

      fireEvent.click(await screen.findByRole('button', { name: 'Options for Example: old-shop' }));
      // eslint-disable-next-line testing-library/no-node-access -- the order of items and separators is the point
      const entries = [...screen.getByRole('menu').querySelectorAll('[role="menuitem"], [role="separator"]')];

      expect(
        entries.map((entry) =>
          entry.getAttribute('role') === 'separator' ? '-' : entry.firstElementChild?.textContent,
        ),
      ).toStrictEqual(['Show in Finder', '-', 'Ignore this item', 'Ignore all Builds of old projects']);
    });

    it("a Safe tile's line names its group and its meaning", async () => {
      await showTiles(toggleCategory('demo.caches'));

      const caches = screen.getByRole('article', { name: 'Example caches' });

      expect(within(caches).getByText('Safe to remove · Apps make these again on their own')).toBeInTheDocument();
      expect(within(caches).queryByText(/tick each one you have checked/u)).toBeNull();
    });

    it("a Look tile's line asks to tick each item", async () => {
      await showTiles(toggleCategory('demo.builds'));

      expect(within(builds()).getByText('Look before removing · tick each one you have checked')).toBeInTheDocument();
    });
  });
});
