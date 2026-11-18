# 017/01/01 — The temporary places and their protection

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
core, macos, system, testing, app

Design sections 1 and 2.

## Files

- `crates/core/src/known_folder.rs` — `KnownFolder::UserTemp` (`user-temp`, "the user's own temporary folder") and
  `KnownFolder::SharedTemp` (`shared-temp`, "the temporary folder every user shares"); `ALL` grows to 16. Every
  exhaustive `match` over `KnownFolder` follows (rules' `under = { folder = … }` reads the two names).
- `crates/macos/src/ffi/confstr.rs` (new), `ffi/mod.rs` — `user_temp_dir() -> io::Result<PathBuf>`: one
  `libc::confstr(_CS_DARWIN_USER_TEMP_DIR, …)` call into a sized buffer (one `unsafe` block, its `// SAFETY:` comment:
  the buffer's length is the one passed); a second call when the first reports a longer value; bytes to a path, no
  UTF-8 assumed. `crates/macos/src/lib.rs` exports it.
- `crates/system/src/known_folders.rs` — `MacKnownFolders::with_user_temp(path)`: keeps the given canonical path;
  `locate(UserTemp)` answers it (none when not given), `locate(SharedTemp)` answers `/private/tmp`. Still pure.
- `app/tauri/src/compose.rs` — reads `user_temp_dir()` once, makes it canonical (`std::fs::canonicalize`: `/var` →
  `/private/var`), and refuses an answer that is not under `/private/var/folders/` (logged, no `UserTemp`).
- `crates/macos/src/protection.rs` — `protection(home, data_folder, app_bundle, user_temp: Option<&Path>)`:
  `never` gains `/private/var/folders`, `/private/var/tmp`, `/private/var/vm`, `/.DocumentRevisions-V100`,
  `~/Library/Autosave Information` and `<user_temp>/TemporaryItems`; `except` gains `<user_temp>`; `whole` gains
  `<user_temp>` and `/private/tmp`. Both forms of each path, as today (`canonical`).
- `crates/core/src/settings/folders.rs` — nothing new to write if the check reads the protection list; its tests gain
  the new places (`/private/tmp`, the user's `T`, `/private/var/folders` refused as scan roots).
- `crates/testing/src/known_folders.rs` — the fixture layout puts `UserTemp` at `<root>/private/var/folders/xx/fixture/T`
  and `SharedTemp` at `<root>/private/tmp`, beside the home.

## Produces

`KnownFolder::{UserTemp, SharedTemp}` resolved on macOS and in fixtures, for 01/04 and the module; the protection for
the cleaner.

## Checks

- Negative first (`protection.rs`): an item in `C/`, `0/`, `X/`, `T/TemporaryItems`, another user's
  `/private/var/folders/yy/other/T`, `/private/var/tmp` and `~/Library/Autosave Information` is refused, in both
  forms (`/var/…`, `/tmp/…`); `T` itself and `/private/tmp` are whole (refused as a whole). Positive: an item directly
  in the user's `T` and in `/private/tmp` passes the list.
- `known_folders.rs`: without a user temp, `UserTemp` is `None`; `SharedTemp` is always `/private/tmp`; with one, the
  given path. `ffi`: `user_temp_dir()` answers an absolute path ending in `/T/` on this Mac (a test that reads, never
  writes).
- `crates/cleaner/tests/never.rs`: the random trees gain the protected places; none is ever removed.
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-system -p fresher-cleaner`; clippy; the crate guard
  (`unsafe` only in `ffi`).

## Conclusions

Built (16f155c, review fixes 0636bf6): `KnownFolder::{UserTemp, SharedTemp}` (`ALL` is 16); `user_temp_dir()` in
`fresher-macos`'s `ffi` (one `confstr` call into a 1,024-byte buffer, a second one when the answer is longer, a 0 return
its own error, bytes to a path); `MacKnownFolders::with_user_temp`; the composition reads it once, makes it canonical
and keeps it only when `is_user_temp` says it is exactly `/private/var/folders/<xx>/<hash>/T` (one shared check, used by
the protection list too: a user temp of any other shape carves nothing, so all of `/private/var/folders` stays
protected); the protection list's entries as the brief lists them.

Changed from the brief, each a ruling of the review:

- The scan roots' settings check is a fixed list, not the protection list: `/private/var/folders`, `/private/var/tmp`,
  `/private/var/vm` and `/.DocumentRevisions-V100` are refused as wholes, but a folder strictly inside a user's `T` (not
  in its `TemporaryItems`) stays allowed, and `/private/tmp` is refused as itself (the folder and above, not what is
  inside). Every test whose scan root is a temporary folder lives under the real `T`. Cost: a user may choose such a
  folder as a scan root (01/04 then leaves the place holding it unwalked).
- `T` itself answers `Ancestor`, not `WholeFolder`: it holds the never place `TemporaryItems`. Both refuse.
- The fixture layout puts the two places inside the fixture root (`<root>/private/var/folders/xx/fixture/T`,
  `<root>/private/tmp`), so a home whose parent is `/` can never point at the real `/private/tmp`.
- The sandbox's own list keeps carving the real `T` (sandboxes are generated in `$TMPDIR`); the sandbox's places join as
  a protection of their own (built in 01/05).

`crates/cleaner/tests/never.rs`: the random trees gain every place (every user's folders, the user's `T`, its
`TemporaryItems`, `C`, another user's `T`, `/private/var/tmp`, `/private/tmp`); none is removed, and emptying
`/private/tmp` is skipped as `WholeFolder`. 1,991 tests passed at the time; the docs were left to 04/01.
