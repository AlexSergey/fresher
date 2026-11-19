# Privacy

## No network

- **Rule:** the app makes no network requests: no telemetry, no analytics, no crash upload, no update check (until an
  update mechanism is agreed with the user).
- **Why:** the app sees the whole file system; nothing about it leaves the Mac.
- **Enforced by:** no HTTP client in the dependency tree (`deny.toml` bans), the Tauri capabilities allow no network,
  the UI's Content Security Policy allows no remote origin (plan 001).

## Local data only

- **Rule:** scan results, the scan cache, history and settings are stored only in the app's own folder in
  `~/Library/Application Support/`. Logs never contain file contents. The webview keeps no data of its own.
- **Why:** paths alone already reveal a lot about a person.
- **Enforced by:** `fresher-store`, the only crate that touches the app's data folder (the app hands it the folder);
  the window runs `incognito`, so the webview writes no cookies, caches or storage to the disk (plan 002); review.
- **Rule:** a development build started by `npm run dev` writes a record of each run (real paths of the Mac, never file
  contents) to `NO_COMMIT/dev-runs/` of the repository, which git ignores; a release build has no such code.
- **Enforced by:** `#[cfg(debug_assertions)]` on `app/tauri/src/dev_runs/`, the variable set only by `scripts/dev.ts`,
  `.gitignore`, the tests of `compose.rs` (`a_release_build_ignores_the_dev_runs_variable`).

## Least permissions

- **Rule:** the app asks for Full Disk Access only when a scan needs it, and explains why in plain English; it works
  with reduced coverage without it and says what it could not see.
- **Why:** trust; macOS shows the permission prompt to the user.
- **Enforced by:** the engine's coverage step and its tests (a scan never enters a guarded place that is not granted
  or opted into, and its result names what was not seen); the `Info.plist` test, which allows exactly five usage
  descriptions (`app/tauri/tests/info_plist.rs`); review. How it works: [`../architecture/permissions.md`](../architecture/permissions.md).
