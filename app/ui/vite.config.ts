import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

import tauriConfig from '../tauri/tauri.conf.json' with { type: 'json' };

// The Tauri dev server: a fixed port the shell's `devUrl` points at.
export default defineConfig({
  build: {
    // macOS 13's WebKit, the minimum system version of the app.
    target: 'safari16',
  },
  clearScreen: false,
  plugins: [react()],
  server: {
    // Tauri injects its CSP only into the pages it serves itself; the dev server's page gets the shell's `devCsp` here.
    headers: { 'Content-Security-Policy': tauriConfig.app.security.devCsp },
    port: 1420,
    strictPort: true,
  },
});
