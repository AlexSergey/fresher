# 012/02/03 — `MacVolumes::space`

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** macos

## Files

- `crates/macos/src/volume.rs` — `MacVolumes::space(path)`:
  - `statfs` of the path: `capacity = f_blocks × f_bsize`, `used = (f_blocks − f_bfree) × f_bsize` (what `identify`
    already computes, shared through one private function), `free = f_bavail × f_bsize`; `mount_point` from
    `f_mntonname`;
  - `kind`: `Network` when `f_flags` lacks `MNT_LOCAL`; otherwise `Internal` when `resources(…).internal` is
    `Some(true)`, else `External`;
  - `purgeable = important_available − free`, saturating at zero; `None` for a network volume or when the system gives
    no value;
  - `name`: the volume's name (`ATTR_VOL_NAME` in the existing `getattrlist` call's list, or the mount point's last
    name when it gives none; `/` reads as the volume's own name);
  - `neighbours`: every other mount of `ffi::volume::mounts()` (the cached list, no volume asked) whose
    `f_mntfromname` is a slice of the same disk — `/dev/disk<N>s…` with the same `<N>` — and whose `f_blocks` equals
    this one's (the same container); each with its used space and a role from its mount point: `/` → `System`,
    `/System/Volumes/VM` → `VirtualMemory`, anything else under `/System/Volumes/` → `System`, otherwise `Other`. A
    device name that is not `/dev/disk…` (a network share, a RAM disk) has no neighbours.
  - Pure parsing in private functions — `disk_of(from: &[u8]) -> Option<u32>`, `role_of(mount_point: &Path) ->
    NeighbourRole`, `kind_of(flags, internal)` — so most of it is tested without a volume.
- The stand-in of 02/01 is removed.

## Produces

The real answer of `Volumes::space` for the app.

## Checks

- Negative first: `disk_of_refuses_a_name_that_is_not_a_disk` (`map auto_home`, `devfs`, `//server/share`, `/dev/rdisk`
  with no slice), `a_volume_on_another_disk_is_not_a_neighbour` (two `statfs` literals with different disk numbers),
  `a_network_mount_is_network_whatever_internal_says`, `purgeable_never_goes_below_zero`,
  `a_path_that_does_not_exist_is_an_error`.
- Positive: `disk_of_reads_the_disk_number` (`/dev/disk3s1s1` and `/dev/disk3s5` → 3), `roles_follow_the_mount_points`,
  `the_space_of_a_temporary_folder_adds_up` (capacity ≥ used + free; free equals `available` read just before, within
  1 MiB), `a_disk_image_has_no_neighbours_and_is_external` (`DiskImage::mount`, skipped without `hdiutil`), and on
  the developer's Mac, `the_data_volume_sees_the_system_as_a_neighbour` — run only when `/System/Volumes/Data` exists,
  otherwise skipped with a printed reason.
- `cargo nextest run -p fresher-macos`.

## Built

- Used space is read per volume from `ATTR_VOL_SPACEUSED` (with the `statfs` formula as the fallback): on APFS
  `statfs` counts the whole container, so every volume of one container would report the same used space. This
  applies to `identify` as well, which shares the function.
- Each neighbour is asked once with `getattrlist` for its name and used space. The mount list is still read
  without asking any volume.
- Known limit: two plain partitions of one disk with equal `f_blocks` read as neighbours.

## Conclusions

Built in `2488b23`, minors fixed in `372742d`, reviewed approved.

- `MacVolumes::space` reads `statfs` and the resource values; the neighbours are the mounts on the same disk number
  with the same block count; each gets a role from its mount point.
- Deviation: the used space of a volume comes from `getattrlist` `ATTR_VOL_SPACEUSED`, not `(f_blocks - f_bfree)`:
  on APFS `statfs` gives the whole container's used space, so every volume read about 428 GB and the parts could never
  add up. Now it matches `diskutil apfs list` (Data 401.78 GB, Macintosh HD 13.76 GB, VM 1.07 GB). The cost is one
  `getattrlist` per neighbour, never a network volume. The attributes come in the documented order, not bit order.
- Measured: `cargo nextest run -p fresher-macos` 147 passed, 1 skipped.
