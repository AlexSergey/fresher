import type { MeaningGroup, Sizes } from '../../ipc/bindings.generated';

import { COMPLETE } from '../storage-map/story-data';

// The by-meaning groups of the meaning list's stories and tests: an app with a category and a data folder, a removed
// app, the demo categories, the Trash and two of your folders, then the sample volume's lines.

const GB = 1_000_000_000;

const sizes = (gigabytes: number): Sizes => ({
  allocated: gigabytes * GB,
  completeness: 'complete',
  logical: gigabytes * GB,
  reclaimable: gigabytes * GB,
});

export const MEANING: readonly MeaningGroup[] = [
  {
    kind: 'app',
    value: {
      app: { id: 'com.example.editor', installed: true, name: 'Example Editor' },
      categories: [{ category: 'demo.images', sizes: sizes(0.42) }],
      data: [
        { name: { kind: 'folder', value: 'com.example.editor' }, node: { index: 11, state: 1 }, sizes: sizes(3.1) },
      ],
      sizes: sizes(3.52),
    },
  },
  {
    kind: 'app',
    value: {
      app: { id: 'com.example.old', installed: false, name: 'Old Example' },
      categories: [],
      data: [{ name: { kind: 'folder', value: 'com.example.old' }, node: { index: 12, state: 1 }, sizes: sizes(1.4) }],
      sizes: sizes(1.4),
    },
  },
  { kind: 'category', value: { category: 'demo.caches', sizes: sizes(1.82) } },
  { kind: 'category', value: { category: 'demo.downloads', sizes: sizes(0.75) } },
  { kind: 'place', value: { name: { kind: 'trash' }, node: { index: 7, state: 1 }, sizes: sizes(4.2) } },
  {
    kind: 'place',
    value: { name: { kind: 'folder', value: 'Movies' }, node: { index: 8, state: 1 }, sizes: sizes(120) },
  },
  ...COMPLETE.parts
    .filter(({ kind }) => !['explained', 'your-folders'].includes(kind.kind))
    .map((part): MeaningGroup => ({ kind: 'part', value: part })),
];
