import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import type { Segment } from '../segment';

import { fakeReducedMotion, setMotion } from '../../../test/animation';
import { StorageRing, type StorageRingProps } from './storage-ring';

const css = readFileSync(path.join(import.meta.dirname, 'storage-ring.module.css'), 'utf8');

const SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', level: 'safe', size: '42 GB', value: 42 },
  { key: 'docker', label: 'Docker', level: 'medium', size: '18 GB', value: 18 },
  { key: 'logs', label: 'Logs', level: 'low', size: '3 GB', value: 3 },
];

const ring = (props: Partial<StorageRingProps> = {}): StorageRingProps => ({
  label: 'Space found',
  segments: SEGMENTS,
  state: 'done',
  total: 100,
  ...props,
});

// The decorative layers have no role; they are found by their markers.
/* eslint-disable testing-library/no-node-access -- the ring's layers are decoration, without roles */
const arcs = (): readonly Element[] => [...document.querySelectorAll('[data-ring="arcs"] [data-level]')];
const track = (): Element | null => document.querySelector('[data-ring="arcs"] [data-track]');
const sweep = (): Element | null => document.querySelector('[data-sweep]');
const sheen = (): Element | null => document.querySelector('[data-sheen]');
/* eslint-enable testing-library/no-node-access */

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  await setMotion(null);
});

describe('StorageRing', () => {
  describe('negative cases', () => {
    it('an empty ring draws only the track', () => {
      render(<StorageRing {...ring({ segments: [], state: 'empty' })} />);

      expect(track()).not.toBeNull();
      expect(arcs()).toHaveLength(0);
      expect(screen.getByRole('img', { name: 'Space found' })).toBeInTheDocument();
    });

    it('a segment of 0 is not drawn', () => {
      render(
        <StorageRing
          {...ring({ segments: [...SEGMENTS, { key: 'none', label: 'None', level: 'high', size: '0 B', value: 0 }] })}
        />,
      );

      expect(arcs()).toHaveLength(3);
    });

    it('the sweep is absent when not scanning', () => {
      render(<StorageRing {...ring({ state: 'done' })} />);

      expect(sweep()).toBeNull();
    });

    it('the sheen plays once, not on every update', () => {
      const { rerender } = render(<StorageRing {...ring({ state: 'scanning' })} />);

      expect(sheen()).toBeNull();

      rerender(<StorageRing {...ring({ state: 'done' })} />);
      const first = sheen();

      expect(first).not.toBeNull();

      rerender(<StorageRing {...ring({ state: 'done', total: 120 })} />);

      expect(sheen()).toBe(first);
    });

    it('a ring that is done from the start plays no sheen', () => {
      render(<StorageRing {...ring({ state: 'done' })} />);

      expect(sheen()).toBeNull();
    });

    it('no sheen under reduced motion', async () => {
      await setMotion('reduced');
      const { rerender } = render(<StorageRing {...ring({ state: 'scanning' })} />);

      rerender(<StorageRing {...ring({ state: 'done' })} />);

      expect(sheen()).toBeNull();
    });

    it('the centre is not hidden in the picture', () => {
      render(
        <StorageRing {...ring({ segments: [], state: 'empty' })}>
          <button type="button">Scan</button>
        </StorageRing>,
      );

      expect(screen.getByRole('button', { name: 'Scan' })).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the arcs keep their order and colours', () => {
      render(<StorageRing {...ring()} />);

      expect(arcs().map((arc) => arc.getAttribute('data-level'))).toStrictEqual(['safe', 'medium', 'low']);
    });

    it("the ring's label lists every segment", () => {
      render(<StorageRing {...ring()} />);

      expect(
        screen.getByRole('img', { name: 'Space found: Xcode 42 GB, Docker 18 GB, Logs 3 GB' }),
      ).toBeInTheDocument();
    });

    it('scanning shows the sweep', () => {
      render(<StorageRing {...ring({ state: 'scanning' })} />);

      expect(sweep()).not.toBeNull();
    });

    it('a partial ring dashes its track', () => {
      render(<StorageRing {...ring({ state: 'partial' })} />);

      expect(track()).toHaveAttribute('data-track', 'partial');
    });

    it('grows its arcs by stroke-dashoffset over the slow duration', () => {
      expect(css).toMatch(/transition:\s*stroke-dashoffset var\(--duration-slow\) var\(--ease-standard\)/u);
    });

    it('a longer arc shows a longer dash', () => {
      render(<StorageRing {...ring()} />);
      const [xcode, docker] = arcs().map((arc) => Number((arc as SVGElement).style.strokeDashoffset));

      // The offset is what is hidden: the larger segment hides less of its dash.
      expect(xcode).toBeLessThan(docker ?? 0);
    });

    it('a finished partial scan plays the sheen too', () => {
      const { rerender } = render(<StorageRing {...ring({ state: 'scanning' })} />);

      rerender(<StorageRing {...ring({ state: 'partial' })} />);

      expect(sheen()).not.toBeNull();
    });
  });
});
