import type {
  Access,
  AccessStatus,
  Capability,
  CategoryInfo,
  Coverage,
  DangerLevel,
  Dictionary,
  Event,
  Finding,
  LeftOutCount,
  ModuleInfo,
  Overview,
  Refusal,
  SettingsView,
} from '../ipc/bindings.generated';

import { iconOf } from './category-icons';

// A scan of the demo module as the core streams it, with the texts it needs: shared by the unit and the e2e tests.

// A scan that saw everything: no guarded place left out, no folder unreadable.
export const FULL_COVERAGE: Coverage = { notEntered: [], unreadable: [] };

// A scan that could not read 312 folders Full Disk Access would open, and did not enter the Desktop.
export const LIMITED_COVERAGE: Coverage = {
  notEntered: [{ access: 'not-asked', capability: { kind: 'folder', value: 'desktop' } }],
  unreadable: [{ folders: 312, reason: 'no-permission', unlockedBy: { kind: 'all-files' } }],
};

// The core's texts of the capabilities the limited coverage names.
export const ACCESS_DICTIONARY: Dictionary = {
  'core.access.all-files.how': '1. Click Open System Settings. 2. Turn on Fresher in the list. 3. Scan again.',
  'core.access.all-files.missing': 'Some folders could not be read because Fresher does not have Full Disk Access.',
  'core.access.all-files.name': 'Full Disk Access',
  'core.access.all-files.unlocks': 'Lets Fresher see every folder on this Mac.',
  'core.access.desktop.denied': 'Turn on Fresher under Files and Folders, then scan again.',
  'core.access.desktop.description': 'macOS will ask you once.',
  'core.access.desktop.how': '1. Click Let Fresher look there. 2. Scan again.',
  'core.access.desktop.missing': 'Your Desktop was not looked at.',
  'core.access.desktop.name': 'Desktop folder',
  'core.access.desktop.unlocks': 'Lets Fresher measure what takes space on your Desktop.',
  'core.access.removable-volume.denied': 'Turn on Removable Volumes for Fresher, then scan again.',
  'core.access.removable-volume.how': '1. Add the drive to the folders to scan. 2. Scan again.',
  'core.access.removable-volume.missing': 'The external drive was not looked at.',
  'core.access.removable-volume.name': 'External drives',
  'core.access.removable-volume.unlocks': 'Lets Fresher measure what takes space on a drive you added.',
};

// A category that needs nothing, so it is ready whatever the access.
export const readyCategory = (id: string, danger: DangerLevel): CategoryInfo => ({
  availability: { kind: 'ready' },
  descriptor: { danger, icon: iconOf(id), id, needs: [] },
});

// What `get_access_status` answers with each capability of `states` in its state, and no other capability.
export const accessStatus = (states: readonly (readonly [Capability, Access])[]): AccessStatus => ({
  capabilities: states.map(([capability, access]) => ({ access, capability, note: null })),
});

// What `get_access_status` answers when Full Disk Access is `access` (the core's `all-files` capability).
export const allFiles = (access: Access): AccessStatus => accessStatus([[{ kind: 'all-files' }, access]]);

const finding = (id: number, category: string, name: string, bytes: number, danger: Finding['danger']): Finding => ({
  category,
  danger,
  id,
  key: { identity: `/demo/${name}`, module: 'demo' },
  module: 'demo',
  reason: { args: {}, key: 'demo.finding.reason' },
  removability: 'removable',
  removal: { kind: 'move-to-trash', value: `/demo/${name}` },
  sizes: { allocated: bytes, completeness: 'complete', logical: bytes, reclaimable: bytes },
  subject: { kind: 'path', value: `/demo/${name}` },
  title: { args: { name: { kind: 'text', value: name } }, key: 'demo.finding.title' },
});

export const DEMO_FINDINGS: readonly Finding[] = [
  finding(1, 'demo.caches', 'editor-cache', 1_500_000_000, 'safe'),
  finding(2, 'demo.downloads', 'installer.dmg', 750_000_000, 'low'),
  finding(3, 'demo.caches', 'browser-cache', 320_000_000, 'safe'),
];

// What `get_modules` answers: the demo module and its categories, as its descriptor registers them.
export const DEMO_MODULES: readonly ModuleInfo[] = [
  {
    categories: [readyCategory('demo.caches', 'safe'), readyCategory('demo.downloads', 'low')],
    choices: [],
    group: 'debug',
    id: 'demo',
    settings: ['demo.enabled'],
  },
];

export const SCAN_STARTED: Event = { modules: ['demo'], scan: 1, type: 'scan-started' };

export const FIRST_BATCH: Event = { findings: DEMO_FINDINGS.slice(0, 2), scan: 1, type: 'findings-added' };

// The one cache the demo module recognizes and leaves out (its app is open), as the engine counts it.
export const DEMO_LEFT_OUT: LeftOutCount = {
  category: 'demo.caches',
  items: 1,
  reason: { args: {}, key: 'demo.left-out.open' },
  sizes: { allocated: 157_286_400, completeness: 'complete', logical: 157_286_400, reclaimable: 157_286_400 },
};

