# 006/03/02 — Settings panes

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** macos

## Files

- `crates/macos/src/privacy/panes.rs` — `Pane` (`FullDiskAccess`, `FilesAndFolders`, `PrivacyAndSecurity`) and
  `pane_address(pane, version: MacosVersion) -> &'static str`: the `x-apple.systempreferences:` addresses measured in
  01/02, per macOS version, with `PrivacyAndSecurity` as the fallback for a version the table does not know;
  `OPEN: &str = "/usr/bin/open"` and `open_args(address) -> [&str; 1]`, `reveal_args(path) -> [&OsStr; 2]` (`-R` and
  the path, as an `OsStr`, never formatted into a string). `MacosVersion` comes from 004/02's system facts; if it has
  none, this step adds a small `macos_version()` reading `kern.osproductversion` (that sysctl is the one FFI call, in the
  crate's FFI module with its `// SAFETY:` comment).
- `crates/macos/src/privacy/mod.rs` — the exports.

## Produces

The addresses and the fixed argument lists 03/03's `MacSystemUi` runs through the process runner.

## Checks

- Negative: `an_unknown_version_falls_back_to_privacy_and_security`, `reveal_keeps_a_path_with_spaces_and_quotes_as_one_argument`,
  `no_address_points_outside_system_settings` (every address starts with `x-apple.systempreferences:`).
- Positive: `each_pane_has_an_address_on_every_measured_version`, `open_takes_exactly_the_address`.
