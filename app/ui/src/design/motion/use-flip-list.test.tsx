import type { JSX } from 'react';

import { render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { type Position, useFlipList } from '.';
import { fakeReducedMotion, recordAnimations, setMotion } from '../../test/animation';

const ROW = 44;

// happy-dom has no layout: each row is placed by its index in the list (`data-index`), 44 px apart.
const placeRowsByIndex = (): void => {
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement): DOMRect {
    const top = Number(this.dataset.index ?? 0) * ROW;

    return { bottom: top + ROW, height: ROW, left: 0, right: 100, top, width: 100, x: 0, y: top } as DOMRect;
  });
};

const List = ({
  keys,
  positions,
}: {
  readonly keys: readonly string[];
  readonly positions?: ReadonlyMap<string, Position>;
}): JSX.Element => {
  const flip = useFlipList(keys, positions === undefined ? {} : { positions });

  return (
    <ul>
      {keys.map((key, index) => (
        <li data-index={index} key={key} ref={flip(key)}>
          {key}
        </li>
      ))}
    </ul>
  );
};

beforeEach(() => {
  fakeReducedMotion(false);
  placeRowsByIndex();
});

afterEach(async () => {
  await setMotion(null);
});

describe('useFlipList', () => {
  describe('negative cases', () => {
    it('a new key does not animate', () => {
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b']} />);

      rerender(<List keys={['c', 'a', 'b']} />);

      expect(animations.map(({ element }) => element.textContent)).not.toContain('c');
    });

    it('an unchanged order plays nothing', () => {
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b', 'c']} />);

      rerender(<List keys={['a', 'b', 'c']} />);

      expect(animations).toHaveLength(0);
    });

    it('plays nothing under reduced motion', async () => {
      await setMotion('reduced');
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b']} />);

      rerender(<List keys={['b', 'a']} />);

      expect(animations).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('plays the inverted translation for a moved key', () => {
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b', 'c']} />);

      rerender(<List keys={['c', 'a', 'b']} />);

      const moves = animations.map(({ element, keyframes, options }) => ({
        from: keyframes[0]?.transform,
        key: element.textContent,
        options,
      }));

      expect(moves).toStrictEqual([
        { from: 'translate(0px, -44px)', key: 'a', options: { duration: 320, easing: 'cubic-bezier(0.2, 0, 0, 1)' } },
        { from: 'translate(0px, -44px)', key: 'b', options: { duration: 320, easing: 'cubic-bezier(0.2, 0, 0, 1)' } },
        { from: 'translate(0px, 88px)', key: 'c', options: { duration: 320, easing: 'cubic-bezier(0.2, 0, 0, 1)' } },
      ]);
    });

    it('sets will-change only while the glide plays', async () => {
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b']} />);

      rerender(<List keys={['b', 'a']} />);

      expect(screen.getByText('a').style.willChange).toBe('transform');

      // One at a time: two `act` calls must not overlap.
      for (const { finish } of animations) {
        await finish();
      }

      expect(screen.getByText('a').style.willChange).toBe('');
    });

    it('ends a running glide before the next reorder', () => {
      const animations = recordAnimations();
      const { rerender } = render(<List keys={['a', 'b']} />);

      rerender(<List keys={['b', 'a']} />);
      rerender(<List keys={['a', 'b']} />);

      expect(animations.slice(0, 2).every(({ cancelled }) => cancelled())).toBe(true);
      expect(animations).toHaveLength(4);
    });

    it('takes the positions from the caller when it gives them', () => {
      const animations = recordAnimations();
      const at = (y: number): Position => ({ x: 0, y });
      const { rerender } = render(
        <List
          keys={['a', 'b']}
          positions={
            new Map([
              ['a', at(0)],
              ['b', at(52)],
            ])
          }
        />,
      );

      rerender(
        <List
          keys={['b', 'a']}
          positions={
            new Map([
              ['a', at(52)],
              ['b', at(0)],
            ])
          }
        />,
      );

      expect(animations.map(({ keyframes }) => keyframes[0]?.transform)).toStrictEqual([
        'translate(0px, -52px)',
        'translate(0px, 52px)',
      ]);
    });
  });
});
