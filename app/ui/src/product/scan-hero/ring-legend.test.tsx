import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { MapPart } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { RingLegend, type RingLegendProps } from './ring-legend';
import { ringParts } from './ring-parts';
import { HERO_LEVELS, HERO_VOLUME, STORY_DICTIONARY } from './story-data';

// The labels are read when the parts are made, so after the core's texts are loaded.
const partsOfHero = (): ReturnType<typeof ringParts> => ringParts(HERO_VOLUME, HERO_LEVELS);

const mountLegend = (overrides: Partial<RingLegendProps> = {}): RingLegendProps => {
  const props: RingLegendProps = {
    highlighted: null,
    onHighlight: vi.fn(),
    parts: partsOfHero(),
    purgeable: 18_000_000_000,
    scanning: false,
    ...overrides,
  };

  render(<RingLegend {...props} />);

  return props;
};

// A row is the list item that holds the name; the list's items are all there is to choose from.
const rowOf = (name: string): HTMLElement => {
  const row = screen.getAllByRole('listitem').find((item) => within(item).queryByText(name) !== null);

  if (row === undefined) {
    throw new Error(`no row for ${name}`);
  }

  return row;
};

const aboutOf = (name: string): HTMLElement => within(rowOf(name)).getByRole('button', { name: `What is ${name}?` });

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('RingLegend', () => {
  describe('negative cases', () => {
    it('shows no explanation until its i is pressed', () => {
      mountLegend();

      expect(aboutOf('macOS')).toHaveAttribute('aria-expanded', 'false');
      expect(screen.queryByText(/looks after this space/u)).not.toBeVisible();
    });

    it('marks no row while nothing is highlighted', () => {
      mountLegend();

      expect(screen.getAllByRole('listitem').filter((row) => row.hasAttribute('data-highlighted'))).toHaveLength(0);
    });

    it('gives Free no level text', () => {
      mountLegend();
      fireEvent.click(aboutOf('Free'));

      expect(within(rowOf('Free')).queryByText(/Comes back|May hold/u)).not.toBeInTheDocument();
    });

    it('gives a part without reasons no list', () => {
      mountLegend();
      fireEvent.click(aboutOf('macOS'));

      expect(within(rowOf('macOS')).queryByRole('list')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('opens a level with what Explained is and what the level means', () => {
      mountLegend();
      fireEvent.click(aboutOf('Low risk'));

      expect(aboutOf('Low risk')).toHaveAttribute('aria-expanded', 'true');
      expect(within(rowOf('Low risk')).getByText(/^What Fresher recognised/u)).toBeVisible();
      expect(within(rowOf('Low risk')).getByText(/^Comes back when needed/u)).toBeVisible();
    });

    it("opens the remainder with the core's description and its reasons", () => {
      const remainder = HERO_VOLUME.parts.find(({ kind }) => kind.kind === 'remainder') as MapPart;
      const parts = partsOfHero().map((part) =>
        part.about.kind === 'part' && part.about.part.kind.kind === 'remainder'
          ? {
              ...part,
              about: {
                kind: 'part' as const,
                part: {
                  ...remainder,
                  kind: {
                    kind: 'remainder' as const,
                    outsideScan: true,
                    reasons: [{ kind: 'snapshots-and-bookkeeping' as const }],
                  },
                },
              },
            }
          : part,
      );
      mountLegend({ parts });
      fireEvent.click(aboutOf('Outside the scanned folders'));

      expect(within(rowOf('Outside the scanned folders')).getByRole('list')).toHaveTextContent(
        "Local snapshots and the file system's own records",
      );
    });

    it('opens Free with the purgeable note', () => {
      mountLegend();
      fireEvent.click(aboutOf('Free'));

      expect(within(rowOf('Free')).getByText('Space you can still use for new files.')).toBeVisible();
      expect(within(rowOf('Free')).getByText(/^macOS can free about 18 GB more/u)).toBeVisible();
    });

    it('points the i at its explanation', () => {
      mountLegend();
      const button = aboutOf('macOS');

      fireEvent.click(button);

      // eslint-disable-next-line testing-library/no-node-access -- the region has no role to query by
      expect(document.getElementById(button.getAttribute('aria-controls') ?? '')).toHaveTextContent(
        /^The system itself/u,
      );
    });

    it('calls onHighlight with the key on pointer enter and with null on leave', () => {
      const { onHighlight } = mountLegend();

      fireEvent.pointerEnter(rowOf('macOS'));
      fireEvent.pointerLeave(rowOf('macOS'));

      expect(onHighlight).toHaveBeenNthCalledWith(1, 'mac-os-3');
      expect(onHighlight).toHaveBeenNthCalledWith(2, null);
    });

    it('calls onHighlight when focus enters the row and when it leaves', () => {
      const { onHighlight } = mountLegend();

      fireEvent.focus(aboutOf('Safe to remove'));
      fireEvent.blur(aboutOf('Safe to remove'));

      expect(onHighlight).toHaveBeenNthCalledWith(1, 'level-safe');
      expect(onHighlight).toHaveBeenNthCalledWith(2, null);
    });

    it('marks the row of the highlighted key', () => {
      mountLegend({ highlighted: 'free-6' });

      expect(rowOf('Free')).toHaveAttribute('data-highlighted');
      expect(rowOf('macOS')).not.toHaveAttribute('data-highlighted');
    });
  });
});
