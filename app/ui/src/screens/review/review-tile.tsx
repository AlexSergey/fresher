import { type CSSProperties, type JSX, useId } from 'react';

import type { CategoryIcon, CategoryId, Finding } from '../../ipc/bindings.generated';
import type { ReviewGroup } from './review-groups';

import { SizeText } from '../../design/data/size-text/size-text';
import { Check } from '../../design/icons/icons';
import { Tile } from '../../design/layout/tile/tile';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { CategoryIconBadge } from '../../product/category-icon/category-icon';
import { BY_HAND } from '../../product/danger-zone/zones';
import { FindingList } from '../../product/finding-list/finding-list';
import { levelMeaning } from '../../product/finding-row/texts';
import { honestSize } from '../../product/sizes';
import { useFindings } from '../../state/use-findings';
import { useRowMenu } from '../use-row-menu';
import styles from './review-tile.module.css';

export type ReviewTileProps = {
  readonly group: ReviewGroup;
  // The home folder the core reports: the items' paths are shown from `~`.
  readonly home: null | string;
  readonly icon: CategoryIcon;
  // The all-checkbox: the user ticks all of the category (`true`), or unticks all of it.
  readonly onAll: (category: CategoryId, ticked: boolean) => void;
  readonly onToggle: (finding: Finding) => void;
};

type ListProps = {
  readonly home: null | string;
  readonly label: string;
  readonly onToggle: (finding: Finding) => void;
};

// A tile's list shows at most this many rows at once; it scrolls past them.
const VISIBLE_ROWS = 6;

const NOTHING_TO_LOAD = (): void => undefined;

// The list's frame, as tall as its rows (up to `VISIBLE_ROWS`).
const ListFrame = ({ children, rows }: { readonly children: JSX.Element; readonly rows: number }): JSX.Element => (
  <div className={styles.list} style={{ '--review-rows': Math.min(rows, VISIBLE_ROWS) } as CSSProperties}>
    {children}
  </div>
);

// All of a category's findings, largest first, paged by the core, each ticked as `isSelected` says: a selected
// category's are ticked but those ticked off inside it; a category to check's only those ticked one by one.
const CategoryRows = ({
  category,
  home,
  isSelected,
  label,
  listed,
  onToggle,
}: ListProps & {
  readonly category: string;
  readonly isSelected: (finding: Finding) => boolean;
  readonly listed: number;
}): JSX.Element => {
  const rows = useFindings({ category, module: null, order: 'largest' });
  const { menuOf, onReveal } = useRowMenu();

  return (
    <ListFrame rows={listed}>
      <FindingList
        failure={rows.failure}
        home={home}
        isSelected={isSelected}
        label={label}
        menuOf={menuOf}
        onRangeChange={rows.onRangeChange}
        onReveal={onReveal}
        onToggle={onToggle}
        rowAt={rows.rowAt}
        // A shown group is never empty: no rows yet means its first page has not answered.
        total={rows.total === 0 && rows.failure === null ? undefined : rows.total}
        variant="review"
      />
    </ListFrame>
  );
};

// Findings picked one by one: listed as the selection keeps them, all ticked; unticking one takes it off the list.
const PickedRows = ({
  findings,
  home,
  label,
  onToggle,
}: ListProps & { readonly findings: readonly Finding[] }): JSX.Element => {
  const { menuOf, onReveal } = useRowMenu();

  return (
    <ListFrame rows={findings.length}>
      <FindingList
        failure={null}
        home={home}
        isSelected={() => true}
        label={label}
        menuOf={menuOf}
        onRangeChange={NOTHING_TO_LOAD}
        onReveal={onReveal}
        onToggle={onToggle}
        rowAt={(index) => findings[index]}
        total={findings.length}
        variant="review"
      />
    </ListFrame>
  );
};

