import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import type { Segment } from '../segment';

import { fakeReducedMotion, setMotion } from '../../../test/animation';
import { StorageRing, type StorageRingProps } from './storage-ring';

const css = readFileSync(path.join(import.meta.dirname, 'storage-ring.module.css'), 'utf8');

const SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', size: '42 GB', tone: 'safe', value: 42 },
  { key: 'docker', label: 'Docker', size: '18 GB', tone: 'medium', value: 18 },
  { key: 'logs', label: 'Logs', size: '3 GB', tone: 'low', value: 3 },
];

const WITH_FREE: readonly Segment[] = [
  ...SEGMENTS,
  { key: 'free', label: 'Free', size: '37 GB', tone: 'free', value: 37 },
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
const arcs = (): readonly Element[] => [...document.querySelectorAll('[data-ring="arcs"] [data-tone]')];
const track = (): Element | null => document.querySelector('[data-ring="arcs"] [data-track]');
const sweep = (): Element | null => document.querySelector('[data-sweep]');
const gapLabelNode = (): Element | null => document.querySelector('[data-gap-label]');
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
          {...ring({ segments: [...SEGMENTS, { key: 'none', label: 'None', size: '0 B', tone: 'high', value: 0 }] })}
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

    it('no gap label without gapLabel, or without a free segment', () => {
      const { rerender } = render(<StorageRing {...ring({ segments: WITH_FREE })} />);

      expect(gapLabelNode()).toBeNull();

      rerender(<StorageRing {...ring({ gapLabel: 'Free' })} />);

      expect(gapLabelNode()).toBeNull();
    });

    it('a free segment is not drawn as an arc', () => {
      render(<StorageRing {...ring({ segments: WITH_FREE })} />);

      expect(arcs()).toHaveLength(3);
    });

    it('no arc is dimmed while nothing is highlighted', () => {
      render(<StorageRing {...ring()} />);

      expect(arcs().filter((arc) => arc.hasAttribute('data-dimmed'))).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('the arcs keep their order and colours', () => {
      render(<StorageRing {...ring()} />);

      expect(arcs().map((arc) => arc.getAttribute('data-tone'))).toStrictEqual(['safe', 'medium', 'low']);
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

    it('puts the label at the free slot, hidden from assistive technology', () => {
      render(<StorageRing {...ring({ gapLabel: 'Free', segments: WITH_FREE })} />);

      expect(gapLabelNode()).toHaveTextContent('Free');
      expect(gapLabelNode()).toHaveAttribute('aria-hidden', 'true');
    });

    it('pointer enter and leave on an arc call onHighlight with its key and null', () => {
      const calls: (null | string)[] = [];

      render(<StorageRing {...ring({ onHighlight: (key) => calls.push(key) })} />);
      const [first] = arcs();

      fireEvent.pointerEnter(first as Element);
      fireEvent.pointerLeave(first as Element);

      expect(calls).toStrictEqual(['xcode', null]);
    });

    it('a highlighted key dims every other arc, and marks the free label', () => {
      const { rerender } = render(<StorageRing {...ring({ highlighted: 'docker' })} />);

      expect(arcs().map((arc) => arc.hasAttribute('data-dimmed'))).toStrictEqual([true, false, true]);

      rerender(<StorageRing {...ring({ gapLabel: 'Free', highlighted: 'free', segments: WITH_FREE })} />);

      expect(gapLabelNode()).toHaveAttribute('data-highlighted');
    });

    it('a look with butt caps, no glow and a thickness draws square-ended arcs of that band', () => {
      render(
        <StorageRing
          {...ring({
            accessibleName: 'The whole disk',
            look: { caps: 'butt', gap: 3, glow: false, minSweep: 5 },
            thickness: 16,
          })}
        />,
      );

      expect(screen.getByRole('img', { name: 'The whole disk' })).toBeInTheDocument();
      expect(arcs().every((arc) => arc.getAttribute('data-cap') === 'square')).toBe(true);
      // eslint-disable-next-line testing-library/no-node-access -- the ring's layers are decoration, without roles
      expect(document.querySelector('[data-glow="off"]')).not.toBeNull();
      expect(arcs()[0]).toHaveAttribute('r', '108');
    });
  });
});
