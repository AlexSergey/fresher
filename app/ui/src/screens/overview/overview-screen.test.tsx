import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Access, NodeId, Overview, PlaceView, StorageMap } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { DESCENT, HOME } from '../../product/place-list/story-data';
import { createNavigation, type Navigation, placeTop } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  allFiles,
  BUSY,
  DEMO_DICTIONARY,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SCAN,
  FIRST_BATCH,
  FULL_COVERAGE,
  IDLE,
  SCAN_STARTED,
} from '../../test/demo-scan';
import {
  DEMO_MAP_COMPLETE,
  DEMO_MAP_NO_SCAN,
  DEMO_MAP_SCANNING,
  DEMO_MEANING,
  EMPTY_PLACE,
  STORAGE_DICTIONARY,
} from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { OverviewScreen } from './overview-screen';

const DICTIONARY = { ...DEMO_DICTIONARY, ...STORAGE_DICTIONARY, 'demo.error.failed': 'The demo folder vanished.' };

const PARTIAL: Overview = {
  ...DEMO_OVERVIEW,
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'partial' },
    scan: 1,
    settingsChanged: false,
  },
};

const FAILED: Overview = {
  ...IDLE,
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'failed', reason: { args: {}, key: 'demo.error.failed' } },
    scan: 1,
    settingsChanged: false,
  },
};

// The overview during scan 1, with only the downloads counted so far.
const SCANNING: Overview = {
  found: { allocated: 750_000_000, completeness: 'complete', logical: 750_000_000, reclaimable: 750_000_000 },
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: null,
  status: { scan: 1, state: 'scanning' },
  totals: DEMO_OVERVIEW.totals.filter(({ category }) => category === 'demo.downloads'),
};

const core = (answers: Partial<CoreAnswers> = {}, access: Access = 'granted'): void => {
  mockCore({
    getAccessStatus: () => allFiles(access),
    getDictionary: () => DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    getStorageMap: () => DEMO_MAP_NO_SCAN,
    getStorageMeaning: () => DEMO_MEANING,
    getStoragePlace: () => EMPTY_PLACE,
    ...answers,
  });
};

const GB = 1_000_000_000;

// The scanning map with `gigabytes` more explained, taken from what is not scanned yet: the sum holds.
const fill = (map: StorageMap, gigabytes: number): StorageMap => ({
  ...map,
  volumes: map.volumes.map((volume) => ({
    ...volume,
    parts: volume.parts.map((part) => {
      const delta =
        part.kind.kind === 'explained' ? gigabytes * GB : part.kind.kind === 'not-scanned-yet' ? -gigabytes * GB : 0;

      return { ...part, sizes: { ...part.sizes, allocated: part.sizes.allocated + delta } };
    }),
  })),
});

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore; readonly unmount: () => void };

// The overview, once the core has answered its dictionary, its modules and its overview.
const renderOverview = async (navigation: Navigation = createNavigation()): Promise<Rendered> => {
  const store = createCoreStore();

  const { unmount } = render(
    <AppProviders navigation={navigation} store={store}>
      <OverviewScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });

  return { navigation, store, unmount };
};

// The cards' visible names, in order; each card's accessible name reads its name as "Open …".
const cardNames = (): readonly string[] =>
  within(screen.getByRole('list', { name: 'Categories' }))
    .getAllByRole('button', { name: /^Open/ })
    .map((button) => within(button).getAllByText(/./u, { selector: '[aria-hidden="true"]' })[0]?.textContent ?? '');

// The places of the demo volume: its root is the place list's home, its Library the descent; a node of another
// state is stale.
const placeOf = (args: unknown): PlaceView => {
  const { node } = args as { readonly node: NodeId };

  if (node.state !== 1) {
    throw new Error('stale');
  }
  const place = node.index === 1 ? DESCENT : HOME;

  return { offset: 0, path: [...place.path], rows: [...place.rows], total: place.rows.length };
};

const STALE = {
  error: { code: 'stale-node' },
  text: { args: {}, key: 'core.error.stale-node' },
} as const;

// The breadcrumb's steps, as they read.
const crumbs = (): readonly string[] =>
  within(screen.getByRole('navigation', { name: 'Where you are' }))
    .getAllByRole('listitem')
    .map(({ textContent }) => textContent.replace('›', ''));

