# 026/02/01 — The record's files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/tauri, docs

The parts of the recorder that need no engine: what the environment asks for, the file name, creating a pair that
never overwrites anything, appending an event line, rewriting the snapshot atomically, UTC times, and a core text in
English. Plain functions on plain values, each a unit test. Debug builds only. The design:
[`../00-design.md`](../00-design.md), sections 2.1 to 2.4 and the texts of 2.5 (agent's calls 8, 9, 15 and 17).

## Files

- `app/tauri/src/lib.rs` — `#[cfg(debug_assertions)] mod dev_runs;` with `#[cfg_attr(not(test), expect(dead_code,
  reason = "the recorder uses it from 026/02/02"))]` (removed by 02/02).
- `app/tauri/src/dev_runs/mod.rs` (new) — the module doc (what a run record is, debug builds only, written only where
  `FRESHER_DEV_RUNS` says; `docs/plans/026-dev-run-record/00-design.md`), and:
  - the variable names `DEV_RUNS_VARIABLE = "FRESHER_DEV_RUNS"`, `COMMIT_VARIABLE = "FRESHER_BUILD_COMMIT"`,
    `DIRTY_VARIABLE = "FRESHER_BUILD_DIRTY"`;
  - `#[derive(Clone, Debug, PartialEq, Eq)] pub(crate) struct Launch { folder: PathBuf, build: BuildName }` and
    `Launch::from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Option<Self>, DevRunsRefusal>`: `None` without
    `FRESHER_DEV_RUNS`; `NotAbsolute` for a relative one; the build from the two other variables;
  - `#[derive(Clone, Debug, PartialEq, Eq, Serialize)] pub(crate) struct BuildName { version: String, commit: String,
    dirty: bool }` with `BuildName::new(version: &str, commit: Option<&OsStr>, dirty: Option<&OsStr>)` (a commit of 4 to
    40 characters of `0-9a-f` is kept, anything else is `unknown`; dirty only for exactly `1`) and `name(&self) ->
    String` (`<version>+<commit>[-dirty]`); serialised through a small `BuildRecord<'a>` with `version`, `commit`,
    `dirty` and `name`;
  - `#[derive(Clone, Debug, PartialEq, Eq)] pub(crate) enum Mode { Mac, Sandbox(String) }`, `Mode::of(build: &Build)
    -> Option<Mode>` (`Release` → `None`; `Development` → `Mac`; `Sandbox { path, .. }` → its last component, every
    character but ASCII letters, digits, `-` and `_` written `-`, empty → `sandbox`) and `Mode::label(&self) ->
    String` (`mac`, `sandbox-<name>`);
  - `#[derive(Debug, thiserror::Error)] pub(crate) enum DevRunsRefusal { NotAbsolute { path }, Folder { path, source:
    io::Error }, NotAFolder { path }, Taken { stem }, Write { path, source } }` with plain messages naming the variable.
- `app/tauri/src/dev_runs/clock.rs` (new) — `pub(crate) fn utc(time: SystemTime) -> Utc` with `Utc { year, month,
  day, hour, minute, second }` (whole seconds since the epoch, a time before it is the epoch; the civil date from the
  day count by the usual days-to-civil arithmetic, written from scratch), `Utc::stamp(&self) -> String`
  (`YYYY-MM-DD_HH-MM-SS`), `Utc::iso(&self) -> String` (`YYYY-MM-DDTHH:MM:SSZ`), `Utc::date(&self) -> String`
  (`YYYY-MM-DD`).
- `app/tauri/src/dev_runs/files.rs` (new) —
  - `pub(crate) fn file_stem(started: SystemTime, build: &BuildName, mode: &Mode, attempt: u32) -> String`:
    `<stamp>_<build>_<mode>` for attempt 1, `…-<attempt>` from 2;
  - `pub(crate) struct RunFiles { folder: PathBuf, snapshot: PathBuf, events: File }` and `RunFiles::create(folder:
    &Path, started, build, mode) -> Result<Self, DevRunsRefusal>`: `fs::create_dir_all(folder)`, then
    `fs::symlink_metadata(folder)` must be a folder and no link (`NotAFolder`); for attempt 1 to 99: when either path
    of the pair exists (`symlink_metadata`), the next attempt; otherwise the `.json` and then the `.events.jsonl` are
    each opened with `OpenOptions::new().write(true).create_new(true)` (the events file with `append(true)`); an
    `AlreadyExists` there (a race) moves to the next attempt, and a file this attempt already made stays empty and is
    never written again (nothing outside the cleaner may delete); after 99, `Taken`. Nothing that existed is ever
    opened;
  - `RunFiles::append(&mut self, line: &serde_json::Value) -> io::Result<()>`: one compact line and `\n`, written with
    one `write_all` on the events file opened in append mode, then `flush`;
  - `RunFiles::rewrite(&self, record: &impl Serialize) -> io::Result<()>`: `tempfile::Builder::new().prefix(".<stem>.")
    .suffix(".tmp").tempfile_in(folder)`, pretty JSON, `sync_all`, `persist(&self.snapshot)` (a failed persist drops the
    temporary file); nothing else in the folder is touched;
  - `RunFiles::snapshot(&self) -> &Path`.
- `app/tauri/src/dev_runs/english.rs` (new) — `pub(crate) fn english(dictionary: &Dictionary, text: &Text) ->
  Option<String>`: the design's section 2.5 (`Count` its number, `Size` `<bytes> bytes`, `Date` `clock::utc(..).date()`,
  `Path` its `display()`, `Text` as it is); a placeholder the text does not set stays as written.
