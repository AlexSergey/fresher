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

import { mockCore, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
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
import { keptCount } from '../../test/left-out';
import { pageOf } from '../../test/page-of';
import { sandboxBuild } from '../../test/sandbox-build';
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
const DOWNLOADS_LEFT_OUT: LeftOutCount = keptCount('demo.downloads', 'demo.left-out.open', 2, {
  allocated: 4_000_000,
  completeness: 'complete',
  logical: 4_000_000,
  reclaimable: 4_000_000,
});

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
    ...sampleAgreementAnswers,
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

    it('the details have no Scan button', async () => {
      core();

      await renderDetails('demo.caches');

      expect(screen.queryByRole('button', { name: 'Scan' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Stop' })).toBeNull();
    });

    it('a Look before removing category has no Select all', async () => {
      core({ getOverview: () => APP_LEFTOVERS_OVERVIEW });

      await renderDetails('app-leftovers.settings', APP_LEFTOVERS);

      expect(await screen.findAllByRole('checkbox')).toHaveLength(2);
      expect(screen.queryByRole('checkbox', { name: /^Select all/u })).toBeNull();
      expect(screen.queryByText(/^Select all/u)).toBeNull();
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
      expect(screen.getByText('Everything found here was left out. The lines below say why.')).toBeVisible();
      expect(leftOutLines()).toHaveTextContent(
        '2 items kept (4 MB): The app that made it is open. Quit the app, then scan again.',
      );
    });

    it('shows no size and no count in the header when everything found was left out', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.downloads');
      const sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 } as const;
      const zero = { category: 'demo.downloads', count: 0, highestDanger: 'low', sizes } as const;

      scanned(store, [{ ...DEMO_LEFT_OUT, category: 'demo.downloads' }], {
        ...DEMO_OVERVIEW,
        totals: [DEMO_OVERVIEW.totals[0] as Overview['totals'][number], zero],
      });

      expect(screen.getByRole('heading', { level: 1, name: 'Example downloads' })).toBeVisible();
      expect(screen.queryByText('0 items')).toBeNull();
      expect(screen.queryByText(/^0 bytes$/u)).toBeNull();
      expect(screen.getByText('Everything found here was left out. The lines below say why.')).toBeVisible();
    });

    it('shows no item held back for now before its toggle is pressed', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT]);

      expect(screen.getByRole('button', { name: /^1 item left out \(157\.3 MB\)/u })).toHaveAttribute(
        'aria-expanded',
        'false',
      );
      expect(screen.queryByRole('list', { name: 'The items left out' })).toBeNull();
    });

    it('shows a count kept for good as a quiet line with no item and no bar', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.downloads');

      scanned(store, [DOWNLOADS_LEFT_OUT]);

      const [line] = within(leftOutLines() as HTMLElement).getAllByRole('listitem');
      expect(line).toHaveAttribute('data-kind', 'kept');
      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative: it has no role to query
      expect(line?.querySelector('svg')).toBeNull();
      expect(screen.queryByRole('list', { name: 'The items left out' })).toBeNull();
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
    it("a tool's object has no Show in Finder and no Ignore this item", async () => {
      core({ getFindings: withToolItem });
      await renderDetails('demo.caches');
      await screen.findByRole('listitem', { name: /^Example: editor-cache/u });

      fireEvent.click(screen.getByRole('button', { name: 'Options for Example: editor-cache' }));

      expect(screen.queryByRole('menuitem', { name: 'Show in Finder' })).toBeNull();
      expect(screen.queryByRole('menuitem', { name: 'Ignore this item' })).toBeNull();
      expect(screen.getAllByRole('menuitem')).toHaveLength(1);
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
      const elsewhere = document.body.appendChild(document.createElement('button'));
      elsewhere.focus();

      act(() => {
        store.dispatch({ overview: WITHOUT_CACHES, type: 'overview-loaded' });
      });

      expect(elsewhere).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Back to Scan' })).not.toHaveFocus();
      elsewhere.remove();
    });
  });

  describe('positive cases', () => {
    it('shows the name, the group, the size and the count', async () => {
      core();

      await renderDetails('demo.caches');

      expect(screen.getByRole('heading', { level: 1, name: 'Example caches' })).toBeVisible();
      expect(screen.getByText('Safe to remove')).toBeVisible();
      expect(screen.getByText('2 items')).toBeVisible();
      expect(screen.getAllByText('1.8 GB').length).toBeGreaterThan(0);
    });

    it('answers what it is, who makes it, what happens if removed and whether it comes back', async () => {
      core();

      await renderDetails('demo.caches');

      const what = screen.getByRole('region', { name: 'What it is' });
      expect(what).toHaveTextContent(text('demo.caches.what'));
      expect(what).toHaveTextContent('Apps that keep caches.');
      const removed = screen.getByRole('region', { name: 'If you remove it' });
      expect(removed).toHaveTextContent('The apps start a little slower once.');
      const back = screen.getByRole('region', { name: 'Comes back?' });
      expect(back).toHaveTextContent('Yes, as the apps run.');
      expect(screen.queryByRole('region', { name: 'Danger level' })).toBeNull();
    });

    it('shows each item with its name, a path from ~ and no level badge', async () => {
      core({ appInfo: () => ({ build: sandboxBuild(), high: 'offered', home: '/demo', version: '0.1.0' }) });

      await renderDetails('demo.caches');

      const row = await screen.findByRole('listitem', { name: /^Example: editor-cache/u });
      expect(row).toHaveAccessibleName(/Safe/u);
      expect(within(row).queryByText('Safe')).toBeNull();
      expect(within(row).getByText('~/editor-cache')).toBeInTheDocument();
    });

    it('lists the left-out lines of this category only', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT, DOWNLOADS_LEFT_OUT]);

      expect(leftOutLines()).toHaveTextContent('1 item left out (157.3 MB)');
      expect(leftOutLines()).not.toHaveTextContent('kept');
    });

    it('shows the left-out lines after the offered items', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT]);

      const findings = await screen.findByRole('region', { name: 'The items of Example caches' });
      expect(findings.compareDocumentPosition(leftOutLines() as HTMLElement)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    });

    it('shows each line held back for now as a bar led by an i icon', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT]);

      const [line] = within(leftOutLines() as HTMLElement).getAllByRole('listitem');
      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative: it has no role to query
      expect(line?.querySelector('svg')).not.toBeNull();
      expect(leftOutLines()).toHaveAttribute('data-bar');
    });

    it('lists each item held back for now by its name, its path and its size', async () => {
      core({ getOverview: () => IDLE });
      const { store } = await renderDetails('demo.caches');

      scanned(store, [DEMO_LEFT_OUT]);
      fireEvent.click(screen.getByRole('button', { name: /^1 item left out/u }));

      const item = within(screen.getByRole('list', { name: 'The items left out' })).getByRole('listitem');
      expect(item).toHaveTextContent('Example: com.example.notes');
      expect(item).toHaveTextContent('/demo/Library/Caches/com.example.notes');
      expect(item).toHaveTextContent('157.3 MB');
    });

    it('Select all selects a Safe category whole', async () => {
      core();
      const { store } = await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select all 2' }));

      expect([...store.getState().selection.categories.keys()]).toStrictEqual(['demo.caches']);
      expect(screen.getByRole('checkbox', { name: 'Select all 2' })).toBeChecked();
    });

    it('a Look before removing category still ticks its rows one by one', async () => {
      core({ getOverview: () => APP_LEFTOVERS_OVERVIEW });
      const { store } = await renderDetails('app-leftovers.settings', APP_LEFTOVERS);

      const [first] = await screen.findAllByRole('checkbox');
      fireEvent.click(first as HTMLElement);

      expect(store.getState().selection.categories.size).toBe(0);
      expect(store.getState().selection.findings.size).toBe(1);
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

    it("a row's menu lists its items in order with its separators", async () => {
      core();
      await renderDetails('demo.caches');

      fireEvent.click(await screen.findByRole('button', { name: 'Options for Example: editor-cache' }));
      // eslint-disable-next-line testing-library/no-node-access -- the order of items and separators is the point
      const entries = [...screen.getByRole('menu').querySelectorAll('[role="menuitem"], [role="separator"]')];

      expect(
        entries.map((entry) =>
          entry.getAttribute('role') === 'separator' ? '-' : entry.firstElementChild?.textContent,
        ),
      ).toStrictEqual(['Show in Finder', '-', 'Ignore this item', 'Ignore all Example caches']);
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
