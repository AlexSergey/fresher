import type { JSX } from 'react';

import type { Sizes } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { ScanningCard } from '../../product/scanning-card/scanning-card';
import { useCore } from '../../state/store';
import { useModules } from '../use-modules';
import { scanPercent } from './progress';
import { useLookingAt } from './use-looking-at';

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

// The running scan's card, wherever a scan is started from: the percent from the core's progress, the "Looking at"
// line and what was found so far. Rendered only while a scan runs.
export const ScanProgressCard = (): JSX.Element => {
  const modules = useModules();
  const progress = useCore((state) => state.progress);
  const finished = useCore((state) => state.modules);
  const started = useCore((state) => state.scanModules);
  const found = useCore((state) => state.overview?.found ?? NOTHING);
  const volumeName = useCore((state) => state.storageMap?.volumes[0]?.name ?? '');
  // The modules the scan runs (a scan learned of from an overview only runs every module the core lists), which of
  // them have finished, and the one the "Looking at" line names: those still running take turns while it runs.
  const running = started ?? modules?.map(({ id }) => id) ?? [];
  const done = finished.map(({ module }) => module);
  const module = useLookingAt(true, running, done);

  return (
    <ScanningCard
      found={found}
      module={module === null ? null : tText({ args: {}, key: `${module}.enabled.label` })}
      percent={scanPercent(progress?.bytes ?? 0, progress?.expected ?? null, {
        finished: done.length,
        started: running.length,
      })}
      volumeName={volumeName}
    />
  );
};
