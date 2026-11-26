import { render, screen as view } from '@testing-library/react';
import { type JSX, useEffect } from 'react';
import { afterEach, describe, expect, it } from 'vitest';

import type { Screen } from '../state/navigation';

import { recordAnimations, setMotion } from '../test/animation';
import { ScreenStack } from './screen-stack';

const CLEANUPS: Screen = { kind: 'cleanups' };
const ONE_CLEANUP: Screen = { cleanup: 1, kind: 'cleanup' };

// A screen that counts its mounts, as a screen that reads the core when it opens would read it.
const counting = (mounts: Map<string, number>) =>
  function Counted({ name }: { readonly name: string }): JSX.Element {
    useEffect(() => {
      mounts.set(name, (mounts.get(name) ?? 0) + 1);
    }, [name]);

    return <p>{name}</p>;
  };

const stackOf = (mounts: Map<string, number>) => {
  const Counted = counting(mounts);

  return (screen: Screen): JSX.Element => (
    <ScreenStack render={(shown) => <Counted name={shown.kind} />} screen={screen} />
  );
};

describe('ScreenStack', () => {
  afterEach(async () => {
    await setMotion(null);
  });

  describe('negative cases', () => {
    it('staying on the same screen does not mount it again', async () => {
      await setMotion('full');
      recordAnimations();
      const mounts = new Map<string, number>();
      const stack = stackOf(mounts);

      const { rerender } = render(stack(CLEANUPS));
      rerender(stack({ kind: 'cleanups' }));

      expect(mounts.get('cleanups')).toBe(1);
    });
  });

  describe('positive cases', () => {
    it('a screen opened again while it is still fading out is mounted anew, so it reads its data again', async () => {
      await setMotion('full');
      const animations = recordAnimations();
      const mounts = new Map<string, number>();
      const stack = stackOf(mounts);

      const { rerender } = render(stack(CLEANUPS));
      rerender(stack(ONE_CLEANUP));
      // The list is still fading out: its exit has not finished.
      expect(animations.some((animation) => !animation.cancelled())).toBe(true);
      rerender(stack(CLEANUPS));

      expect(mounts.get('cleanups')).toBe(2);
      expect(view.getAllByText('cleanups')).toHaveLength(1);
    });

    it('a screen opened again after it left is mounted anew', async () => {
      await setMotion('off');
      const mounts = new Map<string, number>();
      const stack = stackOf(mounts);

      const { rerender } = render(stack(CLEANUPS));
      rerender(stack(ONE_CLEANUP));
      rerender(stack(CLEANUPS));

      expect(mounts.get('cleanups')).toBe(2);
    });
  });
});
