import type { DemoOutcome } from '../ipc/bindings.generated';
import type { CoreStore } from '../state/store';

import { demoFinished, demoPlay } from '../ipc';
import { play } from './player';
import { createPointer } from './pointer';

const sleep = (ms: number): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

// A development build in demo mode: plays the shell's scenario on this page and reports how it ended. Without a
// scenario (or with a shell that has no demo mode) nothing happens at all.
export const startDemo = async (store: CoreStore): Promise<void> => {
  const scenario = await demoPlay().catch(() => null);
  if (scenario === null) {
    return;
  }
  store.dispatch({ type: 'demo-started' });
  document.documentElement.dataset.demo = 'playing';
  const pointer = createPointer(document);
  let outcome: DemoOutcome;
  try {
    outcome = await play(scenario, { now: () => performance.now(), pointer, root: document.body, sleep, store });
  } catch (error) {
    outcome = { kind: 'failed', reason: error instanceof Error ? error.message : String(error), step: 1 };
  }
  pointer.remove();
  await demoFinished(outcome).catch((error: unknown) => {
    // eslint-disable-next-line no-console -- the page has no other log, and a refusal must not end as a silent hang
    console.error('the demo outcome was refused', error);
  });
};
