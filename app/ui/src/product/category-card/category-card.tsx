import { type CSSProperties, type JSX, useId } from 'react';

import type { CategoryTotal } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { useSpotlightProps } from '../../design/effects/spotlight/spotlight';
import { Icon } from '../../design/icons/icon';
import { Settings } from '../../design/icons/icons';
import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Menu, type MenuItem } from '../../design/primitives/menu/menu';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { LEVEL_ICONS } from '../danger-zone/zones';
import { honestSize } from '../sizes';
import styles from './category-card.module.css';

// What the core's availability of the category says, in words: `limited` keeps the (partial) size and says what could
// not be seen; `unavailable` has no size and says why. Either way the text is shown on the card.
export type CardAccess = { readonly kind: 'limited' | 'unavailable'; readonly text: string };

export type CategoryCardProps = {
  readonly access?: CardAccess | undefined;
  // The settings menu: what the user can do with the category besides cleaning it.
  readonly menu: readonly MenuItem[];
  // The menu open from the start, without taking the focus: for the gallery.
  readonly menuOpen?: boolean | undefined;
  readonly onDetails: () => void;
  readonly onToggle: () => void;
  // Whether the whole category is picked for removal; `mixed` when only some of its findings are.
  readonly selected: 'mixed' | boolean;
  // The category's part of everything found, 0 to 1.
  readonly share: number;
  readonly total: CategoryTotal;
};

const textOf = (key: string): string => tText({ args: {}, key });

const clamp = (share: number): number => Math.min(Math.max(share, 0), 1);

// One category the scan found: its level's icon, its name and what it is, its level, size and count, and a bar of its
// share along the bottom. The body is static: Details opens the category, the menu ignores it, the checkbox in the
// corner selects it whole.
export const CategoryCard = ({
  access,
  menu,
  menuOpen,
  onDetails,
  onToggle,
  selected,
  share,
  total,
}: CategoryCardProps): JSX.Element => {
  const spotlight = useSpotlightProps();
  const nameId = useId();
  const unavailable = access?.kind === 'unavailable';
  const { category, count, highestDanger: level, sizes } = total;
  const name = textOf(`${category}.name`);
  const what = textOf(`${category}.what`);
  const drawn = clamp(share);

  return (
    <div
      className={styles.card}
      data-card
      data-level={level}
      data-selected={selected === false ? undefined : selected}
      data-unavailable={unavailable ? true : undefined}
    >
      <Surface
        className={`${spotlight.className} ${styles.surface ?? ''}`}
        elevation={1}
        padding="none"
        ref={spotlight.ref}
      >
        <div className={styles.body}>
          <span className={styles.icon}>
            <Icon icon={LEVEL_ICONS[level]} size={20} />
          </span>
          <div className={styles.text}>
            <h3 className={styles.name} id={nameId}>
              {name}
            </h3>
            <Tooltip content={what} repeatsName>
              {() => <span className={styles.what}>{what}</span>}
            </Tooltip>
            <span className={styles.meta}>
              <DangerBadge label={textOf(`core.danger.${level}.name`)} level={level} size="small" />
              <span className={styles.figures}>
                <SizeText {...honestSize(sizes)} size="title-3" />
                <span className={styles.count}>
                  {t('ui.category-card.count', { count: { kind: 'count', value: count } })}
                </span>
              </span>
            </span>
            {access === undefined ? null : <span className={styles.reason}>{access.text}</span>}
          </div>
        </div>
        {unavailable ? null : (
          <span className={styles.check}>
            <Checkbox
              checked={selected}
              hideLabel
              label={t('ui.category-card.select', { name: { kind: 'text', value: name } })}
              onChange={onToggle}
            />
          </span>
        )}
        <div className={styles.actions}>
          <Button aria-describedby={nameId} onClick={onDetails} size="small" variant="ghost">
            {t('ui.category-card.details')}
          </Button>
          <Menu
            defaultOpen={menuOpen}
            icon={Settings}
            items={menu}
            label={t('ui.category-card.options', { name: { kind: 'text', value: name } })}
          />
        </div>
        {drawn > 0 ? (
          <span aria-hidden="true" className={styles.share} data-share style={{ '--share': drawn } as CSSProperties} />
        ) : null}
      </Surface>
    </div>
  );
};
