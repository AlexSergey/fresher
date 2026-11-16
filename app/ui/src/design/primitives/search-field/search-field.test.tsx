import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { SearchField } from './search-field';

const showSearch = (value: string, onChange = vi.fn()): HTMLElement => {
  render(<SearchField clearLabel="Clear the search" label="Search findings" onChange={onChange} value={value} />);

  return screen.getByRole('searchbox', { name: 'Search findings' });
};

describe('SearchField', () => {
  describe('negative cases', () => {
    it('the clear button is absent when the field is empty', () => {
      showSearch('');

      expect(screen.queryByRole('button', { name: 'Clear the search' })).not.toBeInTheDocument();
    });

    it('Escape on an empty search does nothing', () => {
      const onChange = vi.fn();
      const input = showSearch('', onChange);
      const escape = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, key: 'Escape' });

      fireEvent(input, escape);

      expect(onChange).not.toHaveBeenCalled();
      expect(escape.defaultPrevented).toBe(false);
    });

    it('other keys do not clear it', () => {
      const onChange = vi.fn();
      const input = showSearch('xcode', onChange);

      fireEvent.keyDown(input, { key: 'Enter' });

      expect(onChange).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('the label names the input', () => {
      expect(showSearch('')).toHaveAttribute('type', 'search');
    });

    it('keeps its name when the label is visually hidden', () => {
      render(<SearchField clearLabel="Clear the search" hideLabel label="Search" onChange={vi.fn()} value="" />);

      expect(screen.getByRole('searchbox', { name: 'Search' })).toBeInTheDocument();
    });

    it('typing calls onChange', () => {
      const onChange = vi.fn();
      const input = showSearch('', onChange);

      fireEvent.change(input, { target: { value: 'xcode' } });

      expect(onChange).toHaveBeenCalledWith('xcode');
    });

    it('the clear button empties the search and keeps the focus in it', () => {
      const onChange = vi.fn();
      const input = showSearch('xcode', onChange);

      fireEvent.click(screen.getByRole('button', { name: 'Clear the search' }));

      expect(onChange).toHaveBeenCalledWith('');
      expect(input).toHaveFocus();
    });

    it('Escape empties the search and keeps the focus in it', () => {
      const onChange = vi.fn();
      const input = showSearch('xcode', onChange);
      input.focus();

      fireEvent.keyDown(input, { key: 'Escape' });

      expect(onChange).toHaveBeenCalledWith('');
      expect(input).toHaveFocus();
    });
  });
});
