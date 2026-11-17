import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CategoryId, FindingsQuery, LeftOutCount, ModuleInfo, Overview } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  DEMO_DICTIONARY,
  DEMO_FINDINGS,
  DEMO_LEFT_OUT,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  DEMO_SETTINGS,
  IDLE,
  SCAN_STARTED,
} from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { ModuleScreen } from './module-screen';

const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const DICTIONARY = {
  ...FINDING_DICTIONARY,
  ...DEMO_DICTIONARY,
  'demo.caches.comes-back': 'Yes, as the apps run.',
  'demo.caches.made-by': 'Apps that keep caches.',
  'demo.caches.what': 'Files apps keep to start faster.',
  'demo.downloads.comes-back': 'Only if you download them again.',
  'demo.downloads.made-by': 'Your browser.',
  'demo.downloads.what': 'Installers you downloaded.',
  'demo.failed': 'The demo folder could not be read.',
};

const sized = (bytes: number): Overview['found'] => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// The demo scan with the downloads grown past the caches.
const DOWNLOADS_LARGEST: Overview = {
  ...DEMO_OVERVIEW,
  totals: [
    { category: 'demo.caches', count: 2, highestDanger: 'safe', sizes: sized(1_820_000_000) },
    { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: sized(5_000_000_000) },
  ],
};

// Every `get_findings` the page asked, by its query.
let asked: FindingsQuery[] = [];

const core = (answers: Partial<CoreAnswers> = {}): void => {
  asked = [];
  mockCore({
    getDictionary: () => DICTIONARY,
    getFindings: (args) => {
      const { query } = args as { readonly query: FindingsQuery };
      asked.push(query);

      return pageOf(DEMO_FINDINGS, query);
    },
    getModules: () => DEMO_MODULES,
    getOverview: () => DEMO_OVERVIEW,
    ...sampleStorageAnswers,
    ...answers,
  });
};

type Rendered = {
  readonly navigation: Navigation;
  readonly rerender: (category?: CategoryId) => void;
  readonly store: CoreStore;
};

// The page inside the app's providers, once the core's dictionary has arrived.
const renderPage = async (category?: CategoryId): Promise<Rendered> => {
  const navigation = createNavigation();
  const store = createCoreStore();
  const page = (at?: CategoryId): React.JSX.Element => (
    <AppProviders navigation={navigation} store={store}>
      <ModuleScreen category={at} module={DEMO} />
    </AppProviders>
  );
  const { rerender } = render(page(category));
  await screen.findByRole('heading', { level: 1, name: 'Demo module' });

  return {
    navigation,
    rerender: (at): void => {
      rerender(page(at));
    },
    store,
  };
};

// A finished scan whose module left `leftOut` out, with the demo's totals loaded.
const scanned = (store: CoreStore, leftOut: readonly LeftOutCount[], overview: Overview = DEMO_OVERVIEW): void => {
  act(() => {
    store.dispatch(SCAN_STARTED);
    store.dispatch({
      leftOut: [...leftOut],
      module: 'demo',
      outcome: { kind: 'completed' },
      scan: 1,
      type: 'module-finished',
    });
    store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });
    store.dispatch({ overview, type: 'overview-loaded' });
  });
};

const section = (name: string): HTMLElement => screen.getByRole('region', { name });

const sectionNames = (): readonly (null | string)[] =>
  screen
    .getAllByRole('region')
    .map((region) => region.getAttribute('aria-label'))
    .filter((name) => name?.startsWith('Example') === true);

const disclosure = (name: string): HTMLElement => screen.getByRole('button', { name: `Show the items of ${name}` });

const selectionBar = (): HTMLElement | null => screen.queryByRole('region', { name: 'Selection' });

const askedFor = (category: string): number => asked.filter((query) => query.category === category).length;

beforeEach(async () => {
  await setMotion('off');
  fakeViewport(520);
});

