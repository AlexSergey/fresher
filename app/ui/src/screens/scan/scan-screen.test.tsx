import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Access, Overview, TrashView } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse, sampleAgreementAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { type WallClock } from '../../state/wall-clock';
import { setMotion } from '../../test/animation';
import { cardNames, cardOf } from '../../test/cards';
import {
  ACCESS_DICTIONARY,
  allFiles,
  BUSY,
  DEMO_DICTIONARY,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SCAN,
  FIRST_BATCH,
  FULL_COVERAGE,
  IDLE,
  LIMITED_COVERAGE,
  SCAN_STARTED,
} from '../../test/demo-scan';
import { createFakeClock } from '../../test/fake-clock';
import {
  DEMO_MAP_COMPLETE,
  DEMO_MAP_NO_SCAN,
  DEMO_MAP_SCANNING,
  DEMO_MEANING,
  STORAGE_DICTIONARY,
} from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { ScanScreen } from './scan-screen';

const DICTIONARY = {
  ...ACCESS_DICTIONARY,
  ...DEMO_DICTIONARY,
  ...STORAGE_DICTIONARY,
  'demo.error.failed': 'The demo folder vanished.',
};

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
    ...sampleAgreementAnswers,
    getAccessStatus: () => allFiles(access),
    getDictionary: () => DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    getStorageMap: () => DEMO_MAP_NO_SCAN,
    getStorageMeaning: () => DEMO_MEANING,
    ...answers,
  });
};

const GB = 1_000_000_000;

// The demo overview with a High category beside the Safe and Low ones.
const WITH_HIGH: Overview = {
  ...DEMO_OVERVIEW,
  totals: [
    ...DEMO_OVERVIEW.totals,
    {
      category: 'demo.projects',
      count: 1,
      highestDanger: 'high',
      sizes: { allocated: 2 * GB, completeness: 'complete', logical: 2 * GB, reclaimable: 2 * GB },
    },
  ],
};

const HIGH_DICTIONARY = {
  ...DICTIONARY,
  'core.danger.high.meaning': 'May hold the only copy of your data',
  'core.danger.high.name': 'High',
  'core.danger.high.zone': 'Only if you are sure',
  'demo.projects.name': 'Example projects',
};

// The overview with a High zone, and that zone.
const renderWithHigh = async (): Promise<{ readonly high: HTMLElement; readonly store: CoreStore }> => {
  core({ getDictionary: () => HIGH_DICTIONARY, getOverview: () => WITH_HIGH });
  const { store } = await renderOverview();

  return { high: await screen.findByRole('region', { name: 'Only if you are sure' }), store };
};

const selectedCategories = (store: CoreStore): readonly string[] => [...store.getState().selection.categories.keys()];

const TRASH: TrashView = {
  trashes: [
    {
      fromCleanups: { allocated: GB, completeness: 'complete', logical: GB, reclaimable: GB },
      goneBy: null,
      index: 0,
      volume: 'Macintosh HD',
      waiting: { allocated: 3 * GB, completeness: 'complete', logical: 3 * GB, reclaimable: 3 * GB },
    },
  ],
};

// The time of day of the tests: two minutes after the demo scan finished; its timers never run.
const FIXED_CLOCK: WallClock = {
  epochMs: () => ((DEMO_OVERVIEW.lastScan?.finished ?? 0) + 120) * 1000,
  now: () => 0,
  setTimeout: () => () => undefined,
};

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore; readonly unmount: () => void };

// The overview, once the core has answered its dictionary, its modules and its overview.
const renderOverview = async (
  navigation: Navigation = createNavigation(),
  wallClock: WallClock = FIXED_CLOCK,
): Promise<Rendered> => {
  const store = createCoreStore();

  const { unmount } = render(
    <AppProviders navigation={navigation} store={store} wallClock={wallClock}>
      <ScanScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });

  return { navigation, store, unmount };
};

