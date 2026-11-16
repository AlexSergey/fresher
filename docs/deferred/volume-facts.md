# Volume kinds, free and purgeable space

**Agreed:** 2026-11-16, by delegation (plan 009, decision 3) · **Due at:** plan
[012](../plans/012-storage-map/readme.md), the storage map

## What is postponed

The extension of the core's `Volumes` port that the draft of plan 009 listed: each volume's kind (internal, external,
network, disk image), its free space and the space the system can purge on its own. Nothing reads them before the
storage map and the overview of plan 012, so plan 009 left the port as plans 004 and 008 made it (`identify`,
`available`).

## What doing it needs

- The kind from `statfs` (`f_flags`: `MNT_LOCAL`, `MNT_RDONLY`; the mount's device) in `fresher-macos`'s `MacVolumes`.
- The purgeable space from a Foundation resource value of the volume (the capacity available for important usage),
  one call in `fresher-macos`'s `ffi` with its `// SAFETY:` comment; its answer changes from moment to moment, so its
  test checks only that it is at least the plain free space.
- `FakeVolumes` answering both.
