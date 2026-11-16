# 008/03/03 — The protection list

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos

## Files

- `crates/macos/src/protection.rs` (new) — `protection(home, data_folder, app_bundle) -> Result<Protection,
  ProtectionError>`: the core's `Protection` (02/03) filled with the macOS places of the design, section 5:
  - **never**: `/System`, `/usr` (except `/usr/local`), `/bin`, `/sbin`, `/private/var/db`, `/private/etc`,
    `/cores`, `/Applications`, `/Library/Apple`, every other home under `/Users` (each folder of `/Users` but the
    user's own and `Shared`, read when the list is built), the app's bundle and data folder, `~/.Trash`, `/.Trashes`,
    `~/Library/Mobile Documents`, `~/Library/CloudStorage`, and `.Trashes` at the root of every mounted volume;
  - **whole**: the home; its Desktop, Documents, Downloads, Movies, Music, Pictures, Public, Applications, Library;
    the Library's standard folders (Caches, Application Support, Preferences, Containers, Group Containers, Logs,
    Saved Application State, Cookies, HTTPStorages, WebKit, LaunchAgents, Mail, Messages, Keychains); `/Users`,
    `/Users/Shared`, `/Volumes` and every mounted volume's root;
  - every path given in its canonical form (`/private/var`, not `/var`).
  The list is data in one place, with a comment per group saying why.
- `crates/macos/src/lib.rs` — the export.

## Produces

The protection the app gives `DiskCleaner` (06/04). The OS-neutral checks are the core's; this file only names places.

## Checks

On a temporary folder standing in for the home (the function takes the home; it never looks it up):

- Negative (first): `the_home_and_its_standard_folders_are_whole`, `icloud_and_cloud_storage_are_never`,
  `the_trash_is_never`, `the_apps_own_data_folder_is_never`, `another_users_home_is_never`,
  `usr_is_never_but_usr_local`.
- Positive: `an_item_inside_the_caches_folder_may_go`, `an_item_inside_usr_local_may_go`,
  `every_path_of_the_list_is_canonical` (no path in the list starts with a link of the system: `/var`, `/tmp`, `/etc`).
- A review checklist in the step's conclusions: every group of the list against
  [`restrictions/deletion-safety.md`](../../../restrictions/deletion-safety.md).

## Conclusions

- Built: `protection(home, data_folder, app_bundle: Option<&Path>)` (no bundle in a development build) over a pure
  `Places::of(home, data_folder, app_bundle, volumes)` that holds the list, one constant per group with its reason.
  The given paths are made canonical (links resolved; a data folder not made yet is resolved through its longest
  existing ancestor); a path that cannot be made canonical and plain is refused by the core (`NotPlain`), so the app
  gets no protection and the guard fails closed.
- **Changed: other users' homes by a carve-out, not by a listing.** `/Users` is `never`, with the user's own home and
  `/Users/Shared` carved out of it (the core's `except`: the place itself never as a whole, items inside one by one),
  instead of reading `/Users` and naming each other home. Same answer for every home there is, and safer: a home made
  after the list was built, a file at the top of `/Users` and a name that differs only in case are all refused, and
  there is no read that can fail. `/Users` itself is `never` (stronger than the design's `whole`).
- The mounted volumes are read when the list is built (`getfsstat`, `MNT_NOWAIT`): each root `whole`, its `.Trashes`
  `never`. If they cannot be read the list goes without them (logged): `/.Trashes` stays, and a volume's own Trash
  is then kept by the guard's same-volume check.
- What the core answers first: a `whole` folder that holds a `never` place (the home, `Library`, `Application
  Support`, a volume's root) is refused as `Ancestor` before `WholeFolder`; the tests assert that.
- Tests: on a temporary folder standing in for the home, given through the system's link (`/var/folders/…`), so the
  canonical form is tested too; the other homes and the volumes on fixed paths (`/Users/ann`, `/Volumes/Backup`),
  which the function never opens. Three more than listed: a volume's root and Trash, the user's own and the shared
  items may go, a relative home is refused.
- While running the whole crate, `hdiutil` now and then failed to create or attach an image with 13 tests making
  images at once: `DiskImage` tries a command up to four times, apart, and says `hdiutil`'s own words when it fails.

**Review against [`restrictions/deletion-safety.md`](../../../restrictions/deletion-safety.md)**:

| Group of the list                                                                 | Rule it holds                                                    | Answer |
| --------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------ |
| `/System`, `/usr` (but `/usr/local`), `/bin`, `/sbin`                             | "never touch protected paths"; SIP places also by check 8        | never  |
| `/private/var/db`, `/private/etc`, `/cores`, `/Library/Apple`                     | the same: the system's own data                                  | never  |
| other users' homes, anything at the top of `/Users`                               | "another user's home is never offered"                           | never  |
| `/Applications`, the app's bundle and data folder                                 | apps only by an uninstaller (not planned); the app keeps its own | never  |
| `~/.Trash`, `/.Trashes`, each volume's `.Trashes`                                 | "Trash first": what the user put there is theirs                 | never  |
| `~/Library/Mobile Documents`, `~/Library/CloudStorage`                            | a removal there also removes the copy in the cloud               | never  |
| the home, Desktop, Documents, Downloads, Movies, Music, Pictures, Public, Library | "the user's own folders are hard-blocked"                        | whole  |
| `~/Applications` and the Library's standard folders                               | the same rule, one level down                                    | whole  |
| `/Users/Shared`, `/usr/local`, `/Volumes`, every volume's root                    | never as a whole; items inside one by one                        | whole  |

Symlinks are not this list's: the guard opens with `O_NOFOLLOW_ANY` (check 5). The list leaves nothing of the
restriction out.
