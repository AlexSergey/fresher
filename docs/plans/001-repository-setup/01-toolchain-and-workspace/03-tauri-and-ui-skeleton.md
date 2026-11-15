# 001/01/03 — Tauri shell and UI skeleton

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** app

## Files

- `app/tauri/` — crate `fresher-app` (Tauri 2, exact version): `tauri.conf.json` (app id, window, `frontendDist`
  `../ui/dist`, `devUrl`), `capabilities/default.json` with the minimum permissions (no network, no shell), a strict
  CSP (`default-src 'self'`), one command `app_version` as the IPC example.
- `app/ui/` — Vite + React + TypeScript (strict `tsconfig.json`, `noUncheckedIndexedAccess`,
  `exactOptionalPropertyTypes`), `src/ipc/` — the only module that calls `invoke`, typed; one screen that shows the
  version from the core.
- Root scripts: `tauri` (`tauri` CLI), `dev` (`tauri dev`), `build` (`tauri build`).

## Done when

- `npm run dev` opens the window and shows the version returned by the Rust command.
- `npm run build` produces an unsigned `.app` bundle.
