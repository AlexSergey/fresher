import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Access, Overview } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
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
import { AppProviders } from '../app-providers';
import { OverviewScreen } from './overview-screen';

const DICTIONARY = { ...DEMO_DICTIONARY, 'demo.error.failed': 'The demo folder vanished.' };

const PARTIAL: Overview = {
  ...DEMO_OVERVIEW,
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'partial' },
    scan: 1,
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
  },
};

// The overview during scan 1, with only the downloads counted so far.
const SCANNING: Overview = {
  found: { allocated: 750_000_000, completeness: 'complete', logical: 750_000_000, reclaimable: 750_000_000 },
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
    ...answers,
  });
};

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore };

// The overview, once the core has answered its dictionary, its modules and its overview.
const renderOverview = async (): Promise<Rendered> => {
  const navigation = createNavigation();
  const store = createCoreStore();

  render(
    <AppProviders navigation={navigation} store={store}>
      <OverviewScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });

  return { navigation, store };
};

// The cards' visible names, in order; each card's accessible name reads its name as "Open …".
const cardNames = (): readonly string[] =>
  within(screen.getByRole('list', { name: 'Categories' }))
    .getAllByRole('button', { name: /^Open/ })
    .map((button) => within(button).getAllByText(/./u, { selector: '[aria-hidden="true"]' })[0]?.textContent ?? '');

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

describe('OverviewScreen', () => {
  describe('negative cases', () => {
    it('before a scan there are no cards and no zero sizes', async () => {
      core();

      await renderOverview();

      expect(screen.getByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
      expect(screen.getAllByRole('button', { name: /Scan/ })).toHaveLength(1);
      expect(screen.getByText(/It changes nothing on your disk\./)).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Categories' })).toBeNull();
      expect(screen.queryByText(/\b0 B\b/)).toBeNull();
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
      expect(screen.getByRole('list', { name: 'Parts of the scan' })).toHaveTextContent('Demo module');
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
