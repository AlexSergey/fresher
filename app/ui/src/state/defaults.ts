import type { CategoryTotal, DangerLevel } from '../ipc/bindings.generated';

import { NOTHING_SELECTED, type Selection } from './selection';

// What is selected before the user picks anything: nothing. Recorded here so a later "select all Safe" never reaches
// above Safe (docs/rules.md, rule 2; the idea "cleanup preview and safe removal").
export const PRESELECTED: Selection = NOTHING_SELECTED;

export type SelectedByLevel =
  | { readonly kind: 'refused'; readonly level: DangerLevel }
  | { readonly kind: 'selected'; readonly selection: Selection };

// Adds to the selection every category whose highest level is `level`, so none of their findings is above it. Only
// `safe` may be selected this way: any other level is refused, and the user picks those one by one.
export const selectByLevel = (
  selection: Selection,
  totals: readonly CategoryTotal[],
  level: DangerLevel,
): SelectedByLevel => {
  if (level !== 'safe') {
    return { kind: 'refused', level };
  }

  const categories = new Map(selection.categories);

  for (const total of totals) {
    if (total.highestDanger === 'safe' && !categories.has(total.category)) {
      categories.set(total.category, new Map());
    }
  }

  const findings = new Map([...selection.findings].filter(([, picked]) => !categories.has(picked.category)));

  return { kind: 'selected', selection: { categories, findings } };
};
