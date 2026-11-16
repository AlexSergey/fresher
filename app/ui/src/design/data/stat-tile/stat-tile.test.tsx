import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { StatTile } from './stat-tile';

describe('StatTile', () => {
  describe('negative cases', () => {
    it('a loading tile hides its value', () => {
      render(<StatTile label="Items" loading value="12,408" />);

      expect(screen.queryByText('12,408')).not.toBeInTheDocument();
    });

    it('shows no hint unless given', () => {
      render(<StatTile label="Items" value="12,408" />);

      expect(screen.getByRole('group', { name: 'Items' })).toHaveTextContent(/^Items12,408$/u);
    });
  });

  describe('positive cases', () => {
    it('the stat tile shows a skeleton while loading', () => {
      render(<StatTile label="Items" loading value="12,408" />);
      const tile = screen.getByRole('group', { name: 'Items' });

      expect(tile).toHaveAttribute('aria-busy', 'true');
      // eslint-disable-next-line testing-library/no-node-access -- the skeleton is hidden decoration
      expect(tile.querySelector('[data-shape="line"]')).toBeInTheDocument();
    });

    it('shows its label, its value and its hint', () => {
      render(<StatTile hint="in 6 modules" label="Items" value={<strong>12,408</strong>} />);
      const tile = screen.getByRole('group', { name: 'Items' });

      expect(tile).not.toHaveAttribute('aria-busy');
      expect(screen.getByText('12,408')).toBeInTheDocument();
      expect(screen.getByText('in 6 modules')).toBeInTheDocument();
    });
  });
});
