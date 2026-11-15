import { invoke } from '@tauri-apps/api/core';

// The only module that talks to the Rust core: every command is typed here, components never call `invoke`.

// The Rust names of the commands (`#[tauri::command]` in app/tauri), the only place they are spelled.
export const COMMANDS = {
  appVersion: 'app_version',
} as const;

export type CommandKey = keyof typeof COMMANDS;

export const getAppVersion = (): Promise<string> => invoke<string>(COMMANDS.appVersion);
