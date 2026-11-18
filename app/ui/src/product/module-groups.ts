import type { ModuleGroup, ModuleInfo } from '../ipc/bindings.generated';

// The core's order of the groups; a group a newer core adds comes after them, in the order its modules arrive.
const GROUP_ORDER: readonly string[] = ['system', 'development', 'debug'] satisfies readonly ModuleGroup[];

export type InGroup<T> = {
  // A `ModuleGroup`, or the key of one this build does not know yet.
  readonly group: string;
  readonly items: readonly T[];
};

// `items` split by their group: only the groups that have an item, the core's groups in the core's order, an unknown
// one last, each group's items in the order they came.
export const inGroups = <T>(items: readonly T[], groupOf: (item: T) => string): readonly InGroup<T>[] => {
  const byGroup = new Map<string, T[]>();

  for (const item of items) {
    const group = groupOf(item);

    byGroup.set(group, [...(byGroup.get(group) ?? []), item]);
  }

  const rank = (group: string): number => {
    const index = GROUP_ORDER.indexOf(group);

    return index === -1 ? GROUP_ORDER.length : index;
  };

  return [...byGroup.entries()]
    .map(([group, grouped]): InGroup<T> => ({ group, items: grouped }))
    .toSorted((a, b) => rank(a.group) - rank(b.group));
};

// The modules by group, for the sidebar and the settings.
export const modulesByGroup = (modules: readonly ModuleInfo[]): readonly InGroup<ModuleInfo>[] =>
  inGroups(modules, ({ group }) => group);
