import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { placeTooltip } from './place';
import { HOVER_DELAY, Tooltip } from './tooltip';

const VIEWPORT = { height: 720, width: 1100 };
const TIP = { height: 24, width: 120 };

const showTip = (props: Partial<Parameters<typeof Tooltip>[0]> = {}): HTMLElement => {
  render(
    <Tooltip content="Moves the items to the Trash" {...props}>
      {(describedBy) => (
        <button aria-describedby={describedBy} type="button">
          Clean
        </button>
      )}
    </Tooltip>,
  );

  return screen.getByRole('button', { name: 'Clean' });
};

// The pointer enters and leaves the child's wrapper: `pointerenter` does not bubble, a browser fires it on each.
// eslint-disable-next-line testing-library/no-node-access -- the wrapper has no role to query by
const anchorOf = (child: HTMLElement): HTMLElement => child.parentElement as HTMLElement;

const wait = (ms: number): void => {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('Tooltip', () => {
  describe('negative cases', () => {
    it('the tooltip does not show before 500 ms of hover', () => {
      const button = showTip();

      fireEvent.pointerEnter(anchorOf(button));
      wait(HOVER_DELAY - 1);

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });

    it('the tooltip hides on Escape and keeps the focus on its child', () => {
      const button = showTip();

      act(() => {
        button.focus();
      });
      expect(screen.getByRole('tooltip')).toBeInTheDocument();

      fireEvent.keyDown(button, { key: 'Escape' });

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
      expect(button).toHaveFocus();
    });

    it('ignores other keys', () => {
      const button = showTip({ defaultOpen: true });

      fireEvent.keyDown(button, { key: 'Enter' });

      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    it('hides when the pointer leaves before the delay', () => {
      const button = showTip();

      fireEvent.pointerEnter(anchorOf(button));
      fireEvent.pointerLeave(anchorOf(button));
      wait(HOVER_DELAY);

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });

    it('does not show at once when a press focuses the child', () => {
      const button = showTip();

      fireEvent.pointerDown(button);
      act(() => {
        button.focus();
      });

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });

    it('hides on blur', () => {
      const button = showTip();

      act(() => {
        button.focus();
      });
      act(() => {
        button.blur();
      });

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });

    it('a tooltip that repeats the name does not describe its child', () => {
      const button = showTip({ defaultOpen: true, repeatsName: true });

      expect(screen.getByRole('tooltip')).toBeInTheDocument();
      expect(button).not.toHaveAttribute('aria-describedby');
    });

    it('stays on the preferred side when neither side fits', () => {
      const anchor = { bottom: 712, left: 100, top: 4, width: 40 };

      expect(placeTooltip({ anchor, preferred: 'top', tip: TIP, viewport: VIEWPORT }).side).toBe('top');
    });
  });

  describe('positive cases', () => {
    it('shows after 500 ms of hover, linked as the description', () => {
      const button = showTip();

      fireEvent.pointerEnter(anchorOf(button));
      wait(HOVER_DELAY);

      expect(screen.getByRole('tooltip')).toHaveTextContent('Moves the items to the Trash');
      expect(button).toHaveAccessibleDescription('Moves the items to the Trash');
    });

    it('the tooltip shows at once on keyboard focus', () => {
      const button = showTip();

      act(() => {
        button.focus();
      });

      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    it('is rendered on <body>, outside its child', () => {
      showTip({ defaultOpen: true });

      // eslint-disable-next-line testing-library/no-node-access -- the portal's place is what is checked
      expect(screen.getByRole('tooltip').parentElement).toBe(document.body);
    });

    it('is placed above the child, centred', () => {
      const anchor = { bottom: 140, left: 100, top: 112, width: 40 };

      expect(placeTooltip({ anchor, preferred: 'top', tip: TIP, viewport: VIEWPORT })).toStrictEqual({
        left: 60,
        side: 'top',
        top: 82,
      });
    });

    it('the tooltip flips below near the top edge', () => {
      const anchor = { bottom: 30, left: 100, top: 4, width: 40 };

      expect(placeTooltip({ anchor, preferred: 'top', tip: TIP, viewport: VIEWPORT })).toStrictEqual({
        left: 60,
        side: 'bottom',
        top: 36,
      });
    });

    it('flips above near the bottom edge and stays inside the width', () => {
      const anchor = { bottom: 716, left: 1080, top: 690, width: 20 };

      expect(placeTooltip({ anchor, preferred: 'bottom', tip: TIP, viewport: VIEWPORT })).toStrictEqual({
        left: 972,
        side: 'top',
        top: 660,
      });
    });

    it('places itself from the child it measured', () => {
      vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue(
        DOMRect.fromRect({ height: 26, width: 40, x: 100, y: 4 }),
      );
      showTip({ defaultOpen: true });

      const tip = screen.getByRole('tooltip');

      expect(tip).toHaveAttribute('data-side', 'bottom');
      expect(tip).toHaveAttribute('data-placed');
    });
  });
});
