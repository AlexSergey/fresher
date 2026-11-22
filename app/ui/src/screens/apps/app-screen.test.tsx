import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from 'vitest';

import type { AppDetail, CleanupPlanView, Confirmation, Refusal, UninstallChoice } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  MANAGED_DETAIL,
  NEEDS_ADMIN_DETAIL,
  uninstallPlanOf,
  VIDEO_STUDIO_DENIED_DETAIL,
  VIDEO_STUDIO_DETAIL,
} from '../../test/app-detail';
import { APPS_DICTIONARY } from '../../test/apps-list';
import { CLEANUP_DICTIONARY, cleanupEventsOf, MOVED_REPORT } from '../../test/cleanup-plan';
import {
  ACCESS_DICTIONARY,
  accessStatus,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SETTINGS,
  IDLE,
} from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { AppProviders } from '../app-providers';
import { AppScreen } from './app-screen';

const DICTIONARY = {
  ...FINDING_DICTIONARY,
  ...APPS_DICTIONARY,
  ...ACCESS_DICTIONARY,
  ...CLEANUP_DICTIONARY,
  'core.error.unknown-app': 'This app is not in the list any more.',
};

const APP = VIDEO_STUDIO_DETAIL.summary.app;

const REFUSED = {
  error: { code: 'uninstall-refused', reason: 'open' },
  text: { args: {}, key: 'core.error.uninstall-refused' },
} as unknown as Refusal;

const STALE = {
  error: { code: 'unknown-app', id: APP },
  text: { args: {}, key: 'core.error.unknown-app' },
} as unknown as Refusal;

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore; readonly unmount: () => void };

const show = async (
  detail: AppDetail = VIDEO_STUDIO_DETAIL,
  answers: Partial<CoreAnswers> = {},
  overview = DEMO_OVERVIEW,
): Promise<Rendered> => {
  mockCore({
    getAccessStatus: () => accessStatus([[{ kind: 'app-management' }, 'not-asked']]),
    getApp: () => detail,
    getDictionary: () => DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => overview,
    getSettings: () => DEMO_SETTINGS,
    ...answers,
  });
  const store = createCoreStore();
  const navigation = createNavigation();
  navigation.navigate({ app: detail.summary.app, kind: 'app' });
  const { unmount } = render(
    <AppProviders navigation={navigation} store={store}>
      <AppScreen app={detail.summary.app} />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
  });

  return { navigation, store, unmount };
};

const uninstall = (): void => {
  fireEvent.click(screen.getByRole('button', { name: 'Uninstall…' }));
};

type Prepared = { readonly prepareUninstall: Mock<(args: unknown) => CleanupPlanView> };

const planOf = (choice: UninstallChoice): CleanupPlanView => uninstallPlanOf(VIDEO_STUDIO_DETAIL, choice);