afterEach(async () => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('ModuleScreen', () => {
  describe('negative cases', () => {
    it('shows no category before a scan', async () => {
      core({ getOverview: () => IDLE });

      await renderPage();

      expect(await screen.findByText('Not scanned yet')).toBeInTheDocument();
      expect(sectionNames()).toStrictEqual([]);
    });

    it('a category without findings has no section', async () => {
      core({ getOverview: () => ({ ...DEMO_OVERVIEW, totals: DEMO_OVERVIEW.totals.slice(0, 1) }) });

      await renderPage();

      expect(await screen.findByRole('region', { name: 'Example caches' })).toBeInTheDocument();
      expect(screen.queryByRole('region', { name: 'Example downloads' })).toBeNull();
    });

    it('shows no left-out line when nothing was left out', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();

      scanned(store, []);

      expect(screen.queryByRole('list', { name: 'Left out of this list' })).toBeNull();
    });

    it('says so when the module found nothing', async () => {
      core({ getOverview: () => ({ ...DEMO_OVERVIEW, found: sized(0), totals: [] }) });

      await renderPage();

      expect(await screen.findByText('No findings in this module')).toBeInTheDocument();
    });

    it('a collapsed category requests no page', async () => {
      core();

      await renderPage();

      expect(await screen.findByRole('listitem', { name: /^Example: editor-cache/ })).toBeInTheDocument();
      expect(disclosure('Example downloads')).toHaveAttribute('aria-expanded', 'false');
      expect(askedFor('demo.downloads')).toBe(0);
    });

    it('nothing is selected when the page opens', async () => {
      core();

      await renderPage();
      await screen.findByRole('listitem', { name: /^Example: editor-cache/ });

      for (const box of screen.getAllByRole('checkbox')) {
        expect(box).not.toBeChecked();
      }
      expect(selectionBar()).toBeNull();
    });

    it('a module that failed shows its reason and the findings it sent before', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();

      act(() => {
        store.dispatch(SCAN_STARTED);
        store.dispatch({
          leftOut: [],
          module: 'demo',
          outcome: { kind: 'failed', reason: { args: {}, key: 'demo.failed' } },
          scan: 1,
          type: 'module-finished',
        });
        store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' });
        store.dispatch({ overview: DEMO_OVERVIEW, type: 'overview-loaded' });
      });

      expect(screen.getByText('The demo folder could not be read.')).toBeInTheDocument();
      expect(screen.getByText('Could not finish')).toBeInTheDocument();
      expect(sectionNames()).toStrictEqual(['Example caches', 'Example downloads']);
    });

    it('a category with only left-outs has neither a box nor a disclosure', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();

      scanned(store, [{ ...DEMO_LEFT_OUT, category: 'demo.downloads' }], {
        ...DEMO_OVERVIEW,
        totals: DEMO_OVERVIEW.totals.slice(0, 1),
      });

      const downloads = section('Example downloads');

      expect(within(downloads).queryByRole('checkbox')).toBeNull();
      expect(within(downloads).queryByRole('button', { name: 'Show the items of Example downloads' })).toBeNull();
      expect(within(downloads).getByRole('list', { name: 'Left out of this list' })).toBeVisible();
    });

    it('Ignore on a category with only left-outs removes its section', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();
      act(() => {
        store.dispatch({ settings: DEMO_SETTINGS, type: 'settings-loaded' });
      });
      scanned(store, [{ ...DEMO_LEFT_OUT, category: 'demo.downloads' }], {
        ...DEMO_OVERVIEW,
        totals: DEMO_OVERVIEW.totals.slice(0, 1),
      });
      expect(section('Example downloads')).toBeInTheDocument();

      act(() => {
        store.dispatch({
          key: 'core.ignore.categories',
          type: 'setting-changed',
          value: { kind: 'ids', value: ['demo.downloads'] },
        });
      });

      expect(screen.queryByRole('region', { name: 'Example downloads' })).toBeNull();
      expect(sectionNames()).toStrictEqual(['Example caches']);
    });

    it('a visible module shows no note and offers its selection', async () => {
      core();
      await renderPage();

      expect(screen.queryByText('You ignore this module. Its findings are not shown or cleaned.')).toBeNull();
      expect(await screen.findByRole('button', { name: 'Ignore this module' })).toBeVisible();
      expect(await screen.findByRole('region', { name: 'Example caches' })).toBeVisible();
    });

    it.each([['module'], ['group']] as const)(
      'a module the core hides by its %s shows no category, no left-out and no selection',
      async (by) => {
        core({ getOverview: () => IDLE });
        const { store } = await renderPage();
        scanned(store, [{ ...DEMO_LEFT_OUT, category: 'demo.downloads' }], {
          ...DEMO_OVERVIEW,
          hiddenModules: [{ by, module: 'demo' }],
          totals: [],
        });

        expect(await screen.findByText('You ignore this module. Its findings are not shown or cleaned.')).toBeVisible();
        expect(screen.queryByRole('region', { name: 'Example downloads' })).toBeNull();
        expect(screen.queryByRole('checkbox')).toBeNull();
        expect(screen.queryByRole('button', { name: 'Ignore this module' })).toBeNull();
      },
    );
  });

  describe('positive cases', () => {
    it('shows the line of a left-out under its category', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();

      scanned(store, [DEMO_LEFT_OUT]);

      const line = within(section('Example caches')).getByRole('list', { name: 'Left out of this list' });

      expect(line).toHaveTextContent(
        '1 item left out (157.3 MB): The app that made it is open. Quit the app, then scan again.',
      );
      expect(within(section('Example downloads')).queryByRole('list', { name: 'Left out of this list' })).toBeNull();
    });

    it('shows the line for a category without findings', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderPage();

      scanned(store, [{ ...DEMO_LEFT_OUT, category: 'demo.downloads' }], {
        ...DEMO_OVERVIEW,
        totals: DEMO_OVERVIEW.totals.slice(0, 1),
      });

      expect(within(section('Example downloads')).getByRole('list', { name: 'Left out of this list' })).toBeVisible();
      expect(screen.queryByText('No findings in this module')).toBeNull();
    });

    it.each([
      ['module', 'Restore', { kind: 'module', module: 'demo' }],
      ['group', 'Restore the group', { group: 'debug', kind: 'group' }],
    ] as const)('a module hidden by its %s offers %s', async (by, label, entry) => {
      const restored: unknown[] = [];
      core({
        getOverview: () => ({ ...DEMO_OVERVIEW, hiddenModules: [{ by, module: 'demo' }] }),
        restoreIgnored: (args) => {
          restored.push((args as { readonly entry: unknown }).entry);

          return null;
        },
      });
      const { store } = await renderPage();
      scanned(store, [], { ...DEMO_OVERVIEW, hiddenModules: [{ by, module: 'demo' }] });

      fireEvent.click(await screen.findByRole('button', { name: label }));

      await vi.waitFor(() => {
        expect(restored).toEqual([entry]);
      });
    });

    it('Ignore this module asks the core to ignore the module', async () => {
      const ignored: unknown[] = [];
      core({
        ignore: (args) => {
          ignored.push((args as { readonly target: unknown }).target);

          return null;
        },
      });
      await renderPage();

      fireEvent.click(await screen.findByRole('button', { name: 'Ignore this module' }));

      await vi.waitFor(() => {
        expect(ignored).toEqual([{ kind: 'module', module: 'demo' }]);
      });
    });

    it('Ignore on a category asks the core to ignore it', async () => {
      const ignored: unknown[] = [];
      core({
        ignore: (args) => {
          ignored.push((args as { readonly target: unknown }).target);

          return null;
        },
      });
      await renderPage();

      fireEvent.click(await screen.findByRole('button', { name: 'Ignore Example caches' }));

      await vi.waitFor(() => {
        expect(ignored).toEqual([{ category: 'demo.caches', kind: 'category' }]);
      });
    });

    it('sections follow size order', async () => {
      core({ getOverview: () => DOWNLOADS_LARGEST });

      await renderPage();
      await screen.findByRole('region', { name: 'Example downloads' });

      expect(sectionNames()).toStrictEqual(['Example downloads', 'Example caches']);
      expect(disclosure('Example downloads')).toHaveAttribute('aria-expanded', 'true');
      expect(disclosure('Example caches')).toHaveAttribute('aria-expanded', 'false');
    });

    it("shows a category's four sentences on demand", async () => {
      core();

      await renderPage();
      const caches = await screen.findByRole('region', { name: 'Example caches' });

      expect(within(caches).getByText('About this category')).toBeInTheDocument();
      expect(within(caches).getByText('Files apps keep to start faster.')).toBeInTheDocument();
      expect(within(caches).getByText('Apps that keep caches.')).toBeInTheDocument();
      expect(
        within(caches).getByText('The apps rebuild their caches the next time they need them.'),
      ).toBeInTheDocument();
      expect(within(caches).getByText('Yes, as the apps run.')).toBeInTheDocument();
    });

    it('expanding a category loads its first page', async () => {
      core();
      await renderPage();
      await screen.findByRole('region', { name: 'Example downloads' });

      fireEvent.click(disclosure('Example downloads'));

      expect(await screen.findByRole('listitem', { name: /^Example: installer\.dmg/ })).toBeInTheDocument();
      expect(asked).toContainEqual({
        category: 'demo.downloads',
        limit: 100,
        module: 'demo',
        offset: 0,
        order: 'largest',
      });
    });

    it('ticking rows updates the selection bar', async () => {
      core();
      await renderPage();
      await screen.findByRole('listitem', { name: /^Example: editor-cache/ });

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' }));

      expect(within(selectionBar() ?? document.body).getByText('1 item selected')).toBeInTheDocument();
      expect(screen.getByRole('checkbox', { name: 'Select Example caches' })).toHaveProperty('indeterminate', true);
    });

    it('ticking the category selects it whole', async () => {
      core();
      await renderPage();
      await screen.findByRole('listitem', { name: /^Example: editor-cache/ });

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      expect(screen.getByRole('checkbox', { name: 'Select Example caches' })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: browser-cache' })).toBeChecked();
      expect(within(selectionBar() ?? document.body).getByText('2 items selected')).toBeInTheDocument();
    });

    it('Review goes to the review screen', async () => {
      core();
      const { navigation } = await renderPage();
      await screen.findByRole('region', { name: 'Example caches' });
      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example caches' }));

      fireEvent.click(screen.getByRole('button', { name: 'Review' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'review' });
    });

    it('opened at a category, it is expanded and in view', async () => {
      const scrolled = vi.spyOn(HTMLElement.prototype, 'scrollIntoView');
      core();

      await renderPage('demo.downloads');
      const downloads = await screen.findByRole('region', { name: 'Example downloads' });

      expect(disclosure('Example downloads')).toHaveAttribute('aria-expanded', 'true');
      expect(scrolled).toHaveBeenCalledTimes(1);
      expect(scrolled.mock.contexts[0]).toBe(downloads);
    });

    it('opened again at another category, it expands that one and scrolls to it', async () => {
      const scrolled = vi.spyOn(HTMLElement.prototype, 'scrollIntoView');
      core({ getOverview: () => DOWNLOADS_LARGEST });
      const { rerender } = await renderPage();
      await screen.findByRole('region', { name: 'Example caches' });
      fireEvent.click(disclosure('Example downloads'));
      expect(scrolled).not.toHaveBeenCalled();

      rerender('demo.caches');

      expect(disclosure('Example caches')).toHaveAttribute('aria-expanded', 'true');
      expect(disclosure('Example downloads')).toHaveAttribute('aria-expanded', 'false');
      expect(scrolled.mock.contexts[0]).toBe(section('Example caches'));
    });
  });
});
