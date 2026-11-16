import { act, render, screen } from '@testing-library/react';
import { type JSX, useRef } from 'react';
import { describe, expect, it } from 'vitest';

import { useKeepFocus } from './keep-focus';

type Props = { readonly active?: boolean; readonly scanning: boolean };

// A screen whose Scan button turns into Stop, as the overview's does, and a control outside it.
const Screen = ({ active = true, scanning }: Props): JSX.Element => {
  const ref = useRef<HTMLDivElement>(null);

  useKeepFocus(ref, active);

  return (
    <>
      <div ref={ref}>
        <h1>Overview</h1>
        {scanning ? (
          <button key="stop" type="button">
            Stop
          </button>
        ) : (
          <button key="scan" type="button">
            Scan
          </button>
        )}
      </div>
      <button type="button">Settings</button>
    </>
  );
};

// Re-renders and lets the mutation observer run.
const settle = async (): Promise<void> => {
  await act(async () => {
    await Promise.resolve();
  });
};

describe('useKeepFocus', () => {
  describe('negative cases', () => {
    it('a focus moved out of the screen on purpose stays where it went', async () => {
      const { rerender } = render(<Screen scanning={false} />);
      screen.getByRole('button', { name: 'Scan' }).focus();
      screen.getByRole('button', { name: 'Settings' }).focus();

      rerender(<Screen scanning />);
      await settle();

      expect(screen.getByRole('button', { name: 'Settings' })).toHaveFocus();
    });

    it('a screen that is not the current one does not take the focus', async () => {
      const { rerender } = render(<Screen active={false} scanning={false} />);
      screen.getByRole('button', { name: 'Scan' }).focus();

      rerender(<Screen active={false} scanning />);
      await settle();

      expect(screen.getByRole('heading', { name: 'Overview' })).not.toHaveFocus();
    });
  });

  describe('positive cases', () => {
    it('the focus of a control that leaves the screen goes to its heading', async () => {
      const { rerender } = render(<Screen scanning={false} />);
      screen.getByRole('button', { name: 'Scan' }).focus();

      rerender(<Screen scanning />);
      await settle();

      expect(screen.getByRole('heading', { name: 'Overview' })).toHaveFocus();
    });
  });
});
