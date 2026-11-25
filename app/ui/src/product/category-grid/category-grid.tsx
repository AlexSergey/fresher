import { type CSSProperties, type JSX, useMemo } from 'react';

import type { CategoryIcon, CategoryId, CategoryTotal, IgnoreTarget } from '../../ipc/bindings.generated';

import { useFlipList } from '../../design/motion';
import { type MenuItem } from '../../design/primitives/menu/menu';
import { t, tText } from '../../i18n';
import { type CardAccess, CategoryTile } from '../category-tile/category-tile';
import styles from './category-grid.module.css';
import { PlaceholderTile } from './placeholder-tile';
import { useCoalescedOrder } from './use-coalesced-order';

export type CategoryGridProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access?: ReadonlyMap<CategoryId, CardAccess> | undefined;
  // The icon the core names for a category.
  readonly icon: (category: CategoryId) => CategoryIcon;
  // The list's accessible name: the danger zone's, on the scan screen.
  readonly label: string;
  readonly onDetails: (category: CategoryId) => void;
  readonly onIgnore: (target: IgnoreTarget) => void;
  readonly onToggle: (category: CategoryId) => void;
  // A faded stand-in tile after the last one: more is still being found.
  readonly placeholder?: boolean | undefined;
  // During a scan the totals change several times a second: the tiles then move at most once per move.
  readonly scanning: boolean;
  readonly selected: (category: CategoryId) => 'mixed' | boolean;
  readonly totals: readonly CategoryTotal[];
};

// The categories largest first; ties keep the core's order.
const largestFirst = (totals: readonly CategoryTotal[]): readonly string[] =>
  totals.toSorted((first, second) => second.sizes.allocated - first.sizes.allocated).map(({ category }) => category);

// What a tile's options menu offers: ignore the category. A module is turned off only in Settings.
const menuOf = (category: CategoryId, onIgnore: (target: IgnoreTarget) => void): readonly MenuItem[] => [
  {
    key: 'ignore-category',
    label: t('ui.category-card.ignore-category', {
      name: { kind: 'text', value: tText({ args: {}, key: `${category}.name` }) },
    }),
    onSelect: (): void => {
      onIgnore({ category, kind: 'category' });
    },
  },
];

// The category tiles in two columns, one when its container is narrow; while scanning they have no controls. New tiles
// enter staggered; a tile that changes place glides there.
export const CategoryGrid = ({
  access,
  icon,
  label,
  onDetails,
  onIgnore,
  onToggle,
  placeholder = false,
  scanning,
  selected,
  totals,
}: CategoryGridProps): JSX.Element => {
  const order = useCoalescedOrder(largestFirst(totals), scanning);
  const flip = useFlipList(order);
  const byCategory = new Map(totals.map((total) => [total.category, total]));
  // Built once per change of what they depend on: the menu measures itself again whenever its items change.
  const menus = useMemo(
    (): ReadonlyMap<CategoryId, readonly MenuItem[]> =>
      new Map(order.map((category) => [category, menuOf(category, onIgnore)])),
    [order, onIgnore],
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
              <CategoryTile
                access={access?.get(category)}
                icon={icon(category)}
                menu={menus.get(category) ?? []}
                onDetails={() => {
                  onDetails(category);
                }}
                onToggle={() => {
                  onToggle(category);
                }}
                scanning={scanning}
                selected={selected(category)}
                total={total}
              />
            </li>
          );
        })}
        {placeholder ? <PlaceholderTile /> : null}
      </ul>
    </div>
  );
};
