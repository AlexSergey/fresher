import type { Timestamp } from '../ipc/bindings.generated';

const relative = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

// How long ago `at` (the core's seconds since the epoch) was, read at `nowMs` (milliseconds since the epoch): "now"
// under a minute (and for a time ahead of the clock), then minutes, hours, "yesterday" and days.
export const agoText = (at: Timestamp, nowMs: number): string => {
  const seconds = Math.floor(nowMs / 1000) - at;

  if (seconds < MINUTE) {
    return relative.format(0, 'second');
  }
  if (seconds < HOUR) {
    return relative.format(-Math.floor(seconds / MINUTE), 'minute');
  }
  if (seconds < DAY) {
    return relative.format(-Math.floor(seconds / HOUR), 'hour');
  }

  return relative.format(-Math.floor(seconds / DAY), 'day');
};
