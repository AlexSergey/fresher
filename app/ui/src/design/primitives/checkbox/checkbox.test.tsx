import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { Checkbox } from './checkbox';

const css = readFileSync(path.join(import.meta.dirname, 'checkbox.module.css'), 'utf8');

describe('Checkbox', () => {
  describe('negative cases', () => {
    it('a disabled checkbox ignores clicks', () => {
      const onChange = vi.fn();
      render(<Checkbox checked={false} disabled label="Xcode caches" onChange={onChange} />);

      const checkbox = screen.getByRole('checkbox', { name: 'Xcode caches' });
      fireEvent.click(checkbox);

      expect(checkbox).toBeDisabled();
      expect(onChange).not.toHaveBeenCalled();
    });

    it('the mixed state is not checked for assistive technology', () => {
      render(<Checkbox checked="mixed" label="All caches" onChange={vi.fn()} />);

      const checkbox = screen.getByRole('checkbox', { name: 'All caches' });

      expect(checkbox).not.toBeChecked();
      expect(checkbox).toBePartiallyChecked();
    });

    it('has no description when none is given', () => {
      render(<Checkbox checked={false} label="Xcode caches" onChange={vi.fn()} />);

      expect(screen.getByRole('checkbox', { name: 'Xcode caches' })).not.toHaveAttribute('aria-describedby');
    });

    it('is a tab stop of its own unless told otherwise', () => {
      render(<Checkbox checked={false} label="Xcode caches" onChange={vi.fn()} />);

      expect(screen.getByRole('checkbox', { name: 'Xcode caches' })).not.toHaveAttribute('tabindex');
    });

    it('keeps the mixed state until the parent changes it', () => {
      render(<Checkbox checked="mixed" label="All caches" onChange={vi.fn()} />);

      const checkbox = screen.getByRole('checkbox', { name: 'All caches' });
      fireEvent.click(checkbox);

      expect(checkbox).toBePartiallyChecked();
    });
  });

  describe('positive cases', () => {
    it('toggles on click', () => {
      const onChange = vi.fn();
      const { rerender } = render(<Checkbox checked={false} label="Xcode caches" onChange={onChange} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'Xcode caches' }));

      expect(onChange).toHaveBeenLastCalledWith(true);

      rerender(<Checkbox checked label="Xcode caches" onChange={onChange} />);
      fireEvent.click(screen.getByRole('checkbox', { name: 'Xcode caches' }));

      expect(onChange).toHaveBeenLastCalledWith(false);
    });

    it('leaves the tab order inside a row, and still toggles on click', () => {
      const onChange = vi.fn();
      render(<Checkbox checked={false} hideLabel label="Select Xcode caches" onChange={onChange} tabIndex={-1} />);

      const checkbox = screen.getByRole('checkbox', { name: 'Select Xcode caches' });
      fireEvent.click(checkbox);

      expect(checkbox).toHaveAttribute('tabindex', '-1');
      expect(onChange).toHaveBeenCalledWith(true);
    });

    it('a mixed checkbox becomes checked on click', () => {
      const onChange = vi.fn();
      render(<Checkbox checked="mixed" label="All caches" onChange={onChange} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'All caches' }));

      expect(onChange).toHaveBeenCalledWith(true);
    });

    it('is a real checkbox, which the browser toggles with Space', () => {
      render(<Checkbox checked label="Xcode caches" onChange={vi.fn()} />);

      const checkbox = screen.getByRole('checkbox', { name: 'Xcode caches' });

      expect(checkbox).toBeChecked();
      expect(checkbox.tagName).toBe('INPUT');
    });

    it('links its description', () => {
      render(
        <Checkbox
          checked={false}
          description="They come back the next time you build."
          label="Xcode caches"
          onChange={vi.fn()}
        />,
      );

      expect(screen.getByRole('checkbox', { name: 'Xcode caches' })).toHaveAccessibleDescription(
        'They come back the next time you build.',
      );
    });

    it('keeps its name when the label is visually hidden', () => {
      render(<Checkbox checked={false} hideLabel label="Select Xcode caches" onChange={vi.fn()} />);

      expect(screen.getByRole('checkbox', { name: 'Select Xcode caches' })).toBeInTheDocument();
    });

    it('draws the mark by stroke-dashoffset on the settle curve', () => {
      expect(css).toMatch(/transition: stroke-dashoffset var\(--duration-fast\) var\(--ease-settle\)/u);
    });
  });
});
