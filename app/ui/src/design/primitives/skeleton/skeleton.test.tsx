import { render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { Skeleton } from './skeleton';

const css = readFileSync(path.join(import.meta.dirname, 'skeleton.module.css'), 'utf8');

describe('Skeleton', () => {
  describe('negative cases', () => {
    it('the skeleton is hidden from assistive technology', () => {
      const { container } = render(<Skeleton shape="line" />);

      // eslint-disable-next-line testing-library/no-node-access -- a hidden shape has no role to query by
      expect(container.firstElementChild).toHaveAttribute('aria-hidden', 'true');
    });

    it('stands still unless loops run', () => {
      expect(css).toMatch(/animation-play-state: var\(--loop-state\);/u);
    });
  });

  describe('positive cases', () => {
    it.each(['line', 'block', 'circle'] as const)('draws a %s', (shape) => {
      const { container } = render(<Skeleton shape={shape} />);

      // eslint-disable-next-line testing-library/no-node-access -- a hidden shape has no role to query by
      expect(container.firstElementChild).toHaveAttribute('data-shape', shape);
    });

    it('shimmers by translateX on a 1,400 ms linear loop', () => {
      expect(css).toMatch(/--skeleton-loop: 1400ms;/u);
      expect(css).toMatch(/animation: skeleton-shimmer var\(--skeleton-loop\) var\(--ease-linear\) infinite;/u);
    });
  });
});
