import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { fakeReducedMotion, setMotion } from '../../test/animation';
import { RecoveryHelp } from './recovery-help';

beforeEach(async () => {
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

const BUTTON = 'Removed something you need?';

describe('RecoveryHelp', () => {
  describe('negative cases', () => {
    it('the dialog is closed until the button is pressed', () => {
      render(<RecoveryHelp />);

      expect(screen.getByRole('button', { name: BUTTON })).toBeInTheDocument();
      expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('the address is not a link', () => {
      render(<RecoveryHelp defaultOpen />);

      expect(screen.queryByRole('link')).toBeNull();
      expect(screen.getByText(/^https:\/\/github\.com\/.+template=wrong-finding\.yml$/u).tagName).toBe('CODE');
    });

    it('the help offers no way to empty the Trash', () => {
      render(<RecoveryHelp defaultOpen />);

      expect(screen.queryByRole('button', { name: /empty/i })).toBeNull();
      expect(screen.queryByRole('link', { name: /empty/i })).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('the four parts are shown in order', () => {
      render(<RecoveryHelp />);

      fireEvent.click(screen.getByRole('button', { name: BUTTON }));

      expect(screen.getByRole('dialog', { name: BUTTON })).toBeInTheDocument();
      expect(screen.getAllByRole('heading', { level: 3 }).map((heading) => heading.textContent)).toStrictEqual([
        'Put it back',
        'Find it in the Trash',
        'If you deleted it permanently',
        'Tell us what went wrong',
      ]);
    });

    it('Close has the first focus when the help opens', () => {
      render(<RecoveryHelp />);

      fireEvent.click(screen.getByRole('button', { name: BUTTON }));

      expect(screen.getByRole('button', { name: 'Close' })).toHaveFocus();
    });

    it('Close returns the focus to the button', async () => {
      render(<RecoveryHelp />);
      const button = screen.getByRole('button', { name: BUTTON });
      button.focus();
      fireEvent.click(button);

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(await screen.findByRole('button', { name: BUTTON })).toHaveFocus();
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });
});
