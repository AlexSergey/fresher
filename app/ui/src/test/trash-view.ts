import type { Refusal, Sizes, TrashLine, TrashView } from '../ipc/bindings.generated';

// What the core can answer together: a line only when something waits or a cleanup put something there, a share
// never above what waits, and a date only with a share.
const sizesOf = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const lineOf = (props: Partial<TrashLine> & Pick<TrashLine, 'fromCleanups' | 'index'>): TrashLine => ({
  goneBy: null,
  volume: 'Macintosh HD',
  waiting: null,
  ...props,
});

// One Trash: 23 GB wait, 20 GB of it from earlier cleanups, no date (Finder does not empty it by itself).
export const TRASH_ONE: TrashView = {
  trashes: [lineOf({ fromCleanups: sizesOf(20_000_000_000), index: 0, waiting: sizesOf(23_000_000_000) })],
};

// The same with Finder's 30-day setting: the share leaves by itself on Jan 1, 2026.
export const TRASH_ONE_DATED: TrashView = {
  trashes: [{ ...(TRASH_ONE.trashes[0] as TrashLine), goneBy: 1_767_268_800 }],
};

// Two Trashes: the second volume's was not measured, so only the share is known.
export const TRASH_TWO: TrashView = {
  trashes: [
    lineOf({ fromCleanups: sizesOf(1_200_000_000), index: 0, waiting: sizesOf(5_000_000_000) }),
    lineOf({ fromCleanups: sizesOf(800_000_000), index: 1, volume: 'Backup' }),
  ],
};

// One Trash that was not measured: only the share of the cleanups is known.
export const TRASH_NOT_MEASURED: TrashView = {
  trashes: [lineOf({ fromCleanups: sizesOf(1_200_000_000), index: 0 })],
};

// What waits after the demo plan went to the Trash: its 4.07 GB (shown as 4.1 GB), all of it from this cleanup.
export const TRASH_AFTER_CLEANUP: TrashView = {
  trashes: [lineOf({ fromCleanups: sizesOf(4_070_000_000), index: 0, waiting: sizesOf(4_070_000_000) })],
};

// What `open_trash` refuses for an index the last answer did not have (the index is the mock's to fill in).
export const UNKNOWN_TRASH: Refusal = {
  error: { code: 'unknown-trash', index: 0 },
  text: { args: {}, key: 'core.error.unknown-trash' },
};
