import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Finding, FindingsQuery, Overview } from '../../ipc/bindings.generated';
import type { CommandKey } from '../../ipc/index';

import { type CoreAnswers, mockCore, refuse, sampleStorageAnswers } from '../../ipc/mock';
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
import { FINDING_DICTIONARY, HIGH_FINDING } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
import { BUILD_DICTIONARY, sandboxBuild } from '../../test/sandbox-build';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { ReviewScreen } from './review-screen';

const [EDITOR_CACHE, INSTALLER, BROWSER_CACHE] = DEMO_FINDINGS as readonly [Finding, Finding, Finding];

const FINDINGS: readonly Finding[] = [...DEMO_FINDINGS, HIGH_FINDING];

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
    ...sampleStorageAnswers,
    ...Object.fromEntries(Object.entries(cleanup).map(([key, answer]) => [key, recorded(key as CommandKey, answer)])),
    getDictionary: recorded('getDictionary', () => ({
      ...FINDING_DICTIONARY,
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

const removeButton = (): HTMLElement => screen.getByRole('button', { name: 'Move to Trash…' });

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

const groupNames = (): readonly string[] =>
  screen.getAllByRole('heading', { level: 3 }).map((heading) => heading.textContent);

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
    it('has no Select all: everything listed is already selected', async () => {
      await renderReview(pickCategory('demo.downloads'));

      expect(screen.queryByRole('checkbox', { name: 'Select all' })).toBeNull();
    });

    it('shows no warning icon on a Safe zone', async () => {
      await renderReview(pickCategory('demo.caches'));

      const title = screen.getByRole('heading', { level: 2, name: 'Safe to remove' });

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(title.parentElement?.querySelector('svg')).toBeNull();
    });

    it('an empty selection shows the empty state, with a way back to the overview', async () => {
      const { navigation } = await renderReview(() => undefined);

      expect(screen.getByRole('heading', { level: 1, name: 'Review' })).toBeInTheDocument();
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Move to Trash…' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Clear selection' })).toBeNull();

      fireEvent.click(screen.getByRole('button', { name: 'Go to Scan' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'scan' });
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
      expect(screen.getByRole('button', { name: 'Move to Trash' })).toBeInTheDocument();
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

      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));

      const notice = await screen.findByRole('alert');
      expect(notice).toHaveTextContent(CLEANUP_DICTIONARY[refusal.text.key] ?? '');
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(store.getState().selection.categories.has('demo.downloads')).toBe(true);

      fireEvent.click(within(notice).getByRole('button', { name: 'Review it again' }));

      expect(screen.queryByRole('alert')).toBeNull();
      expect(removeButton()).toBeEnabled();
    });

    it('unticking the last item of a picked group removes the group', async () => {
      await renderReview((store) => {
        pickFinding(HIGH_FINDING)(store);
        pickCategory('demo.downloads')(store);
      });
      expect(groupNames()).toStrictEqual(['Example downloads', 'Example projects']);

      await untick('old-project');

      expect(groupNames()).toStrictEqual(['Example downloads']);
    });

    it('unticking the last item of a selected category removes the group', async () => {
      await renderReview(pickCategory('demo.downloads'));

      await untick('installer.dmg');

      expect(screen.queryAllByRole('heading', { level: 3 })).toHaveLength(0);
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
    });

    it('the highest level never shows by colour alone', async () => {
      await renderReview(pickFinding(HIGH_FINDING, EDITOR_CACHE));
      const summary = summaryRegion();

      expect(within(summary).getByText('Highest danger level:')).toBeInTheDocument();
      expect(within(summary).getByText('High')).toBeInTheDocument();
      expect(within(summary).getByText(FINDING_DICTIONARY['core.danger.high.meaning'] ?? '')).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- the level's icon is decorative, it has no role
      expect(summary.querySelector('[data-level="high"] svg')).not.toBeNull();
    });
  });

  describe('positive cases', () => {
    it('a selection across Safe and High shows two zones, High with its warning and meaning', async () => {
      await renderReview((store) => {
        pickCategory('demo.caches')(store);
        pickFinding(HIGH_FINDING)(store);
      });

      const zones = screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent);
      const high = screen.getByRole('region', { name: 'High risk' });

      expect(zones).toStrictEqual(['Safe to remove', 'High risk']);
      expect(within(high).getByText(FINDING_DICTIONARY['core.danger.high.meaning'] ?? '')).toBeInTheDocument();
      const title = screen.getByRole('heading', { level: 2, name: 'High risk' });

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(title.parentElement?.querySelector('svg')).not.toBeNull();
      expect(within(high).getByRole('region', { name: 'Example projects' })).toBeInTheDocument();
    });

    it('groups follow the zones, then size order, and show what is lost', async () => {
      await renderReview((store) => {
        pickCategory('demo.downloads', 'demo.caches')(store);
        pickFinding(HIGH_FINDING)(store);
      });

      expect(groupNames()).toStrictEqual(['Example caches', 'Example downloads', 'Example projects']);
      for (const category of ['demo.projects', 'demo.caches', 'demo.downloads']) {
        expect(screen.getByText(FINDING_DICTIONARY[`${category}.if-removed`] ?? '')).toBeInTheDocument();
      }
      const caches = screen.getByRole('region', { name: 'Example caches' });
      expect(within(caches).getByText('2 items')).toBeInTheDocument();
      expect(await within(caches).findByRole('list', { name: 'The items of Example caches' })).toBeInTheDocument();
      expect(await within(caches).findAllByRole('listitem')).toHaveLength(2);
    });

    it("the summary matches the selection bar's", async () => {
      await renderReview((store) => {
        pickCategory('demo.caches')(store);
        pickFinding(HIGH_FINDING, INSTALLER)(store);
      }, true);
      const bar = screen.getByRole('region', { name: 'Selection' });
      const summary = summaryRegion();

      expect(within(bar).getByText('4 items selected')).toBeInTheDocument();
      expect(within(summary).getByText('4 items')).toBeInTheDocument();
      expect(within(summary).getByText('4.9 GB', { selector: '[aria-hidden]' })).toBeInTheDocument();
      expect(within(bar).getByText('4.9 GB', { selector: 'span:not([aria-hidden])' })).toBeInTheDocument();
      expect(within(bar).getByText('High')).toBeInTheDocument();
      expect(within(summary).getByText('High')).toBeInTheDocument();
    });

    it('unticking a row updates the summary and keeps the row, unticked', async () => {
      const { store } = await renderReview(pickCategory('demo.caches'));

      await untick('browser-cache');

      expect(within(summaryRegion()).getByText('1 item')).toBeInTheDocument();
      expect(within(summaryRegion()).getByText('1.5 GB', { selector: '[aria-hidden]' })).toBeInTheDocument();
      expect(screen.getByRole('checkbox', { name: 'Select Example: browser-cache' })).not.toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' })).toBeChecked();
      expect(store.getState().selection.categories.get('demo.caches')?.has(BROWSER_CACHE.id)).toBe(true);
    });

    it('Move to Trash checks each item, then opens the core’s plan', async () => {
      let answer: (plan: typeof DEMO_PLAN) => void = () => undefined;
      await renderReview(pickCategory('demo.caches'), false, {
        prepareCleanup: () =>
          new Promise((resolve) => {
            answer = resolve;
          }),
      });

      fireEvent.click(removeButton());

      expect(screen.getByText('Checking each item…')).toHaveAttribute('role', 'status');
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

      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));
      await vi.waitFor(() => {
        expect(store.getState().cleanup).toMatchObject({ cleanup: 1, kind: 'running' });
      });

      expect(changes[1]).toStrictEqual([
        'confirmCleanup',
        { confirmation: { allocated: 2_570_000_000, count: 3, highest: 'low', method: 'trash', plan: 1 } },
      ]);
    });

    it('the permanent choice turns the button red and sends permanent', async () => {
      const { changes } = await renderReview(pickCategory('demo.caches'), false, CLEANUP);
      await openConfirmation();

      fireEvent.click(screen.getByRole('checkbox', { name: /^Delete permanently instead/ }));
      const confirm = screen.getByRole('button', { name: 'Delete permanently' });
      expect(confirm).toHaveAttribute('data-variant', 'danger');
      fireEvent.click(confirm);
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
      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));
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
        appInfo: () => ({ build: sandboxBuild(), version: '0.1.0' }),
      });
      await openConfirmation();
      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));
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

    it('Clear selection empties the selection', async () => {
      const { store } = await renderReview(pickCategory('demo.caches'));

      fireEvent.click(screen.getByRole('button', { name: 'Clear selection' }));

      expect(store.getState().selection.categories.size).toBe(0);
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
    });
  });
});
