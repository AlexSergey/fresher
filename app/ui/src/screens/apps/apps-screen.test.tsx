import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AppsQuery, AppsView, Refusal } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse, sampleAgreementAnswers } from '../../ipc/mock';
import { createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { APPS, APPS_DICTIONARY, APPS_VIEW, MANAGED } from '../../test/apps-list';
import {
  ACCESS_DICTIONARY,
  accessStatus,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SETTINGS,
  IDLE,
} from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { AppsScreen } from './apps-screen';

const DICTIONARY = { ...APPS_DICTIONARY, ...ACCESS_DICTIONARY, 'core.error.no-scan': 'Scan first.' };

const REFUSAL = { error: { code: 'no-scan' }, text: { args: {}, key: 'core.error.no-scan' } } as unknown as Refusal;

const queryOf = (args: unknown): AppsQuery => (args as { readonly query: AppsQuery }).query;

const show = async (answers: Partial<CoreAnswers> = {}, overview = DEMO_OVERVIEW): Promise<void> => {
  mockCore({
    ...sampleAgreementAnswers,
    getAccessStatus: () => accessStatus([[{ kind: 'all-files' }, 'granted']]),
    getApps: () => APPS_VIEW,
    getDictionary: () => DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => overview,
    getSettings: () => DEMO_SETTINGS,
    ...answers,
  });
  const store = createCoreStore();
  render(
    <AppProviders store={store}>
      <AppsScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });
};

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('AppsScreen', () => {
  describe('negative cases', () => {
    it('shows the refusal text when the core refuses', async () => {
      await show({ getApps: refuse(REFUSAL) });

      expect(await screen.findByText('Scan first.')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: 'Apps, largest first' })).not.toBeInTheDocument();
    });

    it('shows the empty sentence when the filter keeps nothing', async () => {
      await show({ getApps: () => ({ apps: [], notice: null }) });

      expect(await screen.findByText('No app fits this filter.')).toBeInTheDocument();
    });

    it("shows nothing offered for a not-offered app's row", async () => {
      await show({ getApps: () => ({ apps: [MANAGED], notice: null }) });

      expect(await screen.findByText(/Homebrew installed this app/u)).toBeInTheDocument();
    });

    it('shows the measuring line alone while the core answers and tells nothing', async () => {
      await show({ getApps: () => new Promise<AppsView>(() => undefined) });

      expect(await screen.findByText('Measuring your apps…')).toBeInTheDocument();
      expect(screen.queryByRole('progressbar')).toBeNull();
    });

    it('a measure that told every bundle shows the line alone until the list comes', async () => {
      await show({ getApps: () => new Promise<AppsView>(() => undefined) });
      await screen.findByText('Measuring your apps…');

      act(() => {
        pushEvents({ done: 12, total: 12, type: 'apps-measuring' });
      });

      expect(screen.getByText('Measuring your apps…')).toBeInTheDocument();
      expect(screen.queryByRole('progressbar')).toBeNull();
    });

    it('a scan running shows its card, not the measuring line', async () => {
      await show({ getApps: () => new Promise<AppsView>(() => undefined) });
      await screen.findByText('Measuring your apps…');

      act(() => {
        pushEvents({ modules: [], scan: 2, type: 'scan-started' });
      });

      expect(await screen.findByRole('region', { name: 'Scanning' })).toBeInTheDocument();
      expect(screen.queryByText(/^Measuring your apps/u)).toBeNull();
    });

    it('asks to scan first before any scan and does not ask the core', async () => {
      const getApps = vi.fn();
      await show({ getApps }, IDLE);

      expect(await screen.findByText('Scan first to see your apps.')).toBeInTheDocument();
      expect(getApps).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it("lists the apps in the core's order", async () => {
      await show();
      const list = await screen.findByRole('list', { name: 'Apps, largest first' });

      const rows = within(list)
        .getAllByRole('listitem')
        .filter((row) => row.tabIndex === 0);

      expect(rows).toHaveLength(APPS.length);
      rows.forEach((row, index) => {
        expect(row.textContent).toMatch(new RegExp(`^${(APPS[index] as (typeof APPS)[number]).name}`, 'u'));
      });
    });

    it("asks again with the filter's value", async () => {
      const getApps = vi.fn((args: unknown) => ({
        ...APPS_VIEW,
        apps: queryOf(args).unused === null ? APPS_VIEW.apps : [],
        notice: null,
      }));
      await show({ getApps });
      await screen.findByRole('list', { name: 'Apps, largest first' });

      fireEvent.click(screen.getByRole('radio', { name: 'A year' }));

      await waitFor(() => {
        expect(getApps).toHaveBeenLastCalledWith({ query: { unused: 'a-year' } });
      });
    });

    it('asks again when a scan ends', async () => {
      const getApps = vi.fn(() => APPS_VIEW);
      await show({ getApps });
      await screen.findByRole('list', { name: 'Apps, largest first' });

      act(() => {
        pushEvents({ modules: [], scan: 2, type: 'scan-started' });
      });
      act(() => {
        pushEvents({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 2, type: 'scan-finished' });
      });

      await waitFor(() => {
        expect(getApps).toHaveBeenCalledTimes(2);
      });
    });

    it('shows how many apps of how many are measured, with its bar', async () => {
      await show({ getApps: () => new Promise<AppsView>(() => undefined) });
      await screen.findByText('Measuring your apps…');

      act(() => {
        pushEvents({ done: 3, total: 12, type: 'apps-measuring' });
      });

      expect(screen.getByText('Measuring your apps · 3 of 12')).toBeInTheDocument();
      expect(screen.getByRole('progressbar', { name: 'Measuring your apps · 3 of 12' })).toHaveAttribute(
        'aria-valuenow',
        '25',
      );
    });

    it("shows the core's notice", async () => {
      await show({ getApps: () => ({ ...APPS_VIEW, notice: { args: {}, key: 'core.apps.other-names' } }) });

      expect(await screen.findByText('Files an app keeps under another name are not listed.')).toBeInTheDocument();
    });
  });
});
