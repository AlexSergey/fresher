import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Finding, FindingsQuery, Overview } from '../../ipc/bindings.generated';
import type { CommandKey } from '../../ipc/index';

import { mockCore } from '../../ipc/mock';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { createNavigation, type Navigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { type CoreStore, createCoreStore, useCore } from '../../state/store';
import { setMotion } from '../../test/animation';
import { DEMO_FINDINGS, DEMO_MODULES, DEMO_OVERVIEW } from '../../test/demo-scan';
import { FINDING_DICTIONARY, HIGH_FINDING } from '../../test/finding-fixtures';
import { pageOf } from '../../test/page-of';
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
  readonly commands: readonly CommandKey[];
  readonly navigation: Navigation;
  readonly store: CoreStore;
};

// The selection bar beside the screen, fed by the same store: what the module page shows for the same selection.
const Bar = (): React.JSX.Element => {
  const selection = useCore((state) => state.selection);
  const overview = useCore((state) => state.overview);

  return <SelectionBar onClear={vi.fn()} onReview={vi.fn()} summary={summarize(selection, overview?.totals ?? [])} />;
};

// The screen, once the core's dictionary and overview have arrived, with what `select` picks.
const renderReview = async (select: (store: CoreStore) => void, withBar = false): Promise<Rendered> => {
  const commands: CommandKey[] = [];
  const recorded =
    (key: CommandKey, answer: (args: unknown) => unknown) =>
    (args: unknown): unknown => {
      commands.push(key);

      return answer(args);
    };
  mockCore({
    getDictionary: recorded('getDictionary', () => FINDING_DICTIONARY),
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

  return { commands, navigation, store };
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
  screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent);

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
    it('an empty selection shows the empty state, with a way back to the overview', async () => {
      const { navigation } = await renderReview(() => undefined);

      expect(screen.getByRole('heading', { level: 1, name: 'Review' })).toBeInTheDocument();
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Move to Trash…' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Clear selection' })).toBeNull();

      fireEvent.click(screen.getByRole('button', { name: 'Go to the overview' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'overview' });
    });

    it('the remove button is disabled and says why', async () => {
      await renderReview(pickCategory('demo.downloads'));

      expect(screen.getByRole('button', { name: 'Move to Trash…' })).toBeDisabled();
      expect(
        screen.getByText('Removal comes in a later version of the app. Nothing is removed from this screen yet.'),
      ).toBeInTheDocument();
    });

    it('confirm is never sent: the remove button opens no dialog, and Escape or Enter sends nothing', async () => {
      const { commands, store } = await renderReview(pickCategory('demo.downloads'));
      await screen.findByRole('listitem', { name: 'Example: installer.dmg, Low, 750 MB' });

      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash…' }));
      const heading = screen.getByRole('heading', { level: 1 });
      fireEvent.keyDown(heading, { key: 'Escape' });
      fireEvent.keyDown(heading, { key: 'Enter' });

      expect(screen.queryByRole('dialog')).toBeNull();
      expect(screen.queryByRole('button', { name: 'Cancel' })).toBeNull();
      expect(commands.filter((command) => !READS.includes(command))).toStrictEqual([]);
      expect(store.getState().selection.categories.has('demo.downloads')).toBe(true);
    });

    it('unticking the last item of a picked group removes the group', async () => {
      await renderReview((store) => {
        pickFinding(HIGH_FINDING)(store);
        pickCategory('demo.downloads')(store);
      });
      expect(groupNames()).toStrictEqual(['Example projects', 'Example downloads']);

      await untick('old-project');

      expect(groupNames()).toStrictEqual(['Example downloads']);
    });

    it('unticking the last item of a selected category removes the group', async () => {
      await renderReview(pickCategory('demo.downloads'));

      await untick('installer.dmg');

      expect(screen.queryAllByRole('heading', { level: 2 })).toHaveLength(0);
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
    it('groups follow size order and show what is lost', async () => {
      await renderReview((store) => {
        pickCategory('demo.downloads', 'demo.caches')(store);
        pickFinding(HIGH_FINDING)(store);
      });

      expect(groupNames()).toStrictEqual(['Example projects', 'Example caches', 'Example downloads']);
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

    it('Clear selection empties the selection', async () => {
      const { store } = await renderReview(pickCategory('demo.caches'));

      fireEvent.click(screen.getByRole('button', { name: 'Clear selection' }));

      expect(store.getState().selection.categories.size).toBe(0);
      expect(screen.getByText('Nothing is selected')).toBeInTheDocument();
    });
  });
});
