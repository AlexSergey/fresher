import type { Level } from '../../design/data/danger-badge/danger-badge';
import type { Segment, Tone } from '../../design/data/segment';
import type { CategoryTotal, MapPart, MeaningGroup, PartKind, RemainderReason } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { sizeLabel } from '../sizes';

const textOf = (key: string): string => tText({ args: {}, key });

// The key of a part's texts in the core's dictionary: `<key>.name` and `<key>.description`.
const keyOf = (kind: PartKind): string => {
  if (kind.kind === 'remainder') {
    return kind.outsideScan ? 'core.storage.remainder.outside' : 'core.storage.remainder.not-measured';
  }

  return `core.storage.part.${kind.kind}`;
};

// A part's name: the core's, or the other volume's own name.
export const partName = (kind: PartKind): string =>
  kind.kind === 'other-volume' && kind.name !== '' ? kind.name : textOf(`${keyOf(kind)}.name`);

// What a part is, in the core's words.
export const partDescription = (kind: PartKind): string => textOf(`${keyOf(kind)}.description`);

// A reason of the remainder, in the core's words.
export const reasonText = (reason: RemainderReason): string => textOf(`core.storage.reason.${reason.kind}`);

// The colour of a part: Explained by the highest danger level of its categories, every other part of a disk its own
// tone, what is not measured (or not yet) unknown, Free free.
export const partTone = (kind: PartKind, explained: Tone): Tone => {
  switch (kind.kind) {
    case 'disk-own':
    case 'mac-os':
    case 'other-volume':
    case 'swap-and-sleep':
      return kind.kind;
    case 'explained':
      return explained;
    case 'free':
      return 'free';
    case 'not-scanned-yet':
    case 'remainder':
      return 'unknown';
    case 'your-folders':
      return 'your-folders';
  }
};

const LEVELS: readonly Level[] = ['safe', 'low', 'medium', 'high'];

// The categories of a volume's by-meaning groups, its apps' among them.
const categoriesOf = (groups: readonly MeaningGroup[]): ReadonlySet<string> =>
  new Set(
    groups.flatMap((group) => {
      if (group.kind === 'category') {
        return [group.value.category];
      }

      return group.kind === 'app' ? group.value.categories.map(({ category }) => category) : [];
    }),
  );

// The tone of a volume's Explained: the highest danger level among its categories (`groups`, the core's by-meaning
// answer for it), or among every counted category while that answer has none (`null`, or a scan's, which holds only
// the volume's lines); `safe` when there are none.
export const explainedTone = (totals: readonly CategoryTotal[], groups: null | readonly MeaningGroup[]): Level => {
  const categories = groups === null ? null : categoriesOf(groups);
  const counted =
    categories === null || categories.size === 0 ? totals : totals.filter(({ category }) => categories.has(category));

  return LEVELS[Math.max(0, ...counted.map(({ highestDanger }) => LEVELS.indexOf(highestDanger)))] ?? 'safe';
};

// The parts the bar draws: every part with space, in the core's order.
export const segmentsOf = (parts: readonly MapPart[], explained: Tone): readonly Segment[] =>
  parts
    .filter(({ sizes }) => sizes.allocated > 0)
    .map(({ kind, sizes }, index) => ({
      key: `${kind.kind}-${String(index)}`,
      label: partName(kind),
      size: sizeLabel(sizes),
      tone: partTone(kind, explained),
      value: sizes.allocated,
    }));
