import { isTauri } from '@tauri-apps/api/core';

// Whether the page runs inside the app's window; a plain browser, the gallery and the tests are outside it.
export const inApp = (): boolean => isTauri();
