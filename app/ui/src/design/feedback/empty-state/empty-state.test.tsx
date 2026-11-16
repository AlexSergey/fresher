import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ListChecks } from '../../icons/icons';
import { EmptyState } from './empty-state';

describe('EmptyState', () => {
  describe('negative cases', () => {
    it('has no action unless given', () => {
      render(<EmptyState icon={ListChecks} text="Nothing here takes space." title="Nothing to clean" />);

      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });

    it('its icon is decoration', () => {
      render(<EmptyState icon={ListChecks} text="Nothing here takes space." title="Nothing to clean" />);

      expect(screen.queryByRole('img')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows its title, its sentence and runs its action', () => {
      const onClick = vi.fn();
      render(
        <EmptyState
          action={{ label: 'Scan again', onClick }}
          icon={ListChecks}
          text="Nothing here takes space."
          title="Nothing to clean"
        />,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Scan again' }));

      expect(screen.getByText('Nothing to clean')).toBeInTheDocument();
      expect(screen.getByText('Nothing here takes space.')).toBeInTheDocument();
      expect(onClick).toHaveBeenCalledOnce();
    });
  });
});