beforeEach(async () => {
  await setMotion('off');
  fakeViewport(440);
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('OverviewScreen', () => {
  describe('negative cases', () => {
    it('says nothing of ignored things or a rescan when there are none', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      await renderOverview();

      expect(screen.queryByText(/you ignored/u)).toBeNull();
      expect(screen.queryByText('Rescan to apply')).toBeNull();
    });

    it('before a scan there are no cards and no zero sizes', async () => {
      core();

      await renderOverview();

      expect(screen.getByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
      expect(screen.getAllByRole('button', { name: /Scan/ })).toHaveLength(1);
      expect(screen.getByText(/It changes nothing on your disk\./)).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Categories' })).toBeNull();
      expect(screen.queryByText(/\b0 B\b/)).toBeNull();
    });

    it('before a scan the map shows the volume only', async () => {
      core();

      await renderOverview();

      expect(await screen.findByRole('heading', { level: 2, name: 'Macintosh HD · internal' })).toBeInTheDocument();
      expect(screen.getByRole('img', { name: /^Space on Macintosh HD: Not scanned yet 364 GB/u })).toBeInTheDocument();
      expect(screen.queryByRole('radiogroup', { name: 'Show the space' })).toBeNull();
      expect(screen.queryByRole('list', { name: 'Places' })).toBeNull();
    });

    it('a partial scan shows the notice', async () => {
      core({ getOverview: () => PARTIAL });

      await renderOverview();

      expect(await screen.findByText('This scan is incomplete')).toBeInTheDocument();
      expect(screen.getByText(/sizes are at least what is shown/)).toBeInTheDocument();
    });

    it('a finished scan shows no notice', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      expect(await screen.findByRole('list', { name: 'Categories' })).toBeInTheDocument();
      expect(screen.queryByText('This scan is incomplete')).toBeNull();
    });

    it.each<Access>(['granted', 'unknown'])('a %s access shows no permission card', async (access) => {
      const probed = vi.fn(() => allFiles(access));
      core({ getAccessStatus: probed });

      await renderOverview();

      await waitFor(() => {
        expect(probed).toHaveBeenCalledOnce();
      });
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
      expect(screen.queryByText('Full Disk Access is not confirmed')).toBeNull();
    });

    it('a refused access probe shows no permission card and leaves the refusal to the frame', async () => {
      core({ getAccessStatus: refuse(BUSY) });

      const { store } = await renderOverview();

      await waitFor(() => {
        expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: BUSY });
      });
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
    });

    it("a failed scan shows the core's reason", async () => {
      core({ getOverview: () => FAILED });

      await renderOverview();

      expect(await screen.findByText('The demo folder vanished.')).toBeInTheDocument();
      expect(screen.getByText('The scan could not finish')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Categories' })).toBeNull();
    });

    it('a stale place reopens the root', async () => {
      const navigation = createNavigation();
      navigation.showOverviewView({ ...placeTop(0), node: { index: 7, state: 0 }, trail: [...DESCENT.path] });
      core({
        getOverview: () => DEMO_OVERVIEW,
        getStorageMap: () => DEMO_MAP_COMPLETE,
        getStoragePlace: (args) =>
          (args as { readonly node: NodeId }).node.state === 1 ? placeOf(args) : refuse(STALE)(),
      });

      await renderOverview(navigation);

      await waitFor(() => {
        expect(crumbs()).toStrictEqual(['home']);
      });
      expect(screen.getByRole('listitem', { name: /^Library,/u })).toBeInTheDocument();
    });

    it('while a scan runs, the place view says the core is busy', async () => {
      core({ getOverview: () => SCANNING, getStorageMap: () => DEMO_MAP_COMPLETE, getStoragePlace: refuse(BUSY) });
      const navigation = createNavigation();
      navigation.showOverviewView(placeTop(0));

      await renderOverview(navigation);

      expect(await screen.findByText('The list by place could not be loaded')).toBeInTheDocument();
    });

    it("a failed scan offers one way to scan again, the hero's Try again", async () => {
      const startScan = vi.fn(() => 2);
      core({ getOverview: () => FAILED, startScan });
      await renderOverview();

      fireEvent.click(await screen.findByRole('button', { name: 'Try again' }));

      expect(screen.queryByRole('button', { name: 'Scan again' })).toBeNull();
      await waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
    });
  });

  describe('positive cases', () => {
    it('counts what is ignored and opens the settings to show it', async () => {
      const lastScan = DEMO_OVERVIEW.lastScan === null ? null : { ...DEMO_OVERVIEW.lastScan, settingsChanged: true };
      core({
        getOverview: () => ({
          ...DEMO_OVERVIEW,
          ignored: { categories: 1, groups: 1, modules: 2, paths: 1 },
          lastScan,
        }),
      });
      const { navigation } = await renderOverview();

      expect(await screen.findByText('5 things you ignored are left out of these results.')).toBeInTheDocument();
      expect(screen.getByText('Rescan to apply')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Show in Settings' }));

      expect(navigation.getScreen()).toEqual({ kind: 'settings', tab: 'groups' });
    });

    it('the bar fills while scanning', async () => {
      core({ getOverview: () => SCANNING, getStorageMap: () => DEMO_MAP_SCANNING });
      const { store } = await renderOverview();
      const bar = async (name: RegExp): Promise<HTMLElement> => screen.findByRole('img', { name });

      expect(await bar(/^Space on Macintosh HD: Explained 2\.3 GB, Not scanned yet 361\.8 GB/u)).toBeInTheDocument();

      act(() => {
        store.dispatch({ storageMap: fill(DEMO_MAP_SCANNING, 120), type: 'storage-map-loaded' });
      });
      expect(await bar(/^Space on Macintosh HD: Explained 122\.3 GB, Not scanned yet 241\.8 GB/u)).toBeInTheDocument();

      act(() => {
        store.dispatch({ storageMap: fill(DEMO_MAP_SCANNING, 300), type: 'storage-map-loaded' });
      });
      expect(await bar(/^Space on Macintosh HD: Explained 302\.3 GB, Not scanned yet 61\.8 GB/u)).toBeInTheDocument();
    });

    it('two volumes show two bars, home first', async () => {
      const home = DEMO_MAP_COMPLETE.volumes[0];
      const archive = home === undefined ? [] : [{ ...home, kind: 'external' as const, name: 'Archive' }];
      core({
        getOverview: () => DEMO_OVERVIEW,
        getStorageMap: () => ({ ...DEMO_MAP_COMPLETE, volumes: [...DEMO_MAP_COMPLETE.volumes, ...archive] }),
      });

      await renderOverview();

      await waitFor(() => {
        expect(screen.getAllByRole('heading', { level: 2 }).map(({ textContent }) => textContent)).toStrictEqual([
          'Macintosh HD · internal',
          'Archive · external',
        ]);
      });
      expect(screen.getByRole('img', { name: /^Space on Macintosh HD/u })).toBeVisible();
      expect(screen.getByRole('button', { name: 'Archive · external' })).toHaveAttribute('aria-expanded', 'false');
    });

    it('after a scan the view by meaning lists the places, and a place opens the place view', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE });
      const { navigation } = await renderOverview();

      expect(screen.getByRole('radio', { name: 'By meaning' })).toBeChecked();
      fireEvent.click(await screen.findByRole('button', { name: 'Look inside Movies' }));

      expect(navigation.getOverviewView()).toStrictEqual({ ...placeTop(0), node: { index: 8, state: 1 } });
      expect(screen.getByRole('radio', { name: 'By place' })).toBeChecked();
      expect(screen.queryByRole('list', { name: 'Categories' })).toBeNull();
    });

    it('the chosen view is kept in the navigation state', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE });
      const { navigation } = await renderOverview();

      fireEvent.click(await screen.findByRole('radio', { name: 'By place' }));
      expect(navigation.getOverviewView()).toStrictEqual(placeTop(0));

      fireEvent.click(screen.getByRole('radio', { name: 'By meaning' }));
      expect(navigation.getOverviewView()).toStrictEqual({ kind: 'meaning' });
      expect(await screen.findByRole('list', { name: 'Categories' })).toBeInTheDocument();
    });

    it('switching views keeps the total', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE, getStoragePlace: placeOf });
      await renderOverview();
      await screen.findByRole('list', { name: 'Categories' });
      const bar = screen.getByRole('img', { name: /^Space on Macintosh HD/u });
      const total = bar.getAttribute('aria-label');

      fireEvent.click(screen.getByRole('radio', { name: 'By place' }));
      await screen.findByRole('listitem', { name: /^Library,/u });

      expect(screen.getByRole('img', { name: /^Space on Macintosh HD/u })).toHaveAttribute('aria-label', total);
    });

    it('by place opens the home and a folder row opens it, the way down kept', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE, getStoragePlace: placeOf });
      const { navigation } = await renderOverview();

      fireEvent.click(await screen.findByRole('radio', { name: 'By place' }));
      fireEvent.click(await screen.findByRole('listitem', { name: /^Library,/u }));

      await waitFor(() => {
        expect(crumbs()).toStrictEqual(['home', 'Library', 'Containers', 'com.example.app']);
      });
      expect(navigation.getOverviewView()).toMatchObject({ descent: 'through', node: { index: 1, state: 1 } });

      fireEvent.click(screen.getByRole('button', { name: 'Back to Containers' }));
      expect(navigation.getOverviewView()).toMatchObject({ descent: 'stay', node: { index: 11, state: 1 } });
    });

    it('by place names a temporary folder walked beside the home by its own name', async () => {
      const tmp: NodeId = { index: 30, state: 1 };
      const map: StorageMap = {
        ...DEMO_MAP_COMPLETE,
        volumes: DEMO_MAP_COMPLETE.volumes.map((volume) => ({
          ...volume,
          places: [{ name: { args: {}, key: 'core.place.shared-temp.name' }, root: tmp }],
          roots: [...volume.roots, tmp],
        })),
      };
      const tmpFolder = { ...HOME.folder, id: tmp, name: 'tmp' };
      const placeWithTmp = (args: unknown): PlaceView => {
        const { node } = args as { readonly node: NodeId };

        return node.index === tmp.index ? { offset: 0, path: [tmpFolder], rows: [], total: 0 } : placeOf(args);
      };
      core({
        getDictionary: () => ({ ...DICTIONARY, 'core.place.shared-temp.name': 'Shared temporary folder' }),
        getOverview: () => DEMO_OVERVIEW,
        getStorageMap: () => map,
        getStoragePlace: placeWithTmp,
      });
      await renderOverview();

      fireEvent.click(await screen.findByRole('radio', { name: 'By place' }));

      const roots = await screen.findByRole('list', { name: 'Scanned folders' });
      expect(
        within(roots)
          .getAllByRole('listitem')
          .map(({ textContent }) => textContent),
      ).toStrictEqual([expect.stringContaining('home'), expect.stringContaining('Shared temporary folder')]);
      expect(within(roots).queryByText('tmp')).toBeNull();
    });

    it('the opened folder survives leaving the overview', async () => {
      const getPlace = vi.fn(placeOf);
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE, getStoragePlace: getPlace });
      const { navigation, unmount } = await renderOverview();
      fireEvent.click(await screen.findByRole('radio', { name: 'By place' }));
      fireEvent.click(await screen.findByRole('listitem', { name: /^Library,/u }));
      await waitFor(() => {
        expect(crumbs()).toHaveLength(4);
      });

      unmount();
      navigation.navigate({ kind: 'settings' });
      navigation.navigate({ kind: 'overview' });
      await renderOverview(navigation);

      await waitFor(() => {
        expect(crumbs()).toStrictEqual(['home', 'Library', 'Containers', 'com.example.app']);
      });
      expect(screen.getByRole('radio', { name: 'By place' })).toBeChecked();
      expect(getPlace).toHaveBeenLastCalledWith(expect.objectContaining({ node: { index: 1, state: 1 } }));
    });

    it('Scan starts a scan and the hero shows scanning', async () => {
      const startScan = vi.fn(() => 1);
      core({ startScan });
      await renderOverview();

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
      await waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
      act(() => {
        pushEvents(SCAN_STARTED);
      });

      expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
      expect(screen.getByRole('group', { name: 'Parts of the scan' })).toHaveTextContent('Demo module');
      expect(screen.queryByRole('button', { name: 'Scan again' })).toBeNull();
    });

    it('cards appear as totals arrive, largest first', async () => {
      let overview = SCANNING;
      core({ getOverview: () => overview });
      await renderOverview();

      expect(await screen.findByRole('button', { name: /^Open Example downloads/u })).toBeInTheDocument();

      overview = DEMO_OVERVIEW;
      act(() => {
        pushEvents(...DEMO_SCAN.slice(2));
      });

      await waitFor(() => {
        expect(cardNames()).toStrictEqual(['Example caches', 'Example downloads']);
      });
    });

    it('cards arrive during a scan the events started', async () => {
      let overview = IDLE;
      core({ getOverview: () => overview });
      await renderOverview();

      overview = SCANNING;
      act(() => {
        pushEvents(SCAN_STARTED, FIRST_BATCH);
      });

      expect(await screen.findByRole('button', { name: /^Open Example downloads/u })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
    });

    it("opening a card goes to its module's page at that category", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation } = await renderOverview();

      fireEvent.click(await screen.findByRole('button', { name: /^Open Example caches/u }));

      expect(navigation.getScreen()).toStrictEqual({ category: 'demo.caches', kind: 'module', module: 'demo' });
    });

    it('selecting a card selects its category', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { store } = await renderOverview();

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select Example caches' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(screen.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
    });

    it('a denied access shows the permission card', async () => {
      core({}, 'denied');

      await renderOverview();

      expect(await screen.findByText('Fresher cannot see every folder')).toBeInTheDocument();
    });

    it('Scan again starts a new scan after a scan', async () => {
      const startScan = vi.fn(() => 2);
      core({ getOverview: () => DEMO_OVERVIEW, startScan });
      await renderOverview();

      fireEvent.click(await screen.findByRole('button', { name: 'Scan again' }));

      await waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
    });
  });
});
