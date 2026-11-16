import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { HOVER_DELAY } from '../../primitives/tooltip/tooltip';
import { SizeText } from './size-text';

afterEach(() => {
  vi.useRealTimers();
});

describe('SizeText', () => {
  describe('negative cases', () => {
    it('an unknown size never shows 0', () => {
      const { container } = render(<SizeText accessibleText="Size not known" completeness="unknown" />);

      expect(container).toHaveTextContent('—');
      expect(container.textContent).not.toMatch(/0/u);
      expect(screen.getByText('—')).toHaveAttribute('aria-hidden', 'true');
      expect(screen.getByText('Size not known')).toBeInTheDocument();
    });

    it('a partial size says "at least" in its accessible text', () => {
      render(<SizeText accessibleText="at least 42 GB" completeness="partial" text="42 GB" />);

      expect(screen.getByText('at least 42 GB')).toHaveClass('u-visually-hidden');
      expect(screen.getByText('42 GB')).toHaveAttribute('aria-hidden', 'true');
      expect(screen.getByText('+')).toHaveAttribute('aria-hidden', 'true');
    });

    it('a complete size shows no plus', () => {
      render(<SizeText accessibleText="42 GB" completeness="complete" text="42 GB" />);

      expect(screen.queryByText('+')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the given text in tabular figures, in the body size by default', () => {
      render(<SizeText accessibleText="42 gigabytes" completeness="complete" text="42 GB" />);

      // eslint-disable-next-line testing-library/no-node-access -- the wrapper carries the figures and the size
      const wrapper = screen.getByText('42 GB').parentElement;

      expect(wrapper).toHaveAttribute('data-numeric');
      expect(wrapper).toHaveAttribute('data-size', 'body');
      expect(screen.getByText('42 gigabytes')).toBeInTheDocument();
    });

    it('takes each size step', () => {
      for (const size of ['body', 'callout', 'title-3', 'display', 'hero'] as const) {
        const { unmount } = render(<SizeText accessibleText="1 GB" completeness="complete" size={size} text="1 GB" />);

        // eslint-disable-next-line testing-library/no-node-access -- the wrapper carries the size
        expect(screen.getByText('1 GB', { selector: '[aria-hidden]' }).parentElement).toHaveAttribute(
          'data-size',
          size,
        );
        unmount();
      }
    });

    it('a partial size explains its plus in a tooltip', () => {
      vi.useFakeTimers();
      render(
        <SizeText
          accessibleText="at least 42 GB"
          completeness="partial"
          hint="Some folders could not be read"
          text="42 GB"
        />,
      );

      // eslint-disable-next-line testing-library/no-node-access -- the pointer enters the tooltip's anchor
      fireEvent.pointerEnter(screen.getByText('+').parentElement as HTMLElement);
      act(() => {
        vi.advanceTimersByTime(HOVER_DELAY);
      });

      expect(screen.getByRole('tooltip')).toHaveTextContent('Some folders could not be read');
    });
  });
});
