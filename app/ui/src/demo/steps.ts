import type { AppState, Step, Target } from '../ipc/bindings.generated';
import type { CoreStore } from '../state/store';
import type { Pointer } from './pointer';

import { findTarget } from './finder';
import { isRefused } from './refused';

export const TARGET_TIMEOUT_MS = 5000;
export const STATE_TIMEOUT_MS = 60_000;
const POLL_MS = 50;
const SCROLL_MS = 400;

// What a step needs; the clock is injected so tests run on fake time.
export type StepDeps = {
  readonly now: () => number;
  readonly pointer: Pointer;
  readonly root: HTMLElement;
  readonly sleep: (ms: number) => Promise<void>;
  readonly store: CoreStore;
};

export type StepResult = { readonly kind: 'done' } | { readonly kind: 'failed'; readonly reason: string };

const DONE: StepResult = { kind: 'done' };

const nameOf = (target: Target): string => target.name.text;

// Polls until `check` answers, or the time is out. `check` answers `null` to keep waiting.
const poll = async <T>(deps: StepDeps, timeoutMs: number, check: () => null | T): Promise<null | T> => {
  const started = deps.now();
  for (;;) {
    const answer = check();
    if (answer !== null) {
      return answer;
    }
    if (deps.now() - started >= timeoutMs) {
      return null;
    }
    await deps.sleep(POLL_MS);
  }
};

type Located =
  { readonly element: HTMLElement; readonly kind: 'one' } | { readonly kind: 'failed'; readonly reason: string };

// The one element of the target, once it is there; two at once fail without waiting.
const locate = async (deps: StepDeps, target: Target): Promise<Located> => {
  const what = `${target.role} “${nameOf(target)}”`;
  const answer = await poll<Located>(deps, TARGET_TIMEOUT_MS, () => {
    const found = findTarget(deps.root, target);
    if (found.kind === 'one') {
      return found;
    }

    return found.kind === 'many' ? { kind: 'failed', reason: `${found.count} elements are the ${what}` } : null;
  });

  if (answer?.kind === 'one' && isRefused(answer.element)) {
    return { kind: 'failed', reason: `the ${what} is a permanent or irreversible removal, which a demo never uses` };
  }

  return answer ?? { kind: 'failed', reason: `no element is the ${what}` };
};

const glideTo = async (deps: StepDeps, element: HTMLElement): Promise<void> => {
  const box = element.getBoundingClientRect();
  await deps.pointer.moveTo(box.left + box.width / 2, box.top + box.height / 2);
};

// The element that scrolls for a target: its nearest ancestor (or itself) that overflows vertically. A
// screen scrolls in a plain container, which no role or name can address.
const scrollerOf = (element: HTMLElement): HTMLElement | null => {
  const view = element.ownerDocument.defaultView;
  for (let node: HTMLElement | null = element; node !== null; node = node.parentElement) {
    const { overflowY } = view?.getComputedStyle(node) ?? { overflowY: '' };
    if ((overflowY === 'auto' || overflowY === 'scroll') && node.scrollHeight > node.clientHeight) {
      return node;
    }
  }

  return null;
};

const STATES: Readonly<Record<AppState, (store: CoreStore) => boolean>> = {
  'cleanup-finished': (store) => store.getState().cleanup.kind === 'finished',
  'scan-finished': (store) => store.getState().outcome !== null,
};

const mouseEvent = (type: string, bubbles: boolean): MouseEvent => new MouseEvent(type, { bubbles, cancelable: true });

// The browser bubbles `over` and does not bubble `enter`.
const HOVER_EVENTS: readonly (readonly [string, boolean])[] = [
  ['pointerover', true],
  ['pointerenter', false],
  ['mouseover', true],
  ['mouseenter', false],
];

const isDisabled = (element: HTMLElement): boolean =>
  element.hasAttribute('disabled') || element.getAttribute('aria-disabled') === 'true';

// A click on a disabled control does nothing, so it fails the step instead of passing as done.
const clickOn = async (deps: StepDeps, element: HTMLElement, name: string): Promise<StepResult> => {
  if (isDisabled(element)) {
    return { kind: 'failed', reason: `the ${name} is disabled` };
  }
  await deps.pointer.press();
  element.click();

  return DONE;
};

export const runStep = async (step: Step, deps: StepDeps): Promise<StepResult> => {
  switch (step.kind) {
    case 'click':
    case 'hover':
    case 'scroll': {
      const located = await locate(deps, step.target);
      if (located.kind === 'failed') {
        return located;
      }
      const { element } = located;
      await glideTo(deps, element);
      if (step.kind === 'click') {
        return await clickOn(deps, element, nameOf(step.target));
      } else if (step.kind === 'hover') {
        for (const [type, bubbles] of HOVER_EVENTS) {
          element.dispatchEvent(mouseEvent(type, bubbles));
        }
      } else {
        const scroller = scrollerOf(element);
        if (scroller === null) {
          return { kind: 'failed', reason: `nothing scrolls at the ${nameOf(step.target)}` };
        }
        scroller.scrollBy({ behavior: 'smooth', top: step.by });
        await deps.sleep(SCROLL_MS);
      }

      return DONE;
    }
    case 'pause':
      await deps.sleep(step.ms);

      return DONE;
    case 'wait': {
      const { until } = step;
      if (until.kind === 'target') {
        const located = await locate(deps, until.target);

        return located.kind === 'failed' ? located : DONE;
      }
      const reached = await poll(deps, STATE_TIMEOUT_MS, () => (STATES[until.state](deps.store) ? true : null));

      return reached === null ? { kind: 'failed', reason: `the app did not reach ${until.state} in 60 s` } : DONE;
    }
  }
};
