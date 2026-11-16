import { render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { fakeReducedMotion, recordAnimations, setMotion } from '../../../test/animation';
import { DURATION } from '../../tokens/motion';
import { ScanSweep } from './scan-sweep';

const css = readFileSync(path.join(import.meta.dirname, 'scan-sweep.module.css'), 'utf8');
const motionCss = readFileSync(path.join(import.meta.dirname, '../../tokens/motion.css'), 'utf8');

// The sweep is decoration: it has no role, only its layer in the DOM.
// eslint-disable-next-line testing-library/no-node-access -- the sweep has no role, by design
const sweep = (): Element | null => document.querySelector('[data-sweep]');

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  await setMotion(null);
});

describe('ScanSweep', () => {
  describe('negative cases', () => {
    it('the sweep is absent when not scanning', () => {
      render(<ScanSweep active={false} />);

      expect(sweep()).toBeNull();
    });

    it('the sweep does not rotate under reduced motion', () => {
      expect(css).toMatch(
        /animation: scan-sweep-turn var\(--duration-sweep\)[^;]*;\s*animation-play-state: var\(--loop-state\);/u,
      );
      // Reduced motion stops every loop at its first frame: a static highlight.
      expect(motionCss).toMatch(/:root\[data-motion='reduced'\] \{[^}]*--loop-state: paused;/u);
    });

    it('is hidden from assistive technology', () => {
      render(<ScanSweep active />);

      expect(sweep()).toHaveAttribute('aria-hidden', 'true');
    });

    it('motion off removes it at once', async () => {
      await setMotion('off');
      const animations = recordAnimations();
      const { rerender } = render(<ScanSweep active />);

      rerender(<ScanSweep active={false} />);

      expect(sweep()).toBeNull();
      expect(animations).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('shows while active, masked to the given band', () => {
      render(<ScanSweep active band={16} />);

      expect(sweep()).toHaveStyle({ '--sweep-band': '16px' });
    });

    it('fades in when it turns on', () => {
      expect(css).toMatch(/animation: scan-sweep-in var\(--duration-base\)/u);
    });

    it('fades out over the move duration when it turns off', async () => {
      const animations = recordAnimations();
      const { rerender } = render(<ScanSweep active />);

      rerender(<ScanSweep active={false} />);

      expect(sweep()).not.toBeNull();
      expect(animations).toHaveLength(1);
      expect(animations[0]?.options.duration).toBe(DURATION.move);
      expect(animations[0]?.keyframes.at(-1)).toStrictEqual({ opacity: 0 });

      await animations[0]?.finish();

      expect(sweep()).toBeNull();
    });
  });
});
