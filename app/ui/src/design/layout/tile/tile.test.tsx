import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { Tile } from './tile';

const css = readFileSync(path.join(import.meta.dirname, 'tile.module.css'), 'utf8');

describe('Tile', () => {
  describe('negative cases', () => {
    it('a plain tile has no medium surface', () => {
      render(<Tile tone="safe">Caches</Tile>);

      expect(screen.getByText('Caches')).toHaveAttribute('data-surface', 'plain');
      expect(screen.getByText('Caches')).toHaveAttribute('data-padding', 'normal');
    });
  });

  describe('positive cases', () => {
    it('takes its tone border', () => {
      render(<Tile tone="low">Caches</Tile>);

      expect(screen.getByText('Caches')).toHaveAttribute('data-level', 'low');
      expect(css).toMatch(/\[data-level='low'\] \{[^}]*--tile-edge: var\(--tile-border-low\);/u);
    });

    it('a medium tile takes the medium surface', () => {
      render(
        <Tile surface="medium" tone="medium">
          Archives
        </Tile>,
      );

      expect(screen.getByText('Archives')).toHaveAttribute('data-surface', 'medium');
      expect(css).toMatch(/\[data-surface='medium'\] \{[^}]*var\(--surface-tile-medium\)/u);
    });

    it('can be a list item without padding', () => {
      render(
        <ul>
          <Tile as="li" padding="none" tone="medium">
            Review
          </Tile>
        </ul>,
      );

      expect(screen.getByRole('listitem')).toHaveAttribute('data-padding', 'none');
    });

    it('an article is named by its heading', () => {
      render(
        <Tile as="article" labelledBy="name">
          <h2 id="name">App caches</h2>
        </Tile>,
      );

      expect(screen.getByRole('article', { name: 'App caches' })).toBeInTheDocument();
    });
  });
});
