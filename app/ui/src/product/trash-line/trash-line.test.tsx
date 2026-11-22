import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Sizes, TrashLine } from '../../ipc/bindings.generated';

import { TrashLines } from './trash-line';

const sizes = (allocated: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated,
  completeness,
  logical: allocated,
  reclaimable: allocated,
});

const line = (props: Partial<TrashLine> = {}): TrashLine => ({
  fromCleanups: sizes(1_200_000_000),
  goneBy: null,
  index: 0,
  volume: 'Macintosh HD',
  waiting: sizes(5_000_000_000),
  ...props,
});

describe('TrashLines', () => {
  describe('negative cases', () => {
    it('renders nothing for no line', () => {
      render(<TrashLines lines={[]} onOpen={vi.fn()} />);

      expect(screen.queryByRole('region', { name: 'The Trash' })).toBeNull();
      expect(screen.queryByRole('button')).toBeNull();
    });

    it('names no volume for a single line', () => {
      render(<TrashLines lines={[line()]} onOpen={vi.fn()} />);

      expect(screen.queryByText('Macintosh HD')).toBeNull();
    });

    it('shows no date without a goneBy', () => {
      render(<TrashLines lines={[line()]} onOpen={vi.fn()} />);

      expect(screen.queryByText(/Finder removes them/u)).toBeNull();
    });

    it('offers no way to empty the Trash', () => {
      render(<TrashLines lines={[line({ goneBy: 1_768_478_400 })]} onOpen={vi.fn()} />);

      expect(screen.getAllByRole('button')).toHaveLength(1);
      expect(screen.getByRole('button', { name: 'Open Trash' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /empty/iu })).toBeNull();
      expect(screen.queryByRole('link', { name: /empty/iu })).toBeNull();
      expect(screen.queryByText(/empty/iu, { ignore: '[id]' })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('says what waits and the share from cleanups', () => {
      render(<TrashLines lines={[line()]} onOpen={vi.fn()} />);

      expect(screen.getByText('Waiting in the Trash: 5 GB, 1.2 GB of it from cleanups here.')).toBeInTheDocument();
    });

    it('says all of it is from cleanups when the share reaches the whole', () => {
      render(<TrashLines lines={[line({ waiting: sizes(1_200_000_000) })]} onOpen={vi.fn()} />);

      expect(screen.getByText('Waiting in the Trash: 1.2 GB, all of it from cleanups here.')).toBeInTheDocument();
    });

    it('says only what waits when no cleanup put anything there', () => {
      render(<TrashLines lines={[line({ fromCleanups: sizes(0) })]} onOpen={vi.fn()} />);

      expect(screen.getByText('Waiting in the Trash: 5 GB.')).toBeInTheDocument();
    });

    it('says the rest was not measured when only the share is known', () => {
      render(<TrashLines lines={[line({ waiting: null })]} onOpen={vi.fn()} />);

      expect(
        screen.getByText('Waiting in the Trash from cleanups here: 1.2 GB. The rest of the Trash was not measured.'),
      ).toBeInTheDocument();
    });

    it('shows a partial size with its plus', () => {
      render(
        <TrashLines
          lines={[line({ fromCleanups: sizes(0), waiting: sizes(5_000_000_000, 'partial') })]}
          onOpen={vi.fn()}
        />,
      );

      expect(screen.getByText('Waiting in the Trash: 5 GB+.')).toBeInTheDocument();
    });

    it('shows the date Finder removes them by', () => {
      render(<TrashLines lines={[line({ goneBy: 1_768_478_400 })]} onOpen={vi.fn()} />);

      expect(screen.getByText('Finder removes them by itself by Jan 15, 2026.')).toBeInTheDocument();
    });

    it('names the volumes when there are two lines', () => {
      render(<TrashLines lines={[line(), line({ index: 1, volume: 'Backup' })]} onOpen={vi.fn()} />);

      expect(screen.getByText('Macintosh HD')).toBeInTheDocument();
      expect(screen.getByText('Backup')).toBeInTheDocument();
    });

    it('describes Open Trash by the invitation', () => {
      render(<TrashLines lines={[line()]} onOpen={vi.fn()} />);

      expect(screen.getByRole('button', { name: 'Open Trash' })).toHaveAccessibleDescription(
        'Look at what is there and empty it yourself if you no longer need it.',
      );
    });

    it('opens the Trash of the line it belongs to', () => {
      const onOpen = vi.fn();
      render(<TrashLines lines={[line(), line({ index: 1, volume: 'Backup' })]} onOpen={onOpen} />);

      fireEvent.click(screen.getAllByRole('button', { name: 'Open Trash' })[1] as HTMLElement);

      expect(onOpen).toHaveBeenCalledExactlyOnceWith(1);
    });
  });
});
