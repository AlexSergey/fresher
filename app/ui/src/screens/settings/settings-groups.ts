import type { ModuleId, ModuleInfo, SettingDescriptor, SettingGroup } from '../../ipc/bindings.generated';

// A group of rows: one of the core's groups, or one module's settings under its name.
export type SettingsGroup =
  | { readonly descriptors: readonly SettingDescriptor[]; readonly group: CoreRowsGroup; readonly kind: 'core' }
  | { readonly descriptors: readonly SettingDescriptor[]; readonly kind: 'module'; readonly module: ModuleId };

// The core's groups shown as rows; the ignored lists have a screen part of their own.
type CoreRowsGroup = Exclude<SettingGroup, 'ignored' | 'module'>;

// The groups of rows in the screen's order, around the module groups: General and Scan before them, Permissions
// after them (the Ignored group, shown between, is not made of rows). A group with no setting is left out.
export const settingsGroups = (
  descriptors: readonly SettingDescriptor[],
  modules: readonly ModuleInfo[],
): { readonly after: readonly SettingsGroup[]; readonly before: readonly SettingsGroup[] } => {
  const byKey = new Map(descriptors.map((descriptor) => [descriptor.key, descriptor] as const));
  const core = (group: CoreRowsGroup): SettingsGroup => ({
    descriptors: descriptors.filter((descriptor) => descriptor.group === group),
    group,
    kind: 'core',
  });
  const filled = (groups: readonly SettingsGroup[]): readonly SettingsGroup[] =>
    groups.filter((group) => group.descriptors.length > 0);
  const ofModules = modules.map((module): SettingsGroup => ({
    descriptors: module.settings.flatMap((key) => byKey.get(key) ?? []),
    kind: 'module',
    module: module.id,
  }));

  return {
    after: filled([core('permissions')]),
    before: filled([core('general'), core('scan'), ...ofModules]),
  };
};
