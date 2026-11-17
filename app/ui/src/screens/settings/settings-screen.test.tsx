import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ModuleInfo, Refusal, SettingsFile, SettingsView, SettingUpdate } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { type CoreStore, createCoreStore } from '../../state/store';
import { DEMO_DICTIONARY, DEMO_MODULES, DEMO_OVERVIEW, IDLE } from '../../test/demo-scan';
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
    {
      default: { kind: 'folders', value: ['~'] },
      effect: 'next-scan',
      group: 'scan',
      key: 'core.scan.roots',
      kind: { kind: 'folders' },
    },
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'demo.enabled',
      kind: { kind: 'switch' },
    },
    {
      default: { kind: 'number', value: 8 },
      effect: 'next-scan',
      group: 'module',
      key: 'demo.count',
      kind: { kind: 'number', max: 50, min: 1, unit: null },
    },
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
    appInfo: () => ({ build: { kind: 'release' }, version: '1.2.3' }),
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

const sentUpdates = (): { readonly answer: (args: unknown) => null; readonly sent: SettingUpdate[] } => {
  const sent: SettingUpdate[] = [];

  return {
    answer: (args): null => {
      sent.push((args as { readonly update: SettingUpdate }).update);

      return null;
    },
    sent,
  };
};

const withValues = (values: SettingsView['values']): Partial<CoreAnswers> => ({
  getSettings: () => ({ ...SETTINGS, values: { ...SETTINGS.values, ...values } }),
});

describe('SettingsScreen', () => {
  describe('negative cases', () => {
    it('a cancelled folder pick or a folder already listed sends nothing', async () => {
      const updates = sentUpdates();
      let picked: null | string = null;
      core({ pickFolder: () => picked, updateSetting: updates.answer });
      await renderScreen();
      const add = await screen.findByRole('button', { name: 'Add folder…' });

      fireEvent.click(add);
      picked = '~';
      fireEvent.click(add);

      await vi.waitFor(() => {
        expect(screen.getByRole('button', { name: 'Add folder…' })).toBeEnabled();
      });
      expect(updates.sent).toEqual([]);
    });

    it('a row at its default has no Reset, and the ignored group says nothing is ignored', async () => {
      core();
      await renderScreen();

      await demoSwitch();
      expect(screen.queryByRole('button', { name: 'Reset' })).toBeNull();
      expect(screen.getByText('Nothing is ignored.')).toBeInTheDocument();
    });

    it('cancelling Reset all settings sends nothing', async () => {
      const resetSettings = vi.fn(() => null);
      core({ resetSettings });
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Reset all settings…' }));
      fireEvent.click(await screen.findByRole('button', { name: 'Keep my settings' }));
      expect(resetSettings).not.toHaveBeenCalled();
    });

    it('a read-only file disables Reset all settings', async () => {
      core(withFile('newer-version'));
      await renderScreen();
      await demoSwitch();
      expect(screen.getByRole('button', { name: 'Reset all settings…' })).toBeDisabled();
    });

    it('no Rescan to apply while the last scan is current', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      await renderScreen();

      await demoSwitch();
      expect(screen.queryByText('Rescan to apply')).toBeNull();
    });
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
      core({ appInfo: refuse(STORAGE) });
      await renderScreen();

      await demoSwitch();
      expect(screen.getByRole('heading', { level: 1, name: 'Settings' })).toBeInTheDocument();
      expect(screen.queryByText(/^Version/u)).toBeNull();
      expect(screen.queryByText(/settings file/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('a picked folder is added to the list and a removed one leaves it', async () => {
      const updates = sentUpdates();
      core({ pickFolder: () => '~/Projects', updateSetting: updates.answer });
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Add folder…' }));
      await vi.waitFor(() => {
        expect(updates.sent).toEqual([
          { key: 'core.scan.roots', value: { kind: 'folders', value: ['~', '~/Projects'] } },
        ]);
      });
      fireEvent.click(screen.getByRole('button', { name: 'Remove ~' }));

      expect(updates.sent.at(-1)).toEqual({ key: 'core.scan.roots', value: { kind: 'folders', value: [] } });
    });

    it('Reset sends the default of a changed row', async () => {
      const updates = sentUpdates();
      core({ ...withValues({ 'demo.count': { kind: 'number', value: 3 } }), updateSetting: updates.answer });
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Reset' }));

      await vi.waitFor(() => {
        expect(updates.sent).toEqual([{ key: 'demo.count', value: { kind: 'number', value: 8 } }]);
      });
    });

    it('every ignored kind of item and path is listed with Restore', async () => {
      const restored: unknown[] = [];
      core({
        ...withValues({
          'core.ignore.categories': { kind: 'ids', value: ['demo.caches'] },
          'core.ignore.paths': { kind: 'paths', value: ['/Users/ann/keep'] },
        }),
        restoreIgnored: (args) => {
          restored.push((args as { readonly entry: unknown }).entry);

          return null;
        },
      });
      await renderScreen();

      const ignored = await screen.findByRole('region', { name: 'Ignored' });
      fireEvent.click(await within(ignored).findByRole('button', { name: 'Restore Example caches' }));
      fireEvent.click(within(ignored).getByRole('button', { name: 'Restore /Users/ann/keep' }));

      await vi.waitFor(() => {
        expect(restored).toEqual([
          { category: 'demo.caches', kind: 'category' },
          { kind: 'path', path: '/Users/ann/keep' },
        ]);
      });
    });

    it('Reset all settings asks first, then resets', async () => {
      const resetSettings = vi.fn(() => null);
      core({ getOwnData: () => ({ budget: 50_000_000, total: 2_000_000 }), resetSettings });
      await renderScreen();

      expect(await screen.findByText(/take 2 MB of the 50 MB/u)).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Reset all settings…' }));
      fireEvent.click(await screen.findByRole('button', { name: 'Reset settings' }));

      await vi.waitFor(() => {
        expect(resetSettings).toHaveBeenCalledOnce();
      });
    });

    it('Rescan to apply offers a scan when a change waits for one', async () => {
      const startScan = vi.fn(() => 2);
      const lastScan = DEMO_OVERVIEW.lastScan === null ? null : { ...DEMO_OVERVIEW.lastScan, settingsChanged: true };
      core({ getOverview: () => ({ ...DEMO_OVERVIEW, lastScan }), startScan });
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Rescan' }));

      await vi.waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
    });
    it('every setting of the demo module has a row', async () => {
      core();
      await renderScreen();

      await demoSwitch();
      expect(screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent)).toEqual([
        'Scan',
        'Demo module',
        'Ignored',
        "Fresher's own data",
        'About',
      ]);
      const general = screen.getByRole('region', { name: 'Scan' });
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

    it('the version and the licence show under About', async () => {
      core();
      await renderScreen();

      const about = await screen.findByRole('region', { name: 'About' });
      expect(await within(about).findByText('Version 1.2.3')).toBeInTheDocument();
      expect(within(about).getByText(/MIT or Apache 2.0/u)).toBeInTheDocument();
    });
  });
});
