import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { AccessStatus, Capability, Coverage, Overview, SettingsView } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, sampleAgreementAnswers } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import {
  ACCESS_DICTIONARY,
  accessStatus,
  DEMO_DICTIONARY,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SETTINGS,
  FULL_COVERAGE,
  IDLE,
  LIMITED_COVERAGE,
  UNEXPLAINED_COVERAGE,
} from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { CoverageNotices } from './coverage-notices';

// The notices' own regions: the app's providers keep a Notifications region beside them.
const noticeRegions = (): HTMLElement[] =>
  screen.queryAllByRole('region').filter((region) => region.getAttribute('aria-label') !== 'Notifications');

const ALL_FILES: Capability = { kind: 'all-files' };
const DESKTOP: Capability = { kind: 'folder', value: 'desktop' };
const DESKTOP_KEY = 'core.access.desktop';

const FULL_DISK: Coverage = {
  notEntered: [],
  unreadable: [{ folders: 312, reason: 'no-permission', unlockedBy: ALL_FILES }],
};
const ONE_FOLDER: Coverage = {
  ...FULL_DISK,
  unreadable: [{ folders: 1, reason: 'no-permission', unlockedBy: ALL_FILES }],
};
const DESKTOP_ONLY: Coverage = { notEntered: [{ access: 'not-asked', capability: DESKTOP }], unreadable: [] };
const EXTERNAL: Coverage = {
  notEntered: [{ access: 'not-asked', capability: { kind: 'volume', value: 'removable' } }],
  unreadable: [],
};
const REMOVABLE: Capability = { kind: 'volume', value: 'removable' };
const DESKTOP_DENIED: Coverage = { notEntered: [{ access: 'denied', capability: DESKTOP }], unreadable: [] };
const EXTERNAL_DENIED: Coverage = { notEntered: [{ access: 'denied', capability: REMOVABLE }], unreadable: [] };
const NOT_NEEDED: Coverage = { notEntered: [{ access: 'not-applicable', capability: DESKTOP }], unreadable: [] };

const overviewOf = (coverage: Coverage): Overview => ({
  ...DEMO_OVERVIEW,
  lastScan: { ...DEMO_OVERVIEW.lastScan, coverage } as NonNullable<Overview['lastScan']>,
});

const statusOf = (
  all: 'denied' | 'granted' | 'unknown',
  desktop: 'granted' | 'not-asked' = 'not-asked',
): AccessStatus =>
  accessStatus([
    [ALL_FILES, all],
    [DESKTOP, desktop],
  ]);

const settingsWith = (desktop: boolean): SettingsView => ({
  ...DEMO_SETTINGS,
  values: { ...DEMO_SETTINGS.values, [DESKTOP_KEY]: { kind: 'switch', value: desktop } },
});

type Setup = {
  readonly answers?: Partial<CoreAnswers>;
  readonly coverage: Coverage | null;
  readonly desktopOn?: boolean;
  readonly status?: AccessStatus;
};

// The notices over the core's answers; the overview carries the coverage of the last scan.
const show = async ({
  answers = {},
  coverage,
  desktopOn = false,
  status = statusOf('denied'),
}: Setup): Promise<CoreStore> => {
  const store = createCoreStore();

  mockCore({
    ...sampleAgreementAnswers,
    getAccessStatus: () => status,
    getDictionary: () => ({ ...DEMO_DICTIONARY, ...ACCESS_DICTIONARY }),
    getModules: () => DEMO_MODULES,
    getOverview: () => (coverage === null ? IDLE : overviewOf(coverage)),
    getSettings: () => settingsWith(desktopOn),
    getStorageMap: () => null,
    getStorageMeaning: () => [],
    ...answers,
  });
  render(
    <AppProviders store={store}>
      <CoverageNotices />
    </AppProviders>,
  );
  await waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
    expect(store.getState().dictionary).not.toBeNull();
    expect(store.getState().access).not.toBeNull();
  });

  return store;
};

