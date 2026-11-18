import type { CategoryId, CleanupSelection, Finding, FindingId } from '../ipc/bindings.generated';

// A finding the user ticked, as the core sent it: the summary counts it and the review screen lists it with no
// request. Only the findings the user ticked one by one are kept, never a page of them.
export type Picked = Finding;

// What the user picked for removal: single findings, and whole categories. A selected category stands for all its
// findings minus the ones ticked off inside it (its map of exclusions), so selecting a category of 100,000 findings
// loads none of them. A finding of a selected category is never in `findings` as well.
export type Selection = {
  readonly categories: ReadonlyMap<CategoryId, ReadonlyMap<FindingId, Picked>>;
  readonly findings: ReadonlyMap<FindingId, Picked>;
};

export type SelectionAction =
  // A module's choice: its categories (`categories`, those of `within` that hold a finding) are selected whole, and
  // nothing of the module's other categories (`within`) stays selected; nothing outside `within` changes.
  | {
      readonly categories: readonly CategoryId[];
      readonly type: 'choose';
      readonly within: readonly CategoryId[];
    }
  | { readonly category: CategoryId; readonly type: 'toggle-category' }
  | { readonly finding: Finding; readonly type: 'toggle-finding' }
  | { readonly type: 'clear' };

export const NOTHING_SELECTED: Selection = { categories: new Map(), findings: new Map() };

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

const toggleCategory = (selection: Selection, category: CategoryId): Selection => {
  if (selection.categories.has(category)) {
    const categories = new Map(selection.categories);
    categories.delete(category);

    return { ...selection, categories };
  }

  // The category now stands for its findings: the ones picked one by one are part of it.
  const findings = new Map([...selection.findings].filter(([, picked]) => picked.category !== category));

  return { categories: new Map([...selection.categories, [category, new Map()]]), findings };
};

const choose = (selection: Selection, categories: readonly CategoryId[], within: readonly CategoryId[]): Selection => ({
  categories: new Map([
    ...[...selection.categories].filter(([category]) => !within.includes(category)),
    ...categories.map((category): [CategoryId, ReadonlyMap<FindingId, Picked>] => [category, new Map()]),
  ]),
  findings: new Map([...selection.findings].filter(([, picked]) => !within.includes(picked.category))),
});

export const reduceSelection = (selection: Selection, action: SelectionAction): Selection => {
  switch (action.type) {
    case 'choose':
      return choose(selection, action.categories, action.within);
    case 'clear':
      return NOTHING_SELECTED;
    case 'toggle-category':
      return toggleCategory(selection, action.category);
    case 'toggle-finding':
      return toggleFinding(selection, action.finding);
  }
};

// A category's checkbox: checked when the category is selected whole, mixed when only some of its findings are
// (picked one by one, or ticked off inside the selected category).
export const checkedOf = (selection: Selection, category: CategoryId): 'mixed' | boolean => {
  const excluded = selection.categories.get(category);

  if (excluded !== undefined) {
    return excluded.size === 0 ? true : 'mixed';
  }

  return [...selection.findings.values()].some((picked) => picked.category === category) ? 'mixed' : false;
};

// The selection as the core takes it: ids only, never a path.
export const cleanupSelectionOf = (selection: Selection): CleanupSelection => ({
  categories: [...selection.categories].map(([category, except]) => ({ category, except: [...except.keys()] })),
  findings: [...selection.findings.keys()],
});
