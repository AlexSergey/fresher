import type { CategoryId, CategoryTotal, DangerLevel, Finding, FindingId, Sizes } from '../../ipc/bindings.generated';
import type { Picked, Selection } from '../../state/selection';

import { NOTHING_SELECTED } from '../../state/selection';
import { summarize } from '../../state/selection-summary';

// One category of the selection as the review screen shows it: what it adds up to, and where its rows come from.
export type ReviewGroup = {
  readonly category: CategoryId;
  readonly count: number;
  readonly highest: DangerLevel;
  readonly rows: ReviewRows;
  readonly sizes: Sizes;
};

// A selected category lists all its findings (the core pages them), the ones ticked off inside it unticked; findings
// picked one by one are listed as the selection keeps them.
type ReviewRows =
  | { readonly excluded: ReadonlyMap<FindingId, Picked>; readonly kind: 'category'; readonly listed: number }
  | { readonly findings: readonly Finding[]; readonly kind: 'picked' };

const groupOf = (
  category: CategoryId,
  rows: ReviewRows,
  part: Selection,
  totals: readonly CategoryTotal[],
): null | ReviewGroup => {
  const { count, highest, sizes } = summarize(part, totals);

  return highest === null || count === 0 ? null : { category, count, highest, rows, sizes };
};

const largestFirst = (a: Finding, b: Finding): number => b.sizes.allocated - a.sizes.allocated || a.id - b.id;

// The selection by category, each summed with the store's rule (`summarize`), largest first; a group with nothing
// left in it is not shown.
export const reviewGroups = (selection: Selection, totals: readonly CategoryTotal[]): readonly ReviewGroup[] => {
  const groups: (null | ReviewGroup)[] = [];

  for (const [category, excluded] of selection.categories) {
    const listed = totals.find((total) => total.category === category)?.count ?? 0;
    const part = { ...NOTHING_SELECTED, categories: new Map([[category, excluded]]) };
    groups.push(groupOf(category, { excluded, kind: 'category', listed }, part, totals));
  }

  const picked = new Map<CategoryId, Finding[]>();

  for (const finding of selection.findings.values()) {
    picked.set(finding.category, [...(picked.get(finding.category) ?? []), finding]);
  }

  for (const [category, findings] of picked) {
    const part = { ...NOTHING_SELECTED, findings: new Map(findings.map((finding) => [finding.id, finding])) };
    groups.push(groupOf(category, { findings: findings.toSorted(largestFirst), kind: 'picked' }, part, totals));
  }

  return groups
    .filter((group) => group !== null)
    .toSorted((a, b) => b.sizes.allocated - a.sizes.allocated || a.category.localeCompare(b.category));
};
