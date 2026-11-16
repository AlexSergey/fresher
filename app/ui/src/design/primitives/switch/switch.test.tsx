import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { Switch } from './switch';

const css = readFileSync(path.join(import.meta.dirname, 'switch.module.css'), 'utf8');

describe('Switch', () => {
  describe('negative cases', () => {
    it('a disabled switch keeps its value', () => {
      const onChange = vi.fn();
      render(<Switch checked disabled label="Include the Trash" onChange={onChange} />);

      const control = screen.getByRole('switch', { name: 'Include the Trash' });
      fireEvent.click(control);

      expect(onChange).not.toHaveBeenCalled();
      expect(control).toBeChecked();
    });

    it('is not a submit button', () => {
      render(<Switch checked={false} label="Include the Trash" onChange={vi.fn()} />);

      expect(screen.getByRole('switch', { name: 'Include the Trash' })).toHaveAttribute('type', 'button');
    });

    it('does not slide its thumb under reduced motion', () => {
      const reduced = /:root\[data-motion='reduced'\] \.thumb \{\s*transition: ([^;]+);/u.exec(css)?.[1];

      expect(reduced).toBe('background-color var(--duration-fast) var(--ease-standard)');
    });
  });

  describe('positive cases', () => {
    it('the switch reports aria-checked', () => {
      const { rerender } = render(<Switch checked={false} label="Include the Trash" onChange={vi.fn()} />);

      expect(screen.getByRole('switch', { name: 'Include the Trash' })).toHaveAttribute('aria-checked', 'false');

      rerender(<Switch checked label="Include the Trash" onChange={vi.fn()} />);

      expect(screen.getByRole('switch', { name: 'Include the Trash' })).toHaveAttribute('aria-checked', 'true');
    });

    it('turns on and off on click', () => {
      const onChange = vi.fn();
      const { rerender } = render(<Switch checked={false} label="Include the Trash" onChange={onChange} />);

      fireEvent.click(screen.getByRole('switch', { name: 'Include the Trash' }));

      expect(onChange).toHaveBeenLastCalledWith(true);

      rerender(<Switch checked label="Include the Trash" onChange={onChange} />);
      fireEvent.click(screen.getByRole('switch', { name: 'Include the Trash' }));

      expect(onChange).toHaveBeenLastCalledWith(false);
    });

    it('keeps its name when the label is visually hidden', () => {
      render(<Switch checked={false} hideLabel label="Include the Trash" onChange={vi.fn()} />);

      expect(screen.getByRole('switch', { name: 'Include the Trash' })).toBeInTheDocument();
    });

    it('slides the thumb by transform on the settle curve', () => {
      expect(css).toMatch(/transform var\(--duration-base\) var\(--ease-settle\)/u);
    });
  });
});