const preparing = (): Prepared => ({
  prepareUninstall: vi.fn((args: unknown) => planOf((args as { readonly choice: UninstallChoice }).choice)),
});

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('AppScreen', () => {
  describe('negative cases', () => {
    it.each([
      ['an app a manager installed', MANAGED_DETAIL],
      ['an app that needs an administrator', NEEDS_ADMIN_DETAIL],
    ])('shows no button and no choice for %s', async (_name, detail) => {
      await show(detail);
      await screen.findByRole('heading', { level: 1, name: detail.summary.name });

      expect(screen.queryByRole('button', { name: 'Uninstall…' })).not.toBeInTheDocument();
      expect(screen.queryByRole('radio')).not.toBeInTheDocument();
    });

    it('shows no Show in Finder for an app a manager installed', async () => {
      await show(MANAGED_DETAIL);
      await screen.findByRole('heading', { level: 1, name: 'Terminal Pro' });

      expect(screen.queryByRole('button', { name: 'Show in Finder' })).not.toBeInTheDocument();
    });

    it('shows no command for an app that needs an administrator', async () => {
      await show(NEEDS_ADMIN_DETAIL);
      await screen.findByRole('heading', { level: 1, name: 'Disk Tool' });

      expect(screen.queryByText(/Run this in Terminal/u)).not.toBeInTheDocument();
    });

    it('goes back to the list when the id is stale', async () => {
      const { navigation } = await show(VIDEO_STUDIO_DETAIL, { getApp: refuse(STALE) });

      await waitFor(() => {
        expect(navigation.getScreen()).toStrictEqual({ kind: 'apps' });
      });
    });

    it('does not empty the selection after an uninstall', async () => {
      const { store } = await show(VIDEO_STUDIO_DETAIL, {
        ...preparing(),
        confirmCleanup: () => 1,
      });
      store.dispatch({ category: 'demo.caches', type: 'toggle-category' });
      await screen.findByRole('button', { name: 'Uninstall…' });

      uninstall();
      fireEvent.click(await screen.findByRole('button', { name: 'Move to Trash' }));
      const events = cleanupEventsOf(planOf('keep-data'), MOVED_REPORT);
      act(() => {
        pushEvents(events.started, ...events.items, events.finished);
      });

      expect(store.getState().cleanup).toMatchObject({ kind: 'finished', origin: 'uninstall' });
      expect(store.getState().selection.categories.size).toBe(1);
    });

    it('does not open another app’s uninstall', async () => {
      const { store } = await show();
      await screen.findByRole('button', { name: 'Uninstall…' });

      act(() => {
        store.dispatch({ app: { index: 1, state: 1 }, origin: 'uninstall', type: 'cleanup-preparing' });
        store.dispatch({ plan: planOf('keep-data'), type: 'cleanup-prepared' });
      });

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('shows the refusal of a prepare and reads the page again', async () => {
      const getApp = vi.fn(() => VIDEO_STUDIO_DETAIL);
      await show(VIDEO_STUDIO_DETAIL, { getApp, prepareUninstall: refuse(REFUSED) });
      await screen.findByRole('button', { name: 'Uninstall…' });

      uninstall();

      expect(
        await screen.findByText('Fresher does not offer to remove this app now. Its page says why.'),
      ).toBeVisible();
      await waitFor(() => {
        expect(getApp).toHaveBeenCalledTimes(2);
      });
    });

    it('offers no permanent delete in the uninstall confirmation', async () => {
      const confirmCleanup = vi.fn<(args: unknown) => number>(() => 1);
      await show(VIDEO_STUDIO_DETAIL, { ...preparing(), confirmCleanup });
      await screen.findByRole('button', { name: 'Uninstall…' });

      uninstall();

      const dialog = await screen.findByRole('dialog');
      expect(within(dialog).queryByRole('checkbox')).toBeNull();
      expect(within(dialog).queryByText(/Delete permanently/u)).toBeNull();
      fireEvent.click(within(dialog).getByRole('button', { name: 'Move to Trash' }));
      expect(
        confirmCleanup.mock.calls.map(
          ([args]) => (args as { readonly confirmation: Confirmation }).confirmation.method,
        ),
      ).toStrictEqual(['trash']);
    });

    it('opens no confirmation while the plan is being prepared', async () => {
      await show(VIDEO_STUDIO_DETAIL, { prepareUninstall: () => new Promise(() => undefined) });
      await screen.findByRole('button', { name: 'Uninstall…' });

      uninstall();

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('shows no open-settings button before the permission was refused', async () => {
      await show();
      await screen.findByRole('heading', { level: 1, name: 'Video Studio' });

      expect(screen.getByText(/macOS may ask you to let Fresher manage apps/u)).toBeVisible();
      expect(screen.queryByRole('button', { name: 'Open System Settings' })).not.toBeInTheDocument();
    });

    it('asks the core for nothing before a scan', async () => {
      const getApp = vi.fn();
      await show(VIDEO_STUDIO_DETAIL, { getApp }, IDLE);

      expect(await screen.findByText('Reading the app.')).toBeVisible();
      expect(getApp).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('shows the name, the version, the size and the core’s sentence', async () => {
      await show();

      expect(await screen.findByRole('heading', { level: 1, name: 'Video Studio' })).toBeVisible();
      expect(screen.getByText('Video Studio takes 4 GB.')).toBeVisible();
      expect(screen.getByText('Version 14.2')).toBeVisible();
    });

    it('lists every file with its kept reason', async () => {
      await show();

      const files = await screen.findByRole('region', { name: 'What it would lose' });

      expect(within(files).getByRole('region', { name: 'Caches' })).toBeVisible();
      expect(within(files).getByText(/Other apps by the same maker may use this folder/u)).toBeVisible();
      expect(screen.getByText('Files an app keeps under another name than its id are not listed.')).toBeVisible();
    });

    it('chooses keep-data when it opens', async () => {
      await show();

      expect(await screen.findByRole('radio', { name: /keep its settings and data/u })).toBeChecked();
      expect(screen.getByRole('radio', { name: /everything it left/u })).not.toBeChecked();
    });

    it.each<[string, RegExp, UninstallChoice]>([
      ['keep-data', /keep its settings and data/u, 'keep-data'],
      ['everything', /everything it left/u, 'everything'],
    ])('asks for the %s plan and opens the confirmation', async (_name, label, choice) => {
      const { prepareUninstall } = preparing();
      await show(VIDEO_STUDIO_DETAIL, { prepareUninstall });
      fireEvent.click(await screen.findByRole('radio', { name: label }));

      uninstall();

      const dialog = await screen.findByRole('dialog');
      expect(prepareUninstall).toHaveBeenCalledWith({ app: APP, choice });
      expect(within(dialog).getByText('Video Studio')).toBeVisible();
    });

    it('cancels the confirmation and tells the core to forget the plan', async () => {
      const discardCleanup = vi.fn(() => null);
      await show(VIDEO_STUDIO_DETAIL, { ...preparing(), discardCleanup });
      await screen.findByRole('button', { name: 'Uninstall…' });
      uninstall();

      fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));

      await waitFor(() => {
        expect(discardCleanup).toHaveBeenCalledWith({ plan: 1 });
      });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('forgets the plan when the page is left during the confirmation', async () => {
      const discardCleanup = vi.fn(() => null);
      const { store, unmount } = await show(VIDEO_STUDIO_DETAIL, { ...preparing(), discardCleanup });
      await screen.findByRole('button', { name: 'Uninstall…' });
      uninstall();
      await screen.findByRole('dialog');

      unmount();

      await waitFor(() => {
        expect(discardCleanup).toHaveBeenCalledWith({ plan: 1 });
      });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
    });

    it('goes to Review once confirmed', async () => {
      const { navigation } = await show(VIDEO_STUDIO_DETAIL, { ...preparing(), confirmCleanup: () => 1 });
      await screen.findByRole('button', { name: 'Uninstall…' });
      uninstall();

      fireEvent.click(await screen.findByRole('button', { name: 'Move to Trash' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'review' });
    });

    it('links each category of the app', async () => {
      const { navigation } = await show();

      fireEvent.click(await screen.findByRole('button', { name: 'Caches of your apps' }));

      expect(navigation.getScreen()).toStrictEqual({ category: 'app-caches.library-caches', kind: 'category' });
    });

    it('goes back to the list', async () => {
      const { navigation } = await show();

      fireEvent.click(await screen.findByRole('button', { name: 'All apps' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'apps' });
    });

    it('shows the notice and its button when the permission was refused', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show(VIDEO_STUDIO_DENIED_DETAIL, {
        getAccessStatus: () => accessStatus([[{ kind: 'app-management' }, 'denied']]),
        openAccessSettings,
      });

      expect(await screen.findByText(/macOS did not let Fresher move an app/u)).toBeVisible();
      fireEvent.click(screen.getByRole('button', { name: 'Open System Settings' }));

      expect(openAccessSettings).toHaveBeenCalledWith({ capability: { kind: 'app-management' } });
    });

    it('shows Show in Finder for an app that needs an administrator', async () => {
      const revealApp = vi.fn(() => null);
      await show(NEEDS_ADMIN_DETAIL, { revealApp });

      fireEvent.click(await screen.findByRole('button', { name: 'Show in Finder' }));

      expect(revealApp).toHaveBeenCalledWith({ app: NEEDS_ADMIN_DETAIL.summary.app });
      expect(screen.getByText(/Removing this app needs an administrator/u)).toBeVisible();
    });

    it('shows the manager’s command as selectable text', async () => {
      await show(MANAGED_DETAIL);

      const command = await screen.findByText('brew uninstall --cask terminal-pro');

      expect(command.tagName).toBe('CODE');
      expect(screen.getByText(/Run this in Terminal to uninstall it with its manager/u)).toBeVisible();
    });

    it('reads the page again when a scan ends', async () => {
      const getApp = vi.fn(() => VIDEO_STUDIO_DETAIL);
      await show(VIDEO_STUDIO_DETAIL, { getApp });
      await screen.findByRole('button', { name: 'Uninstall…' });

      act(() => {
        pushEvents({ modules: [], scan: 2, type: 'scan-started' });
      });
      act(() => {
        pushEvents({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 2, type: 'scan-finished' });
      });

      await waitFor(() => {
        expect(getApp).toHaveBeenCalledTimes(2);
      });
    });
  });
});
