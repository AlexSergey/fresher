import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { IgnoredEntry, IgnoreTarget, ModuleInfo, SettingsView } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';
import type { IgnoredTab } from '../../state/navigation';

import { mockCore, sampleAgreementAnswers } from '../../ipc/mock';
import { createCoreStore } from '../../state/store';
import { DEMO_DICTIONARY, DEMO_MODULES } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { IgnoredGroup } from './ignored-group';

const MODULES: readonly ModuleInfo[] = [
  ...DEMO_MODULES,
  { categories: [], choices: [], group: 'development', id: 'jetbrains', settings: [] },
];

const DICTIONARY = {
  ...DEMO_DICTIONARY,
  'core.group.debug.description': 'Modules of a development build.',
  'core.group.development.description': 'Tools for building software.',
  'core.group.system.description': 'What macOS and your apps keep for themselves.',
  'core.ignore.groups.description': 'Hides everything the modules of this group find.',
  'core.ignore.modules.description': 'Fresher stops looking for these and never removes what they find.',
  'jetbrains.enabled.label': 'JetBrains',
};

const view = (values: SettingsView['values'] = {}): SettingsView => ({
  file: 'loaded',
  problems: [],
  settings: [],
  values,
});

type Sent = { readonly ignored: IgnoreTarget[]; readonly restored: IgnoredEntry[] };

const show = async (
  values: SettingsView['values'] = {},
  extra: { readonly opensOn?: IgnoredTab; readonly readOnly?: boolean } = {},
): Promise<Sent> => {
  const sent: Sent = { ignored: [], restored: [] };
  const answers: Partial<CoreAnswers> = {
    ...sampleAgreementAnswers,
    getDictionary: () => DICTIONARY,
    ignore: (args) => {
      sent.ignored.push((args as { readonly target: IgnoreTarget }).target);

      return null;
    },
    restoreIgnored: (args) => {
      sent.restored.push((args as { readonly entry: IgnoredEntry }).entry);

      return null;
    },
  };
  const store = createCoreStore();

  mockCore(answers);
  const { rerender } = render(<AppProviders store={store}>{null}</AppProviders>);

  await vi.waitFor(() => {
    expect(store.getState().dictionary).not.toBeNull();
  });
  rerender(
    <AppProviders store={store}>
      <IgnoredGroup
        headingId="ignored"
        modules={MODULES}
        readOnly={extra.readOnly ?? false}
        view={view(values)}
        {...(extra.opensOn === undefined ? {} : { opensOn: extra.opensOn })}
      />
    </AppProviders>,
  );

  return sent;
};

const IGNORED_DEVELOPMENT: SettingsView['values'] = { 'core.ignore.groups': { kind: 'ids', value: ['development'] } };

describe('IgnoredGroup', () => {
  describe('negative cases', () => {
    it('a read-only file disables every switch and Restore', async () => {
      await show(
        { ...IGNORED_DEVELOPMENT, 'core.ignore.modules': { kind: 'ids', value: ['demo'] } },
        { readOnly: true },
      );

      for (const control of screen.getAllByRole('switch')) {
        expect(control).toBeDisabled();
      }
      expect(screen.getByRole('button', { name: 'Restore Development' })).toBeDisabled();
      fireEvent.click(screen.getByRole('tab', { name: 'Not looked for · 1' }));
      expect(screen.getByRole('button', { name: 'Restore Demo module' })).toBeDisabled();
    });

    it('a module of an ignored group has its switch on and locked, with the note, and sends nothing', async () => {
      const sent = await show(IGNORED_DEVELOPMENT, { opensOn: 'modules' });

      const lock = screen.getByRole('switch', { name: 'Ignore JetBrains' });
      expect(lock).toBeChecked();
      expect(lock).toBeDisabled();
      expect(screen.getByText('Ignored with its section: Development')).toBeInTheDocument();
      expect(screen.getByRole('switch', { name: 'Ignore Demo module' })).toBeEnabled();
      fireEvent.click(lock);
      expect(sent).toEqual({ ignored: [], restored: [] });
    });

    it('a group with no module is not listed', async () => {
      await show();

      expect(screen.getByRole('switch', { name: 'Ignore Development' })).toBeInTheDocument();
      expect(screen.getAllByRole('switch')).toHaveLength(2);
    });

    it('says nothing is ignored in Items when it is empty', async () => {
      await show({}, { opensOn: 'items' });

      expect(screen.getByText('Nothing is ignored.')).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the counts on the tabs', async () => {
      await show({
        ...IGNORED_DEVELOPMENT,
        'core.ignore.categories': { kind: 'ids', value: ['demo.caches'] },
        'core.ignore.modules': { kind: 'ids', value: ['demo'] },
        'core.ignore.paths': { kind: 'paths', value: ['/Users/ann/keep'] },
      });

      expect(screen.getByRole('tab', { name: 'Sections · 1' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Not looked for · 1' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Items · 2' })).toBeInTheDocument();
    });

    it('opens on the first tab with an entry', async () => {
      await show({ 'core.ignore.paths': { kind: 'paths', value: ['/a'] } });
      expect(screen.getByRole('tab', { name: 'Items · 1' })).toHaveAttribute('aria-selected', 'true');
    });

    it('opens on Sections when nothing is ignored', async () => {
      await show();

      expect(screen.getByRole('tab', { name: 'Sections' })).toHaveAttribute('aria-selected', 'true');
    });

    it('opens on the tab it is told', async () => {
      await show({}, { opensOn: 'modules' });

      expect(screen.getByRole('tab', { name: 'Not looked for' })).toHaveAttribute('aria-selected', 'true');
    });

    it('a group switch ignores the group and a turned-off one restores it', async () => {
      const sent = await show(IGNORED_DEVELOPMENT);

      fireEvent.click(screen.getByRole('switch', { name: 'Ignore Debug' }));
      fireEvent.click(screen.getByRole('switch', { name: 'Ignore Development' }));
      fireEvent.click(screen.getByRole('button', { name: 'Restore Development' }));

      await vi.waitFor(() => {
        expect(sent.ignored).toEqual([{ group: 'debug', kind: 'group' }]);
        expect(sent.restored).toEqual([
          { group: 'development', kind: 'group' },
          { group: 'development', kind: 'group' },
        ]);
      });
    });

    it('a module switch ignores the module and Restore restores it, under the group name', async () => {
      const sent = await show({ 'core.ignore.modules': { kind: 'ids', value: ['demo'] } });
      const debug = screen.getByRole('region', { name: 'Debug' });

      fireEvent.click(within(debug).getByRole('button', { name: 'Restore Demo module' }));
      fireEvent.click(screen.getByRole('switch', { name: 'Ignore JetBrains' }));

      await vi.waitFor(() => {
        expect(sent.restored).toEqual([{ kind: 'module', module: 'demo' }]);
        expect(sent.ignored).toEqual([{ kind: 'module', module: 'jetbrains' }]);
      });
    });

    it('lists an ignored module nobody registers, so that it can be restored', async () => {
      const sent = await show({ 'core.ignore.modules': { kind: 'ids', value: ['gone'] } });

      fireEvent.click(screen.getByRole('button', { name: 'Restore gone' }));

      await vi.waitFor(() => {
        expect(sent.restored).toEqual([{ kind: 'module', module: 'gone' }]);
      });
    });
  });
});
