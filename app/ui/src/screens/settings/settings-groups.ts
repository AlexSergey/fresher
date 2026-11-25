import type { ModuleId, ModuleInfo, SettingDescriptor } from '../../ipc/bindings.generated';
import type { InGroup } from '../../product/module-groups';

import { inGroups } from '../../product/module-groups';

// A module with settings: its rows in the module's order, and its on/off switch apart, to show last.
export type ModuleSettings = {
  readonly module: ModuleId;
  readonly rows: readonly SettingDescriptor[];
  readonly switch: SettingDescriptor | undefined;
};

export type SettingsGroups = {
  readonly general: readonly SettingDescriptor[];
  readonly modules: readonly InGroup<ModuleSettings>[];
  readonly scan: readonly SettingDescriptor[];
};

// The settings of each page: the core's General and Scan rows, and the modules that have settings by group (known
// from the modules alone, before the settings arrive: their rows then are empty). The
// ignored lists and the permission switches have parts of their own.
export const settingsGroups = (
  descriptors: readonly SettingDescriptor[],
  modules: readonly ModuleInfo[],
): SettingsGroups => {
  const byKey = new Map(descriptors.map((descriptor) => [descriptor.key, descriptor] as const));
  const ofCore = (group: 'general' | 'scan'): readonly SettingDescriptor[] =>
    descriptors.filter((descriptor) => descriptor.group === group);
  const ofModule = (module: ModuleInfo): ModuleSettings => {
    const own = module.settings.flatMap((key) => byKey.get(key) ?? []);

    return {
      module: module.id,
      rows: own.filter((descriptor) => descriptor.key !== `${module.id}.enabled`),
      switch: own.find((descriptor) => descriptor.key === `${module.id}.enabled`),
    };
  };
  const withSettings = modules.filter(({ settings }) => settings.length > 0).map(ofModule);
  const groupOf = new Map(modules.map(({ group, id }) => [id, group] as const));

  return {
    general: ofCore('general'),
    modules: inGroups(withSettings, ({ module }) => groupOf.get(module) ?? ''),
    scan: ofCore('scan'),
  };
};
