import type { DangerLevel, Dictionary, Finding } from '../ipc/bindings.generated';

import { DEMO_DICTIONARY, DEMO_FINDINGS } from './demo-scan';

// Findings and texts for the finding rows, the selection bar and the confirmation: every danger level with its word
// and sentence (as the core's dictionary has them), a long path, an external subject. Shared by the unit tests and
// the stories.

export const FINDING_DICTIONARY: Dictionary = {
  ...DEMO_DICTIONARY,
  'core.danger.high.meaning': 'May hold the only copy of your data. Removing it can lose work.',
  'core.danger.high.name': 'High',
  'core.danger.low.meaning': 'Comes back when needed, but that takes time or a download.',
  'core.danger.medium.meaning': 'Can be restored, but not by itself. You may need to set something up again.',
  'core.danger.medium.name': 'Medium',
  'core.danger.safe.meaning': 'Comes back by itself when needed. Removing it costs only a little time.',
  'demo.caches.if-removed': 'The apps rebuild their caches the next time they need them.',
  'demo.downloads.if-removed': 'You download the installers again if you need them.',
  'demo.images.if-removed': 'Docker pulls the image again the next time a container needs it.',
  'demo.images.name': 'Example images',
  'demo.projects.if-removed': 'The work in these folders is gone unless it has a copy elsewhere.',
  'demo.projects.name': 'Example projects',
};

const sized = (bytes: number): Finding['sizes'] => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// A demo finding at `level`, with its own id, name, size and category.
export const findingOf = ({
  bytes = 1_200_000_000,
  category = 'demo.caches',
  id,
  level,
  name,
}: {
  readonly bytes?: number;
  readonly category?: string;
  readonly id: number;
  readonly level: DangerLevel;
  readonly name: string;
}): Finding => ({
  category,
  danger: level,
  id,
  key: { identity: `/demo/${name}`, module: 'demo' },
  module: 'demo',
  reason: { args: {}, key: 'demo.finding.reason' },
  removal: { kind: 'move-to-trash', value: `/demo/${name}` },
  sizes: sized(bytes),
  subject: { kind: 'path', value: `/demo/${name}` },
  title: { args: { name: { kind: 'text', value: name } }, key: 'demo.finding.title' },
});

export const [SAFE_FINDING, LOW_FINDING] = DEMO_FINDINGS as readonly [Finding, Finding, Finding];

export const MEDIUM_FINDING = findingOf({ bytes: 420_000_000, id: 11, level: 'medium', name: 'simulator-runtime' });

export const HIGH_FINDING = findingOf({
  bytes: 2_300_000_000,
  category: 'demo.projects',
  id: 12,
  level: 'high',
  name: 'old-project',
});

export const LONG_PATH_FINDING: Finding = {
  ...findingOf({ bytes: 86_000_000, id: 13, level: 'safe', name: 'derived-data' }),
  subject: {
    kind: 'path',
    value:
      '/Users/someone/Library/Developer/Xcode/DerivedData/AVeryLongProjectName-abcdefghijklmnopqrstuvwxyz/Build/Intermediates.noindex/Index.noindex',
  },
};

export const EXTERNAL_FINDING: Finding = {
  ...findingOf({ bytes: 1_900_000_000, category: 'demo.images', id: 14, level: 'low', name: 'node-image' }),
  removal: { kind: 'tool-command', value: { args: ['image', 'rm', 'sha256:4f1c9a'], tool: 'docker' } },
  subject: { kind: 'external', value: { id: 'sha256:4f1c9a', tool: 'docker' } },
};

export const PARTIAL_FINDING: Finding = {
  ...findingOf({ bytes: 640_000_000, id: 15, level: 'low', name: 'mail-downloads' }),
  sizes: { allocated: 640_000_000, completeness: 'partial', logical: 640_000_000, reclaimable: 640_000_000 },
};
