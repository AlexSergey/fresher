import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, type Mock, vi } from 'vitest';

import type { CategoryId, MeaningGroup, NodeId } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { BUSY } from '../../test/demo-scan';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND, STORY_DICTIONARY } from '../scan-hero/story-data';
import { MeaningList, type MeaningListProps } from './meaning-list';
import { MEANING } from './story-data';

type Handlers = {
  readonly onOpen: Mock<(category: CategoryId) => void>;
  readonly onOpenPlace: Mock<(node: NodeId) => void>;
};

const renderList = (
  groups: null | readonly MeaningGroup[] = MEANING,
  failure: MeaningListProps['failure'] = null,
): Handlers => {
  const handlers: Handlers = { onOpen: vi.fn(), onOpenPlace: vi.fn() };

  render(
    <MeaningList
      failure={failure}
      found={DEMO_TOTALS_FOUND}
      groups={groups}
      onToggle={vi.fn()}
      scanning={false}
      selected={() => false}
      totals={DEMO_TOTALS}
      {...handlers}
    />,
  );

  return handlers;
};

const FIVE_GB = { allocated: 5e9, completeness: 'complete', logical: 5e9, reclaimable: 5e9 } as const;

// The cards, each a button "Open <name>".
const cards = (): readonly HTMLElement[] =>
  within(screen.getByRole('list', { name: 'Categories' })).getAllByRole('button', { name: /^Open/u });

// The accessible names of the lists, in the order they are on the page.
const listNames = (): readonly string[] =>
  screen.getAllByRole('list').map((list) => list.getAttribute('aria-label') ?? '');

beforeEach(() => {
  setCoreDictionary({ ...STORY_DICTIONARY, ...STORAGE_DICTIONARY });
});

describe('MeaningList', () => {
  describe('negative cases', () => {
    it('a place row has no checkbox', () => {
      renderList();

      const places = screen.getByRole('list', { name: 'Places' });

      expect(within(places).queryByRole('checkbox')).toBeNull();
      expect(within(screen.getByRole('list', { name: 'The rest of the disk' })).queryByRole('checkbox')).toBeNull();
      expect(within(screen.getByRole('group', { name: 'Apps' })).queryByRole('checkbox')).toBeNull();
    });

    it('an uninstalled app says so', () => {
      renderList();

      const apps = within(screen.getByRole('group', { name: 'Apps' })).getAllByRole('listitem');

      expect(apps[0]).not.toHaveTextContent('not installed');
      expect(apps[1]).toHaveTextContent('Old Example');
      expect(apps[1]).toHaveTextContent('not installed');
    });

    it('before the core answers, the cards still show', () => {
      renderList(null);

      expect(listNames()).toStrictEqual(['Categories']);
      expect(cards()).toHaveLength(DEMO_TOTALS.length);
    });

    it('a category the core does not list for the volume has no card', () => {
      renderList();

      expect(cards()).toHaveLength(3);
      expect(screen.getByRole('button', { name: /^Open Example caches/u })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /^Open Example downloads/u })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /^Open Example volumes/u })).toBeNull();
    });

    it('a failed answer says so and keeps the cards', () => {
      renderList(null, { kind: 'refused', refusal: BUSY });

      expect(screen.getByText('The list by meaning could not be loaded')).toBeInTheDocument();
      expect(cards()).toHaveLength(DEMO_TOTALS.length);
    });
  });

  describe('positive cases', () => {
    it('apps come first, then categories, then places', () => {
      renderList();

      const sections = [
        screen.getByRole('group', { name: 'Apps' }),
        screen.getByRole('list', { name: 'Categories' }),
        screen.getByRole('list', { name: 'Places' }),
        screen.getByRole('list', { name: 'The rest of the disk' }),
      ];

      for (const [index, section] of sections.slice(1).entries()) {
        expect(sections[index]?.compareDocumentPosition(section) ?? 0).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
      }
    });

    it("a card shows the core's size, not the counted one", () => {
      const counted = DEMO_TOTALS.find(({ category }) => category === 'demo.caches')?.sizes.allocated;
      renderList([{ kind: 'category', value: { category: 'demo.caches', sizes: FIVE_GB } }]);

      const card = screen.getByRole('button', { name: /^Open Example caches/u });

      expect(counted).not.toBe(FIVE_GB.allocated);
      expect(card).toHaveTextContent('5 GB');
      expect(card).toHaveTextContent('2 items');
    });

    it('a category wholly under an app keeps its card, which says the space sits under apps', () => {
      renderList();

      const card = screen.getByRole('button', { name: /^Open Example images/u });

      expect(card).toHaveTextContent('0 bytes');
      expect(card).toHaveTextContent('420 MB more under apps');
      expect(card).toHaveTextContent('4 items');

      const toggle = screen.getByRole('button', { name: 'Show what Example Editor keeps' });
      fireEvent.click(toggle);

      const kept = screen.getByRole('group', { name: 'What Example Editor keeps' });
      expect(toggle).toHaveAttribute('aria-controls', kept.id);
      expect(within(kept).getByRole('button', { name: 'Open Example images' })).toBeVisible();
    });

    it('a category partly under an app shows its own share and what more sits under apps', () => {
      renderList([
        {
          kind: 'app',
          value: {
            app: { id: 'com.example.editor', installed: true, name: 'Example Editor' },
            categories: [{ category: 'demo.caches', sizes: FIVE_GB }],
            data: [],
            sizes: FIVE_GB,
          },
        },
        { kind: 'category', value: { category: 'demo.caches', sizes: { ...FIVE_GB, allocated: 1e9 } } },
      ]);

      const card = screen.getByRole('button', { name: /^Open Example caches/u });

      expect(card).toHaveTextContent('1 GB');
      expect(card).toHaveTextContent('5 GB more under apps');
    });

    it('a card with nothing under apps says nothing of apps', () => {
      renderList();

      expect(screen.getByRole('button', { name: /^Open Example caches/u })).not.toHaveTextContent('under apps');
    });

    it('a category row opens its module', () => {
      const { onOpen } = renderList();

      fireEvent.click(screen.getByRole('button', { name: /^Open Example caches/u }));
      fireEvent.click(screen.getByRole('button', { name: 'Show what Example Editor keeps' }));
      fireEvent.click(screen.getByRole('button', { name: 'Open Example images' }));

      expect(onOpen.mock.calls).toStrictEqual([['demo.caches'], ['demo.images']]);
    });

    it('a place row opens the place view', () => {
      const { onOpenPlace } = renderList();

      fireEvent.click(screen.getByRole('button', { name: 'Look inside Trash' }));
      fireEvent.click(screen.getByRole('button', { name: 'Look inside Movies' }));

      expect(onOpenPlace.mock.calls).toStrictEqual([[{ index: 7, state: 1 }], [{ index: 8, state: 1 }]]);
      expect(screen.getByText('You get this space back when you empty the Trash.')).toBeInTheDocument();
    });
  });
});
