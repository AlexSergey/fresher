# 008/04/02 — Paths: roots, protection, links

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

Checks 2 to 6 of the guard: where the item is, lexically and physically.

## Files

- `crates/cleaner/src/paths.rs` (new):
  - `Roots::new(plan_roots, current_roots)` — the plan's roots that are still scan roots, each made canonical once
    (`std::fs::canonicalize` of the root only: a root may sit under a link of the system such as `/var`), with its
    device;
  - check 2, `Roots::place(path) -> Result<Placed, SkipReason>`: absolute, no `.` or `..` component, strictly inside
    one root (never the root itself), by components; `Placed` holds the root and the relative components;
  - check 3 and 4: `Protection::check` (02/03) on the physical path (the canonical root joined with the relative
    components), with the ignored paths added;
  - check 5, `Handle::open(&Placed) -> Result<Handle, SkipReason>`: `OpenOptions` with `O_NOFOLLOW_ANY | O_RDONLY`
    (the constant measured in 01/02), `ELOOP` and its kin → `ThroughLink`, `ENOENT` → `Vanished`, anything else →
    `CheckFailed`; a link that is itself the item is opened as the link (`O_SYMLINK | O_NOFOLLOW_ANY`: 01/02 measured
    that it opens the last component as the link and still refuses a link in the middle), so the link is what is
    checked and removed; the handle keeps the file open;
  - check 6: the handle's `st_dev` equals its root's (`OtherVolume`).

## Produces

`Handle`: an open, link-free, in-root, unprotected item on its root's volume — the input of 03.

## Checks

On `FixtureTree`s built for each case (negative first):

- `a_path_outside_every_root_is_refused`, `a_root_itself_is_refused`, `a_dotted_path_is_refused`,
  `a_root_no_longer_in_the_settings_is_refused`, `a_shared_prefix_is_not_inside` (`root2` vs `root`),
  `a_protected_path_is_refused`, `an_ancestor_of_a_protected_path_is_refused`, `a_whole_folder_is_refused`,
  `an_ignored_path_is_refused`, `a_link_in_the_middle_is_refused`, `a_link_to_outside_the_root_is_refused`,
  `a_vanished_item_is_reported`, `an_item_on_another_volume_is_refused` (a disk image mounted inside the fixture,
  skipped without `hdiutil`).
- Positive: `an_item_inside_a_root_opens`, `a_link_as_the_item_opens_as_the_link`,
  `a_root_under_a_system_link_is_canonical` (the temporary folder under `/var`).

## Conclusions

- Built in `crates/cleaner/src/paths.rs` with the step's checks and seven more negative cases (an ancestor of a root,
  a relative path, a root that cannot be read, a protected physical path, the gravest answer, a file in the middle of
  the path, a link as a root), all on fixture trees; the volume case mounts a disk image inside the fixture with
  `hdiutil -nobrowse` (a test helper of the crate, `test_support.rs`) and skips itself without `hdiutil`.
- Taken the safest simple way where the step is silent:
  - **Roots are compared as written**: a plan root is kept only when the settings hold the same path; a root that
    cannot be made canonical, or is not a folder, makes every item under it `CheckFailed`, not `OutsideRoots`.
  - **Neither a root nor an ancestor of any root** (of the plan or of the settings, in either form) is ever an item:
    `OutsideRoots`. With nested roots the deepest one places the item, so its volume is the nearest root's.
  - **Checks 3 and 4 run on both forms of the path** (as the plan names it and the physical one under the canonical
    root) and against both protections (the system's, given to the cleaner, and the context's, which adds the ignored
    paths); the gravest answer wins: `Protected` (a `never` place, inside one or above one), then `WholeFolder`, then
    `Ignored`.
  - **The item is always opened with `O_SYMLINK | O_NOFOLLOW_ANY | O_NONBLOCK`**: whatever the last component is, it
    is opened as itself (a link as the link) without a separate look first, so there is no gap between a look and the
    open; a named pipe does not block. `ELOOP` (and `EMLINK`) → `ThroughLink`, not found → `Vanished`, anything else
    (a file in the middle: `ENOTDIR`) → `CheckFailed`.
  - Check 6 is part of `Handle::open`: a handle on another volume than its root is never returned.
  - The snapshot of open files is asked for under both forms of each root (`Roots::within`), since the system names
    held files by their physical paths.
