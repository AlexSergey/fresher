import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CleanupItemView } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import {
  CLEANUP_DICTIONARY,
  cleanupSummaryOf,
  DELETED_DETAIL,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
  STOPPED_REPORT,
} from '../../test/cleanup-plan';
import { DEMO_DICTIONARY } from '../../test/demo-scan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { CleanupItemRow, CleanupRow } from './cleanup-row';

const MOVED_ITEM = MOVED_DETAIL.items[0] as CleanupItemView;

const renderItem = (item: CleanupItemView, putBack: null | string = null, onPutBack = vi.fn()): void => {
  render(
    <div role="list">
      <CleanupItemRow item={item} onPutBack={onPutBack} putBack={putBack} />
    </div>,
  );
};

const openRowOf = (summary = MOVED_DETAIL.summary, onOpen = vi.fn()): HTMLElement => {
  render(
    <ul>
      <CleanupRow onOpen={onOpen} summary={summary} />
    </ul>,
  );

  return screen.getByRole('button');
};

beforeEach(() => {
  setCoreDictionary({ ...DEMO_DICTIONARY, ...FINDING_DICTIONARY, ...CLEANUP_DICTIONARY });
});

describe('CleanupItemRow', () => {
  describe('negative cases', () => {
    it('put back is offered only for items in the trash', () => {
      renderItem({ ...MOVED_ITEM, trash: 'gone' });

      expect(screen.queryByRole('button', { name: 'Put back' })).toBeNull();
      expect(screen.getByRole('listitem')).toHaveTextContent('Example caches · No longer in the Trash.');
    });

    it('a permanent cleanup offers no put back', () => {
      renderItem(DELETED_DETAIL.items[0] as CleanupItemView);

      expect(screen.queryByRole('button', { name: 'Put back' })).toBeNull();
      expect(screen.getByRole('listitem')).toHaveTextContent('Deleted permanently.');
    });

    it('an item a crash left unknown says so', () => {
      renderItem(INTERRUPTED_DETAIL.items[1] as CleanupItemView);

      expect(screen.getByRole('listitem')).toHaveTextContent('Unknown: Fresher stopped during this cleanup.');
    });

    it('an occupied place is explained and the item stays', () => {
      renderItem(MOVED_ITEM, CLEANUP_DICTIONARY['core.cleanup.put-back.occupied'] ?? '');

      expect(screen.getByRole('listitem')).toHaveTextContent('Something is already at its old place');
      expect(screen.queryByRole('button', { name: 'Put back' })).toBeNull();
    });

    it('a skipped item shows why, without the instruction it did not keep', () => {
      renderItem({ ...MOVED_ITEM, outcome: { kind: 'skipped', reason: 'in-use' }, trash: null });

      expect(screen.getByRole('listitem')).toHaveTextContent('An app is using it, so Fresher left it alone.');
      expect(screen.getByRole('listitem')).not.toHaveTextContent('{app}');
    });
  });

  describe('positive cases', () => {
    it('an item in the trash offers put back', () => {
      const onPutBack = vi.fn();
      renderItem(MOVED_ITEM, null, onPutBack);

      expect(screen.getByRole('listitem')).toHaveTextContent('editor-cache');
      fireEvent.click(screen.getByRole('button', { name: 'Put back' }));

      expect(onPutBack).toHaveBeenCalledOnce();
    });

    it('an item put back says where it is', () => {
      renderItem({ ...MOVED_ITEM, putBack: { kind: 'put-back' }, trash: null });

      expect(screen.getByRole('listitem')).toHaveTextContent('Put back where it was.');
    });

    it('a tool item is named by its category', () => {
      renderItem({ ...MOVED_ITEM, action: 'tool', category: 'demo.images', path: null, trash: null });

      expect(screen.getByRole('listitem')).toHaveTextContent('Example images');
    });
  });
});

describe('CleanupRow', () => {
  describe('negative cases', () => {
    it('a deleted, a stopped and an interrupted cleanup say so', () => {
      const { unmount } = render(
        <ul>
          <CleanupRow onOpen={vi.fn()} summary={DELETED_DETAIL.summary} />
          <CleanupRow onOpen={vi.fn()} summary={cleanupSummaryOf(4, STOPPED_REPORT, {})} />
          <CleanupRow onOpen={vi.fn()} summary={INTERRUPTED_DETAIL.summary} />
          <CleanupRow onOpen={vi.fn()} summary={{ ...MOVED_DETAIL.summary, inTrash: 0 }} />
        </ul>,
      );

      expect(screen.getAllByRole('button').map((button) => button.textContent)).toStrictEqual([
        expect.stringContaining('Deleted permanently · 3 itemsDeleted'),
        expect.stringContaining('Stopped'),
        expect.stringContaining('Interrupted'),
        expect.stringContaining('No longer in the Trash'),
      ]);
      unmount();
    });

    it('a cleanup nothing was put back from does not say put back', () => {
      const row = openRowOf({ ...MOVED_DETAIL.summary, putBack: 0 });

      expect(row).not.toHaveTextContent('Put back');
    });
  });

  describe('positive cases', () => {
    it('shows its date, method, count, state and size, and opens', () => {
      const onOpen = vi.fn();
      const row = openRowOf(MOVED_DETAIL.summary, onOpen);

      expect(row).toHaveTextContent('Cleanup of Jan 2, 2025');
      expect(row).toHaveTextContent('Moved to the Trash · 3 items');
      expect(row).toHaveTextContent('In the Trash: 3 of 3');
      expect(row).toHaveTextContent('2.6 GB');

      fireEvent.click(row);

      expect(onOpen).toHaveBeenCalledOnce();
    });

    it('a cleanup put back in full says so', () => {
      const row = openRowOf({ ...MOVED_DETAIL.summary, inTrash: 0, putBack: 3 });

      expect(row).toHaveTextContent('Put back: 3 of 3');
      expect(row).not.toHaveTextContent('In the Trash:');
    });

    it('a cleanup put back in part says what is still in the Trash', () => {
      const row = openRowOf({ ...MOVED_DETAIL.summary, inTrash: 1, putBack: 2 });

      expect(row).toHaveTextContent('Put back: 2 of 3 · In the Trash: 1');
    });

    it('a stopped cleanup put back says put back', () => {
      const row = openRowOf(cleanupSummaryOf(4, STOPPED_REPORT, { inTrash: 0, putBack: 1 }));

      expect(row).toHaveTextContent('Put back: 1 of 3');
      expect(row).not.toHaveTextContent('Stopped');
    });
  });
});
