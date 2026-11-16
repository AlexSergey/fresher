import type { ModuleId, ModuleInfo, ModuleOutcome } from '../../ipc/bindings.generated';
import type { ModuleProgress } from '../../product/scan-progress/scan-progress';

type Finished = { readonly module: ModuleId; readonly outcome: ModuleOutcome };

// The scan's chips: the modules the scan runs, as its `scan-started` listed them, else (a scan learned of from an
// overview only) every module the core lists; each with its outcome once its `module-finished` arrived, running until
// then.
export const moduleProgress = (
  started: null | readonly ModuleId[],
  modules: null | readonly ModuleInfo[],
  finished: readonly Finished[],
): readonly ModuleProgress[] =>
  (started ?? modules?.map(({ id }) => id) ?? []).map((module) => ({
    module,
    state: finished.find((one) => one.module === module)?.outcome ?? { kind: 'running' },
  }));
