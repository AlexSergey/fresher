import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { APPS_DICTIONARY, MANAGED, VIDEO_STUDIO } from '../../test/apps-list';
import { AppRow } from './app-row';

beforeEach(() => {
  setCoreDictionary(APPS_DICTIONARY);
});

describe('AppRow', () => {
  describe('negative cases', () => {
    it('shows no breakdown until it is opened', () => {
      render(<AppRow onOpen={vi.fn()} summary={VIDEO_STUDIO} />);

      expect(screen.getByRole('button', { name: 'What it takes' })).toHaveAttribute('aria-expanded', 'false');
      expect(screen.queryByText('Caches')).not.toBeInTheDocument();
    });

    it('shows the reason of a not-offered app', () => {
      render(<AppRow onOpen={vi.fn()} summary={MANAGED} />);

      expect(screen.getByText(/Homebrew installed this app/u)).toBeVisible();
    });

    it('says nothing of a reason for an offered app', () => {
      render(<AppRow onOpen={vi.fn()} summary={VIDEO_STUDIO} />);

      expect(screen.queryByText(/installed this app/u)).not.toBeInTheDocument();
    });

    it('opening the breakdown does not open the page', () => {
      const onOpen = vi.fn();
      render(<AppRow onOpen={onOpen} summary={VIDEO_STUDIO} />);

      fireEvent.click(screen.getByRole('button', { name: 'What it takes' }));

      expect(onOpen).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it("shows the size and the core's sentence", () => {
      render(<AppRow onOpen={vi.fn()} summary={VIDEO_STUDIO} />);

      expect(screen.getByText('Video Studio')).toBeVisible();
      expect(screen.getByText('Video Studio takes 4 GB.')).toBeVisible();
      expect(screen.getAllByText('4 GB').length).toBeGreaterThan(0);
    });

    it('lists the parts by kind when it is opened', () => {
      render(<AppRow onOpen={vi.fn()} summary={VIDEO_STUDIO} />);

      fireEvent.click(screen.getByRole('button', { name: 'What it takes' }));

      expect(screen.getByText('Caches')).toBeVisible();
      expect(screen.getByText('Files it keeps for you')).toBeVisible();
    });

    it('opens the page on Enter and on a click', () => {
      const onOpen = vi.fn();
      render(<AppRow onOpen={onOpen} summary={VIDEO_STUDIO} />);
      const row = screen.getByRole('listitem');

      fireEvent.keyDown(row, { key: 'Enter' });
      fireEvent.click(row);

      expect(onOpen).toHaveBeenCalledTimes(2);
    });
  });
});
