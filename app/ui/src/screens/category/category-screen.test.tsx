import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  AccessStatus,
  Availability,
  CategoryId,
  Dictionary,
  FindingsQuery,
  LeftOutCount,
  ModuleInfo,
  Overview,
  Refusal,
} from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, refuse, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { type CoreStore, createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  APP_LEFTOVERS_DICTIONARY,
  APP_LEFTOVERS_FINDINGS,
  APP_LEFTOVERS_MODULES,
  APP_LEFTOVERS_OVERVIEW,
} from '../../test/app-leftovers-scan';
import {
  ACCESS_DICTIONARY,
  DEMO_DICTIONARY,
  DEMO_FINDINGS,
  DEMO_LEFT_OUT,
  DEMO_MODULES,
  DEMO_OVERVIEW,
  IDLE,
  SCAN_STARTED,
} from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { CategoryScreen } from './category-screen';

const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];
const [APP_LEFTOVERS] = APP_LEFTOVERS_MODULES as readonly [ModuleInfo];

const DICTIONARY: Dictionary = {
  ...ACCESS_DICTIONARY,
  ...FINDING_DICTIONARY,
  ...DEMO_DICTIONARY,
  ...APP_LEFTOVERS_DICTIONARY,
  'core.danger.safe.zone': 'Safe to remove',
  'demo.caches.comes-back': 'Yes, as the apps run.',
  'demo.caches.if-removed': 'The apps start a little slower once.',
  'demo.caches.made-by': 'Apps that keep caches.',
  'demo.docker.note': 'Docker is installed but not running. Start it, then scan again.',
};

const text = (key: string): string => DICTIONARY[key] ?? key;

const DOCKER = { kind: 'tool', value: 'docker' } as const;

const DOCKER_NOTE = 'Docker is installed but not running. Start it, then scan again.';

const DOCKER_STATUS: AccessStatus = {
  capabilities: [{ access: 'denied', capability: DOCKER, note: { args: {}, key: 'demo.docker.note' } }],
};

// The left-out line of the demo's downloads, beside the caches' own.
const DOWNLOADS_LEFT_OUT: LeftOutCount = {
  category: 'demo.downloads',
  items: 2,
  reason: { args: {}, key: 'demo.left-out.open' },
  sizes: { allocated: 4_000_000, completeness: 'complete', logical: 4_000_000, reclaimable: 4_000_000 },
};

const NOTHING_TO_REVEAL: Refusal = {
  error: { code: 'nothing-to-reveal' },
  text: { args: {}, key: 'core.error.nothing-to-reveal' },
};

// What the core says when a scan replaced the finding the user chose to ignore.
const UNKNOWN_FINDING: Refusal = {
  error: { code: 'unknown-finding', id: 1 },
  text: { args: {}, key: 'core.error.unknown-finding' },
};

// The demo findings with the editor cache owned by another user: the core says it needs an administrator.
const WITH_ADMIN_ITEM = DEMO_FINDINGS.map((finding) =>
  finding.id === 1 ? { ...finding, removability: 'needs-administrator' as const } : finding,
);

// The demo findings with the editor cache an object of a tool: it has no path to ignore.
const WITH_TOOL_ITEM = DEMO_FINDINGS.map((finding) =>
  finding.id === 1
    ? { ...finding, subject: { kind: 'external' as const, value: { id: 'editor', tool: DOCKER.value } } }
    : finding,
);

const withToolItem: CoreAnswers['getFindings'] = (args) =>
  pageOf(WITH_TOOL_ITEM, (args as { readonly query: FindingsQuery }).query);

// The overview of a scan that found no caches any more.
const WITHOUT_CACHES: Overview = {
  ...DEMO_OVERVIEW,
  totals: DEMO_OVERVIEW.totals.filter((total) => total.category !== 'demo.caches'),
};

