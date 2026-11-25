import type { Dictionary, Event, Finding, LeftOutCount, Overview } from '../ipc/bindings.generated';

import { APP_CACHES_DICTIONARY, cacheFinding, sized, text } from './app-caches-scan';
import { FULL_COVERAGE } from './demo-scan';
import { forNowCount, leftOutEntry } from './left-out';

// A scan of the browser and web-engine caches as the core streams it: three cache folders of two Chrome profiles, each
// titled by its browser, profile and folder, two of Redis Insight's, titled by the app and the folder, and Firefox's
// cache, which is left out because Firefox is open. A folder is offered or left out, never both. The texts are the
// module's own. Played by the e2e mock.

const MB = 1_000_000;

export const APP_CACHES_BROWSERS = 'app-caches.browsers';
export const APP_CACHES_WEB_ENGINE = 'app-caches.web-engine';

const CHROME = '/Users/someone/Library/Caches/Google/Chrome';
const REDIS = '/Users/someone/Library/Application Support/RedisInsight';

const chrome = (id: number, profile: string, folder: string, bytes: number): Finding =>
  cacheFinding({
    app: 'com.google.Chrome',
    bytes,
    category: APP_CACHES_BROWSERS,
    id,
    identity: `chrome/${profile}/${folder}`,
    name: 'Google Chrome',
    path: `${CHROME}/${profile}/${folder}`,
    title: {
      args: { app: text('Google Chrome'), folder: text(folder), profile: text(profile) },
      key: 'app-caches.title.profile',
    },
  });

const redis = (id: number, folder: string, bytes: number): Finding =>
  cacheFinding({
    app: 'org.RedisLabs.RedisInsight-V2',
    bytes,
    category: APP_CACHES_WEB_ENGINE,
    id,
    identity: `redis-insight/${folder}`,
    name: 'Redis Insight',
    path: `${REDIS}/${folder}`,
    title: { args: { app: text('Redis Insight'), folder: text(folder) }, key: 'app-caches.title.part' },
  });

export const APP_CACHES_BROWSERS_FINDINGS: readonly Finding[] = [
  chrome(801, 'Default', 'Cache', 400 * MB),
  chrome(802, 'Default', 'Code Cache', 150 * MB),
  chrome(803, 'Profile 1', 'Cache', 90 * MB),
  redis(804, 'Cache', 20 * MB),
  redis(805, 'GPUCache', 5 * MB),
];

// Firefox's cache, left out: the browser is open.
export const APP_CACHES_BROWSERS_LEFT_OUT: readonly LeftOutCount[] = [
  forNowCount(APP_CACHES_BROWSERS, 'app-caches.left-out.app-open', sized(30 * MB), [
    leftOutEntry(
      {
        args: { app: text('Firefox'), folder: text('cache2'), profile: text('default-release') },
        key: 'app-caches.title.profile',
      },
      '/Users/someone/Library/Caches/Firefox/Profiles/default-release/cache2',
      sized(30 * MB),
    ),
  ]),
];

export const APP_CACHES_BROWSERS_SCAN: readonly Event[] = [
  { modules: ['app-caches'], scan: 1, type: 'scan-started' },
  { findings: APP_CACHES_BROWSERS_FINDINGS.slice(0, 3), scan: 1, type: 'findings-added' },
  { findings: APP_CACHES_BROWSERS_FINDINGS.slice(3), scan: 1, type: 'findings-added' },
  {
    insights: [],
    leftOut: [...APP_CACHES_BROWSERS_LEFT_OUT],
    module: 'app-caches',
    outcome: { kind: 'completed' },
    scan: 1,
    type: 'module-finished',
  },
  { coverage: FULL_COVERAGE, scan: 1, type: 'scan-coverage' },
  { kind: { kind: 'full' }, outcome: { kind: 'completed' }, scan: 1, type: 'scan-finished' },
];

export const APP_CACHES_BROWSERS_TOTALS: Overview['totals'] = [
  { category: APP_CACHES_BROWSERS, count: 3, highestDanger: 'low', sizes: sized(640 * MB) },
  { category: APP_CACHES_WEB_ENGINE, count: 2, highestDanger: 'low', sizes: sized(25 * MB) },
];

export const APP_CACHES_BROWSERS_OVERVIEW: Overview = {
  found: sized(665 * MB),
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
  totals: APP_CACHES_BROWSERS_TOTALS,
};

export const APP_CACHES_BROWSERS_DICTIONARY: Dictionary = APP_CACHES_DICTIONARY;
