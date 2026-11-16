import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { LayoutGrid } from '../../icons/icons';
import { Badge } from '../../primitives/badge/badge';
import { NavItem } from './nav-item';

describe('NavItem', () => {
  describe('negative cases', () => {
    it('an inactive nav item has no aria-current', () => {
      render(<NavItem icon={LayoutGrid} label="Overview" onClick={vi.fn()} />);

      expect(screen.getByRole('button', { name: 'Overview' })).not.toHaveAttribute('aria-current');
    });

    it('shows nothing after the label unless given', () => {
      render(<NavItem icon={LayoutGrid} label="Overview" onClick={vi.fn()} />);

      // eslint-disable-next-line testing-library/no-node-access -- the icon and the label are the button's only parts
      expect(screen.getByRole('button', { name: 'Overview' }).children).toHaveLength(2);
    });
  });

  describe('positive cases', () => {
    it('the active nav item has aria-current page', () => {
      render(<NavItem active icon={LayoutGrid} label="Overview" onClick={vi.fn()} />);

      expect(screen.getByRole('button', { name: 'Overview' })).toHaveAttribute('aria-current', 'page');
    });

    it('goes to its screen on click', () => {
      const onClick = vi.fn();
      render(<NavItem icon={LayoutGrid} label="Overview" onClick={onClick} />);

      fireEvent.click(screen.getByRole('button', { name: 'Overview' }));

      expect(onClick).toHaveBeenCalledOnce();
    });

    it('says its count with its name', () => {
      render(
        <NavItem icon={LayoutGrid} label="Review" onClick={vi.fn()} trailing={<Badge count={12} label="12 items" />} />,
      );

      expect(screen.getByRole('button', { name: 'Review 12 items' })).toBeInTheDocument();
    });
  });
});