const withAdminItem: CoreAnswers['getFindings'] = (args) =>
  pageOf(WITH_ADMIN_ITEM, (args as { readonly query: FindingsQuery }).query);

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({
    getDictionary: () => DICTIONARY,
    getFindings: (args) =>
      pageOf([...DEMO_FINDINGS, ...APP_LEFTOVERS_FINDINGS], (args as { query: FindingsQuery }).query),
    getModules: () => [...DEMO_MODULES, ...APP_LEFTOVERS_MODULES],
    getOverview: () => DEMO_OVERVIEW,
    ...sampleStorageAnswers,
    ...answers,
  });
};

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore };

// The details of `category` inside the app's providers, once the core's dictionary has arrived.
const renderDetails = async (category: CategoryId, module: ModuleInfo = DEMO): Promise<Rendered> => {
  const navigation = createNavigation();
  const store = createCoreStore();

  navigation.navigate({ category, kind: 'category' });
  render(
    <AppProviders navigation={navigation} store={store}>
      <CategoryScreen category={category} module={module} />
    </AppProviders>,
  );
  await screen.findByRole('heading', { level: 1, name: text(`${category}.name`) });

  return { navigation, store };
};

// A finished scan of the demo module that left `leftOut` out, with `overview` loaded.
const scanned = (store: CoreStore, leftOut: readonly LeftOutCount[], overview: Overview = DEMO_OVERVIEW): void => {
  act(() => {
    store.dispatch(SCAN_STARTED);
    store.dispatch({
      insights: [],
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

// The demo module with its downloads in the given availability; its caches stay ready.
const demoWith = (availability: Availability): ModuleInfo => ({
  ...DEMO,
  categories: [
    DEMO.categories[0] as ModuleInfo['categories'][number],
    { ...(DEMO.categories[1] as ModuleInfo['categories'][number]), availability },
  ],
});

const selectionBar = (): HTMLElement | null => screen.queryByRole('region', { name: 'Selection' });

const leftOutLines = (): HTMLElement | null => screen.queryByRole('list', { name: 'Left out of this list' });

const choice = (name: string): HTMLElement | null =>
  screen.queryByRole('button', { name: text(`app-leftovers.choice.${name}.label`) });

beforeEach(async () => {
  await setMotion('off');
  fakeViewport(520);
});

afterEach(async () => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  await setMotion(null);
});

describe('CategoryScreen', () => {
  describe('negative cases', () => {
    it('shows the empty state when the last scan found nothing of the category', async () => {
      core({ getOverview: () => ({ ...DEMO_OVERVIEW, totals: DEMO_OVERVIEW.totals.slice(0, 1) }) });

      await renderDetails('demo.downloads');

      expect(await screen.findByText('Nothing found here')).toBeInTheDocument();
      expect(screen.getByText('The last scan found nothing of this kind.')).toBeInTheDocument();
      expect(screen.queryByRole('checkbox', { name: 'Select all Example downloads' })).toBeNull();
      expect(screen.queryByRole('list', { name: 'The items of Example downloads' })).toBeNull();
    });

    it('shows no choice of the module that does not touch the category', async () => {
      core({ getOverview: () => APP_LEFTOVERS_OVERVIEW });

      await renderDetails('app-leftovers.settings', APP_LEFTOVERS);

      expect(choice('everything')).toBeInTheDocument();
      expect(choice('keep-settings')).toBeNull();
    });

    it('shows no empty state while a scan runs', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.downloads');

      act(() => {
        store.dispatch(SCAN_STARTED);
      });

      expect(screen.queryByText('Nothing found here')).toBeNull();
      expect(screen.queryByText('The last scan found nothing of this kind.')).toBeNull();
    });

    it('does not say nothing was found when everything found was left out', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.downloads');

      scanned(store, [DOWNLOADS_LEFT_OUT], { ...DEMO_OVERVIEW, totals: DEMO_OVERVIEW.totals.slice(0, 1) });

      expect(screen.queryByText('Nothing found here')).toBeNull();
      expect(screen.getByText('Everything found here was left out: the lines above say why.')).toBeVisible();
      expect(within(leftOutLines() as HTMLElement).getAllByRole('listitem')).toHaveLength(1);
    });

    it('an unavailable category says why and offers no checkbox', async () => {
      core({ getAccessStatus: () => DOCKER_STATUS, getOverview: () => IDLE });
      const { store } = await renderDetails('demo.downloads', demoWith({ kind: 'unavailable', missing: [DOCKER] }));

      scanned(store, [], { ...DEMO_OVERVIEW, totals: DEMO_OVERVIEW.totals.slice(0, 1) });

      expect(await screen.findByRole('region', { name: 'Not available' })).toHaveAccessibleDescription(DOCKER_NOTE);
      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByText('Nothing found here')).toBeNull();
    });

    it('nothing is selected when the details open', async () => {
      core();

      await renderDetails('demo.caches');
      await screen.findByRole('listitem', { name: /^Example: editor-cache/u });

      for (const box of screen.getAllByRole('checkbox')) {
        expect(box).not.toBeChecked();
      }
      expect(selectionBar()).toBeNull();
    });

    it('a refused Show in Finder is shown like the other command failures', async () => {
      core({ getFindings: withAdminItem, revealFinding: refuse(NOTHING_TO_REVEAL) });
      const { store } = await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Show in Finder' }));

      await vi.waitFor(() => {
        expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: NOTHING_TO_REVEAL });
      });
    });
    it("a tool's object has no row menu", async () => {
      core({ getFindings: withToolItem });
      await renderDetails('demo.caches');

      await screen.findByRole('listitem', { name: /^Example: editor-cache/u });

      expect(screen.queryByRole('button', { name: 'Options for Example: editor-cache' })).toBeNull();
      expect(screen.getByRole('button', { name: 'Options for Example: browser-cache' })).toBeInTheDocument();
    });

    it('a refused ignore keeps the row and shows the failure', async () => {
      core({ ignore: refuse(UNKNOWN_FINDING) });
      const { store } = await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Options for Example: editor-cache' }));
      fireEvent.click(await screen.findByRole('menuitem', { name: 'Ignore this item' }));

      await vi.waitFor(() => {
        expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: UNKNOWN_FINDING });
      });
      expect(screen.getByRole('listitem', { name: /^Example: editor-cache/u })).toBeInTheDocument();
    });

    it('a scan starting on the details does not move the focus to Back to Scan', async () => {
      core();
      const { store } = await renderDetails('demo.caches');
      await screen.findByRole('listitem', { name: /^Example: editor-cache/u });
      document.body.focus();

      act(() => {
        store.dispatch(SCAN_STARTED);
      });

      expect(screen.getByRole('button', { name: 'Back to Scan' })).not.toHaveFocus();
    });

    it('the focus stays where it is when the list empties without the focus in it', async () => {
      core();
      const { store } = await renderDetails('demo.caches');
      await screen.findByRole('listitem', { name: /^Example: editor-cache/u });
      const scan = screen.getByRole('button', { name: 'Scan' });
      scan.focus();

      act(() => {
        store.dispatch({ overview: WITHOUT_CACHES, type: 'overview-loaded' });
      });

      expect(scan).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Back to Scan' })).not.toHaveFocus();
    });
  });

  describe('positive cases', () => {
    it("names the category, what it is and its level with the core's meaning", async () => {
      core();

      await renderDetails('demo.caches');

      expect(screen.getByText(text('demo.caches.what'), { selector: 'header p' })).toBeVisible();
      const level = screen.getByRole('region', { name: 'Danger level' });
      expect(level).toHaveTextContent('Safe');
      expect(level).toHaveTextContent(FINDING_DICTIONARY['core.danger.safe.meaning'] as string);
      expect(level).toHaveTextContent('Safe to remove');
      const about = screen.getByRole('region', { name: 'About it' });
      expect(about).toHaveTextContent('Apps that keep caches.');
      expect(about).toHaveTextContent('The apps start a little slower once.');
      expect(about).toHaveTextContent('Yes, as the apps run.');
    });

    it('lists the left-out lines of this category only', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT, DOWNLOADS_LEFT_OUT]);

      const lines = within(leftOutLines() as HTMLElement).getAllByRole('listitem');
      expect(lines).toHaveLength(1);
      expect(lines[0]).toHaveTextContent('1 item left out (157.3 MB)');
    });

    it("the head's checkbox selects the whole category", async () => {
      core();
      const { store } = await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select all Example caches' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(screen.getByRole('checkbox', { name: 'Select all Example caches' })).toBeChecked();
    });

    it('shows the selection bar once something is selected', async () => {
      core();
      const { navigation } = await renderDetails('demo.caches');

      expect(selectionBar()).toBeNull();

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select Example: browser-cache' }));

      expect(selectionBar()).toHaveTextContent('1 item');

      fireEvent.click(within(selectionBar() as HTMLElement).getByRole('button', { name: 'Review' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'review' });
    });

    it('Back returns to the overview, naming the category it comes from', async () => {
      core();
      const { navigation } = await renderDetails('demo.caches');

      fireEvent.click(screen.getByRole('button', { name: 'Back to Scan' }));

      expect(navigation.getScreen()).toStrictEqual({ from: 'demo.caches', kind: 'scan' });
    });

    it('a limited category shows its partial size and says what could not be seen', async () => {
      core({
        getOverview: () => ({
          ...DEMO_OVERVIEW,
          totals: [
            ...DEMO_OVERVIEW.totals.slice(0, 1),
            {
              category: 'demo.downloads',
              count: 1,
              highestDanger: 'low',
              sizes: { allocated: 750_000_000, completeness: 'partial', logical: 750_000_000, reclaimable: 0 },
            },
          ],
        }),
      });

      await renderDetails('demo.downloads', demoWith({ kind: 'limited', missing: [{ kind: 'all-files' }] }));

      expect(await screen.findByText(/at least 750 MB/u)).toBeInTheDocument();
      expect(screen.getByText('Some of it could not be seen: Full Disk Access.')).toBeInTheDocument();
    });

    it('Show in Finder asks the core to reveal the finding by its id, never its path', async () => {
      const revealed: unknown[] = [];
      core({
        getFindings: withAdminItem,
        revealFinding: (args) => {
          revealed.push(args);

          return null;
        },
      });
      await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Show in Finder' }));

      await vi.waitFor(() => {
        expect(revealed).toStrictEqual([{ finding: 1 }]);
      });
    });

    it('Ignore this item asks the core to ignore that finding', async () => {
      const ignored: unknown[] = [];
      core({
        ignore: (args) => {
          ignored.push(args);

          return null;
        },
      });
      await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Options for Example: editor-cache' }));
      fireEvent.click(await screen.findByRole('menuitem', { name: 'Ignore this item' }));

      await vi.waitFor(() => {
        expect(ignored).toStrictEqual([{ target: { finding: 1, kind: 'finding' } }]);
      });
    });

    it('the menu item carries its sentence', async () => {
      core();
      await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Options for Example: editor-cache' }));

      expect(await screen.findByRole('menuitem', { name: 'Ignore this item' })).toHaveAccessibleDescription(
        'Hides it, and everything inside it, from the results and never removes it. You can restore it in Settings.',
      );
    });

    it('when the last row leaves, the focus goes to Back to Scan', async () => {
      core();
      const { store } = await renderDetails('demo.caches');
      const options = await screen.findByRole('button', { name: 'Options for Example: editor-cache' });
      options.focus();

      act(() => {
        store.dispatch({ overview: WITHOUT_CACHES, type: 'overview-loaded' });
      });

      expect(screen.getByRole('button', { name: 'Back to Scan' })).toHaveFocus();
    });
  });
});