export const DEMO_SCAN: readonly Event[] = [
  SCAN_STARTED,
  FIRST_BATCH,
  { findings: DEMO_FINDINGS.slice(2), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [DEMO_LEFT_OUT],
    module: 'demo',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

// What the core sends when a scan is asked for during a scan: the error and its text, built by the core.
export const BUSY: Refusal = {
  error: { code: 'busy', running: 'scan' },
  text: { args: { running: { kind: 'text', value: 'scan' } }, key: 'core.error.busy' },
};

// What `get_access_status` answers on a Mac that gave the app Full Disk Access.
export const GRANTED: AccessStatus = allFiles('granted');

export const DEMO_SETTINGS: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'switch', value: true },
      effect: 'next-scan',
      group: 'module',
      key: 'demo.enabled',
      kind: { kind: 'switch' },
    },
  ],
  values: { 'demo.enabled': { kind: 'switch', value: true } },
};

export const IDLE: Overview = {
  found: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: null,
  status: { state: 'idle' },
  totals: [],
};

const complete = (bytes: number): Overview['found'] => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

const partial = (bytes: number): Overview['found'] => ({
  allocated: bytes,
  completeness: 'partial',
  logical: bytes,
  reclaimable: bytes,
});

// What `get_overview` answers once the demo scan finished: its totals, as the engine sums them.
export const DEMO_OVERVIEW: Overview = {
  found: complete(2_570_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'completed' },
    scan: 1,
    settingsChanged: false,
  },
  status: { state: 'idle' },
  totals: [
    { category: 'demo.caches', count: 2, highestDanger: 'safe', sizes: complete(1_820_000_000) },
    { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: complete(750_000_000) },
  ],
};

export const DEMO_DICTIONARY: Dictionary = {
  'core.danger.low.name': 'Low',
  'core.danger.safe.name': 'Safe',
  'core.error.busy': 'The app is already running a {running}. Wait for it to finish, or cancel it.',
  'core.group.debug.name': 'Debug',
  'core.group.development.name': 'Development',
  'core.ignore.modules.description': 'Fresher stops looking for these and never removes what they find.',
  'core.left-out.summary.one': '{count} item left out ({size}):',
  'core.left-out.summary.other': '{count} items left out ({size}):',
  'core.scan.outcome.completed': 'The scan is complete.',
  'core.scan.outcome.failed': 'The scan could not finish.',
  'core.scan.outcome.partial': 'The scan stopped early or could not read some folders. What it found so far is shown.',
  'demo.caches.line': 'Made-up caches to try Fresher',
  'demo.caches.name': 'Example caches',
  'demo.caches.what': 'Made-up cache folders of made-up apps.',
  'demo.downloads.line': 'Made-up downloads to try Fresher',
  'demo.downloads.name': 'Example downloads',
  'demo.downloads.what': 'Made-up installers and archives in a made-up Downloads folder.',
  'demo.enabled.description': 'Shows made-up example findings, so the app can be tried without touching your disk.',
  'demo.enabled.label': 'Demo module',
  'demo.finding.reason': 'Made up by the demo; it is not on your disk.',
  'demo.finding.title': 'Example: {name}',
  'demo.left-out.open': 'The app that made it is open. Quit the app, then scan again.',
};

// The live, stopped, partial and failed scans below, the access refused and the settings of every kind: the scripts
// the e2e flows and screenshots play.

// What `get_access_status` answers on a Mac that did not give the app Full Disk Access.
export const DENIED: AccessStatus = allFiles('denied');

// The demo scan with its progress: an e2e script holds it after `LIVE_HOLD` events, mid-scan.
export const DEMO_LIVE_SCAN: readonly Event[] = [
  SCAN_STARTED,
  FIRST_BATCH,
  { bytes: 2_250_000_000, entries: 18_432, expected: 3_000_000_000, scan: 1, type: 'scan-progress' },
  ...DEMO_SCAN.slice(2),
];

export const LIVE_HOLD = 3;

// The first batch's totals, while the scan runs and once it is stopped or failed after it.
const FIRST_TOTALS: Overview['totals'] = [
  { category: 'demo.caches', count: 1, highestDanger: 'safe', sizes: complete(1_500_000_000) },
  { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: complete(750_000_000) },
];

