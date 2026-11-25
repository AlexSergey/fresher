import { useEffect, useState } from 'react';

import type { ModuleId } from '../../ipc/bindings.generated';

import { useWallClock } from '../../state/wall-clock';
import { lookingAt, stillRunning } from './progress';

// How long one module stays on the "Looking at" line while others run too.
export const TURN_MS = 6000;

// The module the "Looking at" line names while a scan runs: the modules still running take turns, one more every
// `TURN_MS` on the injected clock, in the core's order; with one left it stays, and with none (or no scan) there is no
// line. The timer runs only while more than one takes turns, and is cancelled when that changes or the screen goes.
export const useLookingAt = (
  scanning: boolean,
  started: readonly ModuleId[],
  finished: readonly ModuleId[],
): ModuleId | null => {
  const clock = useWallClock();
  const [turn, setTurn] = useState(0);
  const taking = scanning ? stillRunning(started, finished) : [];
  const count = taking.length;

  useEffect(() => {
    if (count < 2) {
      return;
    }

    // eslint-disable-next-line @eslint-react/web-api-no-leaked-timeout -- the clock's timer returns its own cancel
    return clock.setTimeout(() => {
      setTurn((current) => current + 1);
    }, TURN_MS);
  }, [clock, count, turn]);

  return lookingAt(taking, turn);
};
