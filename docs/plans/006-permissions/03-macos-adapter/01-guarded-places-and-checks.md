# 006/03/01 — Guarded places and checks

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** macos

The table of what macOS guards and the checks that never prompt, in `fresher-macos`. It encodes the design's "Probe
results" as they stand (from documentation, unverified: agent's call 24); the user's run of plan 01 may change the
table and the candidates later, in this module only.

## What exists already

- `crates/macos/src/places.rs` — `places_that_prompt(home)`: the twelve places of today's standing avoid list, as
  paths with no guard (`docs/hacks/README.md`, "Places that prompt are avoided until plan 006"). Used by
  `app/tauri/src/compose.rs` and `app/tauri/src/sandbox.rs`.
- `crates/macos/src/errors.rs` — `read_error(&io::Error) -> ReadError` and `from_errno(i32)`: the crate's one errno
  table (`EPERM` → `Guarded`, `EACCES` → `NoPermission`, `ENOENT`/`ENOTDIR` → `Vanished`, …).
- `crates/macos/src/volume.rs` — the private `kind_of(flags, internal) -> MountKind` behind `MacVolumes::space`, on
  `statfs` through `ffi::volume`; a mounted disk image is `External`.
- The crate depends on `fresher-core` since plan 004, not on nothing as the first version of this step said.

## Files

- `crates/macos/src/privacy/mod.rs` (new) — the module's public items, documented: `Guard`, `Probe`,
  `guarded_places`, `places_that_prompt`, `volume_guard`, `classify`, `full_disk_access`, `fda_candidates`,
  `try_place`, `try_inside`. Its own types; no core type in these signatures (the mapping onto the core is `fresher-system`'s, step
  03/03).
