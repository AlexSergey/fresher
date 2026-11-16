import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { createRef } from 'react';
import { describe, expect, it } from 'vitest';

import { Surface } from './surface';

const css = readFileSync(path.join(import.meta.dirname, 'surface.module.css'), 'utf8');
const semantic = readFileSync(path.join(import.meta.dirname, '../../tokens/semantic.css'), 'utf8');

describe('Surface', () => {
  describe('negative cases', () => {
    it('takes the medium radius and padding unless told otherwise', () => {
      render(<Surface elevation={1}>Card</Surface>);

      expect(screen.getByText('Card')).toHaveAttribute('data-radius', 'md');
      expect(screen.getByText('Card')).toHaveAttribute('data-padding', 'medium');
    });

    it('casts no shadow at elevation 1', () => {
      expect(semantic).toMatch(/--elevation-1-shadow: inset 0 1px 0 var\(--white-a4\);/u);
    });
  });

  describe('positive cases', () => {
    it('elevation 3 has the overlay surface', () => {
      render(<Surface elevation={3}>Dialog</Surface>);

      expect(screen.getByText('Dialog')).toHaveAttribute('data-elevation', '3');
      expect(css).toMatch(
        /\.surface\[data-elevation='3'\] \{[^}]*--surface-fill: var\(--elevation-3-surface\);[^}]*\}/u,
      );
      expect(semantic).toMatch(/--elevation-3-surface: var\(--surface-overlay\);/u);
    });

    it('draws the gradient fill, the hairline and the top highlight of its level', () => {
      expect(css).toMatch(/background: var\(--surface-raised-sheen\), var\(--surface-fill\);/u);
      expect(css).toMatch(/border: 1px solid var\(--surface-border\);/u);
      expect(css).toMatch(/box-shadow: var\(--surface-shadow\);/u);
    });

    it('passes its ref and class on, so a card can spread its spotlight on it', () => {
      const ref = createRef<HTMLDivElement>();
      render(
        <Surface className="card" elevation={2} padding="large" radius="xl" ref={ref}>
          Hero
        </Surface>,
      );

      expect(ref.current).toBe(screen.getByText('Hero'));
      expect(screen.getByText('Hero')).toHaveClass('card');
      expect(screen.getByText('Hero')).toHaveAttribute('data-radius', 'xl');
      expect(screen.getByText('Hero')).toHaveAttribute('data-padding', 'large');
    });
  });
});
