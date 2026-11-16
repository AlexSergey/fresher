import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Badge } from './badge';

describe('Badge', () => {
  describe('negative cases', () => {
    it('the badge count shows 999+ but says the full number', () => {
      render(
        <h2>
          Caches <Badge count={1204} label="1,204 items" />
        </h2>,
      );

      expect(screen.getByText('999+')).toHaveAttribute('aria-hidden', 'true');
      expect(screen.getByRole('heading', { name: 'Caches 1,204 items' })).toBeInTheDocument();
    });

    it('does not cap a count of 999', () => {
      render(<Badge count={999} label="999 items" />);

      expect(screen.getByText('999')).toBeInTheDocument();
      expect(screen.queryByText('999+')).not.toBeInTheDocument();
    });

    it('a word badge is neutral unless told otherwise', () => {
      render(<Badge>New</Badge>);

      expect(screen.getByText('New')).toHaveAttribute('data-tone', 'neutral');
    });
  });

  describe('positive cases', () => {
    it('shows a word', () => {
      render(<Badge tone="accent">New</Badge>);

      expect(screen.getByText('New')).toHaveAttribute('data-tone', 'accent');
    });

    it('shows a count in tabular figures', () => {
      render(<Badge count={12} label="12 items" />);

      // eslint-disable-next-line testing-library/no-node-access -- the badge is the count's wrapper
      expect(screen.getByText('12').parentElement).toHaveAttribute('data-numeric');
      expect(screen.getByText('12 items')).toBeInTheDocument();
    });
  });
});
