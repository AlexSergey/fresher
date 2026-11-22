import type { AppSummary, AppsView, Dictionary, NotOffered, Sizes, Text } from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };
import appCachesTexts from '../../../../modules/app-caches/locales/en.json' with { type: 'json' };
import { DEMO_DICTIONARY } from './demo-scan';

// The Apps list as the core answers it for a home of a few apps; the texts are the core's own, read from its locale
// file. Only what the core produces together: a not-offered app carries its reason and text, the sizes of its parts
// add up to its own, the list comes largest first. Shared by the unit tests, the gallery and the e2e mock.

const MB = 1_000_000;
const GB = 1_000 * MB;

const STATE = 1;

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const APPS_KEYS = [
  'core.apps.',
  'core.uninstall',
  'core.access.app-management.',
  'core.error.uninstall-refused',
  'core.error.unknown-app',
];

export const APPS_DICTIONARY: Dictionary = {
  ...DEMO_DICTIONARY,
  ...Object.fromEntries(
    Object.entries(coreTexts as Dictionary).filter(([key]) => APPS_KEYS.some((prefix) => key.startsWith(prefix))),
  ),
  'app-caches.library-caches.name': (appCachesTexts as Dictionary)['app-caches.library-caches.name'] as string,
};

const text = (key: string, args: Text['args'] = {}): Text => ({ args, key });

const name = (value: string): Text['args'] => ({ app: { kind: 'text', value } });

const USED = text('core.apps.summary.used', {
  app: { kind: 'text', value: 'Video Studio' },
  size: { kind: 'size', value: 4 * GB },
});

// An app used lately, offered.
export const VIDEO_STUDIO: AppSummary = {
  app: { index: 0, state: STATE },
  lastUsed: 1_735_689_600,
  name: 'Video Studio',
  offer: { kind: 'offered' },
  parts: [
    { kind: 'bundle', sizes: sized(3_000 * MB) },
    { kind: 'caches', sizes: sized(420 * MB) },
    { kind: 'logs', sizes: sized(30 * MB) },
    { kind: 'saved-state', sizes: sized(20 * MB) },
    { kind: 'launch-agents', sizes: sized(2 * MB) },
    { kind: 'web-data', sizes: sized(18 * MB) },
    { kind: 'settings', sizes: sized(5 * MB) },
    { kind: 'support', sizes: sized(400 * MB) },
    { kind: 'container', sizes: sized(55 * MB) },
    { kind: 'group-container', sizes: sized(50 * MB) },
  ],
  sizes: sized(4 * GB),
  summary: USED,
  version: '14.2',
};

// An app not used for a year, offered.
export const OLD_EDITOR: AppSummary = {
  app: { index: 1, state: STATE },
  lastUsed: 1_672_531_200,
  name: 'Old Editor',
  offer: { kind: 'offered' },
  parts: [
    { kind: 'bundle', sizes: sized(800 * MB) },
    { kind: 'settings', sizes: sized(2 * MB) },
  ],
  sizes: sized(802 * MB),
  summary: text('core.apps.summary.unused', {
    ...name('Old Editor'),
    date: { kind: 'date', value: 1_672_531_200 },
    size: { kind: 'size', value: 802 * MB },
  }),
  version: null,
};

// An app a package manager installed: listed, not offered.
export const MANAGED: AppSummary = {
  app: { index: 2, state: STATE },
  lastUsed: null,
  name: 'Terminal Pro',
  offer: {
    kind: 'not-offered',
    reason: 'installed-by',
    text: text('core.apps.not-offered.installed-by', { manager: { kind: 'text', value: 'Homebrew' } }),
  },
  parts: [{ kind: 'bundle', sizes: sized(300 * MB) }],
  sizes: sized(300 * MB),
  summary: text('core.apps.summary.unknown', { ...name('Terminal Pro'), size: { kind: 'size', value: 300 * MB } }),
  version: '1.0',
};

// An app that is open: listed, not offered.
export const OPEN_NOW: AppSummary = {
  app: { index: 3, state: STATE },
  lastUsed: 1_735_689_600,
  name: 'Chat',
  offer: { kind: 'not-offered', reason: 'open', text: text('core.apps.not-offered.open') },
  parts: [{ kind: 'bundle', sizes: sized(200 * MB) }],
  sizes: sized(200 * MB),
  summary: text('core.apps.summary.used', { ...name('Chat'), size: { kind: 'size', value: 200 * MB } }),
  version: '5.1',
};

const MANAGER_TEXT: Readonly<Record<NotOffered, Text>> = {
  'installed-by': text('core.apps.not-offered.installed-by', { manager: { kind: 'text', value: 'Homebrew' } }),
  locked: text('core.apps.not-offered.locked'),
  'needs-administrator': text('core.apps.not-offered.needs-administrator'),
  open: text('core.apps.not-offered.open'),
  protected: text('core.apps.not-offered.protected'),
  'too-many-files': text('core.apps.not-offered.too-many-files'),
  unmeasured: text('core.apps.not-offered.unmeasured'),
};

// A small app listed, not offered for `reason`, with the core's sentence for it.
export const notOfferedApp = (reason: NotOffered, index: number, appName: string): AppSummary => ({
  app: { index, state: STATE },
  lastUsed: 1_735_689_600,
  name: appName,
  offer: { kind: 'not-offered', reason, text: MANAGER_TEXT[reason] },
  parts: [{ kind: 'bundle', sizes: sized(150 * MB) }],
  sizes: sized(150 * MB),
  summary: text('core.apps.summary.used', { ...name(appName), size: { kind: 'size', value: 150 * MB } }),
  version: '2.0',
});

// An app that needs an administrator to remove: listed, not offered.
export const NEEDS_ADMIN: AppSummary = notOfferedApp('needs-administrator', 4, 'Disk Tool');

// An offered app the core cannot date: it says so and still names its size.
export const UNDATED: AppSummary = {
  ...OLD_EDITOR,
  app: { index: 5, state: STATE },
  lastUsed: null,
  name: 'Sketch Pad',
  summary: text('core.apps.summary.unknown', { ...name('Sketch Pad'), size: { kind: 'size', value: 802 * MB } }),
};

export const APPS: readonly AppSummary[] = [VIDEO_STUDIO, OLD_EDITOR, MANAGED, OPEN_NOW];

export const APPS_VIEW: AppsView = { apps: [...APPS], notice: null };