// The all-checkbox's state: checked when every item of the category goes, mixed when some do. Findings picked one by
// one outside a category to check (a Largest pick, or a row ticked on a category's details) have none: their
// category may hold far more than the user ever saw, so it is never ticked whole from here.
const allOf = (rows: ReviewGroup['rows']): 'mixed' | boolean | null => {
  switch (rows.kind) {
    case 'category':
      return rows.excluded.size === 0 ? true : 'mixed';
    case 'picked':
      return null;
    case 'to-check':
      return rows.ticked.size === 0 ? false : rows.ticked.size >= rows.listed ? true : 'mixed';
  }
};

// Which rows are ticked: a selected category's but those ticked off inside it; a category to check's only those
// ticked one by one.
const isSelectedOf = (
  rows: Exclude<ReviewGroup['rows'], { readonly kind: 'picked' }>,
): ((finding: Finding) => boolean) =>
  rows.kind === 'category'
    ? (finding: Finding): boolean => !rows.excluded.has(finding.id)
    : (finding: Finding): boolean => rows.ticked.has(finding.id);

// The tile's line: its group and what the group means; a "Look before removing" tile of a category asks for each item
// to be ticked. Picked items were ticked one by one already, so their tile only says what the group means.
const lineOf = (group: ReviewGroup): string => {
  const name = { kind: 'text', value: tText({ args: {}, key: `core.danger.${group.highest}.zone` }) } as const;

  return BY_HAND.has(group.highest) && group.rows.kind !== 'picked'
    ? t('ui.screen.review.check-each', { group: name })
    : t('ui.screen.review.line', { group: name, meaning: { kind: 'text', value: levelMeaning(group.highest) } });
};

// One category of the selection on a tile of its group's level: its all-checkbox, icon, name and line, how many of its
// items are checked (green when all are; picked items say how many were picked), its size, and its rows, each with
// its own checkbox.
export const ReviewTile = ({ group, home, icon, onAll, onToggle }: ReviewTileProps): JSX.Element => {
  const nameId = useId();
  const name = tText({ args: {}, key: `${group.category}.name` });
  const label = t('ui.screen.review.items', { category: { kind: 'text', value: name } });
  const { rows } = group;
  const all = allOf(rows);
  const look = BY_HAND.has(group.highest);
  const allChecked = group.unchecked === 0;

  return (
    <Tile as="article" labelledBy={nameId} padding="none" surface={look ? 'medium' : 'plain'} tone={group.highest}>
      <div className={styles.header} data-look={look ? true : undefined}>
        {all === null ? (
          // The checkbox's place stays empty, so the icons of the tiles line up.
          <span aria-hidden="true" className={styles.noCheckbox} />
        ) : (
          <Checkbox
            checked={all}
            hideLabel
            label={t('ui.screen.review.all', { name: { kind: 'text', value: name } })}
            onChange={() => {
              onAll(group.category, all !== true);
            }}
          />
        )}
        <CategoryIconBadge icon={icon} level={group.highest} size="small" />
        <div className={styles.text}>
          <h2 className={styles.name} id={nameId}>
            {name}
          </h2>
          <span className={styles.line}>{lineOf(group)}</span>
        </div>
        <span className={styles.pill} data-all={allChecked ? true : undefined}>
          {allChecked ? <Check aria-hidden color="currentColor" focusable="false" size={11} strokeWidth={2.5} /> : null}
          {rows.kind === 'picked'
            ? t('ui.screen.review.picked', { count: { kind: 'count', value: group.count } })
            : t('ui.screen.review.checked', {
                checked: { kind: 'count', value: group.count },
                count: { kind: 'count', value: group.count + group.unchecked },
              })}
        </span>
        <span className={styles.size}>
          <SizeText {...honestSize(group.sizes)} />
        </span>
      </div>
      <div className={styles.items} data-look={look ? true : undefined}>
        {rows.kind === 'picked' ? (
          <PickedRows findings={rows.findings} home={home} label={label} onToggle={onToggle} />
        ) : (
          <CategoryRows
            category={group.category}
            home={home}
            isSelected={isSelectedOf(rows)}
            label={label}
            listed={rows.listed}
            onToggle={onToggle}
          />
        )}
      </div>
    </Tile>
  );
};
