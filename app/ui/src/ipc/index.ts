import { Channel, invoke, type InvokeArgs } from '@tauri-apps/api/core';

import type {
  AccessStatus,
  CleanupDetail,
  CleanupId,
  CleanupItemRef,
  CleanupPlanView,
  CleanupSelection,
  CleanupsPage,
  CleanupsQuery,
  Confirmation,
  Dictionary,
  Event,
  FindingsPage,
  FindingsQuery,
  IgnoredEntry,
  IgnoreTarget,
  ModuleInfo,
  Overview,
  OwnData,
  PlanId,
  PutBackReport,
  PutBackRequest,
  Refusal,
  ScanId,
  SettingsView,
  SettingUpdate,
} from './bindings.generated';

export { inApp } from './runtime';

// The only module that talks to the Rust core: every command is typed here, components never call `invoke`.

// The Rust names of the commands (`#[tauri::command]` in app/tauri), the only place they are spelled.
export const COMMANDS = {
  appVersion: 'app_version',
  cancelCleanup: 'cancel_cleanup',
  cancelScan: 'cancel_scan',
  confirmCleanup: 'confirm_cleanup',
  discardCleanup: 'discard_cleanup',
  getAccessStatus: 'get_access_status',
  getCleanup: 'get_cleanup',
  getCleanups: 'get_cleanups',
  getDictionary: 'get_dictionary',
  getFindings: 'get_findings',
  getModules: 'get_modules',
  getOverview: 'get_overview',
  getOwnData: 'get_own_data',
  getSettings: 'get_settings',
  ignore: 'ignore',
  pickFolder: 'pick_folder',
  prepareCleanup: 'prepare_cleanup',
  putBack: 'put_back',
  quitAfterCleanup: 'quit_after_cleanup',
  resetSettings: 'reset_settings',
  restoreIgnored: 'restore_ignored',
  revealInTrash: 'reveal_in_trash',
  startScan: 'start_scan',
  subscribe: 'subscribe',
  updateSetting: 'update_setting',
} as const;

export type CommandKey = keyof typeof COMMANDS;

// Why a command failed: the core refused it, with the text to show, or the IPC itself failed.
export type Failure = { readonly kind: 'refused'; readonly refusal: Refusal } | { readonly kind: 'unreachable' };

const isRefusal = (value: unknown): value is Refusal =>
  typeof value === 'object' && value !== null && 'error' in value && 'text' in value;

// A failed command, with its `Failure`.
export class CommandFailed extends Error {
  readonly failure: Failure;

  constructor(cause: unknown) {
    const failure: Failure = isRefusal(cause) ? { kind: 'refused', refusal: cause } : { kind: 'unreachable' };
    super(
      failure.kind === 'refused' ? `The core refused the command: ${failure.refusal.error.code}` : 'The IPC failed',
      {
        cause,
      },
    );
    this.name = 'CommandFailed';
    this.failure = failure;
  }
}

// The failure behind a rejected command; anything that is not a `CommandFailed` is `unreachable`.
export const failureOf = (error: unknown): Failure =>
  error instanceof CommandFailed ? error.failure : { kind: 'unreachable' };

// A command of the core; a failure rejects with `CommandFailed`.
const call = async <T>(key: CommandKey, args?: InvokeArgs): Promise<T> => {
  try {
    return await invoke<T>(COMMANDS[key], args);
  } catch (error) {
    throw new CommandFailed(error);
  }
};

export const getAppVersion = (): Promise<string> => call('appVersion');

// Every event of the core from now on goes to `onEvent`.
export const subscribe = async (onEvent: (event: Event) => void): Promise<void> => {
  let channel: Channel<Event>;
  try {
    // Outside the app (a plain browser) there is no bridge and the channel cannot be made.
    channel = new Channel<Event>(onEvent);
  } catch (error) {
    throw new CommandFailed(error);
  }

  return call('subscribe', { channel });
};

export const startScan = (): Promise<ScanId> => call('startScan');

export const cancelScan = (): Promise<void> => call('cancelScan');

export const getOverview = (): Promise<Overview> => call('getOverview');

export const getModules = (): Promise<readonly ModuleInfo[]> => call('getModules');

export const getFindings = (query: FindingsQuery): Promise<FindingsPage> => call('getFindings', { query });

export const getSettings = (): Promise<SettingsView> => call('getSettings');

export const updateSetting = (update: SettingUpdate): Promise<void> => call('updateSetting', { update });

// Every setting but the ignore lists goes back to its default.
export const resetSettings = (): Promise<void> => call('resetSettings');

// A category, or one finding by its path, leaves the results until it is restored.
export const ignore = (target: IgnoreTarget): Promise<void> => call('ignore', { target });

export const restoreIgnored = (entry: IgnoredEntry): Promise<void> => call('restoreIgnored', { entry });

// The system's folder picker; `null` when it was cancelled. A folder in the home folder comes as `~/…`.
export const pickFolder = (): Promise<null | string> => call('pickFolder');

// What Fresher's own data takes on disk, against its budget.
export const getOwnData = (): Promise<OwnData> => call('getOwnData');

export const getDictionary = (locale: string): Promise<Dictionary> => call('getDictionary', { locale });

export const getAccessStatus = (): Promise<AccessStatus> => call('getAccessStatus');

// The core checks every item of the selection and answers the plan the confirmation shows; nothing is removed.
export const prepareCleanup = (selection: CleanupSelection): Promise<CleanupPlanView> =>
  call('prepareCleanup', { selection });

// The confirmation was cancelled: the core forgets the plan.
export const discardCleanup = (plan: PlanId): Promise<void> => call('discardCleanup', { plan });

// What the confirmation showed, sent back: the core compares it with its plan and starts the cleanup.
export const confirmCleanup = (confirmation: Confirmation): Promise<CleanupId> =>
  call('confirmCleanup', { confirmation });

// Stops the running cleanup after the item in hand.
export const cancelCleanup = (): Promise<void> => call('cancelCleanup');

export const getCleanups = (query: CleanupsQuery): Promise<CleanupsPage> => call('getCleanups', { query });

export const getCleanup = (cleanup: CleanupId): Promise<CleanupDetail> => call('getCleanup', { cleanup });

export const putBack = (request: PutBackRequest): Promise<PutBackReport> => call('putBack', { request });

export const revealInTrash = (item: CleanupItemRef): Promise<void> => call('revealInTrash', { item });

// The user chose to quit during a cleanup: it stops after the item in hand, then the app quits.
export const quitAfterCleanup = (): Promise<void> => call('quitAfterCleanup');
