import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ErrorState } from './error-state';

const CORE_TEXT = 'The folder ~/Library/Caches could not be read: permission denied.';

describe('ErrorState', () => {
  describe('negative cases', () => {
    it('does not retry until asked', () => {
      const onClick = vi.fn();
      render(<ErrorState action={{ label: 'Try again', onClick }} text={CORE_TEXT} title="The list failed" />);

      expect(onClick).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('the error state shows the given text and calls Try again', () => {
      const onClick = vi.fn();
      render(<ErrorState action={{ label: 'Try again', onClick }} text={CORE_TEXT} title="The list failed" />);

      fireEvent.click(screen.getByRole('button', { name: 'Try again' }));

      expect(screen.getByText(CORE_TEXT)).toBeInTheDocument();
      expect(screen.getByText('The list failed')).toBeInTheDocument();
      expect(onClick).toHaveBeenCalledOnce();
    });
  });
});
