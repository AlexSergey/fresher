import { Channel, invoke, type InvokeArgs } from '@tauri-apps/api/core';

import type {
  AccessStatus,
  Dictionary,
  Event,
  FindingsPage,
  FindingsQuery,
  ModuleInfo,
  Overview,
  Refusal,
  ScanId,
  SettingsView,
  SettingUpdate,
} from './bindings.generated';

// The only module that talks to the Rust core: every command is typed here, components never call `invoke`.

// The Rust names of the commands (`#[tauri::command]` in app/tauri), the only place they are spelled.
export const COMMANDS = {
  appVersion: 'app_version',
  cancelScan: 'cancel_scan',
  getAccessStatus: 'get_access_status',
  getDictionary: 'get_dictionary',
  getFindings: 'get_findings',
  getModules: 'get_modules',
  getOverview: 'get_overview',
  getSettings: 'get_settings',
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

export const getDictionary = (locale: string): Promise<Dictionary> => call('getDictionary', { locale });

export const getAccessStatus = (): Promise<AccessStatus> => call('getAccessStatus');
