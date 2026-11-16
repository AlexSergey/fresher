# 009/01/02 — The crate and the macOS folders

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** system, app, workspace, scripts

## Files

- `crates/system/` (new) — `fresher-system`: `Cargo.toml` (core, macos; workspace lints), `src/lib.rs`
  (`#![forbid(unsafe_code)]`), `src/known_folders.rs`: `MacKnownFolders::new(home)`, the macOS layout of every
  `KnownFolder`; pure, opens nothing.
- `src/clock.rs` — `SystemClock`, moved from `app/tauri/src/adapters.rs`.
- `scripts/check-crate-deps.ts` — the row `fresher-system`: core, macos; the app may depend on it.
- `app/tauri/src/compose.rs` — the scanner's anchor from `MacKnownFolders`.

## Produces

`fresher-system` with the known folders the app gives the scanner (and the engine, 02/02).

## Checks

- Negative: a relative home makes no folder (`None` for all); the layout never answers a path outside the home but
  for the two system apps folders.
- Positive: every `KnownFolder` has a place on macOS; `AppCaches` is `<home>/Library/Caches`.
- `npm run check:crate-deps`, `cargo nextest run -p fresher-system -p fresher-app`.

## Conclusions

- Built as written. `fresher-system` depends on the core alone for now (`fresher-macos` joins with 04/01, so
  `cargo machete` stays quiet); the crate guard's table has its row and `system` is a commit scope.
- `MacKnownFolders::new(home)` answers every known folder; a home that is not absolute answers only the two apps
  folders of all users. The app builds the scanner's anchor from it (`known_folders_anchor`).
- `SystemClock` moved from the app's adapters into the crate; the app's stand-ins (`UnknownAccess`, `NoSystemUi`) stay
  where they are until plan 006/03.
