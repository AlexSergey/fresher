# 006/03/03 — The app's adapters

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app

## Files

- `app/tauri/src/access.rs` (new) — `MacAccess { home: PathBuf }` implements `AccessProbe`: `status()` maps
  `full_disk_access` onto `AllFiles` (`Allowed` → `Granted`, `Refused` → `Denied`, otherwise `Unknown`); with `AllFiles`
  granted every folder and other apps' data is `Granted`, otherwise `NotAsked`; other apps' data is `NotApplicable`
  before macOS 14; `guarded_places(roots)` maps `fresher_macos::privacy::guarded_places(home)` and the volume guard of
  each root onto `GuardedPlace`s that lie inside a root; `try_place` maps `Probe`. `MacSystemUi { commands: Arc<dyn
  CommandRunner> }` implements `SystemUi`: `open_access_settings` picks the pane of the capability and runs
  `/usr/bin/open <address>`; `reveal` runs `/usr/bin/open -R <path>`; a `Tool` capability is `Unsupported`.
- `app/tauri/src/adapters.rs` — `UnknownAccess` and `NoSystemUi` removed.
- `app/tauri/src/compose.rs` — `MacAccess` with the home folder Tauri reports, `MacSystemUi` with the existing
  `ProcessCommandRunner`.
- `docs/hacks/README.md` — the entry "Access answers Unknown until plan 006" removed.

If plan 009's `fresher-system` exists when this step starts, the two adapters go there instead and the app only
composes them (design, agent's call 2).

## Produces

The real access of the app. From here on the window shows what macOS actually allows.

## Checks

On temporary trees and `FakeCommandRunner`; no test opens System Settings or reads the real home folder.

- Negative: `a_home_without_any_candidate_gives_unknown_full_disk_access`, `without_full_disk_access_the_folders_are_not_asked`,
  `a_guarded_place_outside_every_root_is_not_reported`, `a_tool_has_no_settings_pane`,
  `a_failed_open_is_reported_as_a_system_ui_error`.
- Positive: `a_readable_candidate_grants_all_files_and_every_folder`, `open_access_settings_runs_open_with_the_pane_address`
  (the exact program and arguments recorded by the fake), `reveal_runs_open_with_dash_r_and_the_path`,
  `guarded_places_of_the_home_root_are_reported`.
