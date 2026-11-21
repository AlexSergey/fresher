import { type JSX, useCallback } from 'react';

import type { CategoryId, CategoryTotal, Sizes } from '../../ipc/bindings.generated';
import type { CardAccess } from '../../product/category-card/category-card';

import { tText } from '../../i18n';
import { CategoryGrid } from '../../product/category-grid/category-grid';
import { DangerZone } from '../../product/danger-zone/danger-zone';
import { useNavigation } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { useCore, useCoreStore } from '../../state/store';
import { useIgnoreCommands } from '../settings/use-ignore-commands';
import { moduleOfCategory, useModules } from '../use-modules';
import { zoneSelectAll, zonesOf } from './zones';

export type ScanZonesProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access: ReadonlyMap<CategoryId, CardAccess>;
  readonly found: Sizes;
  readonly scanning: boolean;
  // The categories the core counted.
  readonly totals: readonly CategoryTotal[];
  // Zero totals of the unavailable categories that found nothing: they are cards too, never absent.
  readonly unavailable: readonly CategoryTotal[];
};

// The category cards in their danger zones, the safest first, each zone with its "Select all".
export const ScanZones = ({ access, found, scanning, totals, unavailable }: ScanZonesProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const modules = useModules();
  const selection = useCore((state) => state.selection);
  const dictionary = useCore((state) => state.dictionary);
  const { ignore } = useIgnoreCommands();
  const moduleOf = useCallback(
    (category: CategoryId): null | { readonly id: string; readonly name: string } => {
      const module = moduleOfCategory(modules, category);

      return module === undefined
        ? null
        : { id: module.id, name: tText({ args: {}, key: `${module.id}.enabled.label` }) };
    },
    // The module's name is the core's text: it is read again when the dictionary arrives.
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- the dictionary is read through `tText`
    [modules, dictionary],
  );

  const cards = [
    ...totals,
    ...unavailable.filter(({ category }) => !totals.some((total) => total.category === category)),
  ];

  return (
    <>
      {zonesOf(cards).map((zone) => {
        const all = zoneSelectAll(selection, zone, (category) => access.get(category)?.kind !== 'unavailable');
        const name = tText({ args: {}, key: `core.danger.${zone.level}.zone` });

        return (
          <DangerZone
            key={zone.level}
            level={zone.level}
            selectAll={
              all === null
                ? undefined
                : {
                    checked: all.checked,
                    onChange: (): void => {
                      store.dispatch({
                        categories: all.checked === true ? [] : all.categories,
                        type: 'choose',
                        within: all.categories,
                      });
                    },
                  }
            }
          >
            <CategoryGrid
              access={access}
              found={found}
              label={name}
              moduleOf={moduleOf}
              onDetails={(category): void => {
                navigation.navigate({ category, kind: 'category' });
              }}
              onIgnore={ignore}
              onToggle={(category) => {
                store.dispatch({ category, type: 'toggle-category' });
              }}
              scanning={scanning}
              selected={(category) => checkedOf(selection, category)}
              totals={zone.totals}
            />
          </DangerZone>
        );
      })}
    </>
  );
};
