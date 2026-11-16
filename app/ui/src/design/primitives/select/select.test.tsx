import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Select } from './select';

type Period = 'month' | 'quarter' | 'week';

const OPTIONS = [
  { label: 'A week', value: 'week' },
  { label: 'A month', value: 'month' },
  { label: 'A quarter', value: 'quarter' },
] as const;

const showSelect = (onChange: (value: Period) => void, disabled = false): HTMLElement => {
  render(<Select disabled={disabled} label="Older than" onChange={onChange} options={OPTIONS} value="month" />);

  return screen.getByRole('combobox', { name: 'Older than' });
};

describe('Select', () => {
  describe('negative cases', () => {
    it('a disabled select does not change', () => {
      const onChange = vi.fn();
      const select = showSelect(onChange, true);

      fireEvent.change(select, { target: { value: 'week' } });

      expect(select).toBeDisabled();
      expect(onChange).not.toHaveBeenCalled();
    });

    it('ignores a value that is not one of its options', () => {
      const onChange = vi.fn();
      const select = showSelect(onChange);

      fireEvent.change(select, { target: { value: 'year' } });

      expect(onChange).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('the label names the select and shows the value', () => {
      const select = showSelect(vi.fn());

      expect(select).toHaveValue('month');
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });

    it('keeps its name when the label is visually hidden', () => {
      render(<Select hideLabel label="Older than" onChange={vi.fn()} options={OPTIONS} value="week" />);

      expect(screen.getByRole('combobox', { name: 'Older than' })).toHaveValue('week');
    });

    it('the select calls onChange with the typed value', () => {
      const onChange = vi.fn<(value: Period) => void>();
      const select = showSelect(onChange);

      fireEvent.change(select, { target: { value: 'quarter' } });

      expect(onChange).toHaveBeenCalledWith('quarter');
    });
  });
});
