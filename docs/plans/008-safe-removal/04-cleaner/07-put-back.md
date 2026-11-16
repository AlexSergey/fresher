# 008/04/07 — Locate and put back

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

## Files

- `crates/cleaner/src/put_back.rs` (new):
  - `locate(items: &[TrashedItem]) -> Vec<TrashState>`: `symlink_metadata` of each landed path; `InTrash` when its
    device and file id are the journalled ones, `Replaced` when something else is there, `Gone` when nothing is;
  - `put_back(item, cx) -> PutBackOutcome`: `locate` says `InTrash`; the original path passes checks 2 to 4 today
    (inside a current root, not protected, not ignored) and its parent opens through no link (check 5 on the parent);
    then the `Trash` port's `put_back`, which never replaces anything (`Occupied`, `NoParent` are outcomes with their
    texts); afterwards the original path must hold the item's identity.
- `crates/cleaner/src/lib.rs` — `Cleaner::{locate, put_back}`.

## Produces

What the history (06/03) shows for each item, and put back.

## Checks

- Negative (first): `an_item_no_longer_in_the_trash_is_gone`, `an_item_replaced_in_the_trash_is_replaced`,
  `put_back_never_replaces_an_item_at_the_original_place` (the original made again: the outcome is `Occupied` and
  both items are as they were), `put_back_into_a_removed_folder_is_refused`,
  `put_back_outside_the_current_roots_is_refused`, `put_back_through_a_link_is_refused` (the original parent replaced
  by a link).
- Positive: `an_item_in_the_trash_is_located`, `put_back_restores_the_item_with_its_identity`,
  `put_back_of_an_emptied_folders_entry_restores_it_into_the_kept_folder`.

## Conclusions

- Built in `crates/cleaner/src/put_back.rs` and wired into `DiskCleaner`, with the step's checks and four more
  negatives (a file where the folder was, an ignored original place, the sandbox marker gone, both answers of
  `locate` checked again by `put_back`); the items are moved to the fake Trash by a real `execute` first.
- Taken the safest simple way where the step is silent:
  - **`locate`** compares the device, the file id and the kind; a landed path that cannot be read is `Gone` (never
    `InTrash`).
  - **Put back runs check 1 too**: in a development build an item goes back only below a sandbox marker, like
    everything else the cleaner does. The order: in the Trash, inside a current root (roots read with `~` as the
    home), in reach, protection and ignored (both lists, both forms), then the original folder opened through no link
    (a link → `Refused { ThroughLink }`, missing or not a folder → `NoParent`, another volume → `Refused
    { OtherVolume }`), then the `Trash` port, then the original path must hold the item's identity (else `Failed`).
  - **The port's answers, as plan 03's `MacTrash` gives them** (`EEXIST` → `Occupied`, `ENOENT` → `NotFound` when the
    landed item is gone, else `NoParent`, `ELOOP` → `ThroughLink`): `Occupied`, `NoParent`, `NotFound` → `Gone`,
    `ThroughLink` → `Refused { ThroughLink }`, anything else → `Failed`. A refusal leaves the item in the Trash.
