# 006/03/02 — Settings panes

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** macos

The System Settings address of each pane and the fixed argument lists of `/usr/bin/open`, in `fresher-macos`. One
address per pane for every supported version (macOS 13 on: agent's call 25), from the design's "Probe results",
unverified until the user's run of plan 01.

## Files

- `crates/macos/src/privacy/panes.rs` (new) — `Pane` (`FullDiskAccess`, `FilesAndFolders`, `PrivacyAndSecurity`)
  and `pane_address(pane: Pane) -> &'static str`:

  | Pane                 | Address                                                                    |
  | -------------------- | -------------------------------------------------------------------------- |
  | `FullDiskAccess`     | `x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`        |
  | `FilesAndFolders`    | `x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders` |
  | `PrivacyAndSecurity` | `x-apple.systempreferences:com.apple.preference.security?Privacy`                 |

  `pane_of(guard: Guard) -> Pane` (03/01's `Guard`): `FullDiskAccess` → `FullDiskAccess`; the three folders and both
  volumes → `FilesAndFolders`; `OtherAppsData` → `PrivacyAndSecurity` (no pane lists it).
  `OPEN: &str = "/usr/bin/open"`, `open_args(pane: Pane) -> [&'static str; 1]` (the address alone), and
  `reveal_args(path: &Path) -> [&OsStr; 2]` (`-R` and the path as an `OsStr`, never formatted into a string).
  No macOS version is read and no FFI call is added; the first version of this step's `MacosVersion` and
  `kern.osproductversion` are dropped. If the user's run finds a version where an address fails, a version split comes
  back, with the version from `fresher-system` (03/03 reads it for data from other apps).
- `crates/macos/src/privacy/mod.rs` — the exports of `Pane`, `pane_address`, `pane_of`, `OPEN`, `open_args`,
  `reveal_args`.

## Interface note for 03/03

`CommandRunner::run(program: &str, args: &[&str], cancel)` takes `&str` arguments, so `MacSystemUi` (03/03) turns the
`OsStr` path of `reveal_args` into `&str` with `to_str()`, and a path that is not valid UTF-8 is refused with
`SystemUiError::Failed` (never lossily converted: a lossy path would reveal another item). `reveal_args` keeps the
`OsStr` so this crate never formats a path.

## Produces

The addresses and the fixed argument lists 03/03's `MacSystemUi` runs through the process runner.

## Checks

- Negative (first): `reveal_keeps_a_path_with_spaces_and_quotes_as_one_argument`,
  `no_address_points_outside_system_settings` (every address starts with `x-apple.systempreferences:`),
  `data_from_other_apps_falls_back_to_privacy_and_security`.
- Positive: `each_pane_has_an_address`, `open_takes_exactly_the_address`, `each_guard_has_its_pane`.
- `cargo clippy -p fresher-macos --all-targets -- -D warnings`; `cargo nextest run -p fresher-macos privacy`.

## Conclusions

- Built as written: `crates/macos/src/privacy/panes.rs` (`Pane`, `pane_address`, `pane_of`, `OPEN`, `open_args`,
  `reveal_args`), exported from `privacy/mod.rs` and `lib.rs`. No `unsafe`, no FFI, no version read.
- The addresses live in one constant, `ADDRESSES`, the only place the user's run of plan 01 may change.
- Nothing opens System Settings or Finder: the module only builds argument lists; the tests check those.
- Tests: the six listed checks, negative first.
