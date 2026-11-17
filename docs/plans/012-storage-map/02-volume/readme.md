# 012/02 — Volume reconciliation

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
testing, macos, engine

The volume facts plan 009 deferred ([`../../../deferred/volume-facts.md`](../../../deferred/volume-facts.md)) — kind,
free and purgeable space — with the capacity and the volumes that share it, and the parts of a volume with the
remainder and its reasons. The design: [`../00-design.md`](../00-design.md), sections 1 to 3.

## Steps

| #   | Step                                                    | Status                                   | Shared files                                                                                   |
| --- | ------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------- |
| 01  | [The space of a volume](./01-space-port.md)             | ✔️ done | `crates/core/src/{lib,volume}.rs`, `crates/testing/src/fakes.rs`, `crates/macos/src/volume.rs` (a stand-in) |
| 02  | [The Foundation call](./02-foundation-call.md)          | ✔️ done | `crates/macos/Cargo.toml`, root `Cargo.toml`, `Cargo.lock`, `docs/dependencies.md`             |
| 03  | [`MacVolumes::space`](./03-mac-space.md)                | ✔️ done | `crates/macos/src/volume.rs`                                                                   |
| 04  | [The parts and the remainder](./04-parts.md)            | ✔️ done | `crates/engine/src/storage/mod.rs`, `crates/core/locales/en.json`                               |

01 after 01/01 (it adds to `volume.rs`, where 01/01 put `MountKind`); 02 from the start, beside anything; 03 after 01
and 02; 04 after 01/03, 01/04 and 03.

## Conclusions

Built on `feat/012-storage-map` on 2026-11-17, task by task, each commit through the hooks and reviewed.

- **What is there now**: `Volumes::space` (`DiskSpace`, `MountKind`, `Neighbour`), the Foundation call, `MacVolumes::space`
  and `volume_map`: the parts and the remainder with its reasons, adding up to the capacity. The deferred volume-facts
  item of plan 009 is closed by it.
- **Changed from the design while building**: the used space of a volume is read with `getattrlist`, not `statfs`:
  on APFS `statfs` gives the container's used space and every volume read the same (02/03). A disk image gives no
  internal flag, so it reads as external (02/02). `disk_changed` is set only in the complete phase (final review).
- **Measured**: the parts add up to the capacity for any facts, phase and roots (property test); the values on this Mac
  match `diskutil apfs list`; `cargo nextest run -p fresher-macos` 147 passed.
