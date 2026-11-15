# 002/03/01 — Data folder and atomic files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** store

## Files

- `crates/store/Cargo.toml` — description without SQLite ("the app's own data folder: settings and other versioned
  files"); dependencies `fresher-core`, `serde`, `serde_json`, `tempfile`, `thiserror`.
- `crates/store/src/data_folder.rs` — `DataFolder::open(path)`: an absolute path, created if missing with mode `0700`;
  refuses a relative path or a path that is a symlink; `DataFolder::file(name)` joins only plain file names (no
  separator, no `..`).
- `crates/store/src/atomic.rs` — `write_atomic(folder, name, bytes)`: `NamedTempFile::new_in(folder)`, write, `fsync`,
  `persist` (a rename), `fsync` of the folder; on any failure the old file is untouched and the temporary file is gone.
- `crates/store/src/versioned.rs` — the envelope `{ "format", "version", … }`: `read_versioned(bytes, format,
  supported, policy)` → `Current`, `Older(version)` (to migrate or discard by the policy), or
  `StoreError::NewerVersion`, `StoreError::Broken`.

## Produces

The file primitives 03/02 builds the settings file on and later plans (004 cache, 008 journal, 010 history) reuse.

## Checks

- Negative: `refuses_a_relative_data_folder`, `refuses_a_data_folder_that_is_a_symlink`,
  `refuses_a_file_name_with_a_separator`, `a_failed_write_leaves_the_old_file_intact`,
  `leaves_no_temporary_file_behind`, `a_newer_version_is_an_error`, `another_format_is_broken`.
- Positive: `writes_and_reads_back`, `creates_the_folder_private_to_the_user`, `an_older_version_is_reported_for_migration`.

## Notes from the implementation

- The data folder's own error is `DataFolderError` (`Relative`, `Symlink`, `NotAFolder`, `BadFileName { name }`,
  `Io { kind }`): the core's `StoreError` speaks of the settings file and has no variant for a bad folder. A path that
  exists but is not a folder is refused too (`NotAFolder`). Only the folder itself is checked for a link: its parents
  may be links (`/var` is one on macOS). An existing folder is opened as it is; a created one is set to `0700` after
  creation, whatever the umask.
- `write_atomic` and `read_file` (a missing file is `None`) live in `atomic.rs` and return the core's `StoreError`; a
  bad file name there is `StoreError::Io { kind: InvalidInput }` (the names are the store's own constants). The written
  file is `0600`, the mode of the temporary file.
- `read_versioned` returns `Current(body)`, `Older { version, body }` under `Migrate`, and `Discarded { version }` under
  `Discard` (the caller rebuilds the file). The body is the JSON object of the fields next to `format` and `version`
  (for the settings file: `values`). Version 0, a missing field or a version that is not a number is `Broken`.
  `write_versioned(format, version, body)` writes pretty JSON, `format` and `version` first, ending with a newline.
- The file primitives are `pub(crate)`: every file the app writes is written by this crate. The crate's public API in
  this step is `DataFolder` and `DataFolderError`.
- **Fixed after review: the write after the rename.** A failed `fsync` of the folder after a successful rename used to
  make the write fail, though the new file was in place; it is now logged (`tracing::warn`, the store's new
  dependency) and the write succeeds. The temporary files are named `.<target>.<random>.tmp`, so one left by a crash
  is recognized as the store's. Removing such leftovers in `DataFolder::open` is not done: only `fresher-cleaner` may
  delete and the store deletes nothing in this plan; see
  [`store-stale-temporary-files.md`](../../../known-issues/store-stale-temporary-files.md).
