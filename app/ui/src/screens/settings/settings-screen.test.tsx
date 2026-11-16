import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ModuleInfo, Refusal, SettingsFile, SettingsView, SettingUpdate } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { SettingsScreen } from './settings-screen';

const MODULES: readonly ModuleInfo[] = DEMO_MODULES.map((module) => ({
  ...module,
  settings: ['demo.enabled', 'demo.count'],
}));

const SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    { default: { kind: 'folders', value: ['~'] }, key: 'core.scan.roots', kind: { kind: 'folders' } },
    { default: { kind: 'switch', value: true }, key: 'demo.enabled', kind: { kind: 'switch' } },
    { default: { kind: 'number', value: 8 }, key: 'demo.count', kind: { kind: 'number', max: 50, min: 1, unit: null } },
  ],
  values: {
    'core.scan.roots': { kind: 'folders', value: ['~'] },
    'demo.count': { kind: 'number', value: 8 },
    'demo.enabled': { kind: 'switch', value: true },
  },
};

const DICTIONARY = {
  ...DEMO_DICTIONARY,
  'core.error.settings-read-only': 'Your settings cannot be changed now. Restart or update the app, then try again.',
  'core.error.storage': 'Your settings could not be saved. Check that the disk has free space, then try again.',
  'core.scan.roots.description': 'The scan looks only inside these folders.',
  'core.scan.roots.label': 'Folders to scan',
  'demo.count.description': 'How many made-up findings the demo module shows.',
  'demo.count.label': 'Example findings',
  'demo.enabled.description': 'Shows made-up example findings.',
};

const STORAGE: Refusal = { error: { code: 'storage' }, text: { args: {}, key: 'core.error.storage' } };

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({
    appVersion: () => '1.2.3',
    getDictionary: () => DICTIONARY,
    getModules: () => MODULES,
    getOverview: () => IDLE,
    getSettings: () => SETTINGS,
    ...answers,
  });
};

// The screen once the dictionary has arrived, as the frame shows it.
const renderScreen = async (): Promise<CoreStore> => {
  const store = createCoreStore();

  render(
    <AppProviders store={store}>
      <SettingsScreen />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });

  return store;
};

const demoSwitch = (): Promise<HTMLElement> => screen.findByRole('switch', { name: 'Demo module' });

const withFile = (file: SettingsFile): Partial<CoreAnswers> => ({ getSettings: () => ({ ...SETTINGS, file }) });

describe('SettingsScreen', () => {
  describe('negative cases', () => {
    it("a refused update shows the core's text in its row and keeps the old value", async () => {
      core({ updateSetting: refuse(STORAGE) });
      await renderScreen();

      fireEvent.click(await demoSwitch());

      const row = screen.getByRole('group', { name: 'Demo module' });
      expect(await within(row).findByText(/could not be saved/u)).toBeInTheDocument();
      expect(screen.getByRole('switch', { name: 'Demo module' })).toBeChecked();
    });

    it('a read-only file disables every row and says why', async () => {
      for (const [file, title] of [
        ['newer-version', 'Your settings were saved by a newer version of Fresher'],
        ['unreadable', 'Your settings file could not be read'],
      ] as const) {
        core(withFile(file));
        const { unmount } = render(
          <AppProviders store={createCoreStore()}>
            <SettingsScreen />
          </AppProviders>,
        );

        expect(await screen.findByText(title)).toBeInTheDocument();
        expect(screen.getByText(/cannot be changed now/u)).toBeInTheDocument();
        expect(await demoSwitch()).toBeDisabled();
        expect(screen.getByRole('spinbutton', { name: 'Example findings' })).toBeDisabled();
        expect(screen.getAllByText(/^Read-only/u)).toHaveLength(SETTINGS.settings.length);
        unmount();
      }
    });

    it('a broken file says the defaults are in use and keeps every row editable', async () => {
      core(withFile('broken'));
      await renderScreen();

      expect(await screen.findByText('Your settings file was damaged')).toBeInTheDocument();
      expect(await demoSwitch()).toBeEnabled();
      expect(screen.queryByText(/^Read-only/u)).toBeNull();
    });

    it('shows no notice and no version line when the core does not tell them', async () => {
      core({ appVersion: refuse(STORAGE) });
      await renderScreen();

      await demoSwitch();
      expect(screen.getByRole('heading', { level: 1, name: 'Settings' })).toBeInTheDocument();
      expect(screen.queryByText(/^Version/u)).toBeNull();
      expect(screen.queryByText(/settings file/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('every setting of the demo module has a row', async () => {
      core();
      await renderScreen();

      await demoSwitch();
      expect(screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent)).toEqual([
        'General',
        'Demo module',
      ]);
      const general = screen.getByRole('region', { name: 'General' });
      const demo = screen.getByRole('region', { name: 'Demo module' });
      expect(within(general).getAllByRole('group')).toHaveLength(1);
      expect(within(general).getByRole('group', { name: 'Folders to scan' })).toBeInTheDocument();
      expect(
        within(demo)
          .getAllByRole('group')
          .map((row) => row.getAttribute('data-kind')),
      ).toEqual(['switch', 'number']);
      expect(within(demo).getByRole('group', { name: 'Demo module' })).toBeInTheDocument();
      expect(within(demo).getByRole('group', { name: 'Example findings' })).toBeInTheDocument();
    });

    it('a switch change is sent and the event updates the row', async () => {
      const sent: SettingUpdate[] = [];
      core({
        updateSetting: (args) => {
          sent.push((args as { readonly update: SettingUpdate }).update);

          return null;
        },
      });
      await renderScreen();

      fireEvent.click(await demoSwitch());
      await vi.waitFor(() => {
        expect(sent).toEqual([{ key: 'demo.enabled', value: { kind: 'switch', value: false } }]);
      });
      expect(screen.getByRole('switch', { name: 'Demo module' })).toBeChecked();

      act(() => {
        pushEvents({ key: 'demo.enabled', type: 'setting-changed', value: { kind: 'switch', value: false } });
      });

      expect(screen.getByRole('switch', { name: 'Demo module' })).not.toBeChecked();
    });

    it('the version shows in the footer', async () => {
      core();
      await renderScreen();

      const footer = await screen.findByRole('contentinfo');
      expect(await within(footer).findByText('Version 1.2.3')).toBeInTheDocument();
    });
  });
});
