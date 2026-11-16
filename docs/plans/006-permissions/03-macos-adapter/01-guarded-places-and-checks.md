# 006/03/01 — Guarded places and checks

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** macos

## Files

- `crates/macos/src/privacy/mod.rs` (new) — the module's public items, documented; no core type (the crate depends on
  nothing).
- `crates/macos/src/privacy/places.rs` — `Guard` (`Desktop`, `Documents`, `Downloads`, `OtherAppsData`,
  `RemovableVolume`, `NetworkVolume`) and `guarded_places(home: &Path) -> Vec<(PathBuf, Guard)>`: the measured table of
  01/01 as data, paths joined with `Path::join`, never strings. Volumes come from 004/02's volume facts for a given
  root: `volume_guard(root: &Path) -> Option<Guard>`.
- `crates/macos/src/privacy/check.rs` — `Probe` (`Allowed`, `Refused`, `Missing`, `Failed`) and
  `classify(&io::Error) -> Probe` (`EPERM` → `Refused`, `ENOENT` → `Missing`, else `Failed`; `EACCES` is POSIX, not
  privacy, so `Failed` here); `full_disk_access(candidates: &[PathBuf]) -> Probe` (the first candidate that exists
  decides; opened read-only with `O_NOFOLLOW`, closed at once, not a byte read); `fda_candidates(home: &Path) ->
  Vec<PathBuf>` (the measured list); `try_place(path: &Path) -> Probe` (opens the folder, lists nothing, closes it).
- `crates/macos/src/lib.rs` — `pub mod privacy;`.

No `unsafe`: `std::fs::File::open` / `OpenOptions` with `custom_flags(libc::O_NOFOLLOW)`, and `libc`'s error constants.
If 01/01 found a check that needs an FFI call, it goes into the crate's FFI module with a `// SAFETY:` comment, behind a
safe function here, and this step lists it.

## Produces

The macOS knowledge 03/03 maps onto `AccessProbe`; `classify` is what plan 004's walk may reuse to tell `Guarded` from
`NoPermission` (offered, not required).

## Checks

Every test runs on a `tempfile` tree; none reads the real home folder.

- Negative: `classify_tells_privacy_from_posix_permissions` (`from_raw_os_error(EPERM)` → `Refused`, `EACCES` →
  `Failed`), `full_disk_access_is_missing_when_no_candidate_exists`, `a_symlinked_candidate_is_not_followed`,
  `an_unreadable_candidate_is_not_allowed` (a `chmod 000` file), `guarded_places_never_leave_the_home_folder`.
- Positive: `full_disk_access_is_allowed_when_the_first_candidate_opens`, `the_first_existing_candidate_decides`,
  `guarded_places_of_a_fake_home_match_the_measured_table`, `try_place_on_a_readable_folder_is_allowed`.
- `cargo clippy -p fresher-macos --all-targets -- -D warnings`; coverage of the module ≥ 90 %.
