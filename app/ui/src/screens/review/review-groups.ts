import type { CategoryId, CategoryTotal, DangerLevel, Finding, FindingId, Sizes } from '../../ipc/bindings.generated';
import type { Picked, Selection } from '../../state/selection';

import { ZONE_ORDER } from '../../product/danger-zone/zones';
import { NOTHING_SELECTED } from '../../state/selection';
import { summarize } from '../../state/selection-summary';

// One category of the selection as the review screen shows it: what of it goes, how many of its listed items are not
// ticked, and where its rows come from.
export type ReviewGroup = {
  readonly category: CategoryId;
  // How many of its items go.
  readonly count: number;
  readonly highest: DangerLevel;
  // The part of the selection it stands for: the summary sums each danger group's from these with the store's rule.
  readonly part: Selection;
  readonly rows: ReviewRows;
  readonly sizes: Sizes;
  // How many of its listed items do not go: a category to check's not ticked yet, a selected category's ticked off.
  readonly unchecked: number;
};

// One danger group of the summary: what of it goes, and how many of its items do not.
export type ReviewLevel = {
  readonly level: DangerLevel;
  readonly sizes: Sizes;
  readonly unchecked: number;
};

// A selected category lists all its findings (the core pages them), the ones ticked off inside it unticked; findings
// picked one by one are listed as the selection keeps them. A category to check lists all its findings too, only
// the ones ticked one by one ticked.
type ReviewRows =
  | { readonly excluded: ReadonlyMap<FindingId, Picked>; readonly kind: 'category'; readonly listed: number }
  | { readonly findings: readonly Finding[]; readonly kind: 'picked' }
  | { readonly kind: 'to-check'; readonly listed: number; readonly ticked: ReadonlyMap<FindingId, Picked> };

const groupOf = (
  category: CategoryId,
  rows: Exclude<ReviewRows, { readonly kind: 'to-check' }>,
  part: Selection,
  totals: readonly CategoryTotal[],
): null | ReviewGroup => {
  const { count, highest, sizes } = summarize(part, totals);
  const unchecked = rows.kind === 'category' ? rows.excluded.size : 0;

  return highest === null || count === 0 ? null : { category, count, highest, part, rows, sizes, unchecked };
};

const largestFirst = (a: Finding, b: Finding): number => b.sizes.allocated - a.sizes.allocated || a.id - b.id;

// A category to check: what of it is ticked, summed with the store's rule, at its total's level; shown even with
// nothing ticked (it sums to nothing then). A category the overview no longer lists is not shown.
const toCheckGroupOf = (
  category: CategoryId,
  ticked: ReadonlyMap<FindingId, Picked>,
  totals: readonly CategoryTotal[],
): null | ReviewGroup => {
  const total = totals.find((one) => one.category === category);

  if (total === undefined) {
    return null;
  }
  const part = { ...NOTHING_SELECTED, findings: ticked };
  const { count, sizes } = summarize(part, totals);

  return {
    category,
    count,
    highest: total.highestDanger,
    part,
    rows: { kind: 'to-check', listed: total.count, ticked },
    sizes,
    unchecked: Math.max(total.count - ticked.size, 0),
  };
};

const levelIndex = (level: DangerLevel): number => ZONE_ORDER.indexOf(level);

// The selection by category, each summed with the store's rule (`summarize`), in the groups' order (Safe, Low,
// Medium, High), largest first within one; a group with nothing left in it is not shown, but a category to check is.
export const reviewGroups = (selection: Selection, totals: readonly CategoryTotal[]): readonly ReviewGroup[] => {
  const groups: (null | ReviewGroup)[] = [];

  for (const category of selection.toCheck) {
    const ticked = new Map([...selection.findings].filter(([, finding]) => finding.category === category));
    groups.push(toCheckGroupOf(category, ticked, totals));
  }

  for (const [category, excluded] of selection.categories) {
    const listed = totals.find((total) => total.category === category)?.count ?? 0;
    const part = { ...NOTHING_SELECTED, categories: new Map([[category, excluded]]) };
    groups.push(groupOf(category, { excluded, kind: 'category', listed }, part, totals));
  }

  const picked = new Map<CategoryId, Finding[]>();

  // The findings of a category to check are listed in its own group.
  for (const finding of selection.findings.values()) {
    if (selection.toCheck.has(finding.category)) {
      continue;
    }
    picked.set(finding.category, [...(picked.get(finding.category) ?? []), finding]);
  }

  for (const [category, findings] of picked) {
    const part = { ...NOTHING_SELECTED, findings: new Map(findings.map((finding) => [finding.id, finding])) };
    groups.push(groupOf(category, { findings: findings.toSorted(largestFirst), kind: 'picked' }, part, totals));
  }

  return groups
    .filter((group) => group !== null)
    .toSorted(
      (a, b) =>
        levelIndex(a.highest) - levelIndex(b.highest) ||
        b.sizes.allocated - a.sizes.allocated ||
        a.category.localeCompare(b.category),
    );
};

// The parts of some groups as one selection.
const together = (groups: readonly ReviewGroup[]): Selection => ({
  categories: new Map(groups.flatMap((group) => [...group.part.categories])),
  findings: new Map(groups.flatMap((group) => [...group.part.findings])),
  toCheck: new Set(),
});

// The danger groups present, the safest first: each sums what of its categories goes with the store's rule
// (`summarize`), and counts how many of their listed items do not go.
export const reviewLevels = (
  groups: readonly ReviewGroup[],
  totals: readonly CategoryTotal[],
): readonly ReviewLevel[] =>
  ZONE_ORDER.flatMap((level) => {
    const inLevel = groups.filter((group) => group.highest === level);

    return inLevel.length === 0
      ? []
      : [
          {
            level,
            sizes: summarize(together(inLevel), totals).sizes,
            unchecked: inLevel.reduce((sum, group) => sum + group.unchecked, 0),
          },
        ];
  });
