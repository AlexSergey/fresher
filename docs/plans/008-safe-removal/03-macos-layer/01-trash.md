# 008/03/01 — The Trash and put back

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos

## Files

- `crates/macos/Cargo.toml` — `objc2` and `objc2-foundation` (workspace pins of 02/01; only the `NSFileManager`,
  `NSURL`, `NSError` and `NSString` features).
- `crates/macos/src/ffi/trash.rs` (new) — one safe function over `NSFileManager.defaultManager`'s
  `trashItemAtURL:resultingItemURL:error:`: a path in, the landed path (when the call gives one: under a race it
  may not, as 01/01 measured) or the `NSError`'s domain and code out; the URL is made with
  `fileURLWithPath:isDirectory:` from the path's bytes, inside an autorelease pool. Beside it, `path_of(fd)` over
  `fcntl(F_GETPATH)`: where the item behind a held descriptor is now (design, call 37).
- `crates/macos/src/ffi/rename.rs` (new) — `rename_exclusive(from, to)` over
  `renamex_np(…, RENAME_EXCL | RENAME_NOFOLLOW_ANY)`, its `errno` kept; `RENAME_NOFOLLOW_ANY` (0x10) is defined
  there with a reference to the SDK's `sys/stdio.h`, since `libc` 0.2.191 lacks it.
- `crates/macos/src/trash.rs` (new) — `MacTrash`, which implements the core's `Trash`:
  - `move_to_trash(path, held)`: `Landed { at, reported }` — `at` from `path_of` on the verified item's held
    handle after the call, `reported` the call's own landed path, if any (call 37) — or the `TrashError` the error
    maps to (the codes measured in 01/01, `NSCocoaErrorDomain`: 3328 "the volume has no Trash" → `NoTrash`, 4 →
    `NotFound`, anything else, such as 513 for no permission or a locked item → `Refused` with a short reason and no
    path);
  - `put_back`: `rename_exclusive(landed, original)`: `EEXIST` → `Occupied`, `ENOENT` of the parent → `NoParent`,
    `ELOOP` (a link in either path) and anything else → `Refused`.
- `crates/macos/src/lib.rs` — `pub use trash::MacTrash;`.

## Produces

The real Trash of the app (wired in 06/04, given to `DiskCleaner` only).

## Checks

All on disk images the tests mount with `-nobrowse` (the existing `disk_image.rs` helper), skipped without `hdiutil`;
the items land in the image's own `.Trashes` and vanish with it.

- Negative (first): `a_missing_item_is_not_found`, `a_volume_without_a_trash_moves_nothing` (an image attached
  `-readonly`, which 01/01 found answers 3328; exFAT images do not mount on the developer's macOS: the item is still in
  place), `put_back_never_replaces_an_existing_item`, `put_back_into_a_missing_folder_is_refused`,
  `put_back_through_a_link_is_refused`, `a_link_is_moved_as_the_link` (its target untouched).
- Positive: `an_item_lands_in_the_trash_of_its_volume_with_its_identity`, `the_held_handle_names_the_landed_path`,
  `put_back_restores_the_item_and_its_identity`, `a_folder_moves_as_one`.
- `cargo nextest run -p fresher-macos`; Clippy; the crate's "no unsafe outside `ffi`" test passes; `cargo deny check`.
- `docs/dependencies.md` gains `objc2` and `objc2-foundation` (version, licence, used by `macos`).

## Conclusions

- Built as written: `ffi/trash.rs` (`trash_item(path, is_directory)` inside an autorelease pool, `path_of(fd)` over
  `F_GETPATH`), `ffi/rename.rs` (`rename_exclusive` with `RENAME_EXCL | RENAME_NOFOLLOW_ANY`) and `MacTrash`. The
  Trash call of `objc2-foundation` 0.3.2 is a safe method, so `ffi/trash.rs` holds one unsafe block (`F_GETPATH`).
- **The error mapping** follows the frozen port (02/04), which is finer than this step's text: Cocoa 4 → `NotFound`,
  3328 → `NoTrash`, **513 → `NotAllowed`** (the port's own variant for no permission, a locked item or a volume that
  refuses a `.Trashes` folder), any other code or domain → `Refused { reason: "<domain> <code>" }`, never a path or
  the localized text (which names the item). Put back: `EEXIST` → `Occupied`; `ENOENT` → `NotFound` when the landed
  item is gone, else `NoParent`; **`ELOOP` → `ThroughLink`** (the port's variant); anything else → `Refused { reason:
  "errno N" }`.
- **`at` when the system cannot say**: if `F_GETPATH` fails after a successful call, `at` is an empty path, which lies
  in no Trash folder, so the cleaner trusts only a `reported` path holding the verified identity (else `Failed`); the
  move is never reported as a refusal, since the item did move.
- Choices the step left open: the URL is made with `fileURLWithFileSystemRepresentation:isDirectory:relativeToURL:`
  (`NSURL::from_path`, the path's bytes as they are, lossless for any name) rather than from an `NSString`;
  `isDirectory` comes from the held handle (`fstat`); a relative path is refused before any call.
- **Put back of a link** works: with `RENAME_NOFOLLOW_ANY` a link that is the item itself is renamed as the link; only a
  link in a parent component fails (`a_link_is_moved_as_the_link` puts it back).
- Tests: all on APFS images the tests mount with `-nobrowse` (`DiskImage` gains `mount_read_only`, which fills the
  image, detaches it and attaches it `-readonly`); every item lands in the image's own `.Trashes/<uid>` and is put back
  or vanishes with the image. Nothing reaches the user's Trash: the one test on a temporary folder (a relative path)
  is refused before the call. Two more negative tests than listed: the mapping of unknown codes and errors, and put
  back of a landed item that is gone.
- Dependencies: `objc2` 0.6.5 (MIT) and `objc2-foundation` 0.3.2 (features `NSError`, `NSFileManager`, `NSString`,
  `NSURL`) in `docs/dependencies.md`. The licence of `objc2-foundation` 0.3.2 is **MIT** (its `Cargo.toml`), not the
  design's "Zlib OR Apache-2.0 OR MIT" of older versions; it is on the allowlist either way. `cargo deny check` passes.
