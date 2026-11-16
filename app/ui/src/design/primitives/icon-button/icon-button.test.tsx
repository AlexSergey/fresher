import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { RotateCw, X } from '../../icons/icons';
import { IconButton } from './icon-button';

describe('IconButton', () => {
  describe('negative cases', () => {
    it('an icon button without a label is a type error', () => {
      // @ts-expect-error -- the label is its accessible name.
      render(<IconButton icon={X} />);

      expect(screen.getByRole('button')).not.toHaveAccessibleName();
    });

    it('does not fire when disabled', () => {
      const onClick = vi.fn();
      render(<IconButton disabled icon={X} label="Close" onClick={onClick} />);

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(onClick).not.toHaveBeenCalled();
    });

    it('its tooltip does not describe it a second time', () => {
      render(<IconButton icon={X} label="Close" />);

      const button = screen.getByRole('button', { name: 'Close' });
      act(() => {
        button.focus();
      });

      expect(screen.getByRole('tooltip')).toHaveTextContent('Close');
      expect(button).not.toHaveAccessibleDescription();
    });
  });

  describe('positive cases', () => {
    it('has its label as its accessible name', () => {
      render(<IconButton icon={RotateCw} label="Scan again" size="large" variant="secondary" />);

      const button = screen.getByRole('button', { name: 'Scan again' });

      expect(button).toHaveAttribute('type', 'button');
      expect(button).toHaveAttribute('data-variant', 'secondary');
      expect(button).toHaveAttribute('data-size', 'large');
    });

    it('fires on click', () => {
      const onClick = vi.fn();
      render(<IconButton icon={X} label="Close" onClick={onClick} />);

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(onClick).toHaveBeenCalledOnce();
    });
  });
});
