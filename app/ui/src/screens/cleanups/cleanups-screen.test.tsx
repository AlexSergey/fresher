import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CleanupDetail, CleanupId, CleanupsPage, CleanupsQuery } from '../../ipc/bindings.generated';

import { type CoreAnswers, mockCore, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../../ipc/mock';
import { createNavigation, type Navigation } from '../../state/navigation';
import { createCoreStore } from '../../state/store';
import { setMotion } from '../../test/animation';
import {
  CLEANUP_DICTIONARY,
  cleanupSummaryOf,
  HISTORY,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
  MOVED_REPORT,
  NOT_IN_TRASH,
} from '../../test/cleanup-plan';
import { DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { fakeViewport } from '../../test/viewport';
import { AppProviders } from '../app-providers';
import { CleanupScreen } from './cleanup-screen';
import { CleanupsScreen } from './cleanups-screen';

const DICTIONARY = { ...DEMO_DICTIONARY, ...FINDING_DICTIONARY, ...CLEANUP_DICTIONARY };

const EMPTY: CleanupsPage = { blocked: null, cleanups: [], offset: 0, total: 0, waiting: 0 };

type Rendered = { readonly navigation: Navigation; readonly seen: readonly (readonly [string, unknown])[] };

// A screen of the history on the mocked core; every cleanup command is recorded with its arguments.
const renderScreen = (content: React.JSX.Element, answers: Partial<CoreAnswers>): Rendered => {
  const seen: (readonly [string, unknown])[] = [];
  const recorded = Object.fromEntries(
    Object.entries(answers).map(([key, answer]) => [
      key,
      (args: unknown): unknown => {
        seen.push([key, args]);

        return answer(args);
      },
    ]),
  );
  mockCore({
    ...sampleAgreementAnswers,
    getDictionary: () => DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    ...sampleStorageAnswers,
    ...recorded,
  });
  const navigation = createNavigation();
  render(
    <AppProviders navigation={navigation} store={createCoreStore()}>
      {content}
    </AppProviders>,
  );

  return { navigation, seen };
};

// 120 cleanups, newest first, answered a page at a time.
const pagesOf =
  (total: number) =>
  (args: unknown): CleanupsPage => {
    const { limit, offset } = (args as { readonly query: CleanupsQuery }).query;
    const count = Math.max(0, Math.min(limit, total - offset));

    return {
      ...HISTORY,
      cleanups: Array.from({ length: count }, (_, at) =>
        cleanupSummaryOf(total - offset - at, MOVED_REPORT, { started: 1_735_819_200 - (offset + at) * 86_400 }),
      ),
      offset,
      total,
    };
  };

const detailOf = (detail: CleanupDetail) => (): CleanupDetail => detail;

const itemRow = (name: string): HTMLElement =>
  screen.getAllByRole('listitem').find((row) => row.textContent.includes(name)) ?? screen.getByText(name);

beforeEach(async () => {
  fakeViewport(720);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('CleanupsScreen', () => {
  describe('negative cases', () => {
    it('an empty history says nothing cleaned yet', async () => {
      renderScreen(<CleanupsScreen />, { getCleanups: () => EMPTY });

      expect(await screen.findByText('Nothing cleaned yet')).toBeInTheDocument();
      expect(screen.queryByText(/^Waiting in the Trash/u)).toBeNull();
    });

    it('a read only journal shows its reason', async () => {
      renderScreen(<CleanupsScreen />, { getCleanups: () => ({ ...HISTORY, blocked: 'journal' }) });

      expect(await screen.findByText('New cleanups cannot be recorded now')).toBeInTheDocument();
      expect(screen.getByText(CLEANUP_DICTIONARY['core.cleanup.blocked.journal'] ?? '')).toBeInTheDocument();
      // The history is still shown.
      expect(screen.getAllByRole('button', { name: /^Cleanup of / })).toHaveLength(3);
    });
  });

  describe('positive cases', () => {
    it('shows the help button', async () => {
      renderScreen(<CleanupsScreen />, { getCleanups: () => HISTORY });

      expect(await screen.findByRole('button', { name: 'Removed something you need?' })).toBeInTheDocument();
    });

    it('the list pages newest first', async () => {
      const { seen } = renderScreen(<CleanupsScreen />, { getCleanups: pagesOf(120) });
      const list = await screen.findByRole('list', { name: 'Cleanups, newest first' });

      expect(within(list).getAllByRole('button')).toHaveLength(50);
      fireEvent.click(screen.getByRole('button', { name: 'Show older cleanups' }));
      await vi.waitFor(() => {
        expect(within(list).getAllByRole('button')).toHaveLength(100);
      });
      fireEvent.click(screen.getByRole('button', { name: 'Show older cleanups' }));
      await vi.waitFor(() => {
        expect(within(list).getAllByRole('button')).toHaveLength(120);
      });

      expect(seen.map(([, args]) => args)).toStrictEqual([
        { query: { limit: 50, offset: 0 } },
        { query: { limit: 50, offset: 50 } },
        { query: { limit: 50, offset: 100 } },
      ]);
      expect(screen.queryByRole('button', { name: 'Show older cleanups' })).toBeNull();
      expect(within(list).getAllByRole('button')[0]).toHaveTextContent('Cleanup of Jan 2, 2025');
    });

    it('shows what waits in the Trash and opens a cleanup', async () => {
      const { navigation } = renderScreen(<CleanupsScreen />, { getCleanups: () => HISTORY });

      expect(await screen.findByText('Waiting in the Trash from cleanups: 4.1 GB')).toBeInTheDocument();
      expect(screen.getByText('This is what Fresher moved. The Trash may hold more of your own.')).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: /^Cleanup of Jan 2, 2025/u }));

      expect(navigation.getScreen()).toStrictEqual({ cleanup: 3, kind: 'cleanup' });
    });
  });
});

describe('CleanupScreen', () => {
  const open = (cleanup: CleanupId, answers: Partial<CoreAnswers>): Rendered =>
    renderScreen(<CleanupScreen cleanup={cleanup} />, answers);

  describe('negative cases', () => {
    it('an occupied place is explained and the item stays', async () => {
      open(3, {
        getCleanup: detailOf(MOVED_DETAIL),
        putBack: () => ({
          items: [
            { index: 0, outcome: { kind: 'occupied' }, text: { args: {}, key: 'core.cleanup.put-back.occupied' } },
          ],
        }),
      });
      await screen.findByText('editor-cache');

      fireEvent.click(within(itemRow('editor-cache')).getByRole('button', { name: 'Put back' }));

      expect(
        await within(itemRow('editor-cache')).findByText(/Something is already at its old place/u),
      ).toBeInTheDocument();
      expect(screen.getAllByRole('listitem')).toHaveLength(3);
    });

    it('a refused put back shows the core’s text', async () => {
      open(3, { getCleanup: detailOf(MOVED_DETAIL), putBack: refuse(NOT_IN_TRASH) });
      await screen.findByText('editor-cache');

      fireEvent.click(within(itemRow('editor-cache')).getByRole('button', { name: 'Put back' }));

      expect(await screen.findByRole('alert')).toHaveTextContent('This item is no longer in the Trash.');
    });

    it('a cleanup Fresher quit during explains its unknown items', async () => {
      open(1, { getCleanup: detailOf(INTERRUPTED_DETAIL) });

      expect(
        await screen.findByText('Fresher stopped during this cleanup. Items marked Unknown may be in the Trash.'),
      ).toBeInTheDocument();
    });

    it('a cleanup with nothing in the Trash offers no put back', async () => {
      open(3, {
        getCleanup: detailOf({
          ...MOVED_DETAIL,
          items: MOVED_DETAIL.items.map((item) => ({ ...item, trash: 'gone' })),
        }),
      });
      await screen.findByText('editor-cache');

      expect(screen.queryByRole('button', { name: /Put back/u })).toBeNull();
      expect(screen.queryByRole('button', { name: 'Show in the Trash' })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('put back all sends the cleanup without items', async () => {
      const { seen } = open(3, { getCleanup: detailOf(MOVED_DETAIL), putBack: () => ({ items: [] }) });
      await screen.findByText('editor-cache');

      fireEvent.click(screen.getByRole('button', { name: 'Put back all still in the Trash' }));
      await vi.waitFor(() => {
        expect(seen.filter(([key]) => key === 'getCleanup')).toHaveLength(2);
      });

      expect(seen).toContainEqual(['putBack', { request: { cleanup: 3, items: null } }]);
    });

    it('a put back updates its row', async () => {
      const back = {
        ...MOVED_DETAIL,
        items: MOVED_DETAIL.items.map((item, index) =>
          index === 0 ? { ...item, putBack: { kind: 'put-back' as const }, trash: null } : item,
        ),
      };
      let detail = MOVED_DETAIL;
      const { seen } = open(3, {
        getCleanup: () => detail,
        putBack: () => {
          detail = back;

          return {
            items: [
              { index: 0, outcome: { kind: 'put-back' }, text: { args: {}, key: 'core.cleanup.put-back.put-back' } },
            ],
          };
        },
      });
      await screen.findByText('editor-cache');

      fireEvent.click(within(itemRow('editor-cache')).getByRole('button', { name: 'Put back' }));

      expect(await within(itemRow('editor-cache')).findByText(/Put back where it was\./u)).toBeInTheDocument();
      expect(within(itemRow('editor-cache')).queryByRole('button', { name: 'Put back' })).toBeNull();
      expect(seen).toContainEqual(['putBack', { request: { cleanup: 3, items: [0] } }]);
    });

    it('show in the trash asks the core', async () => {
      const { seen } = open(3, { getCleanup: detailOf(MOVED_DETAIL), revealInTrash: () => null });
      await screen.findByText('editor-cache');

      fireEvent.click(screen.getByRole('button', { name: 'Show in the Trash' }));
      await vi.waitFor(() => {
        expect(seen).toContainEqual(['revealInTrash', { item: { cleanup: 3, item: 0 } }]);
      });
    });

    it('a long cleanup lists its items in a virtual list', async () => {
      const items = Array.from({ length: 200 }, (_, index) => ({
        ...(MOVED_DETAIL.items[0] as CleanupDetail['items'][number]),
        index,
        path: `/demo/cache-${String(index)}`,
      }));
      open(3, { getCleanup: detailOf({ ...MOVED_DETAIL, items }) });

      const list = await screen.findByRole('list', { name: 'The items of this cleanup' });

      expect(within(list).getAllByRole('listitem').length).toBeLessThan(40);
      expect(within(list).getAllByRole('listitem')[0]).toHaveAttribute('aria-setsize', '200');
    });

    it('All cleanups goes back to the list', async () => {
      const { navigation } = open(3, { getCleanup: detailOf(MOVED_DETAIL) });
      expect(await screen.findByRole('heading', { level: 1, name: 'Cleanup of Jan 2, 2025' })).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'All cleanups' }));

      expect(navigation.getScreen()).toStrictEqual({ kind: 'cleanups' });
    });
  });
});
