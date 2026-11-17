import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CategoryId, FindingsQuery, ModuleInfo, Overview } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { DEMO_FINDINGS, DEMO_MODULES, DEMO_OVERVIEW, IDLE, SCAN_STARTED } from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { ModuleScreen } from './module-screen';

const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const DICTIONARY = {
  ...FINDING_DICTIONARY,
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
  });

  describe('positive cases', () => {
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
