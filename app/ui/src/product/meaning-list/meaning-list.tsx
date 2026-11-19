import type { JSX } from 'react';

import type {
  CategoryId,
  CategoryTotal,
  MapPart,
  MeaningGroup,
  NodeId,
  PlaceShare,
  Refusal,
  Sizes,
} from '../../ipc/bindings.generated';
import type { CardAccess } from '../category-card/category-card';

import { Notice } from '../../design/feedback/notice/notice';
import { t, tText } from '../../i18n';
import { CategoryGrid } from '../category-grid/category-grid';
import { AppRow } from './app-row';
import styles from './meaning-list.module.css';
import { PartRow, PlaceRow } from './rows';

export type MeaningListProps = {
  // The categories that are limited or unavailable and what that says; a category not in it is ready.
  readonly access?: ReadonlyMap<CategoryId, CardAccess> | undefined;
  // Why the core's groups could not be had; the cards then come from the totals, as before an answer.
  readonly failure: MeaningFailure | null;
  // Everything found: the cards' shares are of this.
  readonly found: Sizes;
  // The core's groups for the volume, in their order; `null` before it answered (the cards still show).
  readonly groups: null | readonly MeaningGroup[];
  readonly onOpen: (category: CategoryId) => void;
  // Opens the place view at a folder.
  readonly onOpenPlace: (node: NodeId) => void;
  readonly onToggle: (category: CategoryId) => void;
  readonly scanning: boolean;
  readonly selected: (category: CategoryId) => 'mixed' | boolean;
  // The categories the core counted: the cards before the core's groups arrive, and the count and the danger level of
  // each card after.
  readonly totals: readonly CategoryTotal[];
  // Zero totals of the unavailable categories that found nothing: they are cards after the sized ones, never absent.
  readonly unavailable?: readonly CategoryTotal[];
};

type AppGroup = Extract<MeaningGroup, { readonly kind: 'app' }>['value'];

// Why the core's groups could not be had: the core refused with its text, or could not be reached (`Failure` of
// `ipc/`).
type MeaningFailure = { readonly kind: 'refused'; readonly refusal: Refusal } | { readonly kind: 'unreachable' };

const keyOf = (node: NodeId): string => `${String(node.state)}-${String(node.index)}`;

type Cards = {
  readonly totals: readonly CategoryTotal[];
  readonly underApps: ReadonlyMap<CategoryId, Sizes>;
};

const NO_TOTALS: readonly CategoryTotal[] = [];

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

const COMPLETENESS_ORDER: readonly Sizes['completeness'][] = ['complete', 'partial', 'unknown'];

// Two of the core's shares as one, by the core's rule: the bytes add up and the weakest completeness wins.
const plus = (a: Sizes, b: Sizes): Sizes => ({
  allocated: a.allocated + b.allocated,
  completeness:
    COMPLETENESS_ORDER.indexOf(a.completeness) >= COMPLETENESS_ORDER.indexOf(b.completeness)
      ? a.completeness
      : b.completeness,
  logical: a.logical + b.logical,
  reclaimable: a.reclaimable + b.reclaimable,
});

// The cards: before the core's groups, and while they hold only the volume's lines (a scan's answer), every counted
// category; after, every counted category the core lists for the volume, as a category or under an app. A card's size
// is the core's share outside the apps (so the leaves add up); what sits under apps is told apart, since the card's
// count and selection take the whole category. A category the totals do not know has neither count nor danger level,
// so it gets no card.
const cardsOf = (groups: null | readonly MeaningGroup[], totals: readonly CategoryTotal[]): Cards => {
  if (groups === null || groups.every(({ kind }) => kind === 'part')) {
    return { totals, underApps: new Map() };
  }
  const own = new Map<CategoryId, Sizes>();
  const underApps = new Map<CategoryId, Sizes>();

  for (const group of groups) {
    if (group.kind === 'category') {
      own.set(group.value.category, group.value.sizes);
    } else if (group.kind === 'app') {
      for (const { category, sizes } of group.value.categories) {
        underApps.set(category, plus(underApps.get(category) ?? NOTHING, sizes));
      }
    }
  }

  return {
    totals: totals.flatMap((total) =>
      own.has(total.category) || underApps.has(total.category)
        ? [{ ...total, sizes: own.get(total.category) ?? NOTHING }]
        : [],
    ),
    underApps,
  };
};

const failureText = (failure: MeaningFailure): string =>
  failure.kind === 'refused' ? tText(failure.refusal.text) : t('ui.core.unreachable');

// Where the space went, by what it is: the apps first, then the categories as cards (selectable, as ever), then the
// places of your folders, then the rest of the disk. Only the cards offer a cleanup.
export const MeaningList = ({
  access,
  failure,
  found,
  groups,
  onOpen,
  onOpenPlace,
  onToggle,
  scanning,
  selected,
  totals,
  unavailable = NO_TOTALS,
}: MeaningListProps): JSX.Element => {
  const apps: AppGroup[] = [];
  const places: PlaceShare[] = [];
  const parts: MapPart[] = [];

  for (const group of groups ?? []) {
    if (group.kind === 'app') {
      apps.push(group.value);
    } else if (group.kind === 'place') {
      places.push(group.value);
    } else if (group.kind === 'part' && group.value.sizes.allocated > 0) {
      parts.push(group.value);
    }
  }

  const counted = cardsOf(groups, totals);
  const cards = {
    ...counted,
    totals: [
      ...counted.totals,
      ...unavailable.filter(({ category }) => !counted.totals.some((total) => total.category === category)),
    ],
  };

  return (
    <div className={styles.list}>
      {failure === null ? null : (
        <Notice title={t('ui.meaning-list.failed')} tone="warning">
          {failureText(failure)}
        </Notice>
      )}
      {apps.length > 0 ? (
        <div aria-label={t('ui.meaning-list.apps')} className={styles.rows} role="group">
          {apps.map(({ app, categories, data, sizes }) => (
            <AppRow
              app={app}
              categories={categories}
              data={data}
              key={app.id}
              onOpen={onOpen}
              onOpenPlace={onOpenPlace}
              sizes={sizes}
            />
          ))}
        </div>
      ) : null}
      {cards.totals.length > 0 ? (
        <CategoryGrid
          access={access}
          found={found}
          onOpen={onOpen}
          onToggle={onToggle}
          scanning={scanning}
          selected={selected}
          totals={cards.totals}
          underApps={cards.underApps}
        />
      ) : null}
      {places.length > 0 ? (
        <div aria-label={t('ui.meaning-list.places')} className={styles.rows} role="list">
          {places.map((place) => (
            <PlaceRow
              key={keyOf(place.node)}
              onOpen={() => {
                onOpenPlace(place.node);
              }}
              place={place}
            />
          ))}
        </div>
      ) : null}
      {parts.length > 0 ? (
        <div aria-label={t('ui.meaning-list.rest')} className={styles.rows} role="list">
          {parts.map((part, index) => (
            <PartRow key={`${part.kind.kind}-${String(index)}`} part={part} />
          ))}
        </div>
      ) : null}
    </div>
  );
};
