import { type JSX, useId } from 'react';

import type { CategoryIcon, CategoryTotal } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Ellipsis } from '../../design/icons/icons';
import { Tile } from '../../design/layout/tile/tile';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Menu, type MenuItem } from '../../design/primitives/menu/menu';
import { t, tText } from '../../i18n';
import { CategoryIconBadge } from '../category-icon/category-icon';
import { honestSize } from '../sizes';
import styles from './category-tile.module.css';

// What the core's availability of the category says, in words: `limited` keeps the (partial) size and says what could
// not be seen; `unavailable` has no size and says why. Either way the text is shown on the tile.
export type CardAccess = { readonly kind: 'limited' | 'unavailable'; readonly text: string };

export type CategoryTileProps = {
  readonly access?: CardAccess | undefined;
  // The icon the core names for the category.
  readonly icon: CategoryIcon;
  // The options menu: what the user can do with the category besides cleaning it.
  readonly menu: readonly MenuItem[];
  // The menu open from the start, without taking the focus: for the gallery.
  readonly menuOpen?: boolean | undefined;
  readonly onDetails: () => void;
  // Selects the category whole; absent, the tile has no checkbox.
  readonly onToggle?: (() => void) | undefined;
  // While the scan runs: the tile shows what was found so far, without a checkbox, Details or menu.
  readonly scanning?: boolean | undefined;
  // Whether the whole category is picked for removal; `mixed` when only some of its findings are.
  readonly selected: 'mixed' | boolean;
  readonly total: CategoryTotal;
};

const textOf = (key: string): string => tText({ args: {}, key });

// One category the scan found, on a tile of its level: its icon, its name over its line, the checkbox; then its size,
// its count, Details and the options menu. An unavailable category has no checkbox and says why.
export const CategoryTile = ({
  access,
  icon,
  menu,
  menuOpen,
  onDetails,
  onToggle,
  scanning = false,
  selected,
  total,
}: CategoryTileProps): JSX.Element => {
  const nameId = useId();
  const unavailable = access?.kind === 'unavailable';
  const { category, count, highestDanger: level, sizes } = total;
  const name = textOf(`${category}.name`);

  return (
    <div
      className={styles.tile}
      data-card
      data-level={level}
      data-selected={selected === false ? undefined : selected}
      data-unavailable={unavailable ? true : undefined}
    >
      <Tile surface={level === 'medium' ? 'medium' : 'plain'} tone={level}>
        <div className={styles.top}>
          <CategoryIconBadge icon={icon} level={level} />
          <div className={styles.text}>
            <h3 className={styles.name} id={nameId}>
              {name}
            </h3>
            <span className={styles.line}>{textOf(`${category}.line`)}</span>
            {access === undefined ? null : <span className={styles.reason}>{access.text}</span>}
          </div>
          {unavailable || scanning || onToggle === undefined ? null : (
            <span className={styles.check}>
              <Checkbox
                checked={selected}
                hideLabel
                label={t('ui.category-card.select', { name: { kind: 'text', value: name } })}
                onChange={onToggle}
              />
            </span>
          )}
        </div>
        <div className={styles.bottom}>
          <span className={styles.size}>
            <SizeText {...honestSize(sizes)} size="title-3" />
          </span>
          <span className={styles.count}>
            {t('ui.category-card.count', { count: { kind: 'count', value: count } })}
          </span>
          {scanning ? null : (
            <>
              <span className={styles.spacer} />
              <span className={styles.details}>
                <Button aria-describedby={nameId} onClick={onDetails} size="small" variant="ghost">
                  {t('ui.category-card.details')}
                </Button>
              </span>
              <Menu
                defaultOpen={menuOpen}
                icon={Ellipsis}
                items={menu}
                label={t('ui.category-card.options', { name: { kind: 'text', value: name } })}
              />
            </>
          )}
        </div>
      </Tile>
    </div>
  );
};
