import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { TextField } from './text-field';

const LABEL = 'Older than (days)';

describe('TextField', () => {
  describe('negative cases', () => {
    it('an invalid field announces its error', () => {
      render(<TextField error="Enter a number of days." label={LABEL} onChange={vi.fn()} value="soon" />);

      const input = screen.getByRole('textbox', { name: LABEL });

      expect(input).toBeInvalid();
      expect(input).toHaveAccessibleDescription('Enter a number of days.');
    });

    it('a valid field is not marked invalid and has no description', () => {
      render(<TextField label={LABEL} onChange={vi.fn()} value="30" />);

      const input = screen.getByRole('textbox', { name: LABEL });

      expect(input).not.toHaveAttribute('aria-invalid');
      expect(input).not.toHaveAttribute('aria-describedby');
    });

    it('a disabled field cannot be edited', () => {
      render(<TextField disabled label={LABEL} onChange={vi.fn()} value="30" />);

      expect(screen.getByRole('textbox', { name: LABEL })).toBeDisabled();
    });
  });

  describe('positive cases', () => {
    it('the label names the input', () => {
      render(<TextField label={LABEL} onChange={vi.fn()} value="" />);

      expect(screen.getByRole('textbox', { name: LABEL })).toHaveAttribute('type', 'text');
    });

    it('keeps its name when the label is visually hidden', () => {
      render(<TextField hideLabel label={LABEL} onChange={vi.fn()} type="number" value="30" />);

      expect(screen.getByRole('spinbutton', { name: LABEL })).toBeInTheDocument();
    });

    it('typing calls onChange', () => {
      const onChange = vi.fn();
      render(<TextField label={LABEL} onChange={onChange} value="" />);

      fireEvent.change(screen.getByRole('textbox', { name: LABEL }), { target: { value: '45' } });

      expect(onChange).toHaveBeenCalledWith('45');
    });

    it('links its description, after the error', () => {
      render(
        <TextField
          description="Only caches untouched this long are listed."
          error="Enter a number of days."
          label={LABEL}
          onChange={vi.fn()}
          value="soon"
        />,
      );

      expect(screen.getByRole('textbox', { name: LABEL })).toHaveAccessibleDescription(
        'Enter a number of days. Only caches untouched this long are listed.',
      );
    });
  });
});
