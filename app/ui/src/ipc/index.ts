import { invoke } from '@tauri-apps/api/core';

// The only module that talks to the Rust core: every command is typed here, components never call `invoke`.

export const getAppVersion = (): Promise<string> => invoke<string>('app_version');
