# 002/03 — Settings and app data

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** store

`fresher-store`: the only crate that touches the app's data folder. Atomic, versioned files; `settings.json` behind
the core's `SettingsStore` port; the footprint of the app's own data. No SQLite. The design:
[`../00-design.md`](../00-design.md), "Settings and the app's own data".

Depends on plan 01. Runs in parallel with 02, 04, 06/01–03 and 07/01: it owns `crates/store/` and touches no other
crate. Every test runs on a `tempfile::TempDir` as the data folder; nothing reads the real
`~/Library/Application Support`.

## Steps

| #   | Step                                                   | Status      | Shared files |
| --- | ------------------------------------------------------ | ----------- | ------------ |
| 01  | [Data folder and atomic files](./01-atomic-files.md)   | ✔️ done     | `Cargo.lock` |
| 02  | [The settings file](./02-settings-file.md)             | ✔️ done     | —            |
| 03  | [The app's footprint](./03-footprint.md)               | ✔️ done     | —            |

In order.

## Conclusions

Built (2026-11-15): `fresher-store` holds the app's data folder, with no SQLite; 45 tests, 97.6 % of lines covered
(`cargo llvm-cov nextest -p fresher-store`). It deletes nothing.

- **Public API**: `DataFolder` (`open`, `path`, `file`, `footprint`) with `DataFolderError`; `FileSettingsStore::new`
  implementing the core's `SettingsStore`; `Footprint` with `over()`, and `FOOTPRINT_BUDGET` (50 MB).
- **Inside the crate** (`pub(crate)`, for the cache, the journal and the history of later plans): `write_atomic` and
  `read_file`, the envelope `read_versioned` / `write_versioned` with `Versioned { Current, Older, Discarded }`, and
  `migrate` with its `Migration` chain.

Deviations from the steps (each written in its step file, "Notes from the implementation"):

- The folder's own error type `DataFolderError` (the core's `StoreError` has no variant for a bad folder); a path that
  exists and is not a folder is refused too. `footprint()` returns a `Result`.
- `read_versioned` under `Discard` returns `Discarded { version }` instead of the body; version 0 is `Broken`.
- `save` reads the file first: a file whose migration fails is treated as broken and kept aside as
  `settings.broken.json`, like any broken file. `load` never writes.
- `crates/core` was not changed. Its `StoreError` messages speak of "the settings file"; when the cache or the
  journal reuse the primitives, a later plan may make them name the file.
- `docs/architecture/` still describes `fresher-store` as the SQLite snapshot store; that table is plan 07's to update
  (this plan touches only `crates/store/` and `Cargo.lock`).