describe('CoverageNotices', () => {
  describe('negative cases', () => {
    it('no_notice_for_a_complete_scan', async () => {
      await show({ coverage: FULL_COVERAGE });

      expect(noticeRegions()).toHaveLength(0);
      expect(screen.queryByText('Full Disk Access')).toBeNull();
      expect(screen.queryByRole('heading', { name: 'Some folders were not scanned' })).toBeNull();
    });

    it('no_line_for_folders_a_permission_opens', async () => {
      await show({ coverage: FULL_DISK });
      await screen.findByRole('button', { name: 'Open System Settings' });

      expect(screen.queryByText(/no permission opens/u)).toBeNull();
    });

    it('no_notice_for_a_capability_that_is_not_applicable', async () => {
      await show({ coverage: NOT_NEEDED, status: accessStatus([[DESKTOP, 'not-applicable']]) });

      expect(noticeRegions()).toHaveLength(0);
    });

    it('no_notice_before_any_scan', async () => {
      await show({ coverage: null });

      expect(noticeRegions()).toHaveLength(0);
    });

    it('full_disk_access_denied_without_a_limited_result_shows_no_card', async () => {
      await show({ coverage: FULL_COVERAGE, status: statusOf('denied') });

      expect(screen.queryByRole('button', { name: 'Open System Settings' })).toBeNull();
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
    });

    it('the_open_settings_button_is_not_the_default_action', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show({ answers: { openAccessSettings }, coverage: FULL_DISK });
      await screen.findByRole('button', { name: 'Open System Settings' });

      // Enter with the focus on the page, or on the card's disclosure, does not open System Settings.
      fireEvent.keyDown(document.body, { key: 'Enter' });
      fireEvent.keyDown(screen.getByText('How'), { key: 'Enter' });

      expect(openAccessSettings).not.toHaveBeenCalled();
      expect(screen.getByRole('button', { name: 'Open System Settings' })).not.toHaveFocus();
    });

    it('a_notice_never_names_a_path', async () => {
      await show({ coverage: LIMITED_COVERAGE });
      await screen.findByRole('region', { name: 'Full Disk Access' });

      for (const region of noticeRegions()) {
        expect(region.textContent).not.toMatch(/\/|~/u);
      }
    });

    it('reopen_is_not_offered_before_settings_were_opened', async () => {
      await show({ coverage: FULL_DISK });
      await screen.findByRole('button', { name: 'Open System Settings' });

      expect(screen.queryByRole('button', { name: 'Reopen Fresher' })).toBeNull();
    });

    it('the opt-in is not offered once its switch is on', async () => {
      await show({ coverage: DESKTOP_ONLY, desktopOn: true });

      expect(await screen.findByText('Scan again to look there.')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Let Fresher look there' })).toBeNull();
    });

    it('a volume has no button, only its steps', async () => {
      await show({ coverage: EXTERNAL, status: accessStatus([[{ kind: 'volume', value: 'removable' }, 'not-asked']]) });

      expect(await screen.findByText('1. Add the drive to the folders to scan. 2. Scan again.')).toBeInTheDocument();
      expect(screen.queryByRole('button')).toBeNull();
    });
    it('a_denied_opt_in_no_longer_offers_the_switch_nor_a_scan_to_look', async () => {
      await show({
        coverage: DESKTOP_DENIED,
        desktopOn: true,
        status: accessStatus([[DESKTOP, 'denied']]),
      });

      await screen.findByRole('button', { name: 'Open System Settings' });
      expect(screen.queryByText('Scan again to look there.')).toBeNull();
      expect(screen.queryByRole('button', { name: 'Let Fresher look there' })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('the_heading_stands_above_the_notices', async () => {
      await show({ coverage: LIMITED_COVERAGE });

      const heading = await screen.findByRole('heading', { level: 2, name: 'Some folders were not scanned' });
      const first = noticeRegions()[0];

      expect(first).toBeDefined();
      expect(heading.compareDocumentPosition(first as Node) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    });

    it('folders_no_permission_opens_get_their_own_line_under_the_heading', async () => {
      await show({ coverage: UNEXPLAINED_COVERAGE });

      expect(await screen.findByRole('heading', { name: 'Some folders were not scanned' })).toBeInTheDocument();
      expect(
        screen.getByText(
          '3 folders could not be read, and no permission opens them. They may belong to another user, or they changed during the scan.',
        ),
      ).toBeInTheDocument();
      expect(noticeRegions()).toHaveLength(0);
    });

    it('says one folder no permission opens in the singular', async () => {
      await show({ coverage: { notEntered: [], unreadable: [{ folders: 1, reason: 'failed', unlockedBy: null }] } });

      expect(await screen.findByText(/^1 folder could not be read, and no permission opens it\./u)).toBeInTheDocument();
    });

    it('unreadable_folders_offer_full_disk_access_with_how', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show({ answers: { openAccessSettings }, coverage: FULL_DISK });

      expect(await screen.findByText('312 folders could not be read.')).toBeInTheDocument();
      expect(screen.getByText(/does not have Full Disk Access/u)).toBeInTheDocument();
      expect(screen.getByText('Turn on Fresher in the list.')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: ALL_FILES });
      });
    });

    it('a_denied_opt_in_opens_system_settings_for_its_capability', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show({
        answers: { openAccessSettings },
        coverage: DESKTOP_DENIED,
        desktopOn: true,
        status: accessStatus([[DESKTOP, 'denied']]),
      });

      expect(await screen.findByText('Turn on Fresher under Files and Folders, then scan again.')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: DESKTOP });
      });
    });

    it('a_denied_volume_opens_system_settings_for_its_capability', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show({
        answers: { openAccessSettings },
        coverage: EXTERNAL_DENIED,
        status: accessStatus([[REMOVABLE, 'denied']]),
      });

      expect(await screen.findByText('Turn on Removable Volumes for Fresher, then scan again.')).toBeInTheDocument();
      expect(screen.queryByText('1. Add the drive to the folders to scan. 2. Scan again.')).toBeNull();
      fireEvent.click(screen.getByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: REMOVABLE });
      });
    });

    it('says one folder in the singular', async () => {
      await show({ coverage: ONE_FOLDER });

      expect(await screen.findByText('1 folder could not be read.')).toBeInTheDocument();
    });

    it('skipped_folders_offer_the_opt_in', async () => {
      await show({ coverage: DESKTOP_ONLY });

      expect(await screen.findByText('Your Desktop was not looked at.')).toBeInTheDocument();
      expect(screen.getByText('macOS will ask you once.')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Let Fresher look there' })).toBeInTheDocument();
    });

    it('the_opt_in_turns_on_its_setting_and_asks_to_scan_again', async () => {
      const updateSetting = vi.fn(() => null);
      await show({ answers: { updateSetting }, coverage: DESKTOP_ONLY });

      fireEvent.click(await screen.findByRole('button', { name: 'Let Fresher look there' }));

      await waitFor(() => {
        expect(updateSetting).toHaveBeenCalledWith({
          update: { key: DESKTOP_KEY, value: { kind: 'switch', value: true } },
        });
      });
      act(() => {
        pushEvents({ key: DESKTOP_KEY, type: 'setting-changed', value: { kind: 'switch', value: true } });
      });

      expect(screen.getByText('Scan again to look there.')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Let Fresher look there' })).toBeNull();
    });

    it('a_relevant_grant_turns_the_action_into_scan_again', async () => {
      const startScan = vi.fn(() => 2);
      await show({ answers: { startScan }, coverage: FULL_DISK });
      await screen.findByRole('button', { name: 'Open System Settings' });

      act(() => {
        pushEvents({ status: statusOf('granted'), type: 'access-changed' });
      });

      expect(screen.queryByRole('button', { name: 'Open System Settings' })).toBeNull();
      fireEvent.click(screen.getByRole('button', { name: 'Scan again' }));
      await waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
    });

    it('reopen_calls_relaunch', async () => {
      // The real relaunch never answers: the app restarts.
      const relaunch = vi.fn(() => new Promise<never>(() => undefined));
      await show({ answers: { openAccessSettings: () => null, relaunch }, coverage: FULL_DISK });

      fireEvent.click(await screen.findByRole('button', { name: 'Open System Settings' }));
      expect(await screen.findByText('If you turned it on, Fresher sees it after it reopens.')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Reopen Fresher' }));

      await waitFor(() => {
        expect(relaunch).toHaveBeenCalledOnce();
      });
    });

    it('every notice has its accessible name and sits in one polite live region', async () => {
      await show({ coverage: LIMITED_COVERAGE });

      expect(await screen.findByRole('region', { name: 'Full Disk Access' })).toBeInTheDocument();
      const desktop = screen.getByRole('region', { name: 'Desktop folder' });

      // eslint-disable-next-line testing-library/no-node-access -- the live region has no role to query
      const live = desktop.parentElement;

      expect(live).toHaveAttribute('aria-live', 'polite');
      // eslint-disable-next-line testing-library/no-node-access -- the live region has no role to query
      expect(live?.querySelectorAll(':scope > section')).toHaveLength(2);
    });

    it('lists the notices in the core order', async () => {
      await show({ coverage: LIMITED_COVERAGE });

      await screen.findByRole('region', { name: 'Full Disk Access' });

      expect(noticeRegions().map((region) => region.getAttribute('aria-label'))).toStrictEqual([
        'Desktop folder',
        'Full Disk Access',
      ]);
    });
  });
});
