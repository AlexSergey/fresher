import { render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { fakeReducedMotion, setMotion } from '../../../test/animation';
import { AmbientLight } from './ambient-light';

const css = readFileSync(path.join(import.meta.dirname, 'ambient-light.module.css'), 'utf8');

// The light is decoration, hidden from assistive technology: it is found by its attribute.
const light = (): HTMLElement => {
  // eslint-disable-next-line testing-library/no-node-access -- the light is hidden decoration with no role to query by
  const element = document.querySelector<HTMLElement>('[data-ambient-light]');

  if (element === null) {
    throw new Error('No ambient light is rendered');
  }

  return element;
};

// The tones shown now; the glows are hidden decoration with no role to query by.
const shownTones = (): readonly (string | undefined)[] =>
  // eslint-disable-next-line testing-library/no-node-access -- see above
  [...light().querySelectorAll<HTMLElement>('[data-shown]')].map((glow) => glow.dataset.tone);

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  await setMotion(null);
});

describe('AmbientLight', () => {
  describe('negative cases', () => {
    it('the ambient light does not breathe unless asked', () => {
      render(<AmbientLight />);

      expect(light()).not.toHaveAttribute('data-breathing');
    });

    it('the ambient light is still under reduced motion', () => {
      fakeReducedMotion(true);
      render(<AmbientLight breathing />);

      expect(light()).not.toHaveAttribute('data-breathing');
    });

    it('is still under motion off', async () => {
      await setMotion('off');
      render(<AmbientLight breathing />);

      expect(light()).not.toHaveAttribute('data-breathing');
    });

    it('is off under increased contrast', () => {
      expect(css).toMatch(/@media \(prefers-contrast: more\) \{\s*\.light \{\s*display: none;/u);
    });

    it('is hidden from assistive technology', () => {
      render(<AmbientLight />);

      expect(light()).toHaveAttribute('aria-hidden', 'true');
    });
  });

  describe('positive cases', () => {
    it('glows in Iris by default', () => {
      render(<AmbientLight />);

      expect(shownTones()).toStrictEqual(['iris']);
    });

    it('breathes while asked, between 0.7 and 1, over --duration-breath', () => {
      render(<AmbientLight breathing />);

      expect(light()).toHaveAttribute('data-breathing');
      expect(css).toMatch(
        /animation: ambient-breathe var\(--duration-breath\) var\(--ease-standard\) infinite alternate;/u,
      );
      expect(css).toMatch(/@keyframes ambient-breathe \{\s*to \{\s*opacity: 0\.7;/u);
    });

    it('the tone change crossfades', () => {
      const { rerender } = render(<AmbientLight />);

      rerender(<AmbientLight tone="safe" />);

      // eslint-disable-next-line testing-library/no-node-access -- the glows are hidden decoration
      expect(light().querySelectorAll('[data-tone]')).toHaveLength(2);
      expect(shownTones()).toStrictEqual(['safe']);
      expect(css).toMatch(/\.glow \{[^}]*transition: opacity var\(--duration-deliberate\) var\(--ease-standard\);/u);
    });
  });
});
