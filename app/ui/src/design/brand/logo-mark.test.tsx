/* eslint-disable testing-library/no-container, testing-library/no-node-access -- the mark is an aria-hidden drawing: it has no role */
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { LogoMark } from './logo-mark';

describe('LogoMark', () => {
  describe('negative cases', () => {
    it('is hidden from assistive technology', () => {
      const { container } = render(<LogoMark />);

      expect(container.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    });

    it('gives two marks their own gradient and clip ids', () => {
      const { container } = render(
        <>
          <LogoMark />
          <LogoMark />
        </>,
      );
      const ids = [...container.querySelectorAll('[id]')].map((element) => element.id);

      expect(new Set(ids).size).toBe(ids.length);
      expect(ids).toHaveLength(6);
    });
  });

  describe('positive cases', () => {
    it('draws the leaf, the band, the rim and the star', () => {
      const { container } = render(<LogoMark />);
      const parts = [...container.querySelectorAll('[data-part]')].map((part) => part.getAttribute('data-part'));

      expect(parts).toStrictEqual(['leaf', 'band', 'rim', 'star']);
    });

    it('is 22 px by default and takes a size', () => {
      const { container, rerender } = render(<LogoMark />);

      expect(container.querySelector('svg')).toHaveAttribute('width', '22');

      rerender(<LogoMark size={40} />);

      expect(container.querySelector('svg')).toHaveAttribute('height', '40');
    });
  });
});
