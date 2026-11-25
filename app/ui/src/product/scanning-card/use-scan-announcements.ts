import { useEffect, useRef } from 'react';

import type { Sizes, Text } from '../../ipc/bindings.generated';

import { useAnnounce } from '../../design/feedback/announcer/announcer';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';

// Where the scan is, as the Scan screen derives it (`ScanPhase`).
type Phase = 'failed' | 'finished' | 'idle' | 'partial' | 'scanning';

// The announcements of one scan share a key, so the announcer speaks its progress at most every 5 seconds.
const KEY = 'scan';

const outcomeKey = (phase: Exclude<Phase, 'idle' | 'scanning'>): string => {
  switch (phase) {
    case 'failed':
      return 'core.scan.outcome.failed';
    case 'finished':
      return 'core.scan.outcome.completed';
    case 'partial':
      return 'core.scan.outcome.partial';
  }
};

// The scan's progress and its end, spoken by the `Announcer` above: "Found X so far" while what was found grows
// (the announcer lets one through every 5 seconds), and once when the scan ends, its outcome at once.
export const useScanAnnouncements = (phase: Phase, found: Sizes, failure: null | Text): void => {
  const announce = useAnnounce();
  const previousRef = useRef(phase);
  const said = honestSize(found).accessibleText;

  useEffect(() => {
    if (phase === 'scanning' && found.allocated > 0) {
      announce(t('ui.scanning.announce.progress', { size: { kind: 'text', value: said } }), { key: KEY });
    }
  }, [announce, phase, found.allocated, said]);

  useEffect(() => {
    const previous = previousRef.current;

    previousRef.current = phase;

    if (previous !== 'scanning' || phase === 'scanning' || phase === 'idle') {
      return;
    }

    const outcome = tText({ args: {}, key: outcomeKey(phase) });
    const detail =
      phase === 'failed'
        ? failure === null
          ? ''
          : tText(failure)
        : t('ui.scanning.announce.found', { size: { kind: 'text', value: said } });

    announce(`${outcome} ${detail}`.trim(), { final: true, key: KEY });
  }, [announce, phase, failure, said]);
};
