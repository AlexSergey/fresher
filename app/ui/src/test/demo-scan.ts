import type { Dictionary, Event, Finding, Overview, Refusal } from '../ipc/bindings.generated';

// A scan of the demo module as the core streams it, with the texts it needs: shared by the unit and the e2e tests.

const finding = (id: number, category: string, name: string, bytes: number, danger: Finding['danger']): Finding => ({
  category,
  danger,
  id,
  key: { identity: `/demo/${name}`, module: 'demo' },
  module: 'demo',
  reason: { args: {}, key: 'demo.finding.reason' },
  removal: { kind: 'move-to-trash', value: `/demo/${name}` },
  sizes: { allocated: bytes, completeness: 'complete', logical: bytes },
  subject: { kind: 'path', value: `/demo/${name}` },
  title: { args: { name: { kind: 'text', value: name } }, key: 'demo.finding.title' },
});

export const DEMO_FINDINGS: readonly Finding[] = [
  finding(1, 'demo.caches', 'editor-cache', 1_500_000_000, 'safe'),
  finding(2, 'demo.downloads', 'installer.dmg', 750_000_000, 'low'),
  finding(3, 'demo.caches', 'browser-cache', 320_000_000, 'safe'),
];

export const SCAN_STARTED: Event = { modules: ['demo'], scan: 1, type: 'scan-started' };

export const FIRST_BATCH: Event = { findings: DEMO_FINDINGS.slice(0, 2), scan: 1, type: 'findings-added' };

export const DEMO_SCAN: readonly Event[] = [
  SCAN_STARTED,
  FIRST_BATCH,
  { findings: DEMO_FINDINGS.slice(2), scan: 1, type: 'findings-added' },
  { module: 'demo', outcome: { kind: 'completed' }, scan: 1, type: 'module-finished' },
  { outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

// What the core sends when a scan is asked for during a scan: the error and its text, built by the core.
export const BUSY: Refusal = {
  error: { code: 'busy', running: 'scan' },
  text: { args: { running: { kind: 'text', value: 'scan' } }, key: 'core.error.busy' },
};

export const IDLE: Overview = { lastScan: null, status: { state: 'idle' }, totals: [] };

export const DEMO_DICTIONARY: Dictionary = {
  'core.danger.low.name': 'Low',
  'core.danger.safe.name': 'Safe',
  'core.error.busy': 'The app is already running a {running}. Wait for it to finish, or cancel it.',
  'core.scan.outcome.completed': 'The scan is complete.',
  'core.scan.outcome.failed': 'The scan could not finish.',
  'core.scan.outcome.partial': 'The scan stopped early or could not read some folders. What it found so far is shown.',
  'demo.caches.name': 'Example caches',
  'demo.downloads.name': 'Example downloads',
  'demo.finding.reason': 'Example data from the demo module: it does not exist on your disk.',
  'demo.finding.title': 'Example: {name}',
};
