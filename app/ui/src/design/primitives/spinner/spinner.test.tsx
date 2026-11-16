import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { Spinner } from './spinner';

const css = readFileSync(path.join(import.meta.dirname, 'spinner.module.css'), 'utf8');
const motionCss = readFileSync(path.join(import.meta.dirname, '../../tokens/motion.css'), 'utf8');

describe('Spinner', () => {
  describe('negative cases', () => {
    it('is hidden from assistive technology without a label', () => {
      const { container } = render(<Spinner />);

      expect(screen.queryByRole('img')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-container, testing-library/no-node-access -- a hidden spinner has no role to query by
      expect(container.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    });

    it('the spinner is still under reduced motion', () => {
      const reduced = /:root\[data-motion='reduced'\] \{[^}]*--loop-state: paused;/u;

      expect(css).toMatch(/animation-play-state: var\(--loop-state\);/u);
      expect(motionCss).toMatch(reduced);
    });
  });

  describe('positive cases', () => {
    it('has its label as its accessible name', () => {
      render(<Spinner label="Loading the findings" />);

      expect(screen.getByRole('img', { name: 'Loading the findings' })).toBeInTheDocument();
    });

    it('is 16 pixels, or 12 when small', () => {
      const { rerender } = render(<Spinner label="Loading" />);

      expect(screen.getByRole('img', { name: 'Loading' })).toHaveAttribute('width', '16');

      rerender(<Spinner label="Loading" size="small" />);

      expect(screen.getByRole('img', { name: 'Loading' })).toHaveAttribute('width', '12');
    });

    it('turns by a transform loop', () => {
      expect(css).toMatch(/transform: rotate\(360deg\);/u);
    });
  });
});
