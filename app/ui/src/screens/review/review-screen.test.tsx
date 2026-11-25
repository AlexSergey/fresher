import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Finding, FindingsQuery, Overview } from '../../ipc/bindings.generated';
import type { CommandKey } from '../../ipc/index';

import { type CoreAnswers, mockCore, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { createNavigation, type Navigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { type CoreStore, createCoreStore, useCore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  CLEANUP_DICTIONARY,
  cleanupEventsOf,
  DEMO_PLAN,
  MOVED_REPORT,
  PLAN_GONE,
  PLAN_MISMATCH,
} from '../../test/cleanup-plan';
import { DEMO_FINDINGS, DEMO_MODULES, DEMO_OVERVIEW } from '../../test/demo-scan';
import { FINDING_DICTIONARY, HIGH_FINDING, MEDIUM_FINDING } from '../../test/finding-fixtures';
import { LARGEST_DICTIONARY, LARGEST_ROWS, ownFindingOf } from '../../test/largest-scan';
import { pageOf } from '../../test/page-of';
import { BUILD_DICTIONARY, SANDBOX_HOME, sandboxBuild } from '../../test/sandbox-build';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { ReviewScreen } from './review-screen';

const [, INSTALLER, BROWSER_CACHE] = DEMO_FINDINGS as readonly [Finding, Finding, Finding];

const FINDINGS: readonly Finding[] = [...DEMO_FINDINGS, HIGH_FINDING];

// A Medium finding of a category of its own, picked by itself.
const IMAGE: Finding = { ...MEDIUM_FINDING, category: 'demo.images' };

const OVERVIEW: Overview = {
  ...DEMO_OVERVIEW,
  totals: [
    ...DEMO_OVERVIEW.totals,
    { category: 'demo.projects', count: 1, highestDanger: 'high', sizes: HIGH_FINDING.sizes },
  ],
};

// Only these commands read; anything else the screen sent would be a change.
const READS: readonly CommandKey[] = ['getDictionary', 'getFindings', 'getModules', 'getOverview', 'subscribe'];

type Rendered = {
  // Every command but the reads, with its arguments.
  readonly changes: readonly (readonly [CommandKey, unknown])[];
  readonly commands: readonly CommandKey[];
  readonly navigation: Navigation;
  readonly store: CoreStore;
};

// The selection bar beside the screen, fed by the same store: what the overview shows for the same selection.
const Bar = (): React.JSX.Element => {
  const selection = useCore((state) => state.selection);
  const overview = useCore((state) => state.overview);

  return <SelectionBar onClear={vi.fn()} onReview={vi.fn()} summary={summarize(selection, overview?.totals ?? [])} />;
};

// The screen, once the core's dictionary and overview have arrived, with what `select` picks.
const renderReview = async (
  select: (store: CoreStore) => void,
  withBar = false,
  cleanup: Partial<CoreAnswers> = {},
): Promise<Rendered> => {
  const commands: CommandKey[] = [];
  const changes: (readonly [CommandKey, unknown])[] = [];
  const recorded =
    (key: CommandKey, answer: (args: unknown) => unknown) =>
    (args: unknown): unknown => {
      commands.push(key);
      if (!READS.includes(key)) {
        changes.push([key, args]);
      }

      return answer(args);
    };
  mockCore({
    ...sampleAgreementAnswers,
    ...sampleStorageAnswers,
    ...Object.fromEntries(Object.entries(cleanup).map(([key, answer]) => [key, recorded(key as CommandKey, answer)])),
    getDictionary: recorded('getDictionary', () => ({
      ...FINDING_DICTIONARY,
      ...LARGEST_DICTIONARY,
      ...CLEANUP_DICTIONARY,
      ...BUILD_DICTIONARY,
      ...STORAGE_DICTIONARY,
    })),
    getFindings: recorded('getFindings', (args) => pageOf(FINDINGS, (args as { query: FindingsQuery }).query)),
    getModules: recorded('getModules', () => DEMO_MODULES),
    getOverview: recorded('getOverview', () => OVERVIEW),
    subscribe: recorded('subscribe', () => null),
  });
  const navigation = createNavigation();
  navigation.navigate({ kind: 'review' });
  const store = createCoreStore();

  render(
    <AppProviders navigation={navigation} store={store}>
      <ReviewScreen />
      {withBar ? <Bar /> : null}
    </AppProviders>,
  );
  await vi.waitFor(() => {
    expect(store.getState().overview).not.toBeNull();
  });
  act(() => {
    select(store);
  });

  return { changes, commands, navigation, store };
};

// The core answers the demo plan and names the cleanup 1.
const CLEANUP: Partial<CoreAnswers> = {
  confirmCleanup: () => 1,
  discardCleanup: () => null,
  prepareCleanup: () => DEMO_PLAN,
};

const removeButton = (): HTMLElement => screen.getByRole('button', { name: 'Move to the Trash' });

const openConfirmation = async (): Promise<HTMLElement> => {
  removeButton().focus();
  fireEvent.click(removeButton());

  return screen.findByRole('dialog');
};

const pickFinding =
  (...findings: readonly Finding[]) =>
  (store: CoreStore): void => {
    for (const finding of findings) {
      store.dispatch({ finding, type: 'toggle-finding' });
    }
  };

const pickCategory =
  (...categories: readonly string[]) =>
  (store: CoreStore): void => {
    for (const category of categories) {
      store.dispatch({ category, type: 'toggle-category' });
    }
  };

const tileNames = (): readonly string[] =>
  screen.queryAllByRole('article').map((tile) => within(tile).getByRole('heading', { level: 2 }).textContent);

const tile = (name: string): HTMLElement => screen.getByRole('article', { name });

const reviewBar = (): HTMLElement => screen.getByRole('region', { name: 'Ready to remove' });

const summaryRegion = (): HTMLElement => screen.getByRole('region', { name: 'What you are about to remove' });

const untick = async (title: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('checkbox', { name: `Select Example: ${title}` }));
};

beforeEach(async () => {
  fakeViewport(520);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('ReviewScreen', () => {
  describe('negative cases', () => {
    it('with nothing going and nothing to check it shows the empty state', async () => {
      const { navigation } = await renderReview(() => undefined);

      expect(screen.getByRole('heading', { level: 1, name: 'Review' })).toBeInTheDocument();
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Move to the Trash' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Clear' })).toBeNull();

      fireEvent.click(screen.getByRole('button', { name: 'Go to Scan' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'scan' });
    });

    it('has no Select all: a tile has its own all-checkbox', async () => {
      await renderReview(pickCategory('demo.downloads'));

      expect(screen.queryByRole('checkbox', { name: 'Select all' })).toBeNull();
      expect(within(tile('Example downloads')).getByRole('checkbox', { name: 'All Example downloads' })).toBeChecked();
    });

    it('Move to the Trash stays inactive while only categories to check are selected', async () => {
      const { changes } = await renderReview(pickCategory('demo.projects'), false, CLEANUP);

      expect(tile('Example projects')).toBeInTheDocument();
      expect(removeButton()).toBeDisabled();
      expect(removeButton()).toHaveAccessibleDescription('Tick at least one item first.');
      expect(within(reviewBar()).getByText('Tick at least one item first.')).toBeVisible();
      expect(await screen.findByRole('checkbox', { name: 'Select Example: old-project' })).not.toBeChecked();

      fireEvent.click(removeButton());

      expect(changes).toStrictEqual([]);
    });

    it('findings picked one by one have no checkbox for all of their category', async () => {
      await renderReview(pickFinding(INSTALLER));

      expect(tile('Example downloads')).toBeInTheDocument();
      expect(screen.queryByRole('checkbox', { name: 'All Example downloads' })).toBeNull();
    });

    it('cancel discards the plan', async () => {
      const { changes } = await renderReview(pickCategory('demo.downloads'), false, CLEANUP);
      await openConfirmation();

      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
      await vi.waitFor(() => {
        expect(changes).toHaveLength(2);
      });

      expect(changes).toStrictEqual([
        ['prepareCleanup', { selection: { categories: [{ category: 'demo.downloads', except: [] }], findings: [] } }],
        ['discardCleanup', { plan: DEMO_PLAN.plan }],
      ]);
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(removeButton()).toHaveFocus();
    });

    it('no key confirms', async () => {
      const { changes, store } = await renderReview(pickCategory('demo.downloads'), false, CLEANUP);
      const dialog = await openConfirmation();

      fireEvent.keyDown(dialog, { key: 'Enter' });
      fireEvent.keyDown(screen.getByRole('button', { name: 'Cancel' }), { key: 'Enter' });

      expect(changes.map(([key]) => key)).toStrictEqual(['prepareCleanup']);
      expect(store.getState().cleanup.kind).toBe('confirming');
    });

    it('the permanent choice is off at every opening', async () => {
      await renderReview(pickCategory('demo.downloads'), false, CLEANUP);
      await openConfirmation();
      fireEvent.click(screen.getByRole('checkbox', { name: /^Delete permanently instead/ }));
      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      await openConfirmation();

      expect(screen.getByRole('checkbox', { name: /^Delete permanently instead/ })).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeInTheDocument();
    });

    it.each([
      ['an expired plan', PLAN_GONE],
      ['a refused echo', PLAN_MISMATCH],
    ])('%s shows the core’s text and the way to review it again', async (_, refusal) => {
      const { store } = await renderReview(pickCategory('demo.downloads'), false, {
        ...CLEANUP,
        confirmCleanup: refuse(refusal),
      });
      await openConfirmation();

      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));

      const notice = await screen.findByRole('alert');
      expect(notice).toHaveTextContent(CLEANUP_DICTIONARY[refusal.text.key] ?? '');
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(store.getState().selection.categories.has('demo.downloads')).toBe(true);

      fireEvent.click(within(notice).getByRole('button', { name: 'Review it again' }));

      expect(screen.queryByRole('alert')).toBeNull();
      expect(removeButton()).toBeEnabled();
    });

    it('unticking the last item of a picked group removes its tile', async () => {
      await renderReview((store) => {
        pickFinding(HIGH_FINDING)(store);
        pickCategory('demo.downloads')(store);
      });
      expect(tileNames()).toStrictEqual(['Example downloads', 'Example projects']);

      await untick('old-project');

      expect(tileNames()).toStrictEqual(['Example downloads']);
    });

    it('unticking the last item of a selected category removes its tile', async () => {
      await renderReview(pickCategory('demo.downloads'));

      await untick('installer.dmg');

      expect(tileNames()).toStrictEqual([]);
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
    });

    it('a Safe and Low selection has no chip that counts what is not checked', async () => {
      await renderReview(pickCategory('demo.caches', 'demo.downloads'));

      await untick('browser-cache');

      expect(within(summaryRegion()).queryByText(/not checked/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('a confirm refused after an uninstall was confirmed shows the core’s text', async () => {
      await renderReview((store) => {
        store.dispatch({ app: { index: 0, state: 1 }, origin: 'uninstall', type: 'cleanup-preparing' });
        store.dispatch({ plan: DEMO_PLAN, type: 'cleanup-prepared' });
        store.dispatch({ method: 'trash', type: 'cleanup-confirming' });
        store.dispatch({ refusal: PLAN_GONE, type: 'cleanup-refused' });
      });

      expect(screen.getByText(CLEANUP_DICTIONARY['core.error.plan-gone'] as string)).toBeVisible();
      fireEvent.click(screen.getByRole('button', { name: 'Review it again' }));
      expect(screen.queryByText(CLEANUP_DICTIONARY['core.error.plan-gone'] as string)).not.toBeInTheDocument();
    });

    it("the summary's chips list each group with its size, Look with its not checked count", async () => {
      await renderReview((store) => {
        pickCategory('demo.caches', 'demo.projects')(store);
        pickFinding(IMAGE)(store);
      });
      const summary = summaryRegion();

      expect(within(summary).getByText('1.8 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
      expect(within(summary).getByText('goes to the Trash · 3 items')).toBeInTheDocument();
      expect(within(summary).getByText('You can put them back from Cleanups')).toBeInTheDocument();
      expect(within(summary).getByRole('img')).toHaveAccessibleName(
        'Size of each group: Safe to remove, 1.8 GB; Look before removing, 420 MB; Only if you are sure, 0 bytes',
      );
      expect(within(summary).getByText('· 1 not checked')).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- a chip has no role: its level is data on it
      expect(within(summary).getByText('· 1 not checked').parentElement).toHaveAttribute('data-level', 'high');
    });

    it('a picked own file is a tile of its own in the High group, with no all-checkbox', async () => {
      await renderReview(pickFinding(ownFindingOf(LARGEST_ROWS[0] as (typeof LARGEST_ROWS)[number], 101)));

      const own = tile('Your own files');

      expect(within(own).getByText('Only if you are sure · May hold the only copy of your data')).toBeInTheDocument();
      expect(within(own).getByText('1 picked')).toBeInTheDocument();
      expect(within(own).queryByText(/checked/u)).toBeNull();
      expect(within(own).queryByRole('checkbox', { name: /^All / })).toBeNull();
    });

    it('tiles follow the groups, then size order, each with its line and its pill', async () => {
      await renderReview((store) => {
        pickCategory('demo.downloads', 'demo.caches')(store);
        pickFinding(HIGH_FINDING)(store);
      });

      expect(tileNames()).toStrictEqual(['Example caches', 'Example downloads', 'Example projects']);
      const caches = tile('Example caches');
      expect(within(caches).getByText('Safe to remove · Apps make these again on their own')).toBeInTheDocument();
      expect(within(caches).getByText('2 of 2 checked')).toBeInTheDocument();
      expect(
        within(tile('Example projects')).getByText('Only if you are sure · May hold the only copy of your data'),
      ).toBeInTheDocument();
      expect(await within(caches).findByRole('list', { name: 'The items of Example caches' })).toBeInTheDocument();
      expect(await within(caches).findAllByRole('listitem')).toHaveLength(2);
    });

    it("the summary matches the selection bar's", async () => {
      await renderReview((store) => {
        pickCategory('demo.caches')(store);
        pickFinding(HIGH_FINDING, INSTALLER)(store);
      }, true);
      const selectionBar = screen.getByRole('region', { name: 'Selection' });

      expect(within(selectionBar).getByText('4 items selected')).toBeInTheDocument();
      expect(within(reviewBar()).getByText('4 items')).toBeInTheDocument();
      expect(within(summaryRegion()).getByText('goes to the Trash · 4 items')).toBeInTheDocument();
      expect(within(summaryRegion()).getAllByText('4.9 GB', { selector: '[aria-hidden]' })).not.toHaveLength(0);
      expect(within(reviewBar()).getByText('4.9 GB', { selector: '[aria-hidden]' })).toBeInTheDocument();
      expect(within(selectionBar).getByText('4.9 GB', { selector: 'span:not([aria-hidden])' })).toBeInTheDocument();
    });

    it('unticking an item takes it out of the summary and keeps the row, unticked', async () => {
      const { store } = await renderReview(pickCategory('demo.caches'));

      await untick('browser-cache');

      expect(within(summaryRegion()).getByText('goes to the Trash · 1 item')).toBeInTheDocument();
      expect(within(reviewBar()).getByText('1.5 GB', { selector: '[aria-hidden]' })).toBeInTheDocument();
      expect(within(tile('Example caches')).getByText('1 of 2 checked')).toBeInTheDocument();
      expect(screen.getByRole('checkbox', { name: 'Select Example: browser-cache' })).not.toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'All Example caches' })).toBePartiallyChecked();
      expect(store.getState().selection.categories.get('demo.caches')?.has(BROWSER_CACHE.id)).toBe(true);
    });

    it('an item ticked in Review goes', async () => {
      const { changes } = await renderReview(pickCategory('demo.projects'), false, CLEANUP);

      fireEvent.click(await screen.findByRole('checkbox', { name: 'Select Example: old-project' }));
      await openConfirmation();

      expect(changes).toStrictEqual([
        ['prepareCleanup', { selection: { categories: [], findings: [HIGH_FINDING.id] } }],
      ]);
    });

    it('ticking all of a Look category in Review sends the category', async () => {
      const { changes } = await renderReview(pickCategory('demo.projects'), false, CLEANUP);

      fireEvent.click(await screen.findByRole('checkbox', { name: 'All Example projects' }));
      await openConfirmation();

      expect(changes).toStrictEqual([
        ['prepareCleanup', { selection: { categories: [{ category: 'demo.projects', except: [] }], findings: [] } }],
      ]);
    });

    it('the bar removes through the confirmation: the core checks each item, then opens its plan', async () => {
      let answer: (plan: typeof DEMO_PLAN) => void = () => undefined;
      await renderReview(pickCategory('demo.caches'), false, {
        prepareCleanup: () =>
          new Promise((resolve) => {
            answer = resolve;
          }),
      });

      fireEvent.click(within(reviewBar()).getByRole('button', { name: 'Move to the Trash' }));

      expect(within(reviewBar()).getByRole('status')).toHaveTextContent('Checking each item…');
      expect(screen.queryByRole('dialog')).toBeNull();

      act(() => {
        answer(DEMO_PLAN);
      });
      const dialog = await screen.findByRole('dialog');

      // The plan's figures, not the selection's: the plan holds three items, the selection two.
      expect(dialog).toHaveAccessibleName('Move 3 items to the Trash?');
      expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus();
    });

    it('confirm sends the echo of the shown plan', async () => {
      const { changes, store } = await renderReview(pickCategory('demo.caches'), false, CLEANUP);
      await openConfirmation();

      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));
      await vi.waitFor(() => {
        expect(store.getState().cleanup).toMatchObject({ cleanup: 1, kind: 'running' });
      });

      expect(changes[1]).toStrictEqual([
        'confirmCleanup',
        {
          confirmation: {
            allocated: 2_570_000_000,
            count: 3,
            highest: 'low',
            highRiskUnderstood: false,
            method: 'trash',
            plan: 1,
          },
        },
      ]);
    });

    it('the permanent choice turns the button red and sends permanent', async () => {
      const { changes } = await renderReview(pickCategory('demo.caches'), false, CLEANUP);
      await openConfirmation();

      fireEvent.click(screen.getByRole('checkbox', { name: /^Delete permanently instead/ }));
      const confirmButton = screen.getByRole('button', { name: 'Delete permanently' });
      expect(confirmButton).toHaveAttribute('data-variant', 'danger');
      fireEvent.click(confirmButton);
      await vi.waitFor(() => {
        expect(changes).toHaveLength(2);
      });

      expect(changes[1]).toMatchObject(['confirmCleanup', { confirmation: { method: 'permanent' } }]);
    });

    it('the progress follows the cleanup, then the result; Done returns to the overview', async () => {
      const cancelCleanup = vi.fn(() => null);
      const revealInTrash = vi.fn(() => null);
      const { navigation, store } = await renderReview(pickCategory('demo.caches'), false, {
        ...CLEANUP,
        cancelCleanup,
        revealInTrash,
      });
      await openConfirmation();
      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));
      const events = cleanupEventsOf(DEMO_PLAN, MOVED_REPORT);

      expect(await screen.findByText('Moving 1 of 3 to the Trash…')).toBeInTheDocument();
      act(() => {
        store.dispatch(events.started);
        store.dispatch(events.items[0] as (typeof events.items)[number]);
      });
      expect(screen.getByText('Moving 2 of 3 to the Trash…')).toBeInTheDocument();
      expect(screen.getByText('Now: Example: browser-cache')).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
      expect(screen.getByText('Stopping after this item…')).toBeInTheDocument();
      act(() => {
        store.dispatch(events.finished);
      });

      expect(screen.getByRole('region', { name: 'Moved to the Trash' })).toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', { name: 'Show in the Trash' }));
      fireEvent.click(screen.getByRole('button', { name: 'Done' }));
      await vi.waitFor(() => {
        expect(revealInTrash).toHaveBeenCalledExactlyOnceWith({ item: { cleanup: 1, item: 0 } });
      });

      expect(cancelCleanup).toHaveBeenCalledOnce();
      expect(navigation.getScreen()).toStrictEqual({ kind: 'scan' });
      expect(store.getState().cleanup).toStrictEqual({ kind: 'idle' });
      expect(store.getState().selection.categories.size).toBe(0);
    });

    it("a sandbox build's finished Trash cleanup says where the sandbox's Trash is", async () => {
      const { store } = await renderReview(pickCategory('demo.caches'), false, {
        ...CLEANUP,
        appInfo: () => ({ build: sandboxBuild(), high: 'offered', home: SANDBOX_HOME, version: '0.1.0' }),
      });
      await openConfirmation();
      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));
      await screen.findByText('Moving 1 of 3 to the Trash…');
      const events = cleanupEventsOf(DEMO_PLAN, MOVED_REPORT);
      act(() => {
        store.dispatch(events.started);
        store.dispatch(events.finished);
      });

      const result = screen.getByRole('region', { name: 'Moved to the Trash' });
      expect(
        await within(result).findByText(
          "Cleanups move items to the sandbox's Trash: /Users/dev/sandboxes/trial/home/.Trash",
        ),
      ).toBeInTheDocument();
    });

    it("the bar's Clear empties the selection", async () => {
      const { store } = await renderReview(pickCategory('demo.caches'));

      fireEvent.click(within(reviewBar()).getByRole('button', { name: 'Clear' }));

      expect(store.getState().selection.categories.size).toBe(0);
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
    });
  });
});
