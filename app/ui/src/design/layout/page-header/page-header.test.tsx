import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { PageHeader } from './page-header';

describe('PageHeader', () => {
  describe('negative cases', () => {
    it('shows no subtitle and no actions unless given', () => {
      render(<PageHeader title="Overview" />);

      expect(screen.getByRole('banner')).toHaveTextContent(/^Overview$/u);
      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('names the page with its title', () => {
      render(<PageHeader title="Overview" />);

      expect(screen.getByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
    });

    it('shows its subtitle and actions', () => {
      render(
        <PageHeader
          actions={<button type="button">Scan again</button>}
          subtitle="Last scanned 5 minutes ago"
          title="Overview"
        />,
      );

      expect(screen.getByText('Last scanned 5 minutes ago')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Scan again' })).toBeInTheDocument();
    });
  });
});
