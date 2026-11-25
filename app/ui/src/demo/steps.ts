import type { AppState, Step, Target } from '../ipc/bindings.generated';
import type { CoreStore } from '../state/store';
import type { Pointer } from './pointer';

import { findTarget } from './finder';
import { isRefused } from './refused';

export const TARGET_TIMEOUT_MS = 5000;
export const STATE_TIMEOUT_MS = 60_000;
const POLL_MS = 50;
const SCROLL_MS = 400;
// The longest a smooth scroll that brings a target into view is waited for.
export const REVEAL_TIMEOUT_MS = 2000;

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

// The element that scrolls for a target: its nearest ancestor (or itself) that overflows vertically. A
// screen scrolls in a plain container, which no role or name can address.
const scrollerOf = (element: HTMLElement | null): HTMLElement | null => {
  const view = element?.ownerDocument.defaultView;
  for (let node = element; node !== null; node = node.parentElement) {
    const { overflowY } = view?.getComputedStyle(node) ?? { overflowY: '' };
    if ((overflowY === 'auto' || overflowY === 'scroll') && node.scrollHeight > node.clientHeight) {
      return node;
    }
  }

  return null;
};

type Band = { readonly bottom: number; readonly top: number };

// The band of the window in which the content of `scroller` is seen: its box, or the whole window without one.
const bandOf = (scroller: HTMLElement | null, element: HTMLElement): Band => {
  if (scroller !== null) {
    return scroller.getBoundingClientRect();
  }

  return { bottom: element.ownerDocument.defaultView?.innerHeight ?? 0, top: 0 };
};

// How a target must be in view before the pointer moves to it: a control the pointer acts on wholly, a region the
// pointer scrolls in only in part.
type Reveal = 'part' | 'whole';

const inView = (box: Band, band: Band, reveal: Reveal): boolean =>
  reveal === 'whole'
    ? box.top >= band.top && box.bottom <= band.bottom
    : box.bottom > band.top && box.top < band.bottom;

// Brings the target into view as a user scrolls to what they point at, so the recording shows it: unless it is already
// in view, its scroller glides (a smooth scroll) to centre it, or to its top when it is taller than the view, and the
// step goes on once the scroll has settled.
const reveal = async (deps: StepDeps, element: HTMLElement, how: Reveal): Promise<void> => {
  const scroller = scrollerOf(element.parentElement);
  const box = element.getBoundingClientRect();
  const band = bandOf(scroller, element);
  if (scroller === null || inView(box, band, how)) {
    return;
  }
  const height = band.bottom - band.top;
  const offset = box.height > height ? 0 : (height - box.height) / 2;
  const most = scroller.scrollHeight - scroller.clientHeight;
  const top = Math.round(Math.min(most, Math.max(0, scroller.scrollTop + box.top - band.top - offset)));
  scroller.scrollTo({ behavior: 'smooth', top });
  await poll(deps, REVEAL_TIMEOUT_MS, () => (Math.abs(scroller.scrollTop - top) < 1 ? true : null));
};

// Brings the target into view, then glides the pointer to the middle of the part of it that is in view.
const pointAt = async (deps: StepDeps, element: HTMLElement, how: Reveal): Promise<void> => {
  await reveal(deps, element, how);
  const box = element.getBoundingClientRect();
  const band = bandOf(scrollerOf(element.parentElement), element);
  const top = Math.max(box.top, band.top);
  const bottom = Math.max(top, Math.min(box.bottom, band.bottom));
  await deps.pointer.moveTo(box.left + box.width / 2, (top + bottom) / 2);
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

// Scrolls the scroller of the target by `by` pixels, smoothly, and waits for the scroll.
const scrollIn = async (deps: StepDeps, element: HTMLElement, name: string, by: number): Promise<StepResult> => {
  const scroller = scrollerOf(element);
  if (scroller === null) {
    return { kind: 'failed', reason: `nothing scrolls at the ${name}` };
  }
  scroller.scrollBy({ behavior: 'smooth', top: by });
  await deps.sleep(SCROLL_MS);

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
      await pointAt(deps, element, step.kind === 'scroll' ? 'part' : 'whole');
      if (step.kind === 'click') {
        return await clickOn(deps, element, nameOf(step.target));
      } else if (step.kind === 'hover') {
        for (const [type, bubbles] of HOVER_EVENTS) {
          element.dispatchEvent(mouseEvent(type, bubbles));
        }
      } else {
        return await scrollIn(deps, element, nameOf(step.target), step.by);
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