- `crates/macos/src/privacy/places.rs` (new; the content of `crates/macos/src/places.rs` moves here, the old file is
  removed) — `Guard` (`Desktop`, `Documents`, `Downloads`, `OtherAppsData`, `FullDiskAccess`, `RemovableVolume`,
  `NetworkVolume`) and `guarded_places(home: &Path) -> Vec<(PathBuf, Guard)>`, the design's "Probe results",
  "Places", as data, paths joined with `Path::join` (agent's call 26):

  | Place under the home                                                                                                                              | Guard            |
  | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
  | `Desktop`, `Documents`, `Downloads`                                                                                                               | its folder       |
  | `Library/Containers`, `Library/Group Containers`                                                                                                  | `OtherAppsData`  |
  | `Pictures`, `Movies`, `Music`, `Library/Mobile Documents`, `Library/CloudStorage`, `Library/Calendars`, `Library/Application Support/AddressBook` | `FullDiskAccess` |

  `places_that_prompt(home) -> Vec<PathBuf>` stays (the sandbox's standing avoid list, 03/03) and becomes the paths
  of `guarded_places(home)`: one table. Its three existing tests move with it.
- `crates/macos/src/privacy/volumes.rs` (new) — `volume_guard(root: &Path) -> Option<Guard>`: the root's mount kind
  from `ffi::volume::statfs` and `ffi::volume::resources`, with the same `kind_of` `MacVolumes::space` uses (`kind_of`
  becomes `pub(crate)`; no new FFI): `External` → `RemovableVolume`, `Network` → `NetworkVolume`,
  `Internal` → `None`. A root `statfs` refuses → `None` (its walk then fails to read it and counts it). The pure part,
  `guard_of(MountKind) -> Option<Guard>`, is tested on its own.
- `crates/macos/src/privacy/check.rs` (new) — `Probe` (`Allowed`, `Refused`, `Missing`, `Failed`) and
  `classify(&io::Error) -> Probe`, built on `errors::read_error` (no second errno table): `Guarded` → `Refused`,
  `Vanished` → `Missing`, anything else (`NoPermission` included: POSIX, not privacy) → `Failed`.
  `fda_candidates(home: &Path) -> Vec<PathBuf>`: `Library/Application Support/com.apple.TCC/TCC.db`, then
  `Library/Safari` ("Probe results", "Checks").
  `full_disk_access(candidates: &[PathBuf]) -> Probe`: each candidate in order is opened read-only with
  `custom_flags(libc::O_NOFOLLOW)` and closed at once, not a byte read; the first whose open is not `Missing` decides
  (`Allowed`, `Refused` or `Failed`); every one `Missing` → `Missing`. No `exists()` or `metadata` call first: inside a
  guarded folder they answer `EPERM` too.
  `try_place(path: &Path) -> Probe`: opens the folder the same way, lists nothing, closes it.
  `try_inside(path: &Path) -> Probe`, for the two `OtherAppsData` places: opening `Library/Containers` itself does not
  prompt ("Probe results"), so a try of the parent would answer `Allowed` and the prompt would come later, inside the
  walk. It lists the folder's names (allowed), opens the first sub-folder that is not a link the same way as
  `try_place`, and closes it: that open is the one that may prompt. An empty folder → `Allowed` (nothing guarded to
  see); a listing refused → `classify` of its error.
- `crates/macos/src/lib.rs` — `mod privacy;` with `pub use privacy::{…}` of the items above, as the crate exports its
  other modules; `mod places;` goes, and `places_that_prompt` is re-exported from `privacy`, so `app/tauri` compiles
  unchanged.

No `unsafe` and no new FFI: `std::fs::OpenOptions` with `custom_flags(libc::O_NOFOLLOW)`, the existing `statfs`
wrapper of `ffi::volume`, `libc`'s error constants. If the user's run of plan 01 finds a check that needs an FFI call,
it goes into `src/ffi/` with its `// SAFETY:` comment, behind a safe function here.

## Produces

The macOS knowledge 03/03 maps onto `AccessProbe`: `Guard`, `Probe`, `guarded_places`, `places_that_prompt`,
`volume_guard`, `fda_candidates`, `full_disk_access`, `try_place`, `try_inside`.

## Checks

Every test runs on a `tempfile` tree or a made-up home path; none reads the real home folder, none opens a real guarded
place.

- Negative (first): `classify_tells_privacy_from_posix_permissions` (`from_raw_os_error(EPERM)` → `Refused`,
  `EACCES` → `Failed`, `ENOENT` → `Missing`, no errno → `Failed`), `full_disk_access_is_missing_when_no_candidate_exists`,
  `a_symlinked_candidate_is_not_followed` (a link to a readable file is not `Allowed`),
  `an_unreadable_candidate_is_not_allowed` (a `chmod 000` file: `Failed`), `try_inside_skips_a_link`, `guarded_places_never_leave_the_home_folder`,
  `an_internal_volume_has_no_guard` (`guard_of`), and the moved `places_that_prompt_never_contains_the_home_itself`,
  `places_that_prompt_opens_nothing`.
- Positive: `full_disk_access_is_allowed_when_the_first_candidate_opens`,
  `the_first_candidate_that_is_not_missing_decides` (the first missing, the second readable → `Allowed`),
  `guarded_places_of_a_fake_home_match_the_table` (every row above with its guard),
  `places_that_prompt_are_the_guarded_places`, `try_place_on_a_readable_folder_is_allowed`, `try_inside_opens_the_first_sub_folder` (and an empty folder →
  `Allowed`),
  `external_and_network_volumes_have_their_guard` (`guard_of`), and the moved
  `the_places_are_absolute_under_the_given_home`.
- `cargo clippy -p fresher-macos --all-targets -- -D warnings`; `cargo nextest run -p fresher-macos`; coverage of
  `privacy/` ≥ 90 %; the crate's `no_unsafe_outside_the_ffi_module` test still passes; `cargo check -p fresher-app`
  compiles with no change in `app/tauri`.

## Conclusions

- Built as written: `crates/macos/src/privacy/` (`mod.rs`, `places.rs`, `volumes.rs`, `check.rs`); the old
  `crates/macos/src/places.rs` moved into `privacy/places.rs` and `places_that_prompt` is now the paths of
  `guarded_places`, one table. `lib.rs` re-exports every item; `app/tauri` compiles unchanged. No `unsafe`, no new FFI.
- The tables the user's run of plan 01 may change live in two constants only: `PLACES` (`privacy/places.rs`) and
  `FDA_CANDIDATES` (`privacy/check.rs`). Both encode the "Probe results" as they stand: from documentation, unverified.
- Decision: `is_local` in `volume.rs` became `pub(crate)` with `kind_of`, so `volume_guard` asks a volume's resources
  only when it is local, as `MacVolumes::space` does (a network volume is never asked).
- Decision: an open through a link with `O_NOFOLLOW` answers `ELOOP`, which the errno table calls `Replaced`, so a
  linked candidate is `Failed`, not `Allowed`. `try_inside` tells a link apart by the listing's own entry type, with
  no `stat`; the "first" sub-folder is the first one the listing gives, in no set order.
- Tests added beyond the list: `try_place_on_a_missing_folder_is_missing`, `try_inside_of_a_missing_folder_is_missing`,
  `the_first_candidate_that_is_not_missing_decides_even_when_it_fails`,
  `the_candidates_are_the_permission_database_then_the_browser_data`, `a_root_that_does_not_exist_has_no_guard`,
  `a_temporary_folder_on_the_internal_disk_has_no_guard`, and `a_mounted_disk_image_is_a_removable_volume` (a real
  `volume_guard` on a mounted disk image, skipped where `hdiutil` is missing).
- Not tested: the network branch of `volume_guard`, which needs a network mount; `guard_of(Network)` is tested alone.
- Measured: coverage of `privacy/` with `cargo llvm-cov`: `check.rs` 99.3 % lines, `places.rs` 100 %, `volumes.rs`
  90.9 % (the lines left are the network branch). `cargo clippy --workspace --all-targets -- -D warnings` clean;
  `cargo check -p fresher-app` builds.
