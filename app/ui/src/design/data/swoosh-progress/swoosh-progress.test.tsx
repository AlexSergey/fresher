import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { fakeFrames, setMotion } from '../../../test/animation';
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

// Where the star's transform puts its centre on the measured line: `x` is the length along it.
const starLength = (): number => {
  const transform = screen.getByTestId('swoosh-star').getAttribute('transform') ?? '';

  return Number(/translate\((-?[\d.]+)/u.exec(transform)?.[1]) + 528;
};

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('SwooshProgress', () => {
  describe('negative cases', () => {
    it('a value below 0 or above 1 is clamped', () => {
      const frames = fakeFrames();
      const { rerender } = render(<SwooshProgress label="Scanning" value={-3} />);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');

      rerender(<SwooshProgress label="Scanning" value={1.7} />);
      frames.advance(1000);
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

    it('the star stays on the head of the fill while the fill grows', () => {
      withMeasuring(MEASURING, () => {
        const frames = fakeFrames();
        const { rerender } = render(<SwooshProgress label="Scanning" value={0.2} />);
        rerender(<SwooshProgress label="Scanning" value={0.8} />);
        frames.advance(100);

        const offset = Number(fill().getAttribute('stroke-dashoffset'));
        expect(offset).toBeGreaterThan(200);
        expect(offset).toBeLessThan(800);
        expect(starLength()).toBeCloseTo(1000 - offset, 6);

        frames.advance(1000);
        expect(Number(fill().getAttribute('stroke-dashoffset'))).toBeCloseTo(200, 6);
        expect(starLength()).toBeCloseTo(800, 6);
      });
    });

    it('under reduced motion the fill and the star jump to the value together', async () => {
      await setMotion('reduced');
      withMeasuring(MEASURING, () => {
        const { rerender } = render(<SwooshProgress label="Scanning" value={0.2} />);
        rerender(<SwooshProgress label="Scanning" value={0.8} />);
        expect(Number(fill().getAttribute('stroke-dashoffset'))).toBeCloseTo(200, 6);
        expect(starLength()).toBeCloseTo(800, 6);
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
