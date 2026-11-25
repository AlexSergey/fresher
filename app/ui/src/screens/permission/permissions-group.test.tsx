import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type {
  AccessStatus,
  Capability,
  ModuleInfo,
  SettingDescriptor,
  SettingsView,
} from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, refuse, sampleAgreementAnswers } from '../../ipc/mock';
import { createNavigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { ACCESS_DICTIONARY, BUSY, DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { SettingsScreen } from '../settings/settings-screen';

const ALL_FILES: Capability = { kind: 'all-files' };
const DESKTOP: Capability = { kind: 'folder', value: 'desktop' };
const DOCKER: Capability = { kind: 'tool', value: 'docker' };
const REMOVABLE: Capability = { kind: 'volume', value: 'removable' };

const DICTIONARY = {
  ...DEMO_DICTIONARY,
  ...ACCESS_DICTIONARY,
  'core.access.app-management.how': '1. Open System Settings.',
  'core.access.app-management.missing': 'Fresher may not move apps yet.',
  'core.access.app-management.name': 'App Management',
  'core.access.app-management.unlocks': 'Lets Fresher move apps to the Trash.',
  'core.access.desktop.label': 'Let Fresher look in Desktop',
  'core.access.state.denied': 'Not allowed',
  'core.access.state.granted': 'Allowed',
  'core.access.state.not-applicable': 'Not needed on this Mac',
  'core.access.state.not-asked': 'Not asked yet',
  'core.access.state.unknown': 'Could not be checked',
  'core.access.tool.not-ready': '{tool} is installed but not running. Start it, then scan again.',
  'docker.enabled.label': 'Docker',
};

const SWITCH: SettingDescriptor = {
  default: { kind: 'switch', value: false },
  effect: 'next-scan',
  group: 'permissions',
  key: 'core.access.desktop',
  kind: { kind: 'switch' },
};

const SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [SWITCH],
  values: { 'core.access.desktop': { kind: 'switch', value: false } },
};

const MODULES: readonly ModuleInfo[] = [
  ...DEMO_MODULES,
  {
    categories: [
      {
        availability: { kind: 'ready' },
        descriptor: {
          danger: 'low',
          icon: 'archive',
          id: 'docker.images',
          needs: [{ capability: DOCKER, kind: 'required' }],
        },
      },
    ],
    choices: [],
    group: 'development',
    id: 'docker',
    settings: [],
  },
];

const status = (states: AccessStatus['capabilities']): AccessStatus => ({ capabilities: states });

const FULL: AccessStatus = status([
  { access: 'denied', capability: ALL_FILES, note: null },
  { access: 'not-asked', capability: DESKTOP, note: null },
  { access: 'not-asked', capability: REMOVABLE, note: null },
  {
    access: 'unknown',
    capability: DOCKER,
    note: { args: { tool: { kind: 'text', value: 'Docker' } }, key: 'core.access.tool.not-ready' },
  },
]);

// The settings screen over the core's answers; `null` access: the core has not answered yet.
const show = async (
  access: AccessStatus | null,
  answers: Partial<CoreAnswers> = {},
  settings = SETTINGS,
): Promise<CoreStore> => {
  const store = createCoreStore();

  mockCore({
    ...sampleAgreementAnswers,
    getAccessStatus: () => access ?? new Promise<never>(() => undefined),
    getDictionary: () => DICTIONARY,
    getModules: () => MODULES,
    getOverview: () => IDLE,
    getSettings: () => settings,
    ...answers,
  });
  const navigation = createNavigation();

  navigation.navigate({ kind: 'settings', page: { kind: 'permissions' } });
  render(
    <AppProviders navigation={navigation} store={store}>
      <SettingsScreen />
    </AppProviders>,
  );
  await waitFor(() => {
    if (access !== null) {
      expect(store.getState().access).not.toBeNull();
    }
    expect(store.getState().settings).not.toBeNull();
  });

  return store;
};

const group = async (): Promise<HTMLElement> => screen.findByRole('region', { name: 'Permissions' });

describe('PermissionsGroup', () => {
  describe('negative cases', () => {
    it('a_not_applicable_capability_is_not_listed', async () => {
      await show(status([{ access: 'not-applicable', capability: REMOVABLE, note: null }]));
      const permissions = await group();

      expect(within(permissions).queryByText('External drives')).toBeNull();
    });

    it('a_granted_permission_has_no_action', async () => {
      await show(status([{ access: 'granted', capability: ALL_FILES, note: null }]));
      const permissions = await group();

      expect(within(permissions).getByText('Allowed')).toBeInTheDocument();
      expect(within(permissions).queryByRole('button', { name: 'Open System Settings' })).toBeNull();
    });

    it('the_opt_in_switches_are_not_listed_twice', async () => {
      await show(FULL);
      await group();

      expect(screen.getAllByRole('switch', { name: 'Let Fresher look in Desktop' })).toHaveLength(1);
    });

    it('a_read_only_settings_file_disables_the_switches', async () => {
      await show(FULL, {}, { ...SETTINGS, file: 'newer-version' });
      const permissions = await group();

      expect(within(permissions).getByRole('switch', { name: 'Let Fresher look in Desktop' })).toBeDisabled();
    });

    it('a_not_asked_row_with_no_button_shows_no_numbered_steps', async () => {
      await show(status([{ access: 'not-asked', capability: { kind: 'app-management' }, note: null }]));
      const permissions = await group();

      expect(within(permissions).queryByText('1. Open System Settings.')).toBeNull();
      expect(within(permissions).getByText('Fresher may not move apps yet.')).toBeInTheDocument();
    });

    it('a_tool_nobody_needs_is_not_listed', async () => {
      await show(status([{ access: 'denied', capability: { kind: 'tool', value: 'git' }, note: null }]));
      const permissions = await group();

      expect(within(permissions).queryByText('Not allowed')).toBeNull();
    });
    it('a_tool_without_a_note_has_no_empty_description', async () => {
      await show(status([{ access: 'unknown', capability: DOCKER, note: null }]));
      const permissions = await group();

      // eslint-disable-next-line testing-library/no-node-access -- the row has no role of its own to query
      const row = within(permissions).getByText('Docker').closest('li');

      // eslint-disable-next-line testing-library/no-node-access -- an empty paragraph has no text to query
      expect(row?.querySelectorAll('p')).toHaveLength(0);
    });

    it('a_failed_check_again_reports_the_failure', async () => {
      const store = await show(FULL, { refreshAccess: refuse(BUSY) });

      fireEvent.click(await screen.findByRole('button', { name: 'Check again' }));

      await waitFor(() => {
        expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: BUSY });
      });
    });

    it('a_failed_open_system_settings_reports_the_failure', async () => {
      const store = await show(FULL, { openAccessSettings: refuse(BUSY) });

      fireEvent.click(await screen.findByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: BUSY });
      });
    });

    it('before_the_core_answers_no_state_is_shown', async () => {
      await show(null);
      const permissions = await group();

      expect(within(permissions).queryByText('Full Disk Access')).toBeNull();
      expect(within(permissions).queryByText('Not asked yet')).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('every_capability_shows_its_state_and_what_it_unlocks', async () => {
      await show(FULL);
      const permissions = await group();

      expect(within(permissions).getByText('Full Disk Access')).toBeInTheDocument();
      expect(within(permissions).getByText('Lets Fresher see every folder on this Mac.')).toBeInTheDocument();
      expect(within(permissions).getByText('Not allowed')).toBeInTheDocument();
      expect(within(permissions).getByText('Desktop folder')).toBeInTheDocument();
      expect(within(permissions).getAllByText('Not asked yet')).toHaveLength(2);
      expect(within(permissions).getByText('External drives')).toBeInTheDocument();
      expect(
        within(permissions).getByText('Lets Fresher measure what takes space on a drive you added.'),
      ).toBeInTheDocument();
    });

    it('open_system_settings_passes_the_capability', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show(FULL, { openAccessSettings });

      fireEvent.click(await screen.findByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: ALL_FILES });
      });
      expect(screen.getByRole('button', { name: 'Reopen Fresher' })).toBeInTheDocument();
    });

    it('the_heading_and_the_switches_show_before_the_core_answers', async () => {
      await show(null);
      const permissions = await group();

      expect(within(permissions).getByRole('switch', { name: 'Let Fresher look in Desktop' })).toBeInTheDocument();
      expect(within(permissions).getByRole('button', { name: 'Check again' })).toBeInTheDocument();
    });

    it('a_denied_opt_in_offers_system_settings_with_its_text', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show(status([{ access: 'denied', capability: DESKTOP, note: null }]), { openAccessSettings });
      const permissions = await group();

      expect(
        within(permissions).getByText('Turn on Fresher under Files and Folders, then scan again.'),
      ).toBeInTheDocument();
      expect(within(permissions).getByRole('switch', { name: 'Let Fresher look in Desktop' })).toBeInTheDocument();
      fireEvent.click(within(permissions).getByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: DESKTOP });
      });
    });

    it('a_denied_volume_offers_system_settings_instead_of_its_steps', async () => {
      const openAccessSettings = vi.fn(() => null);
      await show(status([{ access: 'denied', capability: REMOVABLE, note: null }]), { openAccessSettings });
      const permissions = await group();

      expect(
        within(permissions).getByText('Turn on Removable Volumes for Fresher, then scan again.'),
      ).toBeInTheDocument();
      expect(within(permissions).queryByText('1. Add the drive to the folders to scan. 2. Scan again.')).toBeNull();
      fireEvent.click(within(permissions).getByRole('button', { name: 'Open System Settings' }));

      await waitFor(() => {
        expect(openAccessSettings).toHaveBeenCalledWith({ capability: REMOVABLE });
      });
    });

    it('reopen_fresher_relaunches_the_app', async () => {
      const relaunch = vi.fn(() => null);
      await show(FULL, { openAccessSettings: () => null, relaunch });

      fireEvent.click(await screen.findByRole('button', { name: 'Open System Settings' }));
      fireEvent.click(screen.getByRole('button', { name: 'Reopen Fresher' }));

      await waitFor(() => {
        expect(relaunch).toHaveBeenCalled();
      });
    });

    it('check_again_refreshes_the_status', async () => {
      const refreshAccess = vi.fn(() => FULL);
      await show(FULL, { refreshAccess });

      fireEvent.click(await screen.findByRole('button', { name: 'Check again' }));

      await waitFor(() => {
        expect(refreshAccess).toHaveBeenCalled();
      });
    });

    it('a_tool_is_listed_under_its_module_with_its_note', async () => {
      await show(FULL);
      const permissions = await group();

      expect(within(permissions).getByText('Docker')).toBeInTheDocument();
      expect(
        within(permissions).getByText('Docker is installed but not running. Start it, then scan again.'),
      ).toBeInTheDocument();
    });

    it('every_action_is_reachable_by_keyboard', async () => {
      await show(FULL);
      const permissions = await group();
      const open = within(permissions).getByRole('button', { name: 'Open System Settings' });

      open.focus();

      expect(open).toHaveFocus();
      expect(within(permissions).getByRole('switch', { name: 'Let Fresher look in Desktop' })).not.toHaveAttribute(
        'tabindex',
        '-1',
      );
    });
  });
});
