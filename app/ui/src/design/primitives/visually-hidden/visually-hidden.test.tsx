import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { VisuallyHidden } from './visually-hidden';

describe('VisuallyHidden', () => {
  describe('negative cases', () => {
    it('is not hidden from assistive technology', () => {
      render(<VisuallyHidden>42.7 GB</VisuallyHidden>);

      expect(screen.getByText('42.7 GB')).not.toHaveAttribute('aria-hidden');
    });
  });

  describe('positive cases', () => {
    it('visually hidden text is in the accessibility tree', () => {
      render(
        <button type="button">
          <span aria-hidden="true">42.7 GB+</span>
          <VisuallyHidden>At least 42.7 GB</VisuallyHidden>
        </button>,
      );

      expect(screen.getByRole('button', { name: 'At least 42.7 GB' })).toBeInTheDocument();
    });

    it('uses the visually hidden utility', () => {
      render(<VisuallyHidden>42.7 GB</VisuallyHidden>);

      expect(screen.getByText('42.7 GB')).toHaveClass('u-visually-hidden');
    });
  });
});
