import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Divider } from './divider';

describe('Divider', () => {
  describe('negative cases', () => {
    it('is not vertical unless asked', () => {
      render(<Divider />);

      expect(screen.getByRole('separator')).toHaveAttribute('aria-orientation', 'horizontal');
    });
  });

  describe('positive cases', () => {
    it('is a separator for assistive technology', () => {
      render(<Divider />);

      expect(screen.getByRole('separator')).toBeInTheDocument();
    });

    it('can stand vertically', () => {
      render(<Divider orientation="vertical" />);

      expect(screen.getByRole('separator')).toHaveAttribute('aria-orientation', 'vertical');
    });
  });
});
