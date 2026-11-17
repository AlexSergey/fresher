import type { ModuleId, ModuleInfo, ModuleOutcome } from '../../ipc/bindings.generated';
import type { ModuleProgress } from '../../product/scan-progress/scan-progress';

type Finished = { readonly module: ModuleId; readonly outcome: ModuleOutcome };

// The scan's chips: the modules the scan runs, as its `scan-started` listed them, else (a scan learned of from an
// overview only) every module the core lists; each with its group and its outcome once its `module-finished`
// arrived, running until then. A module the core does not list has no group and no name, so it has no chip.
export const moduleProgress = (
  started: null | readonly ModuleId[],
  modules: null | readonly ModuleInfo[],
  finished: readonly Finished[],
): readonly ModuleProgress[] =>
  (started ?? modules?.map(({ id }) => id) ?? []).flatMap((module) => {
    const info = modules?.find(({ id }) => id === module);

    return info === undefined
      ? []
      : [
          {
            group: info.group,
            module,
            state: finished.find((one) => one.module === module)?.outcome ?? { kind: 'running' },
          },
        ];
  });
