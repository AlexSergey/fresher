import type { IgnoredCount } from '../../ipc/bindings.generated';
import type { IgnoredTab } from '../../state/navigation';

// The tab of Settings' Ignored section to open on: the first with an entry, else the first.
export const firstWithEntry = (counts: Readonly<Record<IgnoredTab, number>>): IgnoredTab =>
  counts.groups > 0 ? 'groups' : counts.modules > 0 ? 'modules' : counts.items > 0 ? 'items' : 'groups';

// An overview's counts of what is ignored, as the three tabs of Settings' Ignored section show them.
export const countsOf = ({ categories, groups, modules, paths }: IgnoredCount): Record<IgnoredTab, number> => ({
  groups,
  items: categories + paths,
  modules,
});
