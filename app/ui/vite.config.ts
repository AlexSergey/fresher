import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// The Tauri dev server: a fixed port the shell's `devUrl` points at.
export default defineConfig({
  build: {
    // macOS 13's WebKit, the minimum system version of the app.
    target: 'safari16',
  },
  clearScreen: false,
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true,
  },
});
