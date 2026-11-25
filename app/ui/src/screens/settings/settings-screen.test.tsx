import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ModuleInfo, Refusal, SettingsFile, SettingsView, SettingUpdate } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation, type SettingsPage } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { AGREEMENT_ACCEPTED, AGREEMENT_DICTIONARY, AGREEMENT_NOT_ACCEPTED } from '../../test/agreement';
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
  ...AGREEMENT_DICTIONARY,
  'core.error.settings-read-only': 'Your settings cannot be changed now. Restart or update the app, then try again.',
  'core.error.storage': 'Your settings could not be saved. Check that the disk has free space, then try again.',
  'core.group.files.name': 'Files',
  'core.scan.roots.description': 'The scan looks only inside these folders.',
  'core.scan.roots.label': 'Folders to scan',
  'demo.count.description': 'How many made-up findings the demo module shows.',
  'demo.count.label': 'Example findings',
  'demo.enabled.description': 'Shows made-up example findings.',
};

const STORAGE: Refusal = { error: { code: 'storage' }, text: { args: {}, key: 'core.error.storage' } };

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({
    ...sampleAgreementAnswers,
    appInfo: () => ({ build: { kind: 'release' }, high: 'offered', home: '/Users/someone', version: '1.2.3' }),
    getDictionary: () => DICTIONARY,
    getModules: () => MODULES,
    getOverview: () => IDLE,
    getSettings: () => SETTINGS,
    ...sampleStorageAnswers,
    ...answers,
  });
};

// The navigation with Settings opened on a page (General without one).
const opened = (page?: SettingsPage): Navigation => {
  const navigation = createNavigation();

  navigation.navigate(page === undefined ? { kind: 'settings' } : { kind: 'settings', page });

  return navigation;
};

const DEMO_PAGE: SettingsPage = { kind: 'module', module: 'demo' };

// The screen once the dictionary has arrived, as the frame shows it.
const renderScreen = async (page?: SettingsPage): Promise<CoreStore> => {
  const navigation = opened(page);
  const store = createCoreStore();

  render(
    <AppProviders navigation={navigation} store={store}>
      <SettingsScreen />
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });

  return store;
};

const demoSwitch = (): Promise<HTMLElement> => screen.findByRole('switch', { name: 'Look for Demo module' });

