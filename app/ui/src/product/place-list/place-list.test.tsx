import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from 'vitest';

import type { NodeInfo } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { BUSY } from '../../test/demo-scan';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { fakeViewport } from '../../test/viewport';
import { STORY_DICTIONARY } from '../scan-hero/story-data';
import { PlaceList, type PlaceListProps } from './place-list';
import { DESCENT, EMPTY, HOME, type StoryPlace, UNREADABLE } from './story-data';

type Handlers = {
  readonly onOpen: Mock<(node: NodeInfo) => void>;
  readonly onStep: Mock<(index: number) => void>;
};

const renderPlace = (place: StoryPlace, props: Partial<PlaceListProps> = {}): Handlers => {
  const handlers: Handlers = { onOpen: vi.fn(), onStep: vi.fn() };

  render(
    <PlaceList
      crumbs={place.path.map(({ id, name }) => ({ key: String(id.index), name }))}
      failure={null}
      folder={place.folder}
      onRangeChange={vi.fn()}
      rowAt={(index) => place.rows[index]}
      total={place.rows.length}
      {...handlers}
      {...props}
    />,
  );

  return handlers;
};

// The rows of the folder's list, not the breadcrumb's steps.
const rows = (): readonly HTMLElement[] =>
  within(screen.getByRole('list', { name: /^Inside / })).getAllByRole('listitem');

const rowNamed = (name: string): HTMLElement => screen.getByRole('listitem', { name: new RegExp(`^${name},`, 'u') });

beforeEach(() => {
  setCoreDictionary({ ...STORY_DICTIONARY, ...STORAGE_DICTIONARY });
  fakeViewport(440);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('PlaceList', () => {
  describe('negative cases', () => {
    it('a row has no checkbox', () => {
      renderPlace(HOME);

      expect(screen.queryByRole('checkbox')).toBeNull();
    });

    it('a file row does not open', () => {
      const { onOpen } = renderPlace(HOME);

      fireEvent.click(rowNamed('archive.zip'));
      fireEvent.keyDown(rowNamed('archive.zip'), { key: 'Enter' });
      fireEvent.click(rowNamed('Shared'));

      expect(onOpen).not.toHaveBeenCalled();
    });

    it('an unreadable folder says why', () => {
      const { onOpen } = renderPlace(UNREADABLE);

      expect(rowNamed('Mail')).toHaveAccessibleDescription(
        'Protected by macOS. Full Disk Access lets the next scan measure it.',
      );
      expect(rowNamed('Mobile Documents')).toHaveAccessibleDescription(
        'Not opened: opening it would make macOS ask you for access.',
      );
      expect(rowNamed('Backup')).toHaveAccessibleDescription(
        'On another disk, so it was not scanned with this folder.',
      );

      fireEvent.click(rowNamed('Mail'));
      expect(onOpen).not.toHaveBeenCalled();
    });

    it('a breadcrumb step does not descend again', () => {
      const { onOpen, onStep } = renderPlace(DESCENT);

      fireEvent.click(screen.getByRole('button', { name: 'Back to Containers' }));

      expect(onStep.mock.calls).toStrictEqual([[1]]);
      expect(onOpen).not.toHaveBeenCalled();
    });

    it('a busy core says so instead of rows', () => {
      renderPlace(HOME, { failure: { kind: 'refused', refusal: BUSY }, total: undefined });

      expect(screen.getByText('The list by place could not be loaded')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: /^Inside / })).toBeNull();
    });

    it('an empty folder says so', () => {
      renderPlace(EMPTY);

      expect(screen.getByText('This folder is empty.')).toBeInTheDocument();
      expect(screen.queryByRole('list', { name: /^Inside / })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('rows come largest first', () => {
      renderPlace(HOME);

      const names = rows().map((row) => row.getAttribute('aria-label')?.split(',')[0]);

      expect(names).toStrictEqual(['Library', 'Movies', 'Downloads', 'Documents', 'archive.zip', 'Shared']);
    });

    it('the explained share is drawn', () => {
      renderPlace(HOME);

      const library = rowNamed('Library');
      // eslint-disable-next-line testing-library/no-node-access -- the bar is decoration, it has no role to query by
      const share = library.querySelector<HTMLElement>('[data-part="row"]');
      // eslint-disable-next-line testing-library/no-node-access -- as above
      const explained = library.querySelector<HTMLElement>('[data-part="explained"]');

      expect(library).toHaveAccessibleDescription('9 GB explained');
      expect(share?.style.width).toBe('46.67%');
      expect(explained?.style.width).toBe('6.43%');
      // eslint-disable-next-line testing-library/no-node-access -- as above
      expect(rowNamed('Movies').querySelector('[data-part="explained"]')).toBeNull();
    });

    it('the descent is one breadcrumb', () => {
      renderPlace(DESCENT);

      const breadcrumb = screen.getByRole('navigation', { name: 'Where you are' });

      expect(
        within(breadcrumb)
          .getAllByRole('listitem')
          .map(({ textContent }) => textContent),
      ).toStrictEqual(['Library', '›Containers', '›com.example.app']);
      expect(within(breadcrumb).getByText('com.example.app')).toHaveAttribute('aria-current', 'location');
    });

    it('a folder row opens with a click or Enter', () => {
      const { onOpen } = renderPlace(HOME);

      fireEvent.click(rowNamed('Movies'));
      fireEvent.keyDown(rowNamed('Library'), { key: 'Enter' });

      expect(onOpen.mock.calls.map(([node]) => node.name)).toStrictEqual(['Movies', 'Library']);
    });

    it('the rows and the breadcrumb are reachable by the keyboard', () => {
      renderPlace(DESCENT);

      expect(screen.getByRole('button', { name: 'Back to Library' })).not.toHaveAttribute('tabindex');
      expect(rows().filter((row) => row.tabIndex === 0)).toHaveLength(1);
    });
  });
});
