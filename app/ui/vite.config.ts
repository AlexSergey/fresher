import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

import tauriConfig from '../tauri/tauri.conf.json' with { type: 'json' };

// The e2e build (`--mode e2e`) adds the dev-only gallery and writes to dist-e2e/; the app's build has only index.html.
const E2E = 'e2e';

// The Tauri dev server: a fixed port the shell's `devUrl` points at.
export default defineConfig(({ mode }) => ({
  build: {
    // macOS 13's WebKit, the minimum system version of the app: the baseline is Safari 16.0
    // (docs/plans/003-base-ui/00-design.md, "The platform"); the CSS guard holds the source to it.
    cssTarget: 'safari16',
    ...(mode === E2E ? { outDir: 'dist-e2e', rolldownOptions: { input: ['index.html', 'gallery.html'] } } : {}),
    target: 'safari16',
  },
  clearScreen: false,
  plugins: [react()],
  // The built pages are served with the app's production policy, so a breach fails the e2e tests.
  preview: { headers: { 'Content-Security-Policy': tauriConfig.app.security.csp } },
  server: {
    // Tauri injects its CSP only into the pages it serves itself; the dev server's page gets the shell's `devCsp` here.
    headers: { 'Content-Security-Policy': tauriConfig.app.security.devCsp },
    port: 1420,
    strictPort: true,
  },
}));
