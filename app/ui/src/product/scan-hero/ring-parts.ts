import type { Level } from '../../design/data/danger-badge/danger-badge';
import type { Segment, Tone } from '../../design/data/segment';
import type { CategoryTotal, DangerLevel, MapPart, MeaningGroup, Sizes, VolumeMap } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { addSizes, sizeLabel } from '../sizes';
import { partName, partTone } from '../storage-map/parts';

// What the legend's "i" shows: a level's (Explained's description and the level's meaning) or a part's.
export type RingAbout =
  { readonly kind: 'level'; readonly level: DangerLevel } | { readonly kind: 'part'; readonly part: MapPart };

export type RingPart = {
  readonly about: RingAbout;
  readonly key: string; // `level-safe` … `level-high`, `<part kind>-<index>`
  readonly label: string; // core.danger.<level>.zone, or the part's name
  readonly sizes: Sizes;
  readonly tone: Tone;
};

const LEVELS: readonly Level[] = ['safe', 'low', 'medium', 'high'];

// The category shares of by-meaning groups, the apps' among them.
const sharesOf = (groups: readonly MeaningGroup[]): readonly { category: string; sizes: Sizes }[] =>
  groups.flatMap((group) => {
    if (group.kind === 'category') {
      return [group.value];
    }

    return group.kind === 'app' ? group.value.categories : [];
  });

// The cleanable bytes of the volume by danger level, from the core's answers only: its category shares, each to the
// level of its category's total (a category the totals do not know is not cleanable and is left out); while the
// groups hold no shares (no answer yet, a scan's, a failed query) the totals themselves.
export const levelShares = (
  groups: null | readonly MeaningGroup[],
  totals: readonly CategoryTotal[],
): ReadonlyMap<DangerLevel, Sizes> => {
  const shares = groups === null ? [] : sharesOf(groups);
  const counted =
    groups === null || groups.every(({ kind }) => kind === 'part')
      ? totals.map(({ highestDanger, sizes }) => ({ level: highestDanger, sizes }))
      : shares.flatMap(({ category, sizes }) => {
          const total = totals.find((candidate) => candidate.category === category);

          return total === undefined ? [] : [{ level: total.highestDanger, sizes }];
        });
  const levels = new Map<DangerLevel, Sizes>();

  for (const { level, sizes } of counted) {
    const sum = levels.get(level);

    levels.set(level, sum === undefined ? sizes : addSizes(sum, sizes));
  }

  return levels;
};

// The ring's parts: the levels with something in them, Safe to High, then the volume's other parts in the core's
// order (Explained stands for its levels; a part of zero size is dropped), Free last.
export const ringParts = (volume: VolumeMap, levels: ReadonlyMap<DangerLevel, Sizes>): readonly RingPart[] => [
  ...LEVELS.flatMap((level) => {
    const sizes = levels.get(level);

    return sizes === undefined || sizes.allocated <= 0
      ? []
      : [
          {
            about: { kind: 'level', level } as const,
            key: `level-${level}`,
            label: tText({ args: {}, key: `core.danger.${level}.zone` }),
            sizes,
            tone: level,
          },
        ];
  }),
  ...volume.parts.flatMap((part, index) =>
    part.kind.kind === 'explained' || part.sizes.allocated <= 0
      ? []
      : [
          {
            about: { kind: 'part', part } as const,
            key: `${part.kind.kind}-${String(index)}`,
            label: partName(part.kind),
            sizes: part.sizes,
            // Explained is never drawn as a part (its levels are), so the tone passed for it is unused.
            tone: partTone(part.kind, 'safe'),
          },
        ],
  ),
];

export const ringSegments = (parts: readonly RingPart[]): readonly Segment[] =>
  parts.map(({ key, label, sizes, tone }) => ({
    key,
    label,
    size: sizeLabel(sizes),
    tone,
    value: sizes.allocated,
  }));
