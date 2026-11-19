import type { CSSProperties, JSX } from 'react';

import type { CategoryId, CategoryTotal, Sizes } from '../../ipc/bindings.generated';

import { useFlipList } from '../../design/motion';
import { t } from '../../i18n';
import { type CardAccess, CategoryCard } from '../category-card/category-card';
import styles from './category-grid.module.css';
import { useCoalescedOrder } from './use-coalesced-order';

export type CategoryGridProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access?: ReadonlyMap<CategoryId, CardAccess> | undefined;
  // Everything found: each card's share is its allocated size out of this one's.
  readonly found: Sizes;
  readonly onOpen: (category: CategoryId) => void;
  readonly onToggle: (category: CategoryId) => void;
  // During a scan the totals change several times a second: the cards then move at most once per move.
  readonly scanning: boolean;
  readonly selected: (category: CategoryId) => 'mixed' | boolean;
  readonly totals: readonly CategoryTotal[];
  // What of each category the by-meaning view lists under apps, outside the card's size; absent for the rest.
  readonly underApps?: ReadonlyMap<CategoryId, Sizes>;
};

// The categories largest first; ties keep the core's order.
const largestFirst = (totals: readonly CategoryTotal[]): readonly string[] =>
  totals.toSorted((first, second) => second.sizes.allocated - first.sizes.allocated).map(({ category }) => category);

// The category cards in a grid of one, two or three columns, as wide as its container allows. New cards enter
// staggered; a card that changes place glides there.
export const CategoryGrid = ({
  access,
  found,
  onOpen,
  onToggle,
  scanning,
  selected,
  totals,
  underApps,
}: CategoryGridProps): JSX.Element => {
  const order = useCoalescedOrder(largestFirst(totals), scanning);
  const flip = useFlipList(order);
  const byCategory = new Map(totals.map((total) => [total.category, total]));

  return (
    <div className={styles.container}>
      <ul aria-label={t('ui.category-grid.label')} className={styles.grid}>
        {order.map((category, index) => {
          const total = byCategory.get(category);

          return total === undefined ? null : (
            <li className={styles.cell} key={category} ref={flip(category)} style={{ '--i': index } as CSSProperties}>
              <CategoryCard
                access={access?.get(category)}
                onOpen={() => {
                  onOpen(category);
                }}
                onToggle={() => {
                  onToggle(category);
                }}
                selected={selected(category)}
                share={found.allocated > 0 ? total.sizes.allocated / found.allocated : 0}
                total={total}
                underApps={underApps?.get(category)}
              />
            </li>
          );
        })}
      </ul>
    </div>
  );
};
