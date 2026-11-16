import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { PermissionCard } from './permission-card';

describe('PermissionCard', () => {
  describe('negative cases', () => {
    it('the permission card never blocks the screen', () => {
      render(
        <>
          <PermissionCard access="denied" onLearnMore={vi.fn()} />
          <button type="button">Scan</button>
        </>,
      );

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- no element of the page may be modal
      expect(document.querySelector('[aria-modal]')).toBeNull();
      // The card takes no focus of its own, and what is after it stays reachable.
      expect(document.body).toHaveFocus();
      const scan = screen.getByRole('button', { name: 'Scan' });
      scan.focus();

      expect(scan).toHaveFocus();
    });

    it('has no action unless one is given', () => {
      render(<PermissionCard access="denied" />);

      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the permission card differs for denied and unknown', () => {
      const { unmount } = render(<PermissionCard access="denied" />);
      const denied = screen.getByText('Fresher cannot see every folder');

      // eslint-disable-next-line testing-library/no-node-access -- the tone is drawn, not named
      expect(denied.closest('[data-tone]')).toHaveAttribute('data-tone', 'warning');
      unmount();

      render(<PermissionCard access="unknown" />);
      const unknown = screen.getByText('Full Disk Access is not confirmed');

      // eslint-disable-next-line testing-library/no-node-access -- the tone is drawn, not named
      expect(unknown.closest('[data-tone]')).toHaveAttribute('data-tone', 'info');
      expect(screen.queryByText('Fresher cannot see every folder')).not.toBeInTheDocument();
    });

    it('says the app still works with less', () => {
      render(<PermissionCard access="denied" />);

      expect(screen.getByText(/Everything else is scanned and cleaned as usual\./u)).toBeInTheDocument();
    });

    it('Learn more calls onLearnMore', () => {
      const onLearnMore = vi.fn();
      render(<PermissionCard access="unknown" onLearnMore={onLearnMore} />);

      fireEvent.click(screen.getByRole('button', { name: 'Learn more' }));

      expect(onLearnMore).toHaveBeenCalledOnce();
    });
  });
});
