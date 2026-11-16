# 008/01/01 — The Trash and put back

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos (probe)

## Files

- `NO_COMMIT/probes/008/trash/` — a Cargo project outside the workspace (`objc2`, `objc2-foundation`, `libc`):
  - `trash` — makes a file, a folder of 10 000 files, a symbolic link to a file outside, a folder holding a link, an
    app-like package and an empty folder in a temporary folder, moves each to the Trash with
    `trashItemAtURL:resultingItemURL:error:`, and prints, per item: the result, the landed path, the device and file id
    before and after, the time taken;
  - `trash-image` — the same on disk images it mounts itself: APFS, HFS+ (Mac OS Extended) and exFAT, each with
    `-nobrowse`; and on a network share when one is at hand (else "not measured");
  - `put-back` — renames each landed item back with `renamex_np(…, RENAME_EXCL)`: into a free place, into a place
    taken by a new item of the same name, into a missing parent;
  - `space` — free space (`statfs` `f_bavail`) of the volume before and after moving 1 GiB of new data to the Trash,
    and before and after deleting 1 GiB with `std::fs::remove_dir_all`, read at 0 ms, 100 ms, 1 s and 5 s.

## Questions to answer

1. Does `trashItemAtURL` move a link itself or its target? A folder that holds a link — is anything followed?
2. Is the landed item the same item (device and file id) on every volume kind? What is it named when the Trash already
   holds an item of that name?
3. Which volume kinds have no Trash, and what error comes back (code and domain)? Is anything deleted in that case?
4. Does Finder's own **Put Back** work on an item moved this way?
5. Without Full Disk Access: can the app `lstat` an item it moved into `~/.Trash`, list `~/.Trash`, and rename an item
   out of it? Does any of it raise a prompt? (The Trash folder is guarded by macOS's privacy protection on recent
   versions.) The same with Full Disk Access.
6. Does the call play a sound, show a dialog or need any permission? Does it block on a busy volume?
7. The time per item and for the folder of 10 000 files (moved as one).
8. With a parent folder of the item replaced by a link between the check and the call (a second thread), what is
   moved? (This is the residual risk named in the threat model; the probe says how wide the window is.)
9. `renamex_np` with `RENAME_EXCL`: refuses an existing target atomically; the error for a missing parent; across
   volumes.
10. How soon `f_bavail` shows freed space after a permanent delete on APFS (asynchronous freeing), and that a move to
    the Trash frees nothing.

## Produces

The probe's `RESULTS.md` (in `NO_COMMIT/`), read by step 03: whether `MacTrash` and put back work as section 6 and 8
of the design assume, the error to map to `NoTrash`, whether put back needs Full Disk Access (and so whether the
Cleanups screen must say so), the delay before measuring space.

## Checks

- Every question has an answer or "could not tell, because …".
- No item outside the probe's own folders and images was moved; every trashed item was put back or deleted by the
  probe at its end; `~/.Trash` holds nothing of the probe afterwards.

## Conclusions

Done on 2026-11-16 (Mac15,7, macOS 27.2.1, a terminal without Full Disk Access); the numbers are in the design,
"Measured (01)", items 1–10 of the Trash call. Every item the probe moved was put back by it; nothing of the probe is
left in the Trash.

- The call moves a link as the link and follows nothing inside the item; the landed item keeps its device and file id
  on APFS and HFS+; a name already in the Trash gets a time suffix.
- `NoTrash` is `NSCocoaErrorDomain` 3328 (a read-only volume); 513 (no permission, a locked item, no `.Trashes`
  possible) and 4 (missing) leave the item in place. Nothing is ever deleted instead.
- Put back with `renamex_np` works without Full Disk Access (only listing `~/.Trash` is refused); `RENAME_EXCL` gives
  `EEXIST`, `ENOENT` and `EXDEV` as the design expects. Finder's Put Back records are written by the call.
- A parent swapped for a link makes the call move the item behind it, and some such calls return no landed path: the
  post-check takes the landed path from the kept handle (`F_GETPATH`, exact in 2 000 of 2 000 racing calls) — design,
  call 37, which changes plan 02's `Trash` port.
- Freed space shows when the delete returns: no delay before measuring.
- **Needs the user**: Finder's Put Back on screen, the sound, a run with Full Disk Access, exFAT and FAT32 (their
  images do not mount on this macOS), a network share.