const general = (): Promise<HTMLElement> => screen.findByRole('heading', { level: 1, name: 'General' });

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
    it('About says the agreement is not accepted yet', async () => {
      core({ getAgreement: () => AGREEMENT_NOT_ACCEPTED });
      await renderScreen({ kind: 'about' });

      const about = await screen.findByRole('region', { name: 'About' });

      expect(await within(about).findByText(/Not accepted yet/u)).toBeInTheDocument();
      expect(within(about).queryByText(/You accepted it on/u)).toBeNull();
    });

    it('a cancelled folder pick or a folder already listed sends nothing', async () => {
      const updates = sentUpdates();
      let picked: null | string = null;
      core({ pickFolder: () => picked, updateSetting: updates.answer });
      await renderScreen({ kind: 'scan' });
      const add = await screen.findByRole('button', { name: 'Add folder…' });

      fireEvent.click(add);
      picked = '~';
      fireEvent.click(add);

      await vi.waitFor(() => {
        expect(screen.getByRole('button', { name: 'Add folder…' })).toBeEnabled();
      });
      expect(updates.sent).toEqual([]);
    });

    it('a row at its default has no Reset, and the ignored page says nothing is ignored', async () => {
      core();
      await renderScreen(DEMO_PAGE);

      await demoSwitch();
      expect(screen.queryByRole('button', { name: 'Reset' })).toBeNull();
      fireEvent.click(screen.getByRole('button', { name: 'Ignored' }));
      fireEvent.click(screen.getByRole('tab', { name: 'Items' }));
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
      await general();
      expect(screen.getByRole('button', { name: 'Reset all settings…' })).toBeDisabled();
    });

    it('a module group with no module that has settings is not in the tree', async () => {
      core({
        getModules: () => [...MODULES, { categories: [], choices: [], group: 'files', id: 'bare', settings: [] }],
      });
      await renderScreen();

      expect(await screen.findByRole('button', { name: 'Debug' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Files' })).toBeNull();
    });

    it('no Scan again to apply it while the last scan is current', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      await renderScreen();

      await general();
      expect(screen.queryByText('Scan again to apply it')).toBeNull();
    });
    it("a refused update shows the core's text in its row and keeps the old value", async () => {
      core({ updateSetting: refuse(STORAGE) });
      await renderScreen(DEMO_PAGE);

      fireEvent.click(await demoSwitch());

      const row = screen.getByRole('group', { name: 'Look for Demo module' });
      expect(await within(row).findByText(/could not be saved/u)).toBeInTheDocument();
      expect(screen.getByRole('switch', { name: 'Look for Demo module' })).toBeChecked();
    });

    it('a read-only file disables every row and says why', async () => {
      for (const [file, title] of [
        ['newer-version', 'Your settings were saved by a newer version of Fresher'],
        ['unreadable', 'Your settings file could not be read'],
      ] as const) {
        core(withFile(file));
        const { unmount } = render(
          <AppProviders navigation={opened(DEMO_PAGE)} store={createCoreStore()}>
            <SettingsScreen />
          </AppProviders>,
        );

        expect(await screen.findByText(title)).toBeInTheDocument();
        expect(screen.getByText(/cannot be changed now/u)).toBeInTheDocument();
        expect(await demoSwitch()).toBeDisabled();
        expect(screen.getByRole('spinbutton', { name: 'Example findings' })).toBeDisabled();
        expect(screen.getAllByText(/^Read-only/u)).toHaveLength(2);
        unmount();
      }
    });

    it('a broken file says the defaults are in use and keeps every row editable', async () => {
      core(withFile('broken'));
      await renderScreen(DEMO_PAGE);

      expect(await screen.findByText('Your settings file was damaged')).toBeInTheDocument();
      expect(await demoSwitch()).toBeEnabled();
      expect(screen.queryByText(/^Read-only/u)).toBeNull();
    });

    it('shows no notice and no version line when the core does not tell them', async () => {
      core({ appInfo: refuse(STORAGE) });
      await renderScreen();

      await general();
      expect(screen.getByRole('heading', { level: 1, name: 'General' })).toBeInTheDocument();
      expect(screen.queryByText(/^Version/u)).toBeNull();
      expect(screen.queryByText(/settings file/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('a picked folder is added to the list and a removed one leaves it', async () => {
      const updates = sentUpdates();
      core({ pickFolder: () => '~/Projects', updateSetting: updates.answer });
      await renderScreen({ kind: 'scan' });

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
      await renderScreen(DEMO_PAGE);

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
      await renderScreen({ kind: 'ignored' });

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

    it('Scan again to apply it offers a scan when a change waits for one', async () => {
      const startScan = vi.fn(() => 2);
      const lastScan = DEMO_OVERVIEW.lastScan === null ? null : { ...DEMO_OVERVIEW.lastScan, settingsChanged: true };
      core({ getOverview: () => ({ ...DEMO_OVERVIEW, lastScan }), startScan });
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Scan now' }));

      await vi.waitFor(() => {
        expect(startScan).toHaveBeenCalledOnce();
      });
    });
    it('opens the Ignored section on the tab the navigation names', async () => {
      core();
      const navigation = createNavigation();
      navigation.navigate({ kind: 'settings', tab: 'items' });
      render(
        <AppProviders navigation={navigation} store={createCoreStore()}>
          <SettingsScreen />
        </AppProviders>,
      );

      expect(await screen.findByRole('tab', { name: 'Items' })).toHaveAttribute('aria-selected', 'true');
    });

    it("Fresher's own data is on General, under the core's rows", async () => {
      core();
      await renderScreen();

      await general();
      expect(screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent)).toEqual([
        "Fresher's own data",
      ]);
      expect(screen.getByRole('button', { name: 'Reset all settings…' })).toBeInTheDocument();
    });

    it('the screen opens on General without a page', async () => {
      core();
      await renderScreen();

      expect(await general()).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'General' })).toHaveAttribute('aria-current', 'page');
    });

    it('the current page is marked current and no other is', async () => {
      core();
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Scan' }));

      expect(await screen.findByRole('heading', { level: 1, name: 'Scan' })).toBeInTheDocument();
      const tree = screen.getByRole('navigation', { name: 'Settings sections' });
      expect(within(tree).getByRole('button', { name: 'Scan' })).toHaveAttribute('aria-current', 'page');
      expect(within(tree).getByRole('button', { name: 'General' })).not.toHaveAttribute('aria-current');
    });

    it('a click on a module opens its page with its switch last', async () => {
      core();
      await renderScreen();

      fireEvent.click(await screen.findByRole('button', { name: 'Debug' }));
      fireEvent.click(screen.getByRole('button', { name: 'Demo module' }));

      expect(await screen.findByRole('heading', { level: 1, name: 'Demo module' })).toBeInTheDocument();
      expect(screen.getByText('Shows made-up example findings.')).toBeInTheDocument();
      const rows = screen.getAllByRole('group');
      expect(rows.map((row) => row.getAttribute('data-kind'))).toEqual(['number', 'switch']);
      expect(within(rows[1] as HTMLElement).getByRole('switch')).toBeInTheDocument();
      expect(screen.getByText(/Off: the scan skips them/u)).toBeInTheDocument();
    });

    it('a closed group shows its modules once opened and hides them once closed again', async () => {
      core();
      await renderScreen();
      const group = await screen.findByRole('button', { name: 'Debug' });

      expect(group).toHaveAttribute('aria-expanded', 'false');
      fireEvent.click(group);
      expect(screen.getByRole('button', { name: 'Demo module' })).toBeInTheDocument();
      fireEvent.click(group);
      expect(screen.queryByRole('button', { name: 'Demo module' })).toBeNull();
    });

    it('the group of the current page cannot be closed while it holds the page', async () => {
      core();
      await renderScreen(DEMO_PAGE);
      const group = await screen.findByRole('button', { name: 'Debug' });

      fireEvent.click(group);

      expect(group).toHaveAttribute('aria-expanded', 'true');
      expect(screen.getByRole('button', { name: 'Demo module' })).toBeInTheDocument();
    });

    it('the group of the current page is open', async () => {
      core();
      await renderScreen(DEMO_PAGE);

      expect(await screen.findByRole('button', { name: 'Debug' })).toHaveAttribute('aria-expanded', 'true');
      expect(screen.getByRole('button', { name: 'Demo module' })).toHaveAttribute('aria-current', 'page');
    });

    it('a switch change is sent and the event updates the row', async () => {
      const sent: SettingUpdate[] = [];
      core({
        updateSetting: (args) => {
          sent.push((args as { readonly update: SettingUpdate }).update);

          return null;
        },
      });
      await renderScreen(DEMO_PAGE);

      fireEvent.click(await demoSwitch());
      await vi.waitFor(() => {
        expect(sent).toEqual([{ key: 'demo.enabled', value: { kind: 'switch', value: false } }]);
      });
      expect(screen.getByRole('switch', { name: 'Look for Demo module' })).toBeChecked();

      act(() => {
        pushEvents({ key: 'demo.enabled', type: 'setting-changed', value: { kind: 'switch', value: false } });
      });

      expect(screen.getByRole('switch', { name: 'Look for Demo module' })).not.toBeChecked();
    });

    it('the version and the licence show under About', async () => {
      core();
      await renderScreen({ kind: 'about' });

      const about = await screen.findByRole('region', { name: 'About' });
      expect(await within(about).findByText('Version 1.2.3')).toBeInTheDocument();
      expect(within(about).getByText(/MIT or Apache 2.0/u)).toBeInTheDocument();
    });

    it('About shows the five lines and the acceptance date', async () => {
      core({ getAgreement: () => AGREEMENT_ACCEPTED });
      await renderScreen({ kind: 'about' });

      const about = await screen.findByRole('region', { name: 'About' });

      expect(await within(about).findByText('You accepted it on Jan 1, 2025.')).toBeInTheDocument();
      expect(within(about).getAllByRole('listitem')).toHaveLength(5);
    });

    it('About opens the full licence', async () => {
      core();
      await renderScreen({ kind: 'about' });

      fireEvent.click(await screen.findByRole('button', { name: 'Read the full licence' }));

      expect(screen.getByRole('dialog', { name: 'Licence' })).toBeInTheDocument();
    });

    it('About opens the open-source licences bundled into the app', async () => {
      core();
      await renderScreen({ kind: 'about' });

      fireEvent.click(await screen.findByRole('button', { name: 'Show the open-source licences' }));

      const dialog = screen.getByRole('dialog', { name: 'Open-source licences' });
      expect(await within(dialog).findByText(/react-dom 19/u)).toBeInTheDocument();
      expect(within(dialog).getByText(/crates\.io\/crates\/cssparser\//u)).toBeInTheDocument();
    });
  });
});

describe('the settings search', () => {
  const field = (): Promise<HTMLElement> => screen.findByRole('searchbox', { name: 'Search settings' });
  const search = async (query: string): Promise<HTMLElement> => {
    const input = await field();

    fireEvent.change(input, { target: { value: query } });

    return input;
  };

  describe('negative cases', () => {
    it('says nothing matches and keeps the settings as they were', async () => {
      core();
      await renderScreen();
      await search('zebra');

      expect(screen.getByRole('heading', { level: 1, name: 'Nothing matches “zebra”' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /Example findings/u })).not.toBeInTheDocument();
    });

    it('Esc clears and shows the page again', async () => {
      core();
      await renderScreen();
      const input = await search('findings');

      expect(screen.queryByRole('heading', { level: 1, name: 'General' })).not.toBeInTheDocument();
      fireEvent.keyDown(input, { key: 'Escape' });

      expect(input).toHaveValue('');
      expect(await general()).toBeInTheDocument();
    });

    it('Enter with no hit changes nothing', async () => {
      core();
      await renderScreen();
      const input = await search('zebra');

      fireEvent.keyDown(input, { key: 'Enter' });

      expect(input).toHaveValue('zebra');
    });

    it('Enter on the clear button opens no hit', async () => {
      core();
      await renderScreen();
      await search('findings');

      fireEvent.keyDown(screen.getByRole('button', { name: 'Clear the search' }), { key: 'Enter' });

      expect(screen.getByRole('heading', { level: 1, name: '1 setting matches “findings”' })).toBeInTheDocument();
      expect(screen.queryByRole('heading', { level: 1, name: 'Demo module' })).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('lists the matches, the matched text marked', async () => {
      core();
      await renderScreen();
      await search('findings');

      expect(screen.getByRole('heading', { level: 1, name: '1 setting matches “findings”' })).toBeInTheDocument();
      const hit = screen.getByRole('button', { name: /Example findings/u });

      expect(hit).toHaveTextContent('Debug › Demo module');
      expect(hit).toHaveTextContent('8. How many made-up findings the demo module shows.');
      expect(within(hit).getAllByText('findings', { selector: 'mark' })).toHaveLength(2);
    });

    it('the tree counts matches and dims the rest', async () => {
      core();
      await renderScreen();
      await search('findings');

      const tree = screen.getByRole('navigation', { name: 'Settings sections' });

      expect(within(tree).getByRole('button', { name: 'Demo module 1' })).toBeInTheDocument();
      expect(within(tree).getByRole('button', { name: 'Scan' })).toHaveAttribute('data-dimmed');
      expect(within(tree).getByRole('button', { name: 'Demo module 1' })).not.toHaveAttribute('data-dimmed');
    });

    it('Enter opens the first hit with its setting focused', async () => {
      core();
      await renderScreen();
      const input = await search('findings');

      fireEvent.keyDown(input, { key: 'Enter' });

      expect(input).toHaveValue('');
      expect(await screen.findByRole('heading', { level: 1, name: 'Demo module' })).toBeInTheDocument();
      await vi.waitFor(() => {
        expect(screen.getByRole('slider', { name: 'Example findings' })).toHaveFocus();
      });
    });

    it('a click on a section hit opens its page', async () => {
      core();
      await renderScreen();
      await search('permissions');

      fireEvent.click(within(screen.getByRole('list')).getByRole('button', { name: 'Permissions' }));

      expect(await screen.findByRole('heading', { level: 1, name: 'Permissions' })).toBeInTheDocument();
    });
  });
});
