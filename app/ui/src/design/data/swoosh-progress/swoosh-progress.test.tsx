import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { SwooshProgress } from './swoosh-progress';

const fill = (): SVGPathElement => {
  // eslint-disable-next-line testing-library/no-node-access -- as above
  const paths = screen.getByRole('progressbar').querySelectorAll<SVGPathElement>('path[stroke-dasharray]');

  return paths[paths.length - 1] as SVGPathElement;
};

type Methods = Record<string, unknown>;

// The test DOM measures paths as 0; each test sets the measuring it needs on the path prototype.
const withMeasuring = (methods: Methods, run: () => void): void => {
  const proto = SVGPathElement.prototype as unknown as Methods;
  const saved = { getPointAtLength: proto.getPointAtLength, getTotalLength: proto.getTotalLength };
  Object.assign(proto, methods);
  try {
    run();
  } finally {
    Object.assign(proto, saved);
  }
};

const MEASURING: Methods = {
  getPointAtLength: (length: number) => ({ x: length, y: 50 }),
  getTotalLength: () => 1000,
};

describe('SwooshProgress', () => {
  describe('negative cases', () => {
    it('a value below 0 or above 1 is clamped', () => {
      const { rerender } = render(<SwooshProgress label="Scanning" value={-3} />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');

      rerender(<SwooshProgress label="Scanning" value={1.7} />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
      expect(fill()).toHaveAttribute('stroke-dashoffset', '0');
    });

    it('at 0 the star is hidden', () => {
      withMeasuring(MEASURING, () => {
        render(<SwooshProgress label="Scanning" value={0} />);
        expect(screen.queryByTestId('swoosh-star')).not.toBeInTheDocument();
      });
    });

    it('without getPointAtLength the star stays hidden and nothing throws', () => {
      withMeasuring({ getPointAtLength: undefined }, () => {
        expect(() => render(<SwooshProgress label="Scanning" value={0.5} />)).not.toThrow();
        expect(screen.queryByTestId('swoosh-star')).not.toBeInTheDocument();
      });
    });

    it('a value that is not a finite number counts as 0', () => {
      const { rerender } = render(<SwooshProgress label="Scanning" value={Number.NaN} />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');

      rerender(<SwooshProgress label="Scanning" value={Number.POSITIVE_INFINITY} />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');
    });

    it('an unknown value has no number', () => {
      render(<SwooshProgress label="Scanning" value={null} />);
      expect(screen.getByRole('progressbar')).not.toHaveAttribute('aria-valuenow');
    });
  });

  describe('positive cases', () => {
    it('its value is the progress bar now', () => {
      render(<SwooshProgress label="Scanning" value={0.624} />);
      expect(screen.getByRole('progressbar', { name: 'Scanning' })).toHaveAttribute('aria-valuenow', '62');
    });

    it('the fill dash follows the value', () => {
      render(<SwooshProgress label="Scanning" value={0.25} />);
      expect(fill()).toHaveAttribute('stroke-dashoffset', '750');
    });

    it('the star sits at the head of the fill', () => {
      withMeasuring(MEASURING, () => {
        render(<SwooshProgress label="Scanning" value={0.5} />);
        expect(screen.getByTestId('swoosh-star')).toHaveAttribute('transform', 'translate(-28 -34)');
      });
    });

    it('two on a page have their own gradient ids', () => {
      render(
        <>
          <SwooshProgress label="One" value={0.5} />
          <SwooshProgress label="Two" value={0.5} />
        </>,
      );
      // eslint-disable-next-line testing-library/no-node-access -- the gradients have no role
      const ids = [...document.querySelectorAll('linearGradient')].map((node) => node.id);
      expect(new Set(ids).size).toBe(2);
    });
  });
});
