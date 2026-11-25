/* eslint-disable testing-library/no-container, testing-library/no-node-access -- the line and the icon are decorative, without a role */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { SectionSeparator } from './section-separator';

describe('SectionSeparator', () => {
  describe('negative cases', () => {
    it('hides the line from assistive technology', () => {
      const { container } = render(<SectionSeparator label="What you can remove" />);

      expect(container.querySelector('[aria-hidden="true"]')).not.toBeNull();
      expect(screen.getByText('What you can remove')).not.toHaveAttribute('aria-hidden');
    });
  });

  describe('positive cases', () => {
    it('shows its label', () => {
      render(<SectionSeparator label="What you can remove" />);

      expect(screen.getByText('What you can remove')).toBeVisible();
    });
  });
});
