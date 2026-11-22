import type {
  AppDetail,
  AppSummary,
  ChoiceView,
  CleanupPlanView,
  DangerLevel,
  Finding,
  FootprintItem,
  FootprintKind,
  KeptReason,
  Sizes,
  Text,
  UninstallChoice,
} from '../ipc/bindings.generated';

import { MANAGED, NEEDS_ADMIN, OLD_EDITOR, OPEN_NOW, VIDEO_STUDIO } from './apps-list';
import { planOf } from './cleanup-plan';

// The page of an app as the core answers it, and the uninstall plan of each choice: the sums add up, the dangers are
// the kinds', a kept part goes with neither choice, and the plan is one finding whose removal lists the bundle first.
// Shared by the unit tests, the gallery and the e2e mock.

const MB = 1_000_000;

const LIBRARY = '/Users/someone/Library';

const sized = (bytes: number): Sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// What each kind's part is shown at, and with which choice it goes, as the core's table says.
const DANGERS: Readonly<Record<FootprintKind, DangerLevel>> = {
  bundle: 'medium',
  caches: 'low',
  container: 'high',
  'group-container': 'high',
  'launch-agents': 'medium',
  logs: 'safe',
  'saved-state': 'low',
  settings: 'medium',
  support: 'high',
  'web-data': 'medium',
};

const BOTH: readonly FootprintKind[] = ['bundle', 'caches', 'logs', 'saved-state', 'launch-agents'];

const text = (key: string, args: Text['args'] = {}): Text => ({ args, key });

const itemOf = (kind: FootprintKind, path: string, bytes: number, kept?: KeptReason): FootprintItem => ({
  danger: DANGERS[kind],
  everything: kept === undefined,
  keepData: kept === undefined && BOTH.includes(kind),
  kept: kept === undefined ? null : { reason: kept, text: text(`core.apps.kept.${kept}`) },
  kind,
  path,
  sizes: sized(bytes),
});

const choiceOf = (choice: UninstallChoice, items: readonly FootprintItem[]): ChoiceView => {
  const going = items.filter((item) => (choice === 'keep-data' ? item.keepData : item.everything));
  const bytes = going.reduce((sum, item) => sum + item.sizes.allocated, 0);

  return {
    choice,
    description: text(`core.uninstall.choice.${choice}.description`),
    items: going.length,
    label: text(`core.uninstall.choice.${choice}.label`),
    sizes: sized(bytes),
  };
};

const VIDEO_ITEMS: readonly FootprintItem[] = [
  itemOf('bundle', '/Applications/Video Studio.app', 3_000 * MB),
  itemOf('caches', `${LIBRARY}/Caches/com.example.VideoStudio`, 420 * MB),
  itemOf('logs', `${LIBRARY}/Logs/com.example.VideoStudio`, 30 * MB),
  itemOf('saved-state', `${LIBRARY}/Saved Application State/com.example.VideoStudio.savedState`, 20 * MB),
  itemOf('launch-agents', `${LIBRARY}/LaunchAgents/com.example.VideoStudio.plist`, 2 * MB),
  itemOf('web-data', `${LIBRARY}/WebKit/com.example.VideoStudio`, 18 * MB),
  itemOf('settings', `${LIBRARY}/Preferences/com.example.VideoStudio.plist`, 5 * MB),
  itemOf('support', `${LIBRARY}/Application Support/com.example.VideoStudio`, 400 * MB),
  itemOf('container', `${LIBRARY}/Containers/com.example.VideoStudio`, 55 * MB),
  itemOf('group-container', `${LIBRARY}/Group Containers/ABCDE12345.com.example`, 50 * MB, 'team-shared'),
];

const NOT_ASKED = text('core.apps.management.not-asked');

// An offered app: every kind of part (one kept), both choices, the App Management notice before it was ever asked.
export const VIDEO_STUDIO_DETAIL: AppDetail = {
  categories: ['app-caches.library-caches'],
  choices: [choiceOf('keep-data', VIDEO_ITEMS), choiceOf('everything', VIDEO_ITEMS)],
  command: null,
  items: [...VIDEO_ITEMS],
  notice: NOT_ASKED,
  summary: VIDEO_STUDIO,
};

// The same app after macOS refused to let Fresher move an app: the notice says so and asks for the permission.
export const VIDEO_STUDIO_DENIED_DETAIL: AppDetail = {
  ...VIDEO_STUDIO_DETAIL,
  notice: text('core.access.app-management.denied'),
};

// A not-offered app offers no choice and no notice, whatever else its page lists.
const notOffered = (summary: AppSummary, command: null | string): AppDetail => ({
  categories: [],
  choices: [],
  command,
  items: [itemOf('bundle', `/Applications/${summary.name}.app`, summary.sizes.allocated)],
  notice: null,
  summary,
});

// An app a package manager installed: its manager's command, for the user to copy.
export const MANAGED_DETAIL: AppDetail = notOffered(MANAGED, 'brew uninstall --cask terminal-pro');

// An app that needs an administrator: Show in Finder is the way.
export const NEEDS_ADMIN_DETAIL: AppDetail = notOffered(NEEDS_ADMIN, null);

// An app that is open: the reason says to quit it first.
export const OPEN_DETAIL: AppDetail = notOffered(OPEN_NOW, null);

const OLD_ITEMS: readonly FootprintItem[] = [
  itemOf('bundle', '/Applications/Old Editor.app', 800 * MB),
  itemOf('settings', `${LIBRARY}/Preferences/com.example.OldEditor.plist`, 2 * MB),
];

// An app not used for a year, offered: its settings go only with everything.
export const OLD_EDITOR_DETAIL: AppDetail = {
  categories: [],
  choices: [choiceOf('keep-data', OLD_ITEMS), choiceOf('everything', OLD_ITEMS)],
  command: null,
  items: [...OLD_ITEMS],
  notice: NOT_ASKED,
  summary: OLD_EDITOR,
};

// The page of every app of the list, for the e2e mock.
export const APP_DETAILS: readonly AppDetail[] = [
  VIDEO_STUDIO_DETAIL,
  OLD_EDITOR_DETAIL,
  MANAGED_DETAIL,
  OPEN_DETAIL,
  NEEDS_ADMIN_DETAIL,
];

// The finding the engine makes for the uninstall of `detail` under `choice`: the bundle first, then the parts.
export const uninstallFindingOf = (detail: AppDetail, choice: UninstallChoice, id = 900): Finding => {
  const view = detail.choices.find((one) => one.choice === choice) as ChoiceView;
  const paths = detail.items
    .filter((item) => (choice === 'keep-data' ? item.keepData : item.everything))
    .map((item) => item.path);
  const [bundle] = paths as [string];
  const app = { kind: 'text', value: detail.summary.name } as const;

  return {
    category: `core.uninstall-${choice}`,
    danger: choice === 'keep-data' ? 'medium' : 'high',
    id,
    key: { identity: bundle, module: 'core' },
    module: 'core',
    reason: text(`core.uninstall.reason.${choice}`, { app, size: { kind: 'size', value: view.sizes.allocated } }),
    removability: 'removable',
    removal: { kind: 'move-together', value: paths },
    sizes: view.sizes,
    subject: { kind: 'path', value: bundle },
    title: text('core.uninstall.title', { app }),
  };
};

// The plan the core answers for the uninstall of `detail` under `choice`.
export const uninstallPlanOf = (detail: AppDetail, choice: UninstallChoice, plan = 1): CleanupPlanView => ({
  ...planOf([uninstallFindingOf(detail, choice)], { plan }),
  // An uninstall always goes to the Trash.
  permanentAvailable: false,
});
