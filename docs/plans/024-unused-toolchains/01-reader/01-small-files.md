# 024/01/01 — The small-file reader

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** core, testing, app, e2e, docs

`cx.system().small_file(folder, name)`: a module reads a small text file its own matches showed, in `finish`, through
the reader the walk already uses for `CACHEDIR.TAG`. No walk, scanner or scan cache change. The design:
[`../00-design.md`](../00-design.md), section 1.

## Files

- `crates/core/src/system/small_file.rs` (new) — `SMALL_FILE_LIMIT` (4096), `enum SmallFile { Text(String), Missing,
  NotAFile, TooLong, NotText, Unreadable }`, `trait SmallFiles: Send + Sync { fn head(&self, folder: &Path, name:
  &str, limit: usize) -> Result<Vec<u8>, ReadError>; }`, `pub struct ReaderFiles(pub Arc<dyn FolderReader>)`
  implementing `SmallFiles` by `FolderReader::head`, and `pub(crate) fn judge(read: Result<Vec<u8>, ReadError>) ->
  SmallFile` (more than the limit → `TooLong`; `Vanished` → `Missing`; `Replaced` → `NotAFile`; any other error →
  `Unreadable`; not UTF-8 → `NotText`). Documented: read-only, never follows a link, never logs the bytes.
- `crates/core/src/system/facts.rs` — `SystemPorts` gains `files: Arc<dyn SmallFiles>`; `SystemFacts::small_file(&self,
  folder: &Path, name: &str) -> SmallFile` asks `SMALL_FILE_LIMIT + 1` bytes and judges them, at every call (nothing
  kept); its doc states the rule of use (only a file the module's own matches showed). `nothing_known()` gets a reader
  that answers `Unreadable`. `crates/core/src/system/mod.rs`, `crates/core/src/lib.rs` — the exports.
- `crates/testing/src/small_files.rs` (new) — `DiskFiles` (the standard library: `symlink_metadata` of
  `folder/name`; not there → `ReadError::Vanished`; not a regular file → `Replaced`; else the first `limit` bytes) and
  `NoFiles` (every read `ReadError::Failed`); exported. `system_fakes.rs` — `fake_system(home)` gives `DiskFiles`.
  `run_module.rs` — its `SystemPorts` gives `DiskFiles` (the fixture tree is on the disk); `Ports` gains `files:
  Option<Arc<dyn SmallFiles>>` to replace it in a test.
- Every other `SystemPorts { .. }` literal (the engine's, the modules' and the e2e's tests, about twenty) — `files:
  Arc::new(NoFiles)` or the harness's `DiskFiles` where the home is on the disk (`crates/e2e/tests/support/home_engine.rs`,
  `crates/e2e/tests/jetbrains_home/mod.rs`, `crates/e2e/tests/places.rs`). No other change in them.
- `app/tauri/src/compose.rs` — `files: Arc::new(ReaderFiles(Arc::clone(&reader)))` from the same reader the scanner is
  given (in sandbox mode the sandbox's stated lens over it), so the sandbox's files are read as the walk reads them.
- `docs/architecture/modules.md` — the row `small_file(folder, name)` in "The system's facts": "a text file of at most
  4 KiB, read whole at every call through the walk's reader (a regular file only, never a link); `Missing`,
  `NotAFile`, `TooLong`, `NotText`, `Unreadable` otherwise. Only for a file the module's own matches showed: a name its
  anchored interest holds, the marker of its marker match, or a file it matched"; and a bullet on its tests
  (`DiskFiles`, `NoFiles`).

## Produces

`SystemFacts::small_file`, `SmallFile`, `SmallFiles`, `DiskFiles`, `NoFiles`, for 02 and 03.

## Checks

- `small_file.rs`, negatives first: `mod negative` `a_missing_file_is_missing`, `a_link_or_a_folder_is_not_a_file`,
  `a_file_over_the_limit_is_too_long` (4097 bytes), `bytes_that_are_not_utf8_are_not_text`,
  `any_other_read_error_is_unreadable` (each `ReadError` variant mapped); `mod positive`
  `a_file_at_the_limit_is_read_whole` (4096 bytes), `an_empty_file_is_empty_text`,
  `each_call_reads_the_disk_again` (a counting fake answers twice).
- `small_files.rs` in `fresher-testing` on a temporary folder: a link to a regular file is `Replaced`, a folder is
  `Replaced`, a missing name is `Vanished`; a regular file's first bytes.
- The app's composition test: the system's `files` reads a file of the sandbox's home through the stated lens.
- `cargo nextest run -p fresher-core -p fresher-testing -p fresher-engine -p fresher-app` and the modules' and the e2e's
  tests that build `SystemPorts`; clippy; fmt; `git diff -- crates/scanner crates/cleaner` empty.

## Conclusions

Built as written. `SmallFiles` is a core port beside the folder reader: `SystemFacts::small_file(folder, name)` reads
up to 4 KiB of a regular file, never through a link, and answers `Text`, `Missing`, `NotAFile`, `TooLong`, `NotText` or
`Unreadable`; nothing is kept between calls. The production reader is the scanner's own, so in sandbox mode it reads
through the same facts lens. The walk, the scanner, the cache and the cleaner are unchanged. Calls taken while building:
a `Debug` impl for `dyn SmallFiles` (the test `Ports` derives it); the test reader `DiskFiles` also compares the
opened file's device and inode with the entry it saw, maps a refused permission to `NoPermission`, and (final fix
wave) opens with `O_NOFOLLOW | O_NONBLOCK` so a link or a pipe swapped in between is `Replaced`; no test proves that
race, it cannot be reproduced without one. The app's tests that run a whole sandbox time out only when every crate's
tests share one nextest run, and pass alone.
