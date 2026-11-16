# 004/03/03 — Volumes and the places that prompt

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos

## Files

- `crates/macos/src/ffi/volume.rs` — `statfs` (from `libc`) and `getattrlist` for the volume attributes 01 chose
  (the volume's UUID, the format capabilities for case sensitivity).
- `crates/macos/src/volume.rs` — `MacVolumes` implements `Volumes`: `identify(path)` → `VolumeFacts { id, device,
  case_sensitive, used }`; `used` is `(f_blocks − f_bfree) × f_bsize`. A path on a volume with no UUID (some foreign
  file systems) gets an id derived from the file system type and `f_fsid`, marked so that the scanner never keeps a
  journal position for it (it always scans it in full).
- `crates/macos/src/places.rs` — `pub fn places_that_prompt(home: &Path) -> Vec<PathBuf>`: the stand-in avoid list
  from 01/03 (the home's Desktop, Documents, Downloads, `Library/Containers` and whatever else 01 saw prompt), as
  absolute paths under the given home; pure — it builds paths, it opens nothing. Plan 006 replaces its use with the
  capability state; until then the app passes it as `WalkRequest::avoid`.

## Produces

`MacVolumes` for the walker (05/02) and the scope key (05/05); `places_that_prompt` for the composition root (05/06).

## Checks

- Negative: `a_path_that_does_not_exist_is_an_error`, `places_that_prompt_never_contains_the_home_itself`,
  `places_that_prompt_opens_nothing` (it is given a home that does not exist and still answers).
- Positive: `the_same_volume_has_the_same_id_for_two_paths`, `a_temporary_folder_is_on_the_volume_of_its_parent`,
  `a_disk_image_has_its_own_id_and_device` (a test that creates and detaches an APFS image with `hdiutil`, marked
  `#[ignore]` when `hdiutil` is not available; it never touches another volume), `used_space_is_below_the_total`.
- `cargo nextest run -p fresher-macos`.

## Conclusions

- `MacVolumes::identify` takes the device from the path's own metadata (`st_dev`, as the bulk read reports entries;
  `statfs` and `stat` disagree on `/`, 01), `used` from `statfs` (saturating), and the UUID and case sensitivity from
  `getattrlist(ATTR_VOL_CAPABILITIES | ATTR_VOL_UUID)` on the mount point, parsed by safe code that follows the
  returned mask. Only these two volume attributes are asked, which pack in bit order (01 saw `SPACEUSED` out of bit
  order, so it is not asked; `used` comes from `statfs`). Checked by hand: the data volume's id equals its UUID in
  `diskutil info`.
- **Deviation:** a volume with no UUID (or whose attributes cannot be read) gets an id from its file system type and
  what is mounted (`f_mntfromname`), not from `f_fsid`: `libc` keeps `fsid_t`'s value private, and the mount source is
  steadier across mounts. Its top 32 bits are a fixed tag; `is_derived` tells it apart, and step 04's journal gives
  such a volume no position, so it is always scanned in full.
- `places_that_prompt(home)` is the list of design call 17: Desktop, Documents, Downloads, Pictures, Movies, Music,
  `Library/Containers`, `Library/Group Containers`, `Library/Mobile Documents`, `Library/CloudStorage`,
  `Library/Calendars`, `Library/Application Support/AddressBook`; built name by name with `join`, pure.
- The disk image test mounts a 32 MiB APFS image with `-nobrowse` in its own temporary folder and detaches it when
  dropped (`disk_image.rs`, tests only). Instead of `#[ignore]`, it returns early when `/usr/bin/hdiutil` is missing,
  so it runs wherever it can (about 1.5 s here).
- Checks: 13 new tests; Clippy clean.

