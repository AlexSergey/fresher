import type { Segment, Tone } from '../../design/data/segment';
import type { Sizes, SummaryKind, SummaryPart, VolumeMap } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import { honestSize, sizeLabel } from '../sizes';

export type DiskPart = {
  readonly description: string;
  readonly kind: SummaryKind;
  readonly name: string;
  readonly sizes: Sizes;
  readonly tone: Tone;
};

// The ring's order, from the top clockwise; Free is the track, not an arc.
const RING_ORDER: readonly SummaryKind[] = ['files-and-apps', 'mac-os', 'system-space', 'can-be-freed'];

const textOf = (key: string): string => tText({ args: {}, key });

const partOf = ({ kind, sizes }: SummaryPart): DiskPart => ({
  description: textOf(`core.storage.summary.${kind}.description`),
  kind,
  name: textOf(`core.storage.summary.${kind}.name`),
  sizes,
  tone: kind,
});

// The volume's five parts in the core's order: Can be freed, Your files and apps, macOS, System space, Free.
export const diskParts = (volume: VolumeMap): readonly DiskPart[] => volume.summary.map(partOf);

// The ring's arcs: every part but Free, in the ring's order.
export const ringSegments = (parts: readonly DiskPart[]): readonly Segment[] =>
  RING_ORDER.flatMap((kind) => {
    const part = parts.find((candidate) => candidate.kind === kind);

    return part === undefined
      ? []
      : [{ key: kind, label: part.name, size: sizeLabel(part.sizes), tone: kind, value: part.sizes.allocated }];
  });

// What the ring is called to assistive technology: every part with its honest size, Free included.
export const partsText = (parts: readonly DiskPart[]): string =>
  parts.map(({ name, sizes }) => `${name} ${honestSize(sizes).accessibleText}`).join(', ');
