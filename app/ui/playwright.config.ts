import { defineConfig, devices } from '@playwright/test';

// UI e2e and visual tests (docs/testing.md): the production build served by `vite preview`, the Rust core mocked.
// WebKit is the reference engine: the app runs in WebKit on macOS.
export default defineConfig({
  // A changed word is a few hundred pixels: the tolerance only absorbs anti-aliasing noise.
  expect: { toHaveScreenshot: { maxDiffPixels: 50 } },
  forbidOnly: true,
  fullyParallel: true,
  outputDir: '../../test-results/ui',
  projects: [
    { name: 'webkit', use: { ...devices['Desktop Safari'], viewport: { height: 720, width: 1100 } } },
    { name: 'chromium', use: { ...devices['Desktop Chrome'], viewport: { height: 720, width: 1100 } } },
  ],
  reporter: [['list'], ['html', { open: 'never', outputFolder: '../../playwright-report' }]],
  snapshotPathTemplate: '{testDir}/screenshots/{testFilePath}/{arg}-{projectName}{ext}',
  testDir: 'e2e',
  use: { baseURL: 'http://localhost:4173' },
  webServer: {
    command: 'npm run build && npm run preview -- --port 4173 --strictPort',
    reuseExistingServer: false,
    url: 'http://localhost:4173',
  },
});
