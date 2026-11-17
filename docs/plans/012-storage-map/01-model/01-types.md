# 012/01/01 — The map's types

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core

## Files

- `crates/core/src/storage/mod.rs` (new) — the module's doc and its re-exports.
- `crates/core/src/storage/map.rs` (new) — `StorageMap { phase: MapPhase, volumes: Vec<VolumeMap> }`;
  `MapPhase::{NoScan, Scanning, Complete { state: StateId }}`; `VolumeMap { name, kind: MountKind, capacity, parts,
  purgeable: Option<ByteSize>, roots: Vec<NodeId>, disk_changed: bool }` (`disk_changed`: the scan counted more
  than the volume reports, section 3 of the design); `MapPart { kind: PartKind, sizes: Sizes }`;
  `PartKind::{Explained, YourFolders, NotScannedYet, Remainder { outside_scan, reasons }, MacOs, SwapAndSleep,
  OtherVolume { name }, DiskOwn, Free}`; `RemainderReason::{Unreadable { folders, needs: Option<Capability> },
  SnapshotsAndBookkeeping, ChangedDuringScan}`. `VolumeMap::new(…)` checks the one invariant it can: the parts add up
  to `capacity` (`checked_add`, no overflow) and come in the fixed order of `PartKind` — a `StorageMapError`
  (`thiserror`) otherwise, so a wrong map is never built.
- `crates/core/src/storage/meaning.rs` (new) — `MeaningGroup::{App { app: AppRef, sizes, categories: Vec<CategoryShare>,
  data: Vec<PlaceShare> }, Category(CategoryShare), Place(PlaceShare), Part(MapPart)}`; `AppRef { id: BundleId, name:
  String, installed: bool }`; `CategoryShare { category: CategoryId, sizes }`; `PlaceShare { node: NodeId, name:
  PlaceName, sizes }`; `PlaceName::{Folder(String), Trash}`.
- `crates/core/src/storage/place.rs` (new) — `PlaceView { path: Vec<NodeInfo>, rows: Vec<PlaceRow>, offset, total }`
  (`path`: the node opened and the folders the descent went through); `PlaceRow { node: NodeInfo, explained: Sizes }`.
- `crates/core/src/volume.rs` — `MountKind::{Internal, External, Network}` only (serde kebab-case, `TS`); the port's
  other types come in 02/01. The name: `VolumeKind` is the access capability's.
- `crates/core/src/lib.rs` — `mod storage;` and the exports.
- `crates/core/locales/en.json` — `core.storage.part.<kind>.name` and `.description` for every `PartKind` (rule 7: one
  or two plain sentences, what it is and what the user can do), `core.storage.remainder.outside.*`,
  `core.storage.remainder.not-measured.*`, `core.storage.reason.<reason>`, `core.storage.purgeable` ("macOS can free
  about {size} more on its own when space runs low"), `core.storage.place.trash` ("Trash" — "You get this space back
  when you empty the Trash."), `core.storage.kind.<kind>`, `core.storage.changed` (the negative remainder's note).
- `crates/core/tests/typescript.rs` — the declarations of the new types.

Every serialized type is camelCase with kebab-case tags, as `NodeInfo` and `NotScanned`.

## Produces

The types every later step fills; the texts the UI renders by key.

## Checks

- Negative first: `parts_that_do_not_add_up_to_the_capacity_are_refused`, `parts_out_of_order_are_refused`,
  `an_overflowing_sum_is_refused`, `every_part_kind_has_a_name_and_a_description` (a missing key fails the dictionary
  test, as the existing `texts.rs` tests do), `no_text_says_other` (no `core.storage.*` value contains "Other",
  "Miscellaneous" or "System Data").
- Positive: `a_volume_map_serializes_in_camel_case` (one JSON literal with every part), `a_remainder_serializes_its_reasons`,
  `a_meaning_group_serializes_by_kind`, `the_typescript_declarations_are_written`.
- `cargo nextest run -p fresher-core`.

## Conclusions

Built in `d500870` (2026-11-17), reviewed clean.

- `crates/core/src/storage/` holds `StorageMap`, `MapPhase`, `VolumeMap`, `MapPart`, `PartKind`, `RemainderReason`,
  the meaning types and `PlaceView`; `MountKind` is in `volume.rs`.
- Deviations: `VolumeMap` keeps its fields private behind getters, so only `VolumeMap::new` can build it and the sum
  check cannot be skipped. The order rule is strict by `PartKind`; only `OtherVolume` may repeat. The remainder has
  no `part.remainder` keys: its name and description are the `outside` or `not-measured` pair. `BundleId` gained
  `Serialize`.
- Measured: 30 `core.storage.*` texts, none says "Other"; `cargo nextest run -p fresher-core` 294 passed.
