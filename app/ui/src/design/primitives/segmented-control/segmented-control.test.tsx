import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { SegmentedControl } from './segmented-control';

type Sort = 'count' | 'name' | 'size';

const OPTIONS = [
  { label: 'Size', value: 'size' },
  { label: 'Name', value: 'name' },
  { label: 'Count', value: 'count' },
] as const;

const css = readFileSync(path.join(import.meta.dirname, 'segmented-control.module.css'), 'utf8');
const motionCss = readFileSync(path.join(import.meta.dirname, '../../tokens/motion.css'), 'utf8');

const showSort = (value: Sort, onChange = vi.fn(), disabled = false): ReturnType<typeof vi.fn> => {
  render(<SegmentedControl disabled={disabled} label="Sort by" onChange={onChange} options={OPTIONS} value={value} />);

  return onChange;
};

describe('SegmentedControl', () => {
  describe('negative cases', () => {
    it('the segmented control does not move past the last option with the arrow key', () => {
      const onChange = showSort('count');

      fireEvent.keyDown(screen.getByRole('radio', { name: 'Count' }), { key: 'ArrowRight' });

      expect(onChange).not.toHaveBeenCalled();
    });

    it('does not move before the first option', () => {
      const onChange = showSort('size');

      fireEvent.keyDown(screen.getByRole('radio', { name: 'Size' }), { key: 'ArrowLeft' });
      fireEvent.keyDown(screen.getByRole('radio', { name: 'Size' }), { key: 'Home' });

      expect(onChange).not.toHaveBeenCalled();
    });

    it('ignores other keys and a click on the chosen option', () => {
      const onChange = showSort('name');

      fireEvent.keyDown(screen.getByRole('radio', { name: 'Name' }), { key: 'a' });
      fireEvent.click(screen.getByRole('radio', { name: 'Name' }));

      expect(onChange).not.toHaveBeenCalled();
    });

    it('a disabled control does not change', () => {
      const onChange = showSort('size', vi.fn(), true);

      fireEvent.click(screen.getByRole('radio', { name: 'Name' }));

      expect(screen.getByRole('radio', { name: 'Name' })).toBeDisabled();
      expect(onChange).not.toHaveBeenCalled();
    });

    it('the indicator does not animate under motion off or reduced', () => {
      const transition = /\.indicator \{[^}]*transition: ([^;]+);/u.exec(css)?.[1];
      const off = /:root\[data-motion='off'\] \{[^}]*--duration-base: 0ms;/u;

      expect(transition).toBe('transform var(--duration-base) var(--ease-settle)');
      expect(motionCss).toMatch(off);
      expect(css).toMatch(/:root\[data-motion='reduced'\] \.indicator \{\s*transition: none;/u);
    });
  });

  describe('positive cases', () => {
    it('is a named radio group with its choice checked', () => {
      showSort('name');

      expect(screen.getByRole('radiogroup', { name: 'Sort by' })).toBeInTheDocument();
      expect(screen.getByRole('radio', { name: 'Name' })).toBeChecked();
      expect(screen.getByRole('radio', { name: 'Size' })).not.toBeChecked();
    });

    it('only the chosen option is in the tab order', () => {
      showSort('name');

      expect(screen.getByRole('radio', { name: 'Name' })).toHaveAttribute('tabindex', '0');
      expect(screen.getByRole('radio', { name: 'Size' })).toHaveAttribute('tabindex', '-1');
    });

    it('the segmented control moves the choice with the arrows and Home and End', () => {
      const onChange = showSort('name');
      const name = screen.getByRole('radio', { name: 'Name' });

      fireEvent.keyDown(name, { key: 'ArrowRight' });
      fireEvent.keyDown(name, { key: 'ArrowDown' });
      fireEvent.keyDown(name, { key: 'ArrowLeft' });
      fireEvent.keyDown(name, { key: 'ArrowUp' });
      fireEvent.keyDown(name, { key: 'Home' });
      fireEvent.keyDown(name, { key: 'End' });

      expect(onChange.mock.calls).toStrictEqual([['count'], ['count'], ['size'], ['size'], ['size'], ['count']]);
    });

    it('moves the focus with the choice', () => {
      showSort('name');

      fireEvent.keyDown(screen.getByRole('radio', { name: 'Name' }), { key: 'End' });

      expect(screen.getByRole('radio', { name: 'Count' })).toHaveFocus();
    });

    it('chooses an option on click', () => {
      const onChange = showSort('size');

      fireEvent.click(screen.getByRole('radio', { name: 'Count' }));

      expect(onChange).toHaveBeenCalledWith('count');
    });

    it('places the indicator from the chosen index', () => {
      showSort('count');

      expect(screen.getByRole('radiogroup', { name: 'Sort by' }).style.getPropertyValue('--segment-index')).toBe('2');
    });
  });
});
