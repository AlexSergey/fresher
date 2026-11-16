import type { ModuleId, ModuleInfo, SettingDescriptor } from '../../ipc/bindings.generated';

export type SettingsGroup = {
  readonly descriptors: readonly SettingDescriptor[];
  // `null` for the core's own settings.
  readonly module: ModuleId | null;
};

// The core's settings first (every setting no module lists), then each module's in its own order, in the order the
// core registered the modules. A group with no setting is left out.
export const settingsGroups = (
  descriptors: readonly SettingDescriptor[],
  modules: readonly ModuleInfo[],
): readonly SettingsGroup[] => {
  const byKey = new Map(descriptors.map((descriptor) => [descriptor.key, descriptor] as const));
  const ofModules = new Set(modules.flatMap((module) => module.settings));
  const groups: SettingsGroup[] = [
    { descriptors: descriptors.filter((descriptor) => !ofModules.has(descriptor.key)), module: null },
    ...modules.map((module) => ({
      descriptors: module.settings.flatMap((key) => byKey.get(key) ?? []),
      module: module.id,
    })),
  ];

  return groups.filter((group) => group.descriptors.length > 0);
};
