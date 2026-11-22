# 038/01/02 — Stated sizes and the stated volume

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** testing, sandbox, app

The app shows the space a file takes on disk (`Sizes::allocated`), so a sparse file shows 0 bytes. A demo world with
`sizes = "as-declared"` states, for every sparse file, an allocated size equal to its declared size; sandbox mode
reads it through `FactsLens`. A demo world's `[demo.volume]` is answered as the volume's space, so the ring and the
storage map never show the developer's disk. The design: [`../00-design.md`](../00-design.md), section 1, agent's calls
8 and 9.

Read first: `crates/testing/src/facts_lens.rs` (`Stated`, `apply`), `crates/testing/src/tree_description.rs` (`stated`,
`Fill`), `crates/sandbox/src/world.rs` (`World::of`, `RawWorld`, `RawEntry`, `text`, `parse`), `crates/core/src/size.rs`
(`Sizes`), `crates/core/src/volume.rs` (`Volumes`, `DiskSpace`), `crates/testing/src/fakes.rs` (`FakeVolumes::disk`),
`app/tauri/src/sandbox.rs` (`SandboxPorts::real`). **No file under `crates/core`, `crates/scanner`, `crates/cleaner`,
`crates/engine` or `crates/store` changes**: `Stated` literals elsewhere already use `..Stated::default()`.

## Files

- `crates/testing/src/facts_lens.rs` — `Stated` gains `pub allocated: Option<ByteSize>` ("the space it takes on disk,
  stated: a sparse file of a demo world takes its declared size"). `Stated::apply`: when stated, the entry's sizes
  become `Sizes::complete(entry.sizes.logical, allocated)` (logical kept, allocated and reclaimable stated); a folder's
  entry is never given one.
- `crates/testing/src/tree_description.rs` — `TreeDescription::sparse_files(&self) -> impl Iterator<Item = (&Path,
  u64)>`: the relative path and declared size of every file whose fill is sparse (no `fill`, no `text`, no `kind`) and
  whose size is above 0. Used by the world; `pub` with a doc line.
- `crates/sandbox/src/world.rs`:
  - `RawEntry` gains `allocated: Option<u64>` (`skip_serializing_if = "Option::is_none"`); read into
    `Stated.allocated`; the head comment of `[[entry]]` in `text()` names it.
  - `World::of(files, now)`: when a file's `demo()` has `DemoSizes::AsDeclared`, every sparse file of its cases gets
    an entry with `allocated` equal to its size (merged into the same path's entry when that path already states
    other facts).
  - `World` gains `volume: Option<DemoVolume>` written as a `[volume]` table (`name`, `capacity`, `used`, `free`) and
    read back; `pub fn volume(&self) -> Option<&DemoVolume>`.
- `crates/sandbox/src/stated_space.rs` (new) — `pub struct StatedSpace { inner: Arc<dyn Volumes>, space: DiskSpace }`:
  `StatedSpace::new(inner, volume: &DemoVolume)` builds a `DiskSpace` (`name`, `mount_point` `/`, `MountKind::Internal`,
  `capacity`, `used`, `free`, no purgeable space, no neighbours — as `FakeVolumes::disk`); `Volumes::space` answers it
  for every path, `identify` and `available` are the inner volumes'. Exported from `lib.rs`.
- `app/tauri/src/sandbox.rs` — `SandboxPorts::real`: `volumes` is `StatedSpace::new(Arc::new(MacVolumes::new()),
  volume)` when `world.volume()` is stated, else `MacVolumes::new()` as today. The doc of `SandboxPorts::real` names
  the stated volume.

## Produces

Stated allocated sizes and a stated volume for 01/03's world and for sandbox mode.

## Checks

- `facts_lens.rs`, negatives first: no stated `allocated` leaves the entry's sizes as they are; a folder entry keeps
  its sizes. Then: a stated `allocated` replaces allocated and reclaimable and keeps logical.
- `tree_description.rs`: `sparse_files` skips data, text, cache-directory-tag, stated-kind and zero-size files; lists
  a sparse file with its size.
- `world.rs`, negatives first: a world of a demo with `Sparse` (or a normal case file) states no `allocated`; an
  unknown key in `[volume]` is refused when read. Then: a demo world with `AsDeclared` states each sparse file's size
  (and not a data file's); an entry stating an owner and an allocated size for the same path is one entry; the
  `[volume]` round-trips through `text()` and `read`.
- `stated_space.rs`: `space` answers the stated numbers for any path; `identify` and `available` are passed through
  (a `FakeVolumes` as the inner one, its answers seen).
- `app/tauri/src/sandbox/tests.rs`: a sandbox whose world states a volume reports its capacity through the engine's
  storage map or through `SandboxPorts::real(...).volumes.space(..)` (whichever the existing tests reach), and one
  without keeps the Mac's (`MacVolumes`, checked only to be `Ok`).
- `cargo nextest run -p fresher-testing -p fresher-sandbox -p fresher-app`, `-p fresher-scanner` (its
  `facts_lens_tests` still compile and pass); clippy; fmt.
- `git diff -- crates/core crates/scanner crates/cleaner crates/engine crates/store` empty.

## Conclusions

Built as briefed: `Stated.allocated` (`crates/testing`) replaces the allocated size of a non-folder entry through
`FactsLens`, `TreeDescription::sparse_files`, `[volume]` and the `allocated` lines of `world.toml`, and `StatedSpace`
(`crates/sandbox`), which `SandboxPorts::real` and the sandbox test harness use when the world states a volume.

- The `allocated` line of the `[[entry]]` head comment in `world.toml` is written only when an entry states it: writing
  it always changed every sandbox golden.
- Nothing of the stated sizes or the volume is reachable outside a sandbox (checked in review).
- 1122 tests passed across testing, sandbox, app and scanner; Rust coverage of the touched files 94 to 99 %.
