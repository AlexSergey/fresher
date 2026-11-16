import type { CategoryTotal, DangerLevel, Sizes } from '../ipc/bindings.generated';
import type { Picked, Selection } from './selection';

// What the selection adds up to: a preview for the selection bar. The confirmation shows only what the core's
// cleanup plan says (plan 008).
export type SelectionSummary = {
  readonly count: number;
  // The highest danger level of what is selected; `null` when nothing is.
  readonly highest: DangerLevel | null;
  readonly sizes: Sizes;
};

// The largest byte count the UI holds exactly (2^53 - 1 bytes, 9 PB): a sum past it stays there, as the core's sums
// stay at their largest size.
export const LARGEST_SIZE = Number.MAX_SAFE_INTEGER;

const DANGER_ORDER: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

const COMPLETENESS_ORDER: readonly Sizes['completeness'][] = ['complete', 'partial', 'unknown'];

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

const weakest = (a: Sizes['completeness'], b: Sizes['completeness']): Sizes['completeness'] =>
  COMPLETENESS_ORDER.indexOf(a) >= COMPLETENESS_ORDER.indexOf(b) ? a : b;

const higher = (a: DangerLevel | null, b: DangerLevel): DangerLevel =>
  a !== null && DANGER_ORDER.indexOf(a) >= DANGER_ORDER.indexOf(b) ? a : b;

// The core's rule: the bytes add up and stay at the largest size; the weakest completeness wins.
const add = (a: Sizes, b: Sizes): Sizes => ({
  allocated: Math.min(a.allocated + b.allocated, LARGEST_SIZE),
  completeness: weakest(a.completeness, b.completeness),
  logical: Math.min(a.logical + b.logical, LARGEST_SIZE),
  reclaimable: Math.min(a.reclaimable + b.reclaimable, LARGEST_SIZE),
});

// A category's total without the findings ticked off inside it; its completeness stays the category's.
const without = (total: Sizes, excluded: Sizes): Sizes => ({
  allocated: Math.max(total.allocated - excluded.allocated, 0),
  completeness: total.completeness,
  logical: Math.max(total.logical - excluded.logical, 0),
  reclaimable: Math.max(total.reclaimable - excluded.reclaimable, 0),
});

const sumOf = (picked: Iterable<Picked>): Sizes => [...picked].reduce((sum, one) => add(sum, one.sizes), NOTHING);

// The selection summed from the overview's totals: a selected category counts its total minus its exclusions, a
// finding its own sizes. The highest level of a category is its total's (an exclusion does not lower it: the total
// does not say which finding holds that level). A category the overview no longer lists counts nothing.
export const summarize = (selection: Selection, totals: readonly CategoryTotal[]): SelectionSummary => {
  let summary: SelectionSummary = { count: 0, highest: null, sizes: NOTHING };

  for (const [category, excluded] of selection.categories) {
    const total = totals.find((one) => one.category === category);

    if (total !== undefined) {
      summary = {
        count: summary.count + Math.max(total.count - excluded.size, 0),
        highest: higher(summary.highest, total.highestDanger),
        sizes: add(summary.sizes, without(total.sizes, sumOf(excluded.values()))),
      };
    }
  }

  for (const picked of selection.findings.values()) {
    summary = {
      count: summary.count + 1,
      highest: higher(summary.highest, picked.danger),
      sizes: add(summary.sizes, picked.sizes),
    };
  }

  return summary;
};
