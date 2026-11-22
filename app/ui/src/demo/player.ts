import type { DemoOutcome, DemoPlay } from '../ipc/bindings.generated';
import type { StepDeps } from './steps';

import { runStep } from './steps';

export const LEAD_IN_MS = 1500;

export type PlayerDeps = StepDeps;

// A path under /private/tmp (or /private/var) is also written without /private on macOS.
const withAlias = (text: string): string[] =>
  text.startsWith('/private/') ? [text, text.slice('/private'.length)] : [text];

const failed = (step: number, reason: string): DemoOutcome => ({ kind: 'failed', reason, step });

// Plays the scenario's steps in order; after each, the page's text must hold none of the private texts. The first
// failure ends the play.
export const play = async (demo: DemoPlay, deps: PlayerDeps): Promise<DemoOutcome> => {
  await deps.sleep(LEAD_IN_MS);
  const started = deps.now();
  const secrets = demo.private.filter((text) => text !== '').flatMap(withAlias);
  let n = 0;

  for (const step of demo.scenario.steps) {
    n += 1;
    try {
      const result = await runStep(step, deps);
      if (result.kind === 'failed') {
        return failed(n, result.reason);
      }
    } catch (error) {
      return failed(n, error instanceof Error ? error.message : String(error));
    }
    const page = deps.root.textContent;
    if (secrets.some((text) => page.includes(text))) {
      return failed(n, `the page shows a private path after step ${n}`);
    }
  }

  return { elapsedMs: Math.round(deps.now() - started), kind: 'finished' };
};
