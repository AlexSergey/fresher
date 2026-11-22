# 037/01/01 — The types and the ports

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, macos, system, testing

The types the Trash line is made of, the refusal for an unknown Trash, and the two ports the engine will use: opening
a folder in Finder, and reading Finder's 30-day setting. Each port gets its macOS reader and its fake in the same
step, so every implementor compiles. Nothing calls them yet (02 does). The design:
[`../00-design.md`](../00-design.md), sections 1 and 3.

Read first: `CLAUDE.md`, `docs/rules.md`, `crates/core/src/cleanup/view.rs` (the shape and doc style of the cleanup
views), `crates/core/src/ports.rs` (`SystemUi`, `CommandRunner`), `crates/core/src/error.rs` (how a `CommandError`
variant gets its code, text key and tests: follow `UnknownCleanupItem`), `crates/system/src/system_ui.rs`,
`crates/macos/src/privacy/panes.rs` (`reveal_args`), `crates/testing/src/fake_access.rs` (`FakeSystemUi`).

## Files

- `crates/core/src/cleanup/trash.rs` (new) — the module doc "The Trash as the scan screen shows it: what waits in
  each Trash, and how much of it the app's cleanups put there. Information only: nothing here empties a Trash." Two
  types, `#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]`:
  - `TrashView { pub trashes: Vec<TrashLine> }` — doc: "The answer of `get_trash`: each Trash that holds something,
    the home volume's first."
  - `TrashLine` with `#[serde(rename_all = "camelCase")]`: `index: u32` ("What `open_trash` names it by: its place
    in this answer."), `volume: Option<String>` ("Its volume's name; `None` when the volume could not be read."),
    `waiting: Option<Sizes>` ("Everything in this Trash: the last scan's measure plus what cleanups moved there since
    and is still there; `None` when no scan measured it."), `from_cleanups: Sizes` ("What the app's cleanups moved
    there and is still there, found again now."), `gone_by: Option<Timestamp>` ("With Finder's 30-day setting on:
    when the last of `from_cleanups` leaves the Trash by itself; `None` otherwise.").
  - Tests, `mod negative` first: a serialized view holds no path (`!json.contains('/')` on a view with a volume name
    without a slash) and no snake-case key; then `mod positive`: a line serializes `fromCleanups` and `goneBy`, and a
    line with `waiting: None` serializes `"waiting":null`.
- `crates/core/src/cleanup/mod.rs`, `crates/core/src/lib.rs` — `mod trash;` and the re-export of `TrashView` and
  `TrashLine` beside the other cleanup views.
- `crates/core/src/error.rs` — `CommandError::UnknownTrash { index: u32 }`, doc "No Trash of the last `get_trash`
  answer has this index.", code `unknown-trash`, in every list and match the variant `UnknownCleanupItem` is in
  (its kind, its sample in the tests). `crates/core/locales/en.json`: `"core.error.unknown-trash": "This Trash is no
  longer listed. Look again on the scan screen."`. Fix every exhaustive match the compiler reports elsewhere
  (`app/tauri`'s `Refusal`, if any) the way `UnknownCleanupItem` is handled.
- `crates/core/src/ports.rs`:
  - `SystemUi::open_folder(&self, path: &Path) -> Result<(), SystemUiError>`, doc "Opens a folder in the system's
    file manager, as the user would by double-clicking it."
  - `pub enum AutoEmpty { After30Days, Never, Unknown }` (`Clone, Copy, Debug, PartialEq, Eq`), doc "Whether Finder
    removes items from the Trash 30 days after they were put there (Finder → Settings → Advanced)."; each variant
    documented (`Unknown`: "The setting could not be read.").
  - `pub trait FinderSettings: Send + Sync { fn trash_auto_empty(&self) -> AutoEmpty; }`, doc "Finder's settings the
    app reads, never writes." Re-export both from `lib.rs`.
- `crates/macos/src/privacy/panes.rs` — `pub fn open_folder_args(path: &Path) -> [&OsStr; 1]`: the path alone, kept
  as one `OsStr`, never formatted; re-exported where `reveal_args` is. Test (negative): a path with spaces and quotes
  stays one argument.
- `crates/system/src/system_ui.rs` — `MacSystemUi::open_folder`: a path that is not absolute answers
  `SystemUiError::Failed { reason: "the path is not absolute".into() }` and runs nothing; otherwise `/usr/bin/open`
  with `open_folder_args` (a path not valid UTF-8 refused as `reveal` refuses it). Tests, negatives first: a relative
  path runs nothing; a failing `open` is a `Failed`; then: `open_folder` runs `/usr/bin/open /tmp/x` (the
  `FakeCommandRunner` call list).
- `crates/system/src/finder_settings.rs` (new) — `pub struct MacFinderSettings { commands: Arc<dyn CommandRunner> }`
  with `new(commands)` and a `Debug` that hides the runner (as `MacSystemUi`'s). `trash_auto_empty` runs
  `/usr/bin/defaults` with `["read", "com.apple.finder", "FXRemoveOldTrashItems"]` and a fresh `CancelToken`:
  status 0 and trimmed stdout `1` → `After30Days`; status 0 and `0` → `Never`; a non-zero status whose stderr
  contains `does not exist` → `Never` (Finder's default, off); any other answer or a runner error → `Unknown`, with a
  `tracing::warn!` that names no path. Constants `DEFAULTS: &str = "/usr/bin/defaults"` and the three arguments, each
  with a one-line doc. It only reads: no `write` argument exists in the file. Re-export from `lib.rs`. Tests,
  negatives first: the runner failing, an unexpected stdout (`yes`), another non-zero status → `Unknown`; the missing
  key → `Never`; then `1` → `After30Days`, `0` → `Never`, and the exact program and arguments run.
- `crates/testing/src/fake_access.rs` — `SystemUiCall::OpenFolder(PathBuf)`; `FakeSystemUi::open_folder` records it
  (and answers the configured error, as the other calls do); a test beside the existing ones.
- `crates/testing/src/system_fakes.rs` (or the file the system fakes live in) — `FakeFinderSettings(AutoEmpty)` with
  `FakeFinderSettings::new(answer)`, implementing `FinderSettings`; re-exported from `lib.rs`.

## Produces

`TrashView`, `TrashLine`, `CommandError::UnknownTrash`, `AutoEmpty`, `FinderSettings`, `SystemUi::open_folder`,
`MacFinderSettings`, `FakeFinderSettings`, `SystemUiCall::OpenFolder` — for 02 and 03.

## Checks

- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-system -p fresher-testing`.
- `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`; the crate guard.
- The whole workspace builds (`cargo check --workspace --all-targets`): every `SystemUi` implementor has the new
  method.
- `UPDATE_BINDINGS=1` is not needed yet (no command answers the new types until 03).

## Conclusions

Built as written: `TrashView`/`TrashLine` in `crates/core/src/cleanup/trash.rs`, `AutoEmpty` and `FinderSettings` in
the ports, `SystemUi::open_folder` (`MacSystemUi`, `FakeSystemUi`, `open_folder_args` in `fresher-macos`'s
`privacy/panes.rs`, no FFI), `MacFinderSettings` (`defaults read` through the `CommandRunner`, never a write; a test
reads the file and finds no `write`), `FakeFinderSettings`, `CommandError::UnknownTrash` with the code
`unknown-trash`. 1093 tests ran in `core`, `macos`, `system` and `testing`, all passed.

What changed from the design:

- `UnknownTrash`'s `index` is a field, not a text argument: the dictionary test requires every argument to appear as
  a placeholder, and the text has none.
- `bindings.generated.ts` was regenerated in this step, not in 01/03: `CommandError` is in the bindings and the
  freshness test fails otherwise.
- `FakeCommandRunner::exiting_with_stderr` was added for the "does not exist" answer.
- The manual-testing numbers are 1.17 and 2.15 (plans 033 and 034 took the earlier ones).

Facts still to verify on the Mac: `FXRemoveOldTrashItems` prints `1` with Finder's setting on.
