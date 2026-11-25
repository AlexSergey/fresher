import { describe, expect, it } from 'vitest';

import type { CoreAction } from './reduce';

import { FIRST_BATCH, SCAN_STARTED } from '../test/demo-scan';
import { createFakeClock } from '../test/fake-clock';
import { throttleProgress } from './progress';

const progress = (entries: number): CoreAction => ({
  bytes: entries * 10,
  entries,
  expected: 1000,
  scan: 1,
  type: 'scan-progress',
});

const throttled = (): {
  readonly advance: (ms: number) => void;
  readonly dispatch: (action: CoreAction) => void;
  readonly sent: readonly CoreAction[];
} => {
  const clock = createFakeClock();
  const sent: CoreAction[] = [];

  return {
    advance: clock.advance,
    dispatch: throttleProgress((action) => sent.push(action), { clock, interval: 100 }),
    sent,
  };
};

const entriesOf = (actions: readonly CoreAction[]): readonly number[] =>
  actions.flatMap((action) => (action.type === 'scan-progress' ? [action.entries] : []));

describe('throttleProgress', () => {
  describe('negative cases', () => {
    it('progress beyond 10 a second is dropped except the last', () => {
      const { advance, dispatch, sent } = throttled();

      for (let entries = 1; entries <= 100; entries += 1) {
        dispatch(progress(entries));
        advance(10);
      }
      advance(100);

      expect(entriesOf(sent)).toEqual([1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    });

    it('holds no progress once a new scan starts', () => {
      const { advance, dispatch, sent } = throttled();
      dispatch(progress(1));

      dispatch(progress(2));
      dispatch(SCAN_STARTED);
      advance(1000);

      expect(sent.map((action) => action.type)).toEqual(['scan-progress', 'scan-progress', 'scan-started']);
    });
  });

  describe('positive cases', () => {
    it('lets every other action through at once', () => {
      const { dispatch, sent } = throttled();
      dispatch(progress(1));
      dispatch(progress(2));

      dispatch(FIRST_BATCH);

      expect(sent).toEqual([progress(1), FIRST_BATCH]);
    });

    it('sends the held progress before the scan finishes', () => {
      const { dispatch, sent } = throttled();
      dispatch(progress(1));
      dispatch(progress(2));

      dispatch({ kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' });

      expect(sent.map((action) => action.type)).toEqual(['scan-progress', 'scan-progress', 'scan-finished']);
      expect(entriesOf(sent)).toEqual([1, 2]);
    });
  });
});
