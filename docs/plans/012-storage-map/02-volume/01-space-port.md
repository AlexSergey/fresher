# 012/02/01 — The space of a volume

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
testing

## Files

- `crates/core/src/volume.rs` — beside `MountKind` (01/01):
  - `DiskSpace { name: String, mount_point: PathBuf, kind: MountKind, capacity: ByteSize, used: ByteSize, free:
    ByteSize, purgeable: Option<ByteSize>, neighbours: Vec<Neighbour> }` — `capacity` is the space the volume shares
    (on APFS the container), `used` this volume's own, `free` what the user can still write, `purgeable` what the
    system can free on its own (`None` when it does not say);
  - `Neighbour { name: String, role: NeighbourRole, used: ByteSize }`, `NeighbourRole::{System, VirtualMemory,
    Other}`;
  - a third method of `Volumes`: `fn space(&self, path: &Path) -> Result<DiskSpace, ReadError>`, documented as "the
    space of the volume that holds `path`; a link in `path` is followed, as `identify`".
  `VolumeSpace` is the cleanup journal's (a volume's available space before and after), hence `DiskSpace`.
- `crates/core/src/lib.rs` — the exports.
- `crates/macos/src/volume.rs` — `MacVolumes::space` answers `ReadError::Failed` in this step, so the workspace
  builds; 03 fills it in. The one stand-in, removed in the same plan.
- `crates/testing/src/fakes.rs` — `FakeVolumes::with_space(root, space)`: `space` answers the `DiskSpace` of the
  deepest root that holds the path, `ReadError::Vanished` outside every root, like `identify`; and
  `FakeVolumes::disk(name, capacity, used, free)` building a plain `DiskSpace` (internal, no neighbours, no purgeable)
  for the tests in one line.

## Produces

The port the parts (04) read and the adapter (03) answers.

## Checks

- Negative first: `space_outside_every_root_is_vanished`, `the_deepest_root_answers_the_space`.
- Positive: `with_space_answers_its_space`, `disk_builds_a_plain_internal_volume`.
- `cargo nextest run -p fresher-core -p fresher-testing`; `cargo clippy --workspace --all-targets -- -D warnings`.

## Conclusions

Built in `477606a`, reviewed clean.

- `Volumes::space` with `DiskSpace` and `Neighbour` with `NeighbourRole`; `FakeVolumes::with_space` answers by the
  deepest root and `FakeVolumes::disk` builds a disk. `MacVolumes::space` was a stand-in until 02/03.
- No deviation from the step.
- Measured: four new fake tests; `cargo nextest run -p fresher-core -p fresher-testing` 424 passed.