beforeEach(async () => {
  await setMotion('off');
  fakeViewport(440);
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('ScanScreen', () => {
  describe('negative cases', () => {
    it('shows no Trash line when the core cannot tell what waits there', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      await screen.findByRole('list', { name: 'Safe to remove' });
      expect(screen.queryByRole('region', { name: 'The Trash' })).toBeNull();
    });

    it('shows no Trash line during a scan', async () => {
      core({ getOverview: () => SCANNING, getTrash: () => TRASH });

      await renderOverview();

      expect(screen.queryByRole('region', { name: 'The Trash' })).toBeNull();
    });

    it('says nothing of ignored things or a rescan when there are none', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      await renderOverview();

      expect(screen.queryByText(/you ignored/u)).toBeNull();
      expect(screen.queryByText('Scan again to apply it')).toBeNull();
    });

    it('before a scan there are no cards and no zero sizes', async () => {
      core();

      await renderOverview();

      expect(screen.getByRole('heading', { level: 1, name: 'Scan' })).toBeInTheDocument();
      expect(screen.getAllByRole('button', { name: /Scan/ })).toHaveLength(1);
      expect(screen.getByText(/It changes nothing on your disk\./)).toBeInTheDocument();
      expect(screen.queryByRole('region', { name: 'Safe to remove' })).toBeNull();
      expect(screen.queryByText(/\b0 B\b/)).toBeNull();
    });

    it('before a scan the disk card names the volume and its centre holds Scan', async () => {
      core();

      await renderOverview();

      const disk = await screen.findByRole('region', { name: 'Your disk' });
      expect(within(disk).getByText('Macintosh HD')).toBeInTheDocument();
      expect(within(disk).getByRole('button', { name: 'Scan' })).toBeInTheDocument();
      expect(within(disk).getByText('Not scanned yet')).toBeInTheDocument();
      expect(screen.queryByRole('radiogroup', { name: 'Show the space' })).toBeNull();
      expect(screen.queryByRole('list', { name: 'Places' })).toBeNull();
    });

    it('before a scan there is no separator and no group', async () => {
      core();

      await renderOverview();

      await screen.findByRole('region', { name: 'Your disk' });
      expect(screen.queryByText('What you can remove')).toBeNull();
      expect(screen.queryByRole('heading', { level: 2 })).toBeNull();
      expect(screen.queryByText(/ago$/u)).toBeNull();
    });

    it('before the core names a volume, Scan stands alone', async () => {
      core({ getStorageMap: refuse(BUSY) });

      await renderOverview();

      expect(screen.getByRole('button', { name: 'Scan' })).toBeInTheDocument();
      expect(screen.queryByRole('region', { name: 'Your disk' })).toBeNull();
    });

    it('a partial scan shows the notice', async () => {
      core({ getOverview: () => PARTIAL });

      await renderOverview();

      expect(await screen.findByText('This scan is not complete')).toBeInTheDocument();
      expect(screen.getByText(/the real sizes may be larger/)).toBeInTheDocument();
    });

    it('a finished scan shows no notice', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      expect(await screen.findByRole('list', { name: 'Safe to remove' })).toBeInTheDocument();
      expect(screen.queryByText('This scan is not complete')).toBeNull();
    });

    it.each<Access>(['granted', 'unknown'])('a %s access shows no permission card', async (access) => {
      const probed = vi.fn(() => allFiles(access));
      core({ getAccessStatus: probed });

      await renderOverview();

      await waitFor(() => {
        expect(probed).toHaveBeenCalledOnce();
      });
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
      expect(screen.queryByText('Fresher cannot tell if it has Full Disk Access')).toBeNull();
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
      expect(screen.queryByRole('region', { name: 'Safe to remove' })).toBeNull();
    });

    it('after a scan there is no view switch, no apps, no places and no rest of the disk', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE });

      await renderOverview();

      expect(await screen.findByRole('region', { name: 'Safe to remove' })).toBeInTheDocument();
      expect(screen.queryByRole('radiogroup')).toBeNull();
      expect(screen.queryByRole('radio', { name: 'By place' })).toBeNull();
      expect(screen.queryByRole('group', { name: 'Apps' })).toBeNull();
      expect(screen.queryByRole('list', { name: 'Places' })).toBeNull();
      expect(screen.queryByRole('list', { name: 'The rest of the disk' })).toBeNull();
      expect(screen.queryByRole('button', { name: /Look inside/u })).toBeNull();
    });

    it('shows no selection bar while nothing is selected', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      await screen.findByRole('region', { name: 'Safe to remove' });
      expect(screen.queryByRole('region', { name: 'Selection' })).toBeNull();
    });

    it('an Only if you are sure group has no Select all and its tile ticked selects none of its items', async () => {
      const { high, store } = await renderWithHigh();

      expect(within(high).queryByRole('checkbox', { name: 'Select all' })).toBeNull();
      expect(within(high).getByRole('heading', { level: 3, name: 'Example projects' })).toBeInTheDocument();
      expect(within(high).getByRole('button', { name: 'Details' })).toBeInTheDocument();

      fireEvent.click(within(high).getByRole('checkbox', { name: 'Select Example projects' }));

      expect(selectedCategories(store)).toStrictEqual([]);
      expect([...store.getState().selection.toCheck]).toStrictEqual(['demo.projects']);
      expect(screen.getByRole('region', { name: 'Selection' })).toHaveTextContent(/^1 to check in Review/u);
    });

    it('Select all of a Safe zone does not ask', async () => {
      const { store } = await renderWithHigh();

      fireEvent.click(
        within(screen.getByRole('region', { name: 'Safe to remove' })).getByRole('checkbox', { name: 'Select all' }),
      );

      expect(screen.queryByRole('dialog')).toBeNull();
      expect(selectedCategories(store)).toStrictEqual(['demo.caches']);
    });

    it("a Select all of the Safe zone leaves the other zones' cards unticked", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { store } = await renderOverview();
      const safe = await screen.findByRole('region', { name: 'Safe to remove' });

      fireEvent.click(within(safe).getByRole('checkbox', { name: 'Select all' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(screen.getByRole('checkbox', { name: 'Select Example downloads' })).not.toBeChecked();
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

    it('a tile found while scanning has no checkbox, Details or menu', async () => {
      core({ getOverview: () => SCANNING, getStorageMap: () => DEMO_MAP_SCANNING });
      await renderOverview();

      expect(await screen.findByRole('heading', { level: 3, name: 'Example downloads' })).toBeInTheDocument();
      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByRole('button', { name: 'Details' })).toBeNull();
      expect(screen.queryByRole('button', { name: /^Options for/u })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('shows what waits in the Trash and opens it by its index', async () => {
      const opened = vi.fn(() => null);
      core({ getOverview: () => DEMO_OVERVIEW, getTrash: () => TRASH, openTrash: opened });
      await renderOverview();

      expect(await screen.findByText('Waiting in the Trash: 3 GB, 1 GB of it from cleanups here.')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Open Trash' }));

      await waitFor(() => {
        expect(opened).toHaveBeenCalledWith({ index: 0 });
      });
    });

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

      expect(await screen.findByText('5 things you ignored are not shown here.')).toBeInTheDocument();
      expect(screen.getByText('Scan again to apply it')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Show in Settings' }));

      expect(navigation.getScreen()).toEqual({ kind: 'settings', tab: 'groups' });
    });

    it('a second volume has its compact disk card under the first', async () => {
      const home = DEMO_MAP_COMPLETE.volumes[0];
      const archive = home === undefined ? [] : [{ ...home, kind: 'external' as const, name: 'Archive' }];
      core({
        getOverview: () => DEMO_OVERVIEW,
        getStorageMap: () => ({ ...DEMO_MAP_COMPLETE, volumes: [...DEMO_MAP_COMPLETE.volumes, ...archive] }),
      });

      await renderOverview();

      const second = await screen.findByRole('region', { name: 'Archive' });
      const first = screen.getByRole('region', { name: 'Your disk' });
      expect(second).toHaveAttribute('data-compact');
      expect(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(within(second).queryByRole('button', { name: 'Scan' })).toBeNull();
    });

    it('the disk card, then the separator, then the groups', async () => {
      core({ getOverview: () => DEMO_OVERVIEW, getStorageMap: () => DEMO_MAP_COMPLETE });

      await renderOverview();

      const safe = await screen.findByRole('region', { name: 'Safe to remove' });
      const disk = screen.getByRole('region', { name: 'Your disk' });
      const separator = screen.getByText('What you can remove');
      expect(disk.compareDocumentPosition(separator) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(separator.compareDocumentPosition(safe) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(within(safe).getByText('· Apps make these again on their own')).toBeInTheDocument();
    });

    it('Scan again shows after a finished scan with when it was scanned, read again each minute', async () => {
      const clock = createFakeClock();
      const finished = DEMO_OVERVIEW.lastScan?.finished ?? 0;
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview(createNavigation(), { ...clock, epochMs: () => (finished + 150) * 1000 + clock.now() });

      expect(await screen.findByRole('button', { name: 'Scan again' })).toBeInTheDocument();
      expect(screen.getByText('2 minutes ago')).toBeInTheDocument();

      act(() => {
        clock.advance(60_000);
      });

      expect(screen.getByText('3 minutes ago')).toBeInTheDocument();
    });

    it('the cards sit in their danger zones, the safest first', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      await screen.findByRole('region', { name: 'Safe to remove' });
      expect(screen.getAllByRole('heading', { level: 2 }).map(({ textContent }) => textContent)).toStrictEqual([
        'Safe to remove',
        'Usually safe',
      ]);
      expect(cardNames('Safe to remove')).toStrictEqual(['Example caches']);
      expect(cardNames('Usually safe')).toStrictEqual(['Example downloads']);
    });

    it("a zone's Select all selects every category of that zone and nothing else", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { store } = await renderOverview();
      const low = await screen.findByRole('region', { name: 'Usually safe' });

      fireEvent.click(within(low).getByRole('checkbox', { name: 'Select all' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.downloads']);
      expect(within(low).getByRole('checkbox', { name: 'Select all' })).toBeChecked();
      expect(
        within(screen.getByRole('region', { name: 'Safe to remove' })).getByRole('checkbox', { name: 'Select all' }),
      ).not.toBeChecked();
    });

    it("turning a zone's Select all off clears only that zone", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { store } = await renderOverview();
      const safe = await screen.findByRole('region', { name: 'Safe to remove' });
      const low = screen.getByRole('region', { name: 'Usually safe' });
      fireEvent.click(within(safe).getByRole('checkbox', { name: 'Select all' }));
      fireEvent.click(within(low).getByRole('checkbox', { name: 'Select all' }));

      fireEvent.click(within(safe).getByRole('checkbox', { name: 'Select all' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.downloads']);
    });

    it('the selection bar shows the count, Clear empties it and Review opens Review', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation, store } = await renderOverview();
      const safe = await screen.findByRole('region', { name: 'Safe to remove' });

      fireEvent.click(within(safe).getByRole('checkbox', { name: 'Select all' }));

      const bar = await screen.findByRole('region', { name: 'Selection' });
      expect(within(bar).getByText('2 items selected')).toBeInTheDocument();
      fireEvent.click(within(bar).getByRole('button', { name: 'Review' }));
      expect(navigation.getScreen()).toStrictEqual({ kind: 'review' });

      fireEvent.click(within(bar).getByRole('button', { name: 'Clear' }));
      expect(store.getState().selection.categories.size).toBe(0);
    });

    it('while scanning the card holds the swoosh and the toolbar holds Stop', async () => {
      const startScan = vi.fn(() => 1);
      core({ startScan });
      await renderOverview();

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
      await waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
      act(() => {
        pushEvents(SCAN_STARTED, {
          bytes: 1_500_000_000,
          entries: 10,
          expected: 3_000_000_000,
          scan: 1,
          type: 'scan-progress',
        });
      });

      const card = screen.getByRole('region', { name: 'Scanning' });
      expect(within(card).getByRole('progressbar', { name: 'Scanning, 50 percent' })).toBeInTheDocument();
      expect(within(card).getByText('Scanning Macintosh HD')).toBeInTheDocument();
      expect(within(card).getByText('Looking at Demo module')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Scan again' })).toBeNull();
    });

    it('cards appear as totals arrive, each in the zone of its level', async () => {
      let overview = SCANNING;
      core({ getOverview: () => overview });
      await renderOverview();

      expect(await screen.findByRole('heading', { level: 3, name: 'Example downloads' })).toBeInTheDocument();

      overview = DEMO_OVERVIEW;
      act(() => {
        pushEvents(...DEMO_SCAN.slice(2));
      });

      await waitFor(() => {
        expect(cardNames('Safe to remove')).toStrictEqual(['Example caches']);
      });
      expect(cardNames('Usually safe')).toStrictEqual(['Example downloads']);
    });

    it('cards arrive during a scan the events started', async () => {
      let overview = IDLE;
      core({ getOverview: () => overview });
      await renderOverview();

      overview = SCANNING;
      act(() => {
        pushEvents(SCAN_STARTED, FIRST_BATCH);
      });

      expect(await screen.findByRole('heading', { level: 3, name: 'Example downloads' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
    });

    it("a card's Details opens the category's details", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation } = await renderOverview();

      await screen.findByRole('heading', { level: 3, name: 'Example caches' });
      fireEvent.click(within(cardOf('Example caches', 'Safe to remove')).getByRole('button', { name: 'Details' }));

      expect(navigation.getScreen()).toStrictEqual({ category: 'demo.caches', kind: 'category' });
    });

    it('selecting a card selects its category', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { store } = await renderOverview();

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select Example caches' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(screen.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
    });

    it('a denied access without a limited result shows no permission card', async () => {
      const probed = vi.fn(() => allFiles('denied'));
      core({ getAccessStatus: probed, getOverview: () => DEMO_OVERVIEW });

      await renderOverview();

      await waitFor(() => {
        expect(probed).toHaveBeenCalledOnce();
      });
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
    });

    it('a result limited by Full Disk Access shows the permission card', async () => {
      const limited: Overview = {
        ...DEMO_OVERVIEW,
        lastScan: { ...DEMO_OVERVIEW.lastScan, coverage: LIMITED_COVERAGE } as NonNullable<Overview['lastScan']>,
      };
      core({ getOverview: () => limited }, 'denied');

      await renderOverview();

      expect(await screen.findByText('Fresher cannot see every folder')).toBeInTheDocument();
      expect(screen.getByText('312 folders could not be read.')).toBeInTheDocument();
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
