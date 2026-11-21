import type { CategoryId, CategoryTotal, DangerLevel } from '../../ipc/bindings.generated';

import { ZONE_ORDER } from '../../product/danger-zone/zones';
import { checkedOf, type Selection } from '../../state/selection';

// The categories of one danger level, in the order they were given.
export type Zone = { readonly level: DangerLevel; readonly totals: readonly CategoryTotal[] };

// The categories by their highest level, in the zones' order, empty zones left out.
export const zonesOf = (totals: readonly CategoryTotal[]): readonly Zone[] =>
  ZONE_ORDER.map((level) => ({ level, totals: totals.filter(({ highestDanger }) => highestDanger === level) })).filter(
    ({ totals: inZone }) => inZone.length > 0,
  );

// A zone's "Select all": checked when every selectable category of it is selected whole, mixed when some are (or some
// of their findings), unchecked otherwise; `null` when nothing in it can be selected. A selectable category has
// findings and is not unavailable.
export const zoneSelectAll = (
  selection: Selection,
  zone: Zone,
  selectable: (category: CategoryId) => boolean,
): null | { readonly categories: readonly CategoryId[]; readonly checked: 'mixed' | boolean } => {
  const categories = zone.totals
    .filter(({ category, count }) => count > 0 && selectable(category))
    .map(({ category }) => category);

  if (categories.length === 0) {
    return null;
  }
  const states = categories.map((category) => checkedOf(selection, category));

  if (states.every((state) => state === true)) {
    return { categories, checked: true };
  }

  return { categories, checked: states.some((state) => state !== false) ? 'mixed' : false };
};
