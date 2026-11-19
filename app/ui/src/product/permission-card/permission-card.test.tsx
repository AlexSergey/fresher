import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { PermissionCard } from './permission-card';

const HOW = ['Open it.', 'Turn it on.'];
const noop = (): void => undefined;

describe('PermissionCard', () => {
  describe('negative cases', () => {
    it('the permission card never blocks the screen', () => {
      render(
        <>
          <PermissionCard access="denied" how={HOW} onOpenSettings={noop}>
            <p>Not seen.</p>
          </PermissionCard>
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

    it('offers Reopen Fresher only when it is given', () => {
      render(
        <PermissionCard access="denied" how={HOW} onOpenSettings={noop}>
          <p>Not seen.</p>
        </PermissionCard>,
      );

      expect(screen.queryByRole('button', { name: 'Reopen Fresher' })).not.toBeInTheDocument();
      expect(screen.queryByText('If you turned it on, Fresher sees it after it reopens.')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the permission card differs for denied and unknown', () => {
      const { unmount } = render(
        <PermissionCard access="denied" how={HOW} onOpenSettings={noop}>
          <p>Not seen.</p>
        </PermissionCard>,
      );
      const denied = screen.getByText('Fresher cannot see every folder');

      // eslint-disable-next-line testing-library/no-node-access -- the tone is drawn, not named
      expect(denied.closest('[data-tone]')).toHaveAttribute('data-tone', 'warning');
      unmount();

      render(
        <PermissionCard access="unknown" how={HOW} onOpenSettings={noop}>
          <p>Not seen.</p>
        </PermissionCard>,
      );
      const unknown = screen.getByText('Full Disk Access is not confirmed');

      // eslint-disable-next-line testing-library/no-node-access -- the tone is drawn, not named
      expect(unknown.closest('[data-tone]')).toHaveAttribute('data-tone', 'info');
      expect(screen.queryByText('Fresher cannot see every folder')).not.toBeInTheDocument();
    });

    it('shows its text and the numbered steps', () => {
      render(
        <PermissionCard access="denied" how={HOW} onOpenSettings={noop}>
          <p>Not seen.</p>
        </PermissionCard>,
      );

      expect(screen.getByText('Not seen.')).toBeInTheDocument();
      expect(screen.getAllByRole('listitem').map((item) => item.textContent)).toStrictEqual(HOW);
    });

    it('Open System Settings calls onOpenSettings', () => {
      const onOpenSettings = vi.fn();
      render(
        <PermissionCard access="unknown" how={HOW} onOpenSettings={onOpenSettings}>
          <p>Not seen.</p>
        </PermissionCard>,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Open System Settings' }));

      expect(onOpenSettings).toHaveBeenCalledOnce();
    });

    it('Reopen Fresher calls onReopen', () => {
      const onReopen = vi.fn();
      render(
        <PermissionCard access="denied" how={HOW} onOpenSettings={noop} onReopen={onReopen}>
          <p>Not seen.</p>
        </PermissionCard>,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Reopen Fresher' }));

      expect(onReopen).toHaveBeenCalledOnce();
    });
  });
});
