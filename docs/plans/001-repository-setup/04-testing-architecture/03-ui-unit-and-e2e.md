# 001/04/03 — UI unit and e2e tests

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** app

## Files

- `app/ui/vitest.config.ts` — happy-dom, Testing Library setup file, `restoreMocks`, v8 coverage.
- `app/ui/src/ipc/mock.ts` — a typed wrapper over `@tauri-apps/api/mocks` `mockIPC` shared by unit and e2e tests, so a
  test states what the core returns.
- One component test in the negative/positive shape.
- `app/ui/playwright.config.ts` — runs against `vite preview` with the IPC mock injected; Chromium and WebKit (Tauri
  uses WebKit on macOS, so WebKit is the reference); screenshots stored next to the tests, `maxDiffPixelRatio` 0.01.
- `app/ui/e2e/smoke.spec.ts` (the version screen renders) and `app/ui/e2e/visual.spec.ts` (one screenshot).
- Root scripts: `test:ui`, `test:e2e:ui`, `test:e2e` (core + UI).

## Checks

- `npm run test:ui` and `npm run test:e2e:ui` pass; changing the screen's text fails the screenshot test.
- The known limit is written down: the full app cannot be driven by WebDriver on macOS (`tauri-driver` supports Linux
  and Windows only), recorded in [`../../../testing.md`](../../../testing.md).
