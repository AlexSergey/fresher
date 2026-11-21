import { type CSSProperties, type JSX, useMemo } from 'react';

import type { CategoryId, CategoryTotal, IgnoreTarget, ModuleId, Sizes } from '../../ipc/bindings.generated';

import { useFlipList } from '../../design/motion';
import { type MenuItem } from '../../design/primitives/menu/menu';
import { t, tText } from '../../i18n';
import { type CardAccess, CategoryCard } from '../category-card/category-card';
import styles from './category-grid.module.css';
import { useCoalescedOrder } from './use-coalesced-order';

export type CategoryGridProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access?: ReadonlyMap<CategoryId, CardAccess> | undefined;
  // Everything found: each card's share is its allocated size out of this one's.
  readonly found: Sizes;
  // The list's accessible name: the danger zone's, on the scan screen.
  readonly label: string;
  // The module that finds a category, for the card's "Ignore <module>"; `null` when it is not known.
  readonly moduleOf: (category: CategoryId) => null | { readonly id: ModuleId; readonly name: string };
  readonly onDetails: (category: CategoryId) => void;
  readonly onIgnore: (target: IgnoreTarget) => void;
  readonly onToggle: (category: CategoryId) => void;
  // During a scan the totals change several times a second: the cards then move at most once per move.
  readonly scanning: boolean;
  readonly selected: (category: CategoryId) => 'mixed' | boolean;
  readonly totals: readonly CategoryTotal[];
};

// The categories largest first; ties keep the core's order.
const largestFirst = (totals: readonly CategoryTotal[]): readonly string[] =>
  totals.toSorted((first, second) => second.sizes.allocated - first.sizes.allocated).map(({ category }) => category);

// What a card's settings menu offers: ignore the category, and ignore its module when it is known.
const menuOf = (
  category: CategoryId,
  module: null | { readonly id: ModuleId; readonly name: string },
  onIgnore: (target: IgnoreTarget) => void,
  moduleDescription: string,
): readonly MenuItem[] => {
  const ignoreCategory: MenuItem = {
    description: t('ui.category-card.ignore-category.description'),
    key: 'ignore-category',
    label: t('ui.category-card.ignore-category'),
    onSelect: (): void => {
      onIgnore({ category, kind: 'category' });
    },
  };

  if (module === null) {
    return [ignoreCategory];
  }

  return [
    ignoreCategory,
    {
      description: moduleDescription,
      key: 'ignore-module',
      label: t('ui.category-card.ignore-module', { module: { kind: 'text', value: module.name } }),
      onSelect: (): void => {
        onIgnore({ kind: 'module', module: module.id });
      },
    },
  ];
};

// The category cards in a grid of one, two or three columns, as wide as its container allows. New cards enter
// staggered; a card that changes place glides there.
export const CategoryGrid = ({
  access,
  found,
  label,
  moduleOf,
  onDetails,
  onIgnore,
  onToggle,
  scanning,
  selected,
  totals,
}: CategoryGridProps): JSX.Element => {
  const order = useCoalescedOrder(largestFirst(totals), scanning);
  const flip = useFlipList(order);
  const byCategory = new Map(totals.map((total) => [total.category, total]));
  // Built once per change of what they depend on: the menu measures itself again whenever its items change.
  // The core's sentence is read here, so a dictionary that arrives later rebuilds the items.
  const moduleDescription = tText({ args: {}, key: 'core.ignore.modules.description' });
  const menus = useMemo(
    (): ReadonlyMap<CategoryId, readonly MenuItem[]> =>
      new Map(order.map((category) => [category, menuOf(category, moduleOf(category), onIgnore, moduleDescription)])),
    [order, moduleOf, onIgnore, moduleDescription],
  );

  return (
    <div className={styles.container}>
      <ul aria-label={label} className={styles.grid}>
        {order.map((category, index) => {
          const total = byCategory.get(category);

          return total === undefined ? null : (
            <li
              className={styles.cell}
              data-category={category}
              key={category}
              ref={flip(category)}
              style={{ '--i': index } as CSSProperties}
            >
              <CategoryCard
                access={access?.get(category)}
                menu={menus.get(category) ?? []}
                onDetails={() => {
                  onDetails(category);
                }}
                onToggle={() => {
                  onToggle(category);
                }}
                selected={selected(category)}
                share={found.allocated > 0 ? total.sizes.allocated / found.allocated : 0}
                total={total}
              />
            </li>
          );
        })}
      </ul>
    </div>
  );
};
