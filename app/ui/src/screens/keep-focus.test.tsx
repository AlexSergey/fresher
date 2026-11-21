import { act, render, screen } from '@testing-library/react';
import { type JSX, useRef } from 'react';
import { describe, expect, it } from 'vitest';

import { focusCard, useKeepFocus } from './keep-focus';

type Props = { readonly active?: boolean; readonly scanning: boolean };

// A screen whose Scan button turns into Stop, as the overview's does, and a control outside it.
const Screen = ({ active = true, scanning }: Props): JSX.Element => {
  const ref = useRef<HTMLDivElement>(null);

  useKeepFocus(ref, active);

  return (
    <>
      <div ref={ref}>
        <h1>Scan</h1>
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

      expect(screen.getByRole('heading', { name: 'Scan' })).not.toHaveFocus();
    });
  });

  describe('positive cases', () => {
    it('the focus of a control that leaves the screen goes to its heading', async () => {
      const { rerender } = render(<Screen scanning={false} />);
      screen.getByRole('button', { name: 'Scan' }).focus();

      rerender(<Screen scanning />);
      await settle();

      expect(screen.getByRole('heading', { name: 'Scan' })).toHaveFocus();
    });
  });
});

describe('focusCard', () => {
  const Cards = (): JSX.Element => (
    <div data-testid="screen">
      <h1>Scan</h1>
      <ul>
        <li data-category="a.one">
          <input aria-label="Select one" type="checkbox" />
          <button type="button">Details of one</button>
          <button type="button">Options of one</button>
        </li>
      </ul>
    </div>
  );

  describe('negative cases', () => {
    it('says no and moves nothing when the card is not on the screen', () => {
      render(<Cards />);

      expect(focusCard(screen.getByTestId('screen'), 'a.gone')).toBe(false);
      expect(document.body).toHaveFocus();
    });

    it('says no when there is no screen', () => {
      expect(focusCard(null, 'a.one')).toBe(false);
    });
  });

  describe('positive cases', () => {
    it("focuses the card's Details button", () => {
      render(<Cards />);

      expect(focusCard(screen.getByTestId('screen'), 'a.one')).toBe(true);
      expect(screen.getByRole('button', { name: 'Details of one' })).toHaveFocus();
    });
  });
});
