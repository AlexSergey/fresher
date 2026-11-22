import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Step, Target } from '../ipc/bindings.generated';
import type { CoreStore } from '../state/store';
import type { Pointer } from './pointer';
import type { StepDeps } from './steps';

import { createCoreStore } from '../state/store';
import { runStep, STATE_TIMEOUT_MS, TARGET_TIMEOUT_MS } from './steps';

const target = (name: string): Target => ({ name: { kind: 'exact', text: name }, role: 'button' });

type Rig = { clock: { time: number }; deps: StepDeps; pointer: { [K in keyof Pointer]: ReturnType<typeof vi.fn> } };

const rig = (store: CoreStore = createCoreStore()): Rig => {
  const clock = { time: 0 };
  const pointer = { moveTo: vi.fn(() => Promise.resolve()), press: vi.fn(() => Promise.resolve()), remove: vi.fn() };
  const deps: StepDeps = {
    now: () => clock.time,
    pointer,
    root: document.body,
    sleep: (ms) => {
      clock.time += ms;

      return Promise.resolve();
    },
    store,
  };

  return { clock, deps, pointer };
};

const click = (name: string): Step => ({ kind: 'click', target: target(name) });

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  document.body.replaceChildren();
});

describe('runStep', () => {
  describe('negative cases', () => {
    it('fails a click whose target never appears, after 5 s', async () => {
      const { clock, deps } = rig();

      expect(await runStep(click('Scan'), deps)).toEqual({ kind: 'failed', reason: 'no element is the button “Scan”' });
      expect(clock.time).toBeGreaterThanOrEqual(TARGET_TIMEOUT_MS);
      expect(clock.time).toBeLessThan(TARGET_TIMEOUT_MS + 100);
    });

    it.each(['<button disabled>Scan</button>', '<button aria-disabled="true">Scan</button>'])(
      'fails a click on a disabled control and presses nothing: %s',
      async (html) => {
        document.body.innerHTML = html;
        const { deps, pointer } = rig();

        expect(await runStep(click('Scan'), deps)).toEqual({ kind: 'failed', reason: 'the Scan is disabled' });
        expect(pointer.press).not.toHaveBeenCalled();
      },
    );

    it('fails at once when two elements are the target', async () => {
      document.body.innerHTML = '<button>Scan</button><button>Scan</button>';
      const { clock, deps } = rig();

      expect(await runStep(click('Scan'), deps)).toEqual({
        kind: 'failed',
        reason: '2 elements are the button “Scan”',
      });
      expect(clock.time).toBe(0);
    });

    it('fails a wait for a state that never comes, after 60 s', async () => {
      const { clock, deps } = rig();
      const result = await runStep({ kind: 'wait', until: { kind: 'state', state: 'scan-finished' } }, deps);

      expect(result).toEqual({ kind: 'failed', reason: 'the app did not reach scan-finished in 60 s' });
      expect(clock.time).toBeGreaterThanOrEqual(STATE_TIMEOUT_MS);
    });

    it('fails a wait for a cleanup that never finishes', async () => {
      const result = await runStep({ kind: 'wait', until: { kind: 'state', state: 'cleanup-finished' } }, rig().deps);

      expect(result.kind).toBe('failed');
    });

    it('fails a wait for a target that never appears', async () => {
      const result = await runStep({ kind: 'wait', until: { kind: 'target', target: target('Done') } }, rig().deps);

      expect(result).toEqual({ kind: 'failed', reason: 'no element is the button “Done”' });
    });
  });

  describe('negative cases (permanent removal)', () => {
    const CHOICE = 'Delete permanently instead — this cannot be undone';

    it.each([
      ['the choice, by its aria-label', `<input type="checkbox" aria-label="${CHOICE}">`, 'checkbox', CHOICE],
      [
        'the choice, by its label text',
        `<input type="checkbox" id="c"><label for="c">${CHOICE}<span>Frees the space now.</span></label>`,
        'checkbox',
        'Delete permanently instead — this cannot be undone Frees the space now.',
      ],
      ['the Delete button', '<button>Delete permanently</button>', 'button', 'Delete permanently'],
      ['the tool removal button', '<button>Remove</button>', 'button', 'Remove'],
    ] as const)('fails a click, a hover and a scroll on %s and presses nothing', async (_, html, role, name) => {
      document.body.innerHTML = html;
      const clicked = vi.fn();
      document.body.addEventListener('click', clicked);
      const { deps, pointer } = rig();
      const named: Target = { name: { kind: 'exact', text: name }, role };

      for (const step of [
        { kind: 'click', target: named },
        { kind: 'hover', target: named },
        { by: 100, kind: 'scroll', target: named },
        { kind: 'wait', until: { kind: 'target', target: named } },
      ] as const) {
        const result = await runStep(step, deps);

        expect(result.kind).toBe('failed');
      }
      expect(clicked).not.toHaveBeenCalled();
      expect(pointer.press).not.toHaveBeenCalled();
    });

    it('fails a scroll when nothing overflows', async () => {
      document.body.innerHTML = '<button>Scan</button>';
      const { deps } = rig();

      expect(await runStep({ by: 100, kind: 'scroll', target: target('Scan') }, deps)).toEqual({
        kind: 'failed',
        reason: 'nothing scrolls at the Scan',
      });
    });
  });

  describe('positive cases', () => {
    it('glides to the target and clicks it', async () => {
      document.body.innerHTML = '<button>Scan</button>';
      const handler = vi.fn();
      document.querySelector('button')?.addEventListener('click', handler);
      const { deps, pointer } = rig();

      expect(await runStep(click('Scan'), deps)).toEqual({ kind: 'done' });
      expect(pointer.moveTo).toHaveBeenCalledTimes(1);
      expect(pointer.press).toHaveBeenCalledTimes(1);
      expect(handler).toHaveBeenCalledTimes(1);
    });

    it('clicks a target that appears after a while', async () => {
      const { deps } = rig();
      const original = deps.sleep;
      let slept = 0;
      const late: StepDeps = {
        ...deps,
        sleep: (ms) => {
          slept += 1;
          if (slept === 3) {
            document.body.innerHTML = '<button>Scan</button>';
          }

          return original(ms);
        },
      };

      expect(await runStep(click('Scan'), late)).toEqual({ kind: 'done' });
    });

    it('hovers: enter events fire and do not bubble', async () => {
      document.body.innerHTML = '<div id="outer"><button>Scan</button></div>';
      const seen: string[] = [];
      const outer = document.querySelector('#outer');
      const inner = document.querySelector('button');
      for (const type of ['pointerover', 'pointerenter', 'mouseover', 'mouseenter']) {
        inner?.addEventListener(type, () => seen.push(`${type}:inner`));
        outer?.addEventListener(type, () => seen.push(`${type}:outer`));
      }
      const { deps, pointer } = rig();

      expect(await runStep({ kind: 'hover', target: target('Scan') }, deps)).toEqual({ kind: 'done' });
      expect(pointer.press).not.toHaveBeenCalled();
      expect(seen).toEqual([
        'pointerover:inner',
        'pointerover:outer',
        'pointerenter:inner',
        'mouseover:inner',
        'mouseover:outer',
        'mouseenter:inner',
      ]);
    });

    it('scrolls the target smoothly and waits 400 ms', async () => {
      document.body.innerHTML = '<button style="overflow-y: auto">Scan</button>';
      const scrollBy = vi.fn();
      const element = document.querySelector('button');
      if (element !== null) {
        element.scrollBy = scrollBy;
        Object.defineProperty(element, 'scrollHeight', { value: 900 });
        Object.defineProperty(element, 'clientHeight', { value: 300 });
      }
      const { clock, deps } = rig();

      await runStep({ by: 300, kind: 'scroll', target: target('Scan') }, deps);

      expect(scrollBy).toHaveBeenCalledWith({ behavior: 'smooth', top: 300 });
      expect(clock.time).toBe(400);
    });

    it('scrolls the nearest ancestor that overflows when the target itself does not', async () => {
      document.body.innerHTML = '<div id="area" style="overflow-y: auto"><button>Scan</button></div>';
      const area = document.querySelector<HTMLElement>('#area');
      const scrollBy = vi.fn();
      if (area !== null) {
        area.scrollBy = scrollBy;
        Object.defineProperty(area, 'scrollHeight', { value: 900 });
        Object.defineProperty(area, 'clientHeight', { value: 300 });
      }
      const { deps } = rig();

      await runStep({ by: 400, kind: 'scroll', target: target('Scan') }, deps);

      expect(scrollBy).toHaveBeenCalledWith({ behavior: 'smooth', top: 400 });
    });

    it('pauses for the time asked', async () => {
      const { clock, deps } = rig();

      expect(await runStep({ kind: 'pause', ms: 700 }, deps)).toEqual({ kind: 'done' });
      expect(clock.time).toBe(700);
    });

    it('sees scan-finished once the store gets the finishing action', async () => {
      const store = createCoreStore();
      store.dispatch({ modules: ['demo'], scan: 1, type: 'scan-started' });
      const { deps } = rig(store);
      const original = deps.sleep;
      const later: StepDeps = {
        ...deps,
        sleep: (ms) => {
          store.dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });

          return original(ms);
        },
      };

      expect(await runStep({ kind: 'wait', until: { kind: 'state', state: 'scan-finished' } }, later)).toEqual({
        kind: 'done',
      });
    });

    it('waits for a target until it is added', async () => {
      document.body.innerHTML = '<button>Done</button>';
      const result = await runStep({ kind: 'wait', until: { kind: 'target', target: target('Done') } }, rig().deps);

      expect(result).toEqual({ kind: 'done' });
    });
  });
});
