import { useEffect, useState } from 'react';

import type { Timestamp } from '../../ipc/bindings.generated';

import { agoText } from '../../i18n/relative-time';
import { useWallClock } from '../../state/wall-clock';

const MINUTE_MS = 60_000;

// How long ago the last scan finished ("2 minutes ago"): the clock is read when the scan changes and again every
// minute while one is shown; `null` without a last scan.
export const useScannedAgo = (finished: null | Timestamp): null | string => {
  const clock = useWallClock();
  const [read, setRead] = useState(() => ({ finished, now: clock.epochMs() }));

  if (read.finished !== finished) {
    setRead({ finished, now: clock.epochMs() });
  }

  useEffect(() => {
    if (finished === null) {
      return;
    }

    // eslint-disable-next-line @eslint-react/web-api-no-leaked-timeout -- the clock's timer returns its own cancel
    return clock.setTimeout(() => {
      setRead({ finished, now: clock.epochMs() });
    }, MINUTE_MS);
  }, [clock, finished, read]);

  return finished === null ? null : agoText(finished, read.now);
};
