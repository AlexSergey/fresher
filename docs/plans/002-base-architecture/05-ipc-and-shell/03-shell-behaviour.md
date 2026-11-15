# 002/05/03 — Shell behaviour

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** app

## Files

- `app/tauri/Cargo.toml` — `tauri-plugin-single-instance = "=2.5.5"` (MIT / Apache-2.0; checked against Tauri 2.12 in
  this step, and with `cargo deny check`).
- `app/tauri/src/lib.rs` — the single-instance plugin registered first: a second launch focuses the open window and
  exits; on `RunEvent::ExitRequested` (the window closed) the engine is shut down (scan cancelled and joined) before the
  process ends. No tray, no background thread survives the window.
- `app/tauri/tauri.conf.json` — the main window `"incognito": true` (a non-persistent web data store); the CSP stays as
  it is, with no remote origin.

## Produces

Section 8 of [`goal.md`](../../../goal.md) for the shell: the app runs only while its window is open.

## Checks

- Negative: `exit_requested_shuts_the_engine_down` (the exit handler, tested as a function on an engine with a blocking
  scripted walk), `the_config_keeps_no_remote_origin_in_the_csp` (reads `tauri.conf.json`).
- Positive: `the_main_window_is_incognito` (reads `tauri.conf.json`).
- By hand, written in the plan's conclusions: a second `npm run tauri dev` launch focuses the first window; closing the
  window ends the process; `~/Library/WebKit/dev.macfresher.app` is not created.

## Notes from the implementation

- `tauri-plugin-single-instance` 2.5.5 builds with Tauri 2.12.5 and passes `cargo deny check` (MIT / Apache-2.0;
  it brings `tokio`, already in Tauri's tree).
- The behaviour is in `app/tauri/src/shell.rs`: `on_second_launch` (unminimize, show, focus the `main` window; the
  plugin ends the second process), `on_run_event` (`RunEvent::ExitRequested` → `on_exit`) and `on_exit` (the managed
  engine's `shutdown`: cancel and join; nothing when the engine was never set up). `run()` now builds the app and runs
  it with that handler.
- `exit_requested_shuts_the_engine_down` calls `on_exit` on a mocked app (`tauri::test`) holding an engine whose walk
  blocks until it is cancelled: the engine is idle afterwards and the last event is `ScanFinished { Partial }`.
  `ExitRequestApi` cannot be built outside Tauri, so the one-line `match` of `on_run_event` is not tested.
- The mock runtime does not track focus or visibility: `a_second_launch_reaches_the_main_window` proves only that the
  window is found and every call on it accepted.
- The CSP check allows only `'self'`, `data:`, `ipc:` and `http://ipc.localhost` in any directive, and
  `default-src 'self'` first. `the_main_window_is_incognito` also checks that `main` is the only window.
- Not checked by hand (the agent cannot open a window): a second `npm run tauri dev` focusing the first window, closing
  the window ending the process, and `~/Library/WebKit/dev.macfresher.app` not being created. To do before the plan's
  conclusions are final.
- **Fixed after review**: `devCsp` was `null`. It is now the production CSP plus `'unsafe-inline'` in `script-src`
  (Vite's React refresh preamble) and `style-src` (Vite injects styles), and `ws://localhost:1420` and
  `http://localhost:1420` in `connect-src` (hot reload); `the_dev_csp_is_the_production_csp_plus_what_the_dev_server_needs`
  checks exactly that. On desktop Tauri injects a CSP only into the pages it serves itself, never into a `devUrl` page,
  so `app/ui/vite.config.ts` sends `devCsp` as the dev server's `Content-Security-Policy` header (read from
  `tauri.conf.json`). The dev page was loaded in WebKit and Chromium under it: no violation, hot reload connected.
