# The system's Trash call takes a path

**Status:** open · **Updated:** 2026-11-25

## Symptom

If a parent folder of an item is swapped for a link in the instant between the cleaner's last check and the system's
Trash call, the call moves whatever lies behind the link instead of the item. A permanent delete has a smaller window
of its own: an entry replaced inside a held folder between its check and `unlinkat` is removed.

## Cause

The system's Trash call (`trashItemAtURL:resultingItemURL:error:`, `fresher-macos`'s `MacTrash`) names the item by a
path; no call moves an item to the Trash by its handle. `unlinkat` removes an entry by name beneath a held folder; no
call removes it by its identity.

## What was tried

- Plan 008/01 measured the Trash window: a parent swapped for a link makes the call move the item behind it, and some
  such calls return no landed path. The post-check asks the kept handle where the verified item is now (`F_GETPATH`,
  exact in 2 000 of 2 000 racing calls; plan 008, call 37): an item moved in its place is reported as `Failed`
  ("Another item was moved to the Trash in its place") (`crates/cleaner/src/remove/trash.rs`). The item it moved
  instead is not journalled (security audit 2026-11-25): its original place is unknown, and journalling it under the
  verified item's path made put back bring it there. Finder's own Put Back restores it to where it came from.
- The permanent delete acts beneath held folders since plan 008/04's hardening
  ([`fixed/cleaner-act-by-path.md`](./fixed/cleaner-act-by-path.md)): each entry is checked against its listing right before
  `unlinkat`, so a replacement can only be an entry of the same verified folder, on the same volume, never behind a
  link.

## Options left

- None on the Trash call while macOS has no handle-based form; a move by `renameatx_np` into the Trash folder would
  skip the Trash's own records (Finder's Put Back, the name on a clash).
- The window needs a process of the same user racing the cleaner within microseconds; the Trash makes the result
  recoverable and the report names it.