- `app/tauri/Cargo.toml` — `serde.workspace = true`, `serde_json.workspace = true` and `tempfile.workspace = true`
  move to (or join) `[dependencies]`, each with the comment "the development run record (`src/dev_runs/`), debug
  builds only"; removed from `[dev-dependencies]` where they were (the tests keep them through `[dependencies]`).
- `docs/dependencies.md` — the `serde` row's "Used by" gains `app`; the paragraph under "Build and checks only" that
  says `serde_json` and `tempfile` reach the release graph through `fresher-testing` adds that the app now names them
  itself, for the development run record compiled in debug builds only.

## Produces

`Launch`, `BuildName`, `Mode`, `DevRunsRefusal`, `RunFiles` (`create`, `append`, `rewrite`, `snapshot`), `clock::utc`,
`english`, for 02/02.

## Checks

Tests in each file, negatives first; every folder is a `tempfile::tempdir()`, no test reads the process environment.

- `mod.rs`:
  - `mod negative`: `no_variable_starts_no_run` (`from_env` with a `var` that answers nothing: `Ok(None)`);
    `a_relative_folder_is_refused` (`NotAbsolute`); `a_commit_that_is_not_hex_is_unknown` (`"26E4CB0"`, `"abc"`,
    `"26e4cb0; rm"`, 41 digits: `unknown`); `dirty_is_only_one` (`"true"`, `"yes"`, `""`, absent: clean);
    `a_release_build_has_no_mode`;
  - `mod positive`: `the_build_name_is_version_plus_commit_and_dirty` (`0.1.0+26e4cb0-dirty`, `0.1.0+unknown`);
    `a_sandbox_mode_is_named_by_its_folder` (`/x/sandbox/trial` → `sandbox-trial`, `/x/my box!` → `sandbox-my-box-`);
    `a_development_build_is_mac`.
- `clock.rs`:
  - `mod negative`: `a_time_before_the_epoch_is_the_epoch` (`1970-01-01T00:00:00Z`);
  - `mod positive`: `known_instants_are_written_in_utc` (0; 951 782 400 → `2000-02-29T00:00:00Z`; 1 763 633 071 →
    `2025-11-20T10:04:31Z`; 4 107 542 399 → `2100-02-28T23:59:59Z`), `the_stamp_has_no_colon` (`2025-11-20_10-04-31`).
- `files.rs`:
  - `mod negative`: `an_existing_record_is_never_opened` (a `.json` of the first name written beforehand with known
    bytes: the new pair is `-2`, the old file's bytes are unchanged); `an_existing_events_file_moves_to_the_next_name`
    (only the `.events.jsonl` of the first name exists: the pair is `-2`, the old one unchanged);
    `ninety_nine_taken_names_refuse` (attempts 1 to 99 made beforehand: `Taken`, nothing changed);
    `a_folder_that_is_a_link_is_refused` (`NotAFolder`; nothing written through the link);
    `a_file_in_place_of_the_folder_is_refused`; `a_rewrite_leaves_no_temporary_file_and_touches_nothing_else` (another
    file in the folder keeps its bytes; no `.tmp` left);
  - `mod positive`: `the_first_pair_is_named_by_time_build_and_mode`
    (`2025-11-20_10-04-31_0.1.0+26e4cb0-dirty_mac.json` and `….events.jsonl`); `the_folder_is_created`;
    `events_are_appended_one_line_each` (three appends: three lines, in order, each valid JSON);
    `the_snapshot_is_replaced_whole` (two rewrites: the file holds the second, parsed).
- `english.rs`:
  - `mod negative`: `a_key_the_dictionary_lacks_has_no_english` (`None`); `an_unset_placeholder_stays_as_written`;
  - `mod positive`: `every_kind_of_argument_is_written` (a dictionary parsed from a small JSON with one template
    naming a text, a count, a size, a date and a path).
- `cargo nextest run -p fresher-app --lib dev_runs`; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo clippy -p fresher-app --release --all-targets -- -D warnings`; `cargo fmt --check`; `cargo machete`;
  `cargo deny check`. `export CARGO_TARGET_DIR=…/fresher/target`.

## Conclusions

Done 2026-11-19. 25 tests pass; clippy (workspace and `fresher-app --release`), fmt, machete and deny are clean.

- `file_stem` (the stem both files share, no extension) is the design's pure `file_name`; the design's name stays only
  in `00-design.md` (ruling C8).
- `BuildName` implements `Serialize` by hand through `BuildRecord<'a>` (borrowed, adds `name`), instead of a derive, so
  02/02's `Session` can hold the owned `BuildName` (C8).
- `clock`, `english` and `files` are `pub(crate) mod` of `dev_runs`; 02/02 reaches them as `super::files::RunFiles`.
  Added `clock::at_secs(u64) -> SystemTime`, used by `english` for a `Timestamp` date.
- `a_file_in_place_of_the_folder_is_refused` ends in `Folder` (from `create_dir_all`); the test accepts that or
  `NotAFolder`.
- A leftover empty file of a lost race stays (nothing outside the cleaner deletes).
- `Cargo.lock` gained `serde` under `fresher-app` only (the crates were already in the graph).
