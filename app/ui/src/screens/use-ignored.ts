import type { HiddenBy, ModuleId, ModuleInfo, Overview } from '../ipc/bindings.generated';

// Why the core hides `module` (its own entry or its group's); `undefined` when it does not. The core says it in the
// overview, so a restore shows at once, with the next overview.
export const hiddenBy = (overview: null | Overview, module: ModuleId): HiddenBy | undefined =>
  overview?.hiddenModules.find((hidden) => hidden.module === module)?.by;

// Whether the core hides the modules of `group` because the group is ignored.
export const isGroupIgnored = (overview: null | Overview, modules: readonly ModuleInfo[], group: string): boolean =>
  modules.some((module) => module.group === group && hiddenBy(overview, module.id) === 'group');
