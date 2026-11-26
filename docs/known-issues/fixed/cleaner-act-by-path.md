# The cleaner acts on a path after checking a handle

**Status:** fixed ([plan 008/04, Conclusions](../../plans/008-safe-removal/04-cleaner/readme.md#conclusions)) ·
**Updated:** 2026-11-16

## Fix

A permanent delete, the emptying of a folder and the growth walk act beneath held folders
([`architecture/removal.md`](../../architecture/removal.md), "Kinds of removal"). `fresher-macos`'s `beneath` module
gives the cleaner safe calls that name one entry of a folder held open, never a path: `open_folder`
(`O_DIRECTORY | O_NOFOLLOW_ANY`), `open_folder_at` (`openat` with `O_NOFOLLOW_ANY`), `identity_at` (`fstatat` with
`AT_SYMLINK_NOFOLLOW`), `read_folder` (the bulk read) and `remove_at` (`unlinkat`, `AT_REMOVEDIR` for a folder). The
deletion point (`crates/cleaner/src/remove/permanent.rs`) opens the item's parent through no link, checks the entry
against the verified identity, and removes a folder depth first: each folder opened beneath the one above and checked
against its listing and the root's volume, each entry checked again right before `unlinkat`. The first refusal fails
the item; what was removed before it stays removed and the rest stays in place. An emptied folder's place must still
hold it; its entries are listed from its handle and deleted beneath it. `remove_at` is in Clippy's
`disallowed-methods`, so the deletion point is still the only caller. Proved by the race tests of
`remove/permanent.rs` (a parent and a folder inside swapped for a link during the delete, 100 times each; both failed
against the path-based delete), its locked-entry and mounted-volume tests, and the swap case of `tests/never.rs`.

Still left: an entry replaced inside a held folder between its check and `unlinkat` (no call removes by identity) —
the replacement lies in the same verified folder and on the same volume, never behind a link. The Trash call still
takes a path; a swap there is detected after the move (plan 008, call 37).

## Symptom

Every item is checked through a handle opened through no link (plan 008/04: identity, removability, use, growth), but
the act itself takes a path: the system's Trash call, `remove_file` and `remove_dir_all`. If a parent folder is swapped
for a link in the microseconds between the check and the act, the act lands behind the link.

- After a move to the Trash this is detected: the landed item's identity is checked through the held handle (plan
  008, call 37) and a mismatch is reported.
- After a permanent delete it is not detected.
- A permanent delete of a folder that holds a locked entry deletes part of the folder before it fails; the item is
  reported as failed.
- Emptying a folder re-checks each entry only for link, volume and identity; removability and use rely on the checks
  of the whole folder.
- The size walk of the growth check works by path, not below the held handle.

## Cause

The standard library's removal functions and the Trash API take paths; acting relative to the held folder handle
(`unlinkat`, `renameatx_np`, a walk by descriptors) needs FFI, which lives only in `fresher-macos`.

## What was tried

Nothing yet: found and written down by plan 008/04. The window is microseconds and needs a process of the same user
racing the cleaner; the Trash path, the default, detects it.

## Options left

- A permanent delete and the emptying of a folder through descriptors in `fresher-macos`: open each folder with
  `O_NOFOLLOW_ANY`, remove entries with `unlinkat` relative to it, never by path. Planned for plan 008/08 or a
  follow-up before the first release.
- Keep permanent delete off by default (it already is a separate, explicit choice per confirmation).
