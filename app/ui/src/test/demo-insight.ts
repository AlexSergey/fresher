import type { Dictionary, Insight, InsightPart } from '../ipc/bindings.generated';

// An insight as a module could describe one, with made-up texts: three figures, parts of three levels (two with a
// command to copy), two notes. The demo module does not produce it; the insight components' tests and stories do.

const text = (key: string): { args: Record<string, never>; key: string } => ({ args: {}, key });

const part = (name: string, size: number, danger: InsightPart['danger'], line: null | string): InsightPart => ({
  command: line === null ? null : { args: line.split(' ').slice(1), line, tool: 'demo-tool' },
  danger,
  detail: text(`demo.insight.${name}.detail`),
  effect: text(`demo.insight.${name}.effect`),
  label: text(`demo.insight.${name}.label`),
  size,
});

export const DEMO_INSIGHT: Insight = {
  figures: [
    { label: text('demo.insight.figure.maximum'), size: 68_719_476_736 },
    { label: text('demo.insight.figure.on-disk'), size: 21_474_836_480 },
    { label: text('demo.insight.figure.inside'), size: 9_663_676_416 },
  ],
  identity: 'demo-tool',
  notes: [text('demo.insight.note.first'), text('demo.insight.note.second')],
  parts: [
    part('cache', 4_294_967_296, 'safe', 'demo-tool cache prune --all'),
    part('images', 3_221_225_472, 'low', 'demo-tool image prune --all'),
    part('volumes', 1_073_741_824, 'high', null),
  ],
  summary: text('demo.insight.summary'),
  title: text('demo.insight.title'),
};

// The same, stopped: two figures, no part.
export const DEMO_INSIGHT_STOPPED: Insight = {
  figures: DEMO_INSIGHT.figures.slice(0, 2),
  identity: 'demo-tool-stopped',
  notes: [],
  parts: [],
  summary: text('demo.insight.stopped.summary'),
  title: text('demo.insight.stopped.title'),
};

// Nothing known: no figure, no part, no note.
export const DEMO_INSIGHT_UNKNOWN: Insight = {
  figures: [],
  identity: 'demo-tool-unknown',
  notes: [],
  parts: [],
  summary: text('demo.insight.unknown.summary'),
  title: text('demo.insight.unknown.title'),
};

export const DEMO_INSIGHT_DICTIONARY: Dictionary = {
  'core.danger.high.name': 'High',
  'core.danger.low.name': 'Low',
  'core.danger.medium.name': 'Medium',
  'core.danger.safe.name': 'Safe',
  'demo.insight.cache.detail': 'Made-up build leftovers of a made-up tool.',
  'demo.insight.cache.effect': 'The next build is slower; nothing else changes.',
  'demo.insight.cache.label': 'Build cache',
  'demo.insight.figure.inside': 'Used inside',
  'demo.insight.figure.maximum': 'Largest it can grow',
  'demo.insight.figure.on-disk': 'Takes on the disk',
  'demo.insight.images.detail': 'Made-up images no container uses.',
  'demo.insight.images.effect': 'They are downloaded again when needed.',
  'demo.insight.images.label': 'Unused images',
  'demo.insight.note.first': 'Freeing space inside does not shrink the file on the disk at once.',
  'demo.insight.note.second': 'Fresher never runs these commands. You run them yourself.',
  'demo.insight.stopped.summary': 'The made-up tool is not running, so what is inside is not known.',
  'demo.insight.stopped.title': 'Example tool (stopped)',
  'demo.insight.summary': 'Made-up data of a made-up tool: a disk file and what is inside it.',
  'demo.insight.title': 'Example tool',
  'demo.insight.unknown.summary': 'Nothing is known about it.',
  'demo.insight.unknown.title': 'Example tool (unknown)',
  'demo.insight.volumes.detail': 'Made-up volumes no container uses; they can hold your data.',
  'demo.insight.volumes.effect': 'Their contents are gone for good.',
  'demo.insight.volumes.label': 'Unused volumes',
};
