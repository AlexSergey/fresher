import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

// UI unit tests (docs/testing.md): happy-dom, Testing Library, a 90 % coverage floor on every metric.
export default defineConfig({
  plugins: [react()],
  test: {
    coverage: {
      exclude: ['src/main.tsx', 'src/ipc/mock.ts', 'src/test/**', 'src/**/*.test.{ts,tsx}'],
      include: ['src/**/*.{ts,tsx}'],
      provider: 'v8',
      reportsDirectory: '../../coverage/ui',
      thresholds: { branches: 90, functions: 90, lines: 90, statements: 90 },
    },
    // Dates in tests are formatted in UTC, whatever the machine's time zone.
    env: { TZ: 'UTC' },
    environment: 'happy-dom',
    include: ['src/**/*.test.{ts,tsx}'],
    restoreMocks: true,
    setupFiles: ['src/test/setup.ts'],
  },
});
