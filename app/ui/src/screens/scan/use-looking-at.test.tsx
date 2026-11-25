import { act, renderHook, type RenderHookResult } from '@testing-library/react';
import { type JSX, type ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import type { ModuleId } from '../../ipc/bindings.generated';

import { WallClockContext } from '../../state/wall-clock';
import { createFakeClock, type FakeClock } from '../../test/fake-clock';
import { TURN_MS, useLookingAt } from './use-looking-at';

type Props = { readonly finished: readonly ModuleId[]; readonly scanning: boolean };

const STARTED: readonly ModuleId[] = ['jetbrains', 'logs', 'temp'];

type Mounted = {
  readonly advance: (ms: number) => void;
  readonly clock: FakeClock;
  readonly view: RenderHookResult<ModuleId | null, Props>;
};

// The hook on a fake clock: the line's module, and the clock to move it.
const mount = (initial: Props): Mounted => {
  const clock: FakeClock = createFakeClock();
  const wrapper = ({ children }: { readonly children: ReactNode }): JSX.Element => (
    <WallClockContext value={{ ...clock, epochMs: () => 0 }}>{children}</WallClockContext>
  );
  const view = renderHook(({ finished, scanning }: Props) => useLookingAt(scanning, STARTED, finished), {
    initialProps: initial,
    wrapper,
  });
  const advance = (ms: number): void => {
    act(() => {
      clock.advance(ms);
    });
  };

  return { advance, clock, view };
};

describe('useLookingAt', () => {
  describe('negative cases', () => {
    it('names nothing and waits for nothing when no scan runs', () => {
      const { clock, view } = mount({ finished: [], scanning: false });

      expect(view.result.current).toBeNull();
      expect(clock.waiting()).toBe(0);
    });

    it('one module left stays and no timer runs', () => {
      const { advance, clock, view } = mount({ finished: ['jetbrains', 'temp'], scanning: true });

      advance(3 * TURN_MS);

      expect(view.result.current).toBe('logs');
      expect(clock.waiting()).toBe(0);
    });

    it('the line does not change before its turn is over', () => {
      const { advance, view } = mount({ finished: [], scanning: true });

      advance(TURN_MS - 1);

      expect(view.result.current).toBe('jetbrains');
    });

    it('its timer is cancelled when the screen goes', () => {
      const { clock, view } = mount({ finished: [], scanning: true });

      view.unmount();

      expect(clock.waiting()).toBe(0);
    });
  });

  describe('positive cases', () => {
    it('the modules still running take turns in the core order and wrap around', () => {
      const { advance, view } = mount({ finished: [], scanning: true });
      const seen = [view.result.current];

      for (let turn = 0; turn < 3; turn += 1) {
        advance(TURN_MS);
        seen.push(view.result.current);
      }

      expect(seen).toStrictEqual(['jetbrains', 'logs', 'temp', 'jetbrains']);
    });

    it('a module that finished leaves the turns', () => {
      const { advance, view } = mount({ finished: [], scanning: true });

      view.rerender({ finished: ['logs'], scanning: true });
      const seen = [view.result.current];
      for (let turn = 0; turn < 2; turn += 1) {
        advance(TURN_MS);
        seen.push(view.result.current);
      }

      expect(seen).toStrictEqual(['jetbrains', 'temp', 'jetbrains']);
    });
  });
});
