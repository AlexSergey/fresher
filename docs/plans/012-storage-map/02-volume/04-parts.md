# 012/02/04 — The parts and the remainder

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** engine,
core

## Files

- `crates/engine/src/storage/volume.rs` (new) — `volume_map(input: VolumeInput) -> Result<VolumeMap,
  StorageMapError>`, pure. `VolumeInput` holds the volume's `DiskSpace` (or none), the roots on it with their sizes and
  explained bytes (from `Shares`, or the findings so far during a scan), whether the roots cover the mount point, the
  unreadable places on it (count, and the capability that would show them from the access status), and the phase.
  - **Scanned parts**: `Explained` and `YourFolders` (complete phase) or `Explained` and `NotScannedYet` (scanning,
    no-scan).
  - **The remainder**: `space.used − scanned`; `outside_scan` unless a root on the volume is its `mount_point`
    (compared by components; the engine knows no macOS path, so a root of `/` on a Mac whose data volume is mounted
    elsewhere reads as outside — measured in the run by hand, 06/02); reasons in a fixed order —
    `Unreadable { folders, needs }` when any folder was not read, then `SnapshotsAndBookkeeping` and
    `ChangedDuringScan` always (the disk always changes during a scan). Negative: the remainder is zero, `used` is
    taken as the scanned sum, and `disk_changed` is set (the UI shows `core.storage.changed`).
  - **Neighbours**: `MacOs` (the sum of the `System` neighbours), `SwapAndSleep` (`VirtualMemory`), one `OtherVolume`
    per `Other` neighbour.
  - **`DiskOwn`**: `capacity − free − used − Σ neighbours`, never below zero; when below, `capacity` is taken as the
    sum of the parts and `free`.
  - **`Free`**; `purgeable` copied to the map.
  - **Without volume facts** (no `Volumes` port, or it failed — logged): the scanned parts only, `capacity` their sum,
    `kind` `Internal`, the name of the root, no remainder.
  Parts of zero size are kept (the UI hides them); their order is `PartKind`'s.
- `crates/engine/src/storage/meaning.rs` — the system's parts, the remainder and Free are appended to the by-meaning
  groups as `MeaningGroup::Part`, so both views end with the same lines.
- `crates/engine/src/storage/mod.rs` — `mod volume;`.
- `crates/core/locales/en.json` — any text the build shows missing (01/01 wrote them).

## Produces

`VolumeMap` for the query (03/02).

## Checks

- Negative first: `a_negative_remainder_is_zero_with_the_note`, `a_negative_disk_own_space_is_zero`,
  `without_volume_facts_only_the_scanned_parts_are_drawn`, `a_home_root_is_outside_scan`,
  `unreadable_folders_are_a_reason_not_a_part`, `no_part_is_named_other` (every `PartKind` maps to its own text key).
- Positive: `the_developers_mac_adds_up` (the numbers of `df -k` on the developer's Mac, measured on 2026-11-17, as a
  literal: 482 797 652 KiB capacity, 77 429 216 free, Data 380 727 388, `/` 13 434 928, Preboot 9 541 224, VM 24,
  Update 23 756 — parts add up to the capacity, `DiskOwn` about 1.6 GiB), `a_root_at_the_mount_point_is_not_measured`,
  `scanning_splits_used_into_explained_and_not_scanned_yet`; a **property test**: for any `DiskSpace`, scanned sizes
  and neighbours, the parts add up to `capacity` and none is negative.
- `cargo nextest run -p fresher-engine`.

## Conclusions

Built in `0309e0e`, test gaps closed in `55a6ecc`, reviewed clean.

- `volume_map` (pure) gives Explained, your folders, the remainder with its reasons, macOS, swap and sleep image, each
  other volume, the disk's own space and Free. Without volume facts only the scanned parts are drawn. A negative
  remainder is zero with the "disk changed" note.
- Deviations: whether the roots cover the mount point is computed inside from the root paths. While scanning there
  is no remainder, "Not scanned yet" takes all the rest. Later (final review): `disk_changed` is set only in the
  complete phase, because findings arriving during a scan can pass the used space.
- Measured: the property test says the parts add up to the capacity for any facts, phase and roots; the developer's
  Mac literal (`df -k`) adds up with 1.57 GiB of disk's own space.