// What `get_overview` answers while the live scan is held after its first batch.
export const DEMO_SCANNING: Overview = {
  found: complete(2_250_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: null,
  status: { scan: 1, state: 'scanning' },
  totals: FIRST_TOTALS,
};

// What the core streams when the live scan is stopped after its first batch, and the overview it answers then.
export const DEMO_STOP: readonly Event[] = [
  { insights: [], leftOut: [], module: 'demo', outcome: { kind: 'cancelled' }, scan: 1, type: 'module-finished' },
  { kind: { kind: 'full' }, outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' },
];

export const DEMO_STOPPED: Overview = {
  found: complete(2_250_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'partial' },
    scan: 1,
    settingsChanged: false,
  },
  status: { state: 'idle' },
  totals: FIRST_TOTALS,
};

// A scan that could not read some folders: every finding, the caches' size only a lower bound.
export const DEMO_PARTIAL_SCAN: readonly Event[] = [
  ...DEMO_SCAN.slice(0, -1),
  { kind: { kind: 'full' }, outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' },
];

export const DEMO_PARTIAL: Overview = {
  found: partial(2_570_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'partial' },
    scan: 1,
    settingsChanged: false,
  },
  status: { state: 'idle' },
  totals: [
    { category: 'demo.caches', count: 2, highestDanger: 'safe', sizes: partial(1_820_000_000) },
    { category: 'demo.downloads', count: 1, highestDanger: 'low', sizes: complete(750_000_000) },
  ],
};

const FAILURE = { args: {}, key: 'demo.error.failed' } as const;

// A scan whose only module failed after its first batch, so the scan failed with the module's reason.
export const DEMO_FAILED_SCAN: readonly Event[] = [
  SCAN_STARTED,
  FIRST_BATCH,
  {
    insights: [],
    leftOut: [],
    module: 'demo',
    outcome: { kind: 'failed', reason: FAILURE },
    scan: 1,
    type: 'module-finished',
  },
  { kind: { kind: 'full' }, outcome: { kind: 'failed', reason: FAILURE }, scan: 1, type: 'scan-finished' },
];

export const DEMO_FAILED: Overview = {
  found: complete(2_250_000_000),
  hiddenModules: [],
  ignored: { categories: 0, groups: 0, modules: 0, paths: 0 },
  lastScan: {
    coverage: FULL_COVERAGE,
    finished: 1_735_689_600,
    kind: { kind: 'full' },
    outcome: { kind: 'failed', reason: FAILURE },
    scan: 1,
    settingsChanged: false,
  },
  status: { state: 'idle' },
  totals: FIRST_TOTALS,
};

// Settings of every kind: three of the core's (a number, a choice, folders) and the demo module's switch.
export const SETTINGS_OF_EVERY_KIND: SettingsView = {
  file: 'loaded',
  problems: [],
  settings: [
    {
      default: { kind: 'number', value: 90 },
      effect: 'next-scan',
      group: 'scan',
      key: 'core.scan.stale-after',
      kind: { kind: 'number', max: 365, min: 1, unit: 'days' },
    },
    {
      default: { kind: 'choice', value: 'system' },
      effect: 'now',
      group: 'general',
      key: 'core.motion',
      kind: { kind: 'choice', options: ['system', 'reduced'] },
    },
    {
      default: { kind: 'folders', value: [] },
      effect: 'next-scan',
      group: 'scan',
      key: 'core.scan.roots',
      kind: { kind: 'folders' },
    },
    ...DEMO_SETTINGS.settings,
  ],
  values: {
    ...DEMO_SETTINGS.values,
    'core.scan.roots': { kind: 'folders', value: ['~/Projects', '~/Work/clients'] },
  },
};

// The same settings in a file a newer version of the app saved: none can be changed.
export const SETTINGS_READ_ONLY: SettingsView = { ...SETTINGS_OF_EVERY_KIND, file: 'newer-version' };

// The texts of the scripts above, beside the demo scan's.
export const SCRIPTS_DICTIONARY: Dictionary = {
  ...DEMO_DICTIONARY,
  'core.completeness.partial': 'Some files could not be read, so the real size is a little bigger.',
  'core.danger.low.meaning': 'You may need to download them again',
  'core.danger.safe.meaning': 'Apps make these again on their own',
  'core.error.settings-read-only':
    'Your settings cannot be changed now: a newer version of the app saved them, or the file could not be read. ' +
    'Restart or update the app, then try again.',
  'core.motion.description': 'How much the app animates.',
  'core.motion.label': 'Motion',
  'core.motion.options.reduced': 'Reduced',
  'core.motion.options.system': 'Follow your Mac',
  'core.scan.roots.description': 'The scan looks only inside these folders.',
  'core.scan.roots.label': 'Folders to scan',
  'core.scan.stale-after.description': 'A project untouched for this long counts as stale.',
  'core.scan.stale-after.label': 'Stale after',
  'demo.caches.comes-back': 'Yes: the example apps would fill their caches again.',
  'demo.caches.if-removed': 'Nothing is lost: an app rebuilds its cache the next time it needs it.',
  'demo.caches.made-by': 'The made-up apps of the demo module.',
  'demo.downloads.comes-back': 'No: a removed download stays removed until you download it again.',
  'demo.downloads.if-removed': 'The installers are gone; download them again if you need them.',
  'demo.downloads.made-by': 'You, by downloading the made-up installers.',
  'demo.error.failed': 'The demo module stopped with an error, as it was told to.',
};
