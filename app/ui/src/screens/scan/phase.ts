import type { EngineStatus, LastScan, ScanOutcome, Text } from '../../ipc/bindings.generated';

// Where the scan is, as the screen derives it from the core's status and the last scan's outcome.
export type ScanPhase = 'failed' | 'finished' | 'idle' | 'partial' | 'scanning';

// How the last scan ended: as the events told it (`outcome`, the store's), which can be newer than the overview's
// last scan until the overview is asked for again; `null` before any scan.
const endOf = (lastScan: LastScan | null, outcome: null | ScanOutcome): null | ScanOutcome =>
  outcome ?? lastScan?.outcome ?? null;

// The hero's phase: a running scan (the store's status, which an older overview cannot undo), else how the last scan
// ended, else idle.
export const phaseOf = (status: EngineStatus, lastScan: LastScan | null, outcome: null | ScanOutcome): ScanPhase => {
  if (status.state === 'scanning') {
    return 'scanning';
  }

  const end = endOf(lastScan, outcome);

  switch (end?.kind) {
    case 'completed':
      return 'finished';
    case 'failed':
      return 'failed';
    case 'partial':
      return 'partial';
    case undefined:
      return 'idle';
  }
};

// The core's reason of a failed last scan; `null` when it did not fail.
export const scanFailure = (lastScan: LastScan | null, outcome: null | ScanOutcome): null | Text => {
  const end = endOf(lastScan, outcome);

  return end?.kind === 'failed' ? end.reason : null;
};
