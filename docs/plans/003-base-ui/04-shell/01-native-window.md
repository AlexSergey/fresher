# 003/04/01 — The native window

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/tauri, app/ui

## Files

- `app/tauri/tauri.conf.json` — the `main` window: `titleBarStyle: "Overlay"`, `hiddenTitle: true`, the traffic
  lights inset (`trafficLightPosition` about `{ x: 18, y: 22 }`, so they sit centred in the 52 px strip of the
  sidebar), `transparent: true`, `windowEffects: { effects: ["sidebar"], state: "followsWindowActiveState" }`;
  `app.macOSPrivateApi: true`. Every key is checked against the schema of the pinned `@tauri-apps/cli`
  (`node_modules/@tauri-apps/cli/config.schema.json`) before it is written; a key the pinned version lacks is
  replaced by its runtime equivalent in `app/tauri/src/` (the window builder) and noted below.
- `app/tauri/Cargo.toml` — the `macos-private-api` feature of `tauri` (required by `macOSPrivateApi`).
- `app/tauri/capabilities/default.json` — adds `core:window:allow-start-dragging` and
  `core:window:allow-internal-toggle-maximize` (the drag region and its double-click zoom); the description says why.
- The capability test in `app/tauri/tests/` — allows exactly these two core permissions besides the app's commands;
  any other `core:` or plugin permission still fails it.
- `app/ui/src/ipc/runtime.ts` — `inApp(): boolean` (`isTauri()` of `@tauri-apps/api/core`), exported from
  `ipc/index.ts`; still the only place importing the API.
- `app/ui/src/main.tsx` — sets `data-window="vibrant"` on `<html>` when `inApp()`; otherwise nothing (the solid
  fallback).
- `app/ui/src/design/base/window.css` — `html, body` transparent under `[data-window='vibrant']`, the window surface
  painted by the main pane only; without the attribute, `html` is `--surface-window`.
- Page zoom: whether `zoomHotkeysEnabled` exists in the pinned schema and needs a permission is checked; if it needs
  more than a window permission, it is left for plan 005 and noted here.

## Produces

The window the shell (04/02) lays out: a transparent webview over the sidebar material, a drag region by attribute.

## Checks

- Negative: `the_capability_allows_no_core_permission_beyond_dragging` (a third `core:` permission fails),
  `inApp is false outside the app` (Vitest, no Tauri internals).
- Positive: `the_capability_allows_dragging_and_zoom_by_double_click`, `main sets data-window only in the app`.
- `cargo nextest run -p fresher-app` and `npm run lint` pass; `npm run build` bundles the app.
- **By hand** (the agent cannot open a window; written in the plan's "Checks by hand"): `npm run tauri dev` shows the
  translucent sidebar with the desktop colours through it, the traffic lights inset, the window draggable by its top
  strip, double-click zooms; with "Reduce transparency" on in System Settings the sidebar turns opaque; an inactive
  window's sidebar dims like a native app's.

## Notes

- Every key exists in the schema of the pinned `@tauri-apps/cli` 2.12.4 and was written to `tauri.conf.json` as
  planned; nothing moved to the window builder. `windowEffects` takes `effects`, `state`, `radius`, `color` and
  `interactive`; only the first two are set.
- `macOSPrivateApi` and the `macos-private-api` feature are documented as no-ops since Tauri 2.12.1 (the transparent
  background API is always on), but `tauri-build` still checks that the config flag and the Cargo feature agree, so both
  are set. The feature adds no crate: `Cargo.lock` is unchanged.
- The drag region: Tauri's drag script (on `data-tauri-drag-region`) invokes `plugin:window|start_dragging` on a press
  and, on macOS, `plugin:window|internal_toggle_maximize` on a double click's release, which are exactly the two
  permissions added.
- **Page zoom (agent's call 28):** `zoomHotkeysEnabled` exists in the pinned schema (default `false`). On macOS it
  injects a script that calls `plugin:webview|set_webview_zoom`, so it needs `core:webview:allow-set-webview-zoom`, a
  webview permission, not a window one. It is left off and left for plan 005; the capability test keeps it out.
- The capability test also checks the window's keys (`the_main_window_is_transparent_over_the_sidebar_material`).
- Run by the agent: `npm run tauri dev` built and started the app without an error, and the process ran until stopped;
  the agent could not look at the window (no screen access), so every check by hand above is still the user's.

- 2026-11-15 — the user opened the app (`npm run tauri dev`): the window shows correctly. The screens of the
  design preview come with plans 06 and 07; the by-hand checks of the material, dragging and zoom stay in 08.
