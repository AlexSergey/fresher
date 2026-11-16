import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { ProgressBar } from './progress-bar';

const css = readFileSync(path.join(import.meta.dirname, 'progress-bar.module.css'), 'utf8');

describe('ProgressBar', () => {
  describe('negative cases', () => {
    it('an indeterminate progress bar has no aria-valuenow', () => {
      render(<ProgressBar label="Scanning" value={null} />);

      expect(screen.getByRole('progressbar', { name: 'Scanning' })).not.toHaveAttribute('aria-valuenow');
    });

    it('keeps a value outside 0 to 100 inside the bar', () => {
      const { rerender } = render(<ProgressBar label="Scanning" value={140} />);

      expect(screen.getByRole('progressbar', { name: 'Scanning' })).toHaveAttribute('aria-valuenow', '100');

      rerender(<ProgressBar label="Scanning" value={-5} />);

      expect(screen.getByRole('progressbar', { name: 'Scanning' })).toHaveAttribute('aria-valuenow', '0');
    });

    it('the sweep stands still unless loops run', () => {
      expect(css).toMatch(/animation-play-state: var\(--loop-state\);/u);
    });
  });

  describe('positive cases', () => {
    it('the progress bar reports its value', () => {
      render(<ProgressBar label="Scanning" value={40} />);

      const bar = screen.getByRole('progressbar', { name: 'Scanning' });

      expect(bar).toHaveAttribute('aria-valuenow', '40');
      expect(bar).toHaveAttribute('aria-valuemin', '0');
      expect(bar).toHaveAttribute('aria-valuemax', '100');
    });

    it('grows its fill by transform', () => {
      render(<ProgressBar label="Scanning" value={40} />);

      // eslint-disable-next-line testing-library/no-node-access -- the fill is decoration inside the bar
      const fill = screen.getByRole('progressbar', { name: 'Scanning' }).firstElementChild as HTMLElement;

      expect(fill.style.getPropertyValue('--progress')).toBe('0.4');
      expect(css).toMatch(/transition: transform var\(--duration-slow\) var\(--ease-standard\);/u);
    });
  });
});
