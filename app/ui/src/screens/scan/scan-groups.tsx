import { type JSX, useId } from 'react';

import type { CategoryId, CategoryTotal } from '../../ipc/bindings.generated';
import type { CardAccess } from '../../product/category-tile/category-tile';
import type { Selection } from '../../state/selection';

import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { CategoryGrid } from '../../product/category-grid/category-grid';
import { iconOf } from '../../product/category-icon/icon-of';
import { BY_HAND, GROUP_ICONS } from '../../product/danger-zone/zones';
import { useNavigation } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { useCore, useCoreStore } from '../../state/store';
import { useIgnoreCommands } from '../settings/use-ignore-commands';
import { useModules } from '../use-modules';
import styles from './scan-groups.module.css';
import { type Zone, zoneSelectAll, zonesOf } from './zones';

export type ScanGroupsProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access: ReadonlyMap<CategoryId, CardAccess>;
  // The categories the core counted.
  readonly totals: readonly CategoryTotal[];
  // Zero totals of the unavailable categories that found nothing: they are tiles too, never absent.
  readonly unavailable: readonly CategoryTotal[];
};

type SelectAllProps = {
  readonly headingId: string;
  readonly onChoose: (categories: readonly CategoryId[], within: readonly CategoryId[]) => void;
  readonly selectable: (category: CategoryId) => boolean;
  readonly selection: Selection;
  readonly zone: Zone;
};

// A group's "Select all", at the end of its heading: ticks every selectable category of the group, or unticks them
// all; what it does is said on demand.
const SelectAll = ({ headingId, onChoose, selectable, selection, zone }: SelectAllProps): JSX.Element | null => {
  const all = zoneSelectAll(selection, zone, selectable);

  if (all === null) {
    return null;
  }

  return (
    <span className={styles.select}>
      <Tooltip content={t('ui.danger-zone.select-all.description')}>
        {(describedBy) => (
          <Checkbox
            aria-describedby={describedBy === undefined ? headingId : `${describedBy} ${headingId}`}
            checked={all.checked}
            label={t('ui.danger-zone.select-all')}
            onChange={() => {
              onChoose(all.checked === true ? [] : all.categories, all.categories);
            }}
          />
        )}
      </Tooltip>
    </span>
  );
};

type GroupProps = Omit<SelectAllProps, 'headingId'> & {
  readonly children: JSX.Element;
};

// One level's group: its icon, its name and its line, then its tiles. Only the groups whose items may go whole have a
// "Select all".
const Group = ({ children, onChoose, selectable, selection, zone }: GroupProps): JSX.Element => {
  const headingId = useId();
  const { level } = zone;
  const Shape = GROUP_ICONS[level];

  return (
    <section aria-labelledby={headingId} className={styles.group} data-level={level}>
      <div className={styles.head}>
        <span aria-hidden="true" className={styles.icon}>
          <Shape color="currentColor" focusable="false" size={16} strokeWidth={2.2} />
        </span>
        <h2 className={styles.name} id={headingId}>
          {tText({ args: {}, key: `core.danger.${level}.zone` })}
        </h2>
        <span className={styles.line}>
          {t('ui.screen.scan.group-line', {
            line: { kind: 'text', value: tText({ args: {}, key: `core.danger.${level}.meaning` }) },
          })}
        </span>
        {BY_HAND.has(level) ? null : (
          <SelectAll
            headingId={headingId}
            onChoose={onChoose}
            selectable={selectable}
            selection={selection}
            zone={zone}
          />
        )}
      </div>
      {children}
    </section>
  );
};

// The category tiles of a finished scan in their groups, the safest first.
// Nothing is selected until the user ticks it; a "Look before removing" tile ticked here only brings its category to
// Review, where its items are ticked one by one (`Selection.toCheck`): its tile reads checked, mixed once some are.
export const ScanGroups = ({ access, totals, unavailable }: ScanGroupsProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const modules = useModules();
  const selection = useCore((state) => state.selection);
  const { ignore } = useIgnoreCommands();

  const choose = (categories: readonly CategoryId[], within: readonly CategoryId[]): void => {
    store.dispatch({ categories, type: 'choose', within });
  };

  const tiles = [
    ...totals,
    ...unavailable.filter(({ category }) => !totals.some((total) => total.category === category)),
  ];
  const selectable = (category: CategoryId): boolean => access.get(category)?.kind !== 'unavailable';

  return (
    <>
      {zonesOf(tiles).map((zone) => (
        <Group key={zone.level} onChoose={choose} selectable={selectable} selection={selection} zone={zone}>
          <CategoryGrid
            access={access}
            icon={(category) => iconOf(modules, category)}
            label={tText({ args: {}, key: `core.danger.${zone.level}.zone` })}
            onDetails={(category): void => {
              navigation.navigate({ category, kind: 'category' });
            }}
            onIgnore={ignore}
            onToggle={(category) => {
              store.dispatch({ category, type: 'toggle-category' });
            }}
            scanning={false}
            selected={(category) => checkedOf(selection, category)}
            totals={zone.totals}
          />
        </Group>
      ))}
    </>
  );
};
