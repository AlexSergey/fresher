import { defineConfig, devices } from '@playwright/test';

// UI e2e and visual tests (docs/testing.md): the e2e build (the app and the gallery) served by `vite preview` with the
// app's production CSP, the Rust core mocked. WebKit is the reference engine: the app runs in WebKit on macOS.
const viewport = { height: 720, width: 1100 };
const GALLERY = 'gallery.spec.ts';
// The performance measurement runs only on its own (`npm run test:perf`): headed, full motion, one test at a time.
const PERF = 'perf.spec.ts';
const perf = { headless: false, reducedMotion: 'no-preference', viewport } as const;

export default defineConfig({
  // A changed word is a few hundred pixels: the tolerance only absorbs anti-aliasing noise.
  expect: { toHaveScreenshot: { maxDiffPixels: 50 } },
  forbidOnly: true,
  fullyParallel: true,
  outputDir: '../../test-results/ui',
  projects: [
    // Every story of the gallery, photographed in the reference engine only.
    { name: 'gallery', testMatch: GALLERY, use: { ...devices['Desktop Safari'], viewport } },
    { name: 'webkit', testIgnore: [GALLERY, PERF], use: { ...devices['Desktop Safari'], viewport } },
    { name: 'chromium', testIgnore: [GALLERY, PERF], use: { ...devices['Desktop Chrome'], viewport } },
    // The budget is measured in the reference engine; Chromium adds what only it reports: long tasks and the JS heap.
    { name: 'perf', testMatch: PERF, use: { ...devices['Desktop Safari'], ...perf } },
    {
      name: 'perf-chromium',
      testMatch: PERF,
      use: { ...devices['Desktop Chrome'], ...perf },
    },
  ],
  reporter: [['list'], ['html', { open: 'never', outputFolder: '../../playwright-report' }]],
  snapshotPathTemplate: '{testDir}/screenshots/{testFilePath}/{arg}-{projectName}{ext}',
  testDir: 'e2e',
  use: { baseURL: 'http://localhost:4173' },
  webServer: {
    command: 'npm run build:e2e && npm run preview -- --mode e2e --port 4173 --strictPort',
    reuseExistingServer: false,
    url: 'http://localhost:4173',
  },
});
