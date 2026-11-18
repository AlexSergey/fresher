# 019/01/03 — Spotlight's reach

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, macos, system, testing, modules, app, docs

Before a module takes "Spotlight found no app with this id" as "not installed", the system asks whether Spotlight can
see every mounted volume; a volume left out of the index, a network share or a timeout makes the search incomplete,
and every module that asks (`jetbrains`, `app-leftovers`) keeps everything. Closes the known issue
[Spotlight misses an excluded volume](../../../known-issues/app-spotlight-misses-an-excluded-volume.md). The design:
[`../00-design.md`](../00-design.md), section 1.3 (agent's call 7).

## Files

- `crates/core/src/system/apps.rs` — `InstalledApps` gains `fn search_complete(&self, cancel: &CancelToken) ->
  Result<(), SystemError>`, documented as in the design; no default body.
- `crates/core/src/system/facts.rs` — `SystemFacts` gains `search: OnceLock<Result<(), SystemError>>`; `find_apps`
  first gets `search_complete` (asked once per scan, its answer kept) and returns its error before any `find`; the
  `Nothing` and `Counting` test ports answer `Ok(())`.
- `crates/macos/src/volume.rs` — `pub struct MountedVolume { pub mount_point: PathBuf, pub local: bool, pub browsable:
  bool }` and `pub fn mounted_volumes() -> io::Result<Vec<MountedVolume>>`, from `ffi::volume::mounts()` (`MNT_LOCAL`,
  `MNT_DONTBROWSE` in `f_flags`, `f_mntonname`); `devfs` and `autofs` mounts are left out (they hold no apps).
  Re-exported from `lib.rs`. No new `unsafe`.
- `crates/system/src/spotlight.rs` (new) — `pub trait Mounts: Send + Sync { fn mounted(&self) ->
  io::Result<Vec<MountedVolume>>; }`, `pub struct MacMounts` (calls `fresher_macos::mounted_volumes`);
  `pub const INDEX_CHECK_TIMEOUT: Duration = Duration::from_secs(10)`; `pub struct SpotlightSearch { find:
  Arc<dyn CommandRunner>, index: Arc<dyn CommandRunner>, mounts: Arc<dyn Mounts> }` with `new(find, index, mounts)`
  and `search_complete(cancel)`:
  1. the Finder probe (moved here from `find`): no answer for `com.apple.finder` → "Spotlight does not know the
     Finder: its index is off";
  2. the mounts cannot be listed → an error;
  3. for every browsable mount: not local → "a network volume is mounted" (never asked: the command may hang); local →
     `mdutil -s <mount point>` through `index`; indexed only when the status is 0 and the output holds
     `Indexing enabled.` (to verify in 04/02 on the developer's Mac, with an indexed and an unindexed volume); anything
     else, a timeout included, → "a volume is not indexed" (the mount point logged, never in the error's text).
- `crates/system/src/apps.rs` — `AppSearch::Spotlight(SpotlightSearch)` (was a runner); `AppSearch::Listed` becomes
  `AppSearch::Declared(DeclaredSearch)`, `pub struct DeclaredSearch { pub index_on: bool, pub elsewhere:
  Vec<(BundleId, PathBuf)> }` with `DeclaredSearch::listed()` (index on, nothing elsewhere: today's `Listed`);
  `MacInstalledApps::find` keeps Spotlight's per-id query without the probe, and for `Declared` answers the listed apps
  with the ids plus the `elsewhere` paths with them; `search_complete` asks `SpotlightSearch`, or answers `Ok` when
  `index_on` and an error ("the sandbox's Spotlight is off") otherwise.
- `crates/testing/src/system_fakes.rs` — `FakeInstalledApps::search_complete`: an error when built with
  `failing_search(reason)`, `Ok` otherwise; `find` answers the `with_elsewhere` apps as today (its own failure kept
  for a test that needs it, if any uses it; otherwise removed); `search_calls()` counts. A `FakeMounts` beside it.
- `app/tauri/src/compose.rs` — `AppSearch::Spotlight(SpotlightSearch::new(commands, index, Arc::new(MacMounts)))`,
  `index` a `ProcessCommandRunner::new(INDEX_CHECK_TIMEOUT)`; `app/tauri/src/sandbox.rs` —
  `AppSearch::Declared(DeclaredSearch::listed())` (03/01 fills it from the sandbox's world).
- `modules/jetbrains/locales/en.json` — `jetbrains.left-out.spotlight-unread`: "Fresher could not ask Spotlight about
  every drive, so this IDE may be installed elsewhere. It keeps its data. If Spotlight is off, or a drive is not
  indexed, turn it on and scan again." `docs/categories/jetbrains-uninstalled.md`: the same reason in its decision.
- `modules/jetbrains/src/tests.rs` — a removed IDE's data kept `spotlight-unread` when `search_complete` fails while
  `find` would find nothing.
- `docs/known-issues/app-spotlight-misses-an-excluded-volume.md` removed and its row in `docs/known-issues/README.md`
  removed (the README's rule for a fixed issue, if it keeps fixed ones instead: status `fixed` with this plan's link).
- `docs/architecture/modules.md` — the `find_apps` row: "after checking once per scan that Spotlight sees every
  mounted volume; an error when it cannot".

## Produces

`InstalledApps::search_complete`, `SpotlightSearch`, `Mounts`, `MacMounts`, `INDEX_CHECK_TIMEOUT`,
`mounted_volumes`, `AppSearch::Declared(DeclaredSearch)`, for 02/02 (rows 10 and 11), 03/01 (the sandbox's
Spotlight) and `jetbrains`.

## Checks

- `spotlight.rs`, negatives first, with `FakeCommandRunner` answers and `FakeMounts`: the Finder unknown; `mdfind`
  failing; the mounts failing; a network volume (no command run for it); `mdutil` answering `Indexing disabled.`,
  `Error: unknown indexing state.`, status 1, failing, or cancelled; then two local volumes indexed → `Ok`; a volume
  not browsable skipped; each command run with exactly `["-s", "<mount point>"]`, never through a shell.
- `apps.rs`: `Declared` answers its elsewhere paths for their ids only; `Declared` with the index off fails
  `search_complete`; `find` no longer runs the Finder probe.
- `facts.rs`: `search_complete` asked once for two `find_apps` calls; its error returned without calling `find`.
- `volume.rs` (`crates/macos`): on this Mac, `/` is listed, local and browsable; a `-nobrowse` disk image the test
  mounts (as the crate's tests do) is local and not browsable.
- `jetbrains`: its tests and golden pass; the new test above.
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-system -p fresher-testing -p fresher-module-jetbrains
  -p fresher-app`; clippy; fmt; the crate guard (`unsafe` only in `ffi`).

## Conclusions

Built (b747c5d): `InstalledApps::search_complete(cancel)` in the core, asked once per scan by `find_apps` before any
search (the answer is kept for the scan, a cancelled first ask included); in `fresher-macos`, `mounted_volumes()` and
`mount_point_of(path)` (`getfsstat`, `statfs`; no new `unsafe`); in `fresher-system`, `spotlight.rs`: the Finder probe,
the mounts, then `mdutil -s <mount point>` for every browsable volume and the one that holds the home (10 s each,
indexed only on status 0 and `Indexing enabled.`); a network volume counts as not indexed without being asked; any
volume not indexed or unknown keeps everything. `AppSearch::Declared` for a sandbox's declared apps. The jetbrains text
and `docs/categories/jetbrains-uninstalled.md` carry the reason. `FakeInstalledApps::failing_search` now fails
`search_complete` only (preflight F9).

Deviations and calls (both stricter than the brief):

- **The home's volume is found by `statfs`** (`mount_point_of`), not by the longest prefix of the home path, and is
  always asked, browsable or not: on APFS the home is textually under `/`, but its data lives on
  `/System/Volumes/Data`, a `nobrowse` mount. `search_complete` takes the home from `KnownFolders`; no home or an
  unlocated volume keeps everything.
- `FakeMounts` lives in `spotlight.rs`'s tests, not in `fresher-testing`: the trait is in `fresher-system`, which
  dev-depends on `fresher-testing`.
- The known issue stays with the status `fixed` and a link (the folder's rule), not removed (preflight F19).
- A mounted local, browsable volume without an index turns the detection off for the scan, in both modules. Known issue
  [`app-unindexed-volume-turns-off-removed-app-detection.md`](../../../known-issues/app-unindexed-volume-turns-off-removed-app-detection.md);
  the privacy-list folders and nested apps Spotlight cannot answer are in
  [`app-spotlight-children-and-privacy-list.md`](../../../known-issues/app-spotlight-children-and-privacy-list.md).

Tests (negatives first): the core asks once and answers the incomplete search without `find`; `mounted.rs` (devfs and
autofs left out, an unreadable mount point is an error, a disk image local and not browsable, a missing path); the
Finder unknown, `mdfind` failing, a failing mount list, a home volume not located, a network volume never asked, `mdutil`
disabled, unknown, failing, cancelled or timed out, the data volume asked and not indexed, a non-browsable volume that
does not hold the home skipped, then all indexed with the exact call order and arguments (a mount point with a space
stays one argument). `cargo nextest run -p fresher-core -p fresher-macos -p fresher-system -p fresher-testing
-p fresher-module-jetbrains -p fresher-app`: 1141 passed.

To verify by hand (04/02): what `mdutil -s` says on `/` and `/System/Volumes/Data`, indexed and not, and on an attached
backup disk.

Parked minors: no test of a status other than 0 together with `Indexing enabled.` (the fake gives no stdout with an exit
status); `apps.rs` tests reach the `spotlight::tests` helpers.
