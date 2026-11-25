import type { CategoryId, CleanupSelection, DangerLevel, Finding, FindingId } from '../ipc/bindings.generated';

import { BY_HAND } from '../product/danger-zone/zones';

// A category's highest level, as the overview's totals tell it; `undefined` when they do not list it.
export type LevelOf = (category: CategoryId) => DangerLevel | undefined;

// A finding the user ticked, as the core sent it: the summary counts it and the review screen lists it with no
// request. Only the findings the user ticked one by one are kept, never a page of them.
export type Picked = Finding;

// What the user picked for removal: single findings, and whole categories. A selected category stands for all its
// findings minus the ones ticked off inside it (its map of exclusions), so selecting a category of 100,000 findings
// loads none of them. A finding of a selected category is never in `findings` as well.
export type Selection = {
  readonly categories: ReadonlyMap<CategoryId, ReadonlyMap<FindingId, Picked>>;
  readonly findings: ReadonlyMap<FindingId, Picked>;
  // The "Look before removing" categories (a `BY_HAND` level) the user brought to Review: none of their findings goes
  // unless it is in `findings` (ticked one by one) or the category is in `categories` (the user ticked all of it in
  // Review). A category here is never in `categories` as well.
  readonly toCheck: ReadonlySet<CategoryId>;
};

export type SelectionAction =
  // A module's choice: its categories (`categories`, those of `within` that hold a finding) are selected whole (a
  // `BY_HAND` one is only brought to Review), and nothing of the module's other categories (`within`) stays selected
  // or to check; nothing outside `within` changes.
  | {
      readonly categories: readonly CategoryId[];
      readonly type: 'choose';
      readonly within: readonly CategoryId[];
    }
  // Review's all-checkbox of a category: the user ticks all of it (`tick-all`), or unticks all of it (`untick-all`).
  // Only Review's all-checkbox may send `tick-all`: it selects a "Look before removing" category whole, which is the
  // user's own act there and nowhere else.
  | { readonly category: CategoryId; readonly type: 'tick-all' }
  | { readonly category: CategoryId; readonly type: 'toggle-category' }
  | { readonly category: CategoryId; readonly type: 'untick-all' }
  | { readonly finding: Finding; readonly type: 'toggle-finding' }
  | { readonly type: 'clear' };

export const NOTHING_SELECTED: Selection = { categories: new Map(), findings: new Map(), toCheck: new Set() };

// A level the overview does not tell is ticked by hand: a category is never selected whole for the user unless its
// level says it may be.
const byHand = (levels: LevelOf, category: CategoryId): boolean => {
  const level = levels(category);

  return level === undefined || BY_HAND.has(level);
};

const without = <T>(set: ReadonlySet<T>, value: T): ReadonlySet<T> => new Set([...set].filter((one) => one !== value));

const findingsOutside = (selection: Selection, category: CategoryId): ReadonlyMap<FindingId, Picked> =>
  new Map([...selection.findings].filter(([, picked]) => picked.category !== category));

// A copy of `map` with `key` removed when it is there, added with `value` when it is not.
const toggled = <K, V>(map: ReadonlyMap<K, V>, key: K, value: V): ReadonlyMap<K, V> => {
  const next = new Map(map);

  if (!next.delete(key)) {
    next.set(key, value);
  }

  return next;
};

const toggleFinding = (selection: Selection, finding: Finding): Selection => {
  const excluded = selection.categories.get(finding.category);

  // Inside a selected category, a finding is ticked off (or back on) in the category's exclusions.
  if (excluded !== undefined) {
    const categories = new Map(selection.categories);
    categories.set(finding.category, toggled(excluded, finding.id, finding));

    return { ...selection, categories };
  }

  return { ...selection, findings: toggled(selection.findings, finding.id, finding) };
};

// The category now stands for its findings: the ones picked one by one are part of it, and it is no longer to check.
const tickAll = (selection: Selection, category: CategoryId): Selection => ({
  categories: new Map([...selection.categories, [category, new Map()]]),
  findings: findingsOutside(selection, category),
  toCheck: without(selection.toCheck, category),
});

// Nothing of the category goes; a `BY_HAND` one stays to check in Review.
const untickAll = (selection: Selection, category: CategoryId, levels: LevelOf): Selection => {
  const categories = new Map(selection.categories);
  categories.delete(category);
  const toCheck = byHand(levels, category)
    ? new Set([...selection.toCheck, category])
    : without(selection.toCheck, category);

  return { categories, findings: findingsOutside(selection, category), toCheck };
};

// A `BY_HAND` category is brought to Review with nothing of it ticked for the user (its findings ticked one by one
// stay ticked); toggled again, everything of it leaves. Any other category is selected whole, or no longer.
const toggleCategory = (selection: Selection, category: CategoryId, levels: LevelOf): Selection => {
  if (byHand(levels, category)) {
    if (selection.toCheck.has(category) || selection.categories.has(category)) {
      const categories = new Map(selection.categories);
      categories.delete(category);

      return {
        categories,
        findings: findingsOutside(selection, category),
        toCheck: without(selection.toCheck, category),
      };
    }

    return { ...selection, toCheck: new Set([...selection.toCheck, category]) };
  }

  if (selection.categories.has(category)) {
    const categories = new Map(selection.categories);
    categories.delete(category);

    return { ...selection, categories };
  }

  return tickAll(selection, category);
};

const choose = (
  selection: Selection,
  categories: readonly CategoryId[],
  within: readonly CategoryId[],
  levels: LevelOf,
): Selection => ({
  categories: new Map([
    ...[...selection.categories].filter(([category]) => !within.includes(category)),
    ...categories
      .filter((category) => !byHand(levels, category))
      .map((category): [CategoryId, ReadonlyMap<FindingId, Picked>] => [category, new Map()]),
  ]),
  findings: new Map([...selection.findings].filter(([, picked]) => !within.includes(picked.category))),
  toCheck: new Set([
    ...[...selection.toCheck].filter((category) => !within.includes(category)),
    ...categories.filter((category) => byHand(levels, category)),
  ]),
});

// `levels` tells each category's highest level: a "Look before removing" category is never selected whole for the user.
export const reduceSelection = (selection: Selection, action: SelectionAction, levels: LevelOf): Selection => {
  switch (action.type) {
    case 'choose':
      return choose(selection, action.categories, action.within, levels);
    case 'clear':
      return NOTHING_SELECTED;
    case 'tick-all':
      return tickAll(selection, action.category);
    case 'toggle-category':
      return toggleCategory(selection, action.category, levels);
    case 'toggle-finding':
      return toggleFinding(selection, action.finding);
    case 'untick-all':
      return untickAll(selection, action.category, levels);
  }
};

// A category's checkbox: checked when the category is selected whole, mixed when only some of its findings are
// (picked one by one, or ticked off inside the selected category). A category to check reads checked (it is brought
// to Review), mixed once some of its findings are ticked.
export const checkedOf = (selection: Selection, category: CategoryId): 'mixed' | boolean => {
  const excluded = selection.categories.get(category);

  if (excluded !== undefined) {
    return excluded.size === 0 ? true : 'mixed';
  }

  const picked = [...selection.findings.values()].some((one) => one.category === category);

  if (selection.toCheck.has(category)) {
    return picked ? 'mixed' : true;
  }

  return picked ? 'mixed' : false;
};

// The selection as the core takes it: ids only, never a path. A category to check adds nothing: only what goes.
export const cleanupSelectionOf = (selection: Selection): CleanupSelection => ({
  categories: [...selection.categories].map(([category, except]) => ({ category, except: [...except.keys()] })),
  findings: [...selection.findings.keys()],
});
