# 008/03 — macOS layer

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos

The macOS adapters of the ports plan 02 defined: the system Trash and put back, the open-files snapshot, the current
user, and the protection list for a home folder. Every FFI call goes into `fresher-macos`'s `mod ffi`, one unsafe
operation per block with its `// SAFETY:` comment, behind a safe function (plan 004/03's rule); the rest of the crate
is safe Rust. The design: [`../00-design.md`](../00-design.md), sections 5, 6, 8 and 12; the measured behaviour:
"Measured (01)".

Depends on 01 (the answers) and 02 (the ports). Runs in parallel with 04, 05 and 06/01–06/03, which use the fakes.
Every test runs on a temporary folder or on a disk image the test mounts with `-nobrowse`; nothing moves an item of
the user's, and nothing lands in the user's own Trash.

## Steps

| #   | Step                                                   | Status   | Shared files                                                        |
| --- | ------------------------------------------------------ | -------- | ------------------------------------------------------------------- |
| 01  | [The Trash and put back](./01-trash.md)                | ✔️ done    | `crates/macos/{Cargo.toml,src/lib.rs,src/ffi/mod.rs}`, `Cargo.lock` |
| 02  | [Open files and the user](./02-open-files-and-user.md) | ✔️ done    | `crates/macos/src/{lib.rs,ffi/mod.rs,process.rs}`                   |
| 03  | [The protection list](./03-protection-list.md)         | ✔️ done    | `crates/macos/src/lib.rs`                                           |

The three steps touch separate files apart from one export line each and may run in parallel.

## Conclusions

Done on 2026-11-16: the three steps are built in order, one commit each; `npm run pre-push` passes on the last one.
`fresher-macos` exports `MacTrash`, `MacOpenFiles`, `current_user` and `protection`, for 06/04 to wire. Every FFI call
is in `mod ffi` (`trash.rs`, `rename.rs`, `libproc.rs`), one unsafe operation per block with its `// SAFETY:`
comment; the crate's "no unsafe outside `ffi`" test holds.

- **The Trash** (01): `MacTrash::move_to_trash` answers `Landed { at, reported }`, `at` from the held handle
  (`F_GETPATH`), empty when the system cannot say; the codes map to the frozen port's variants (513 → `NotAllowed`,
  `ELOOP` of put back → `ThroughLink`, both finer than the step's text). Put back is `renamex_np` with `RENAME_EXCL |
  RENAME_NOFOLLOW_ANY`, and it works for a link that is the item itself.
- **Open files** (02): descriptors, working folders and mapped files (the region walk reused for 30 s) of the user's
  own processes, **this process left out** (the cleaner holds the item it checks); 11 ms per snapshot in a release
  build, 0.56 s with the region walk.
- **Protection** (03): the design's list, with other homes kept by carving the user's own home and `Shared` out of a
  `never` `/Users` rather than by listing `/Users`.
- **Nothing reached the user's Trash**: every test that moves an item does it on an APFS image it mounts with
  `-nobrowse`, and puts it back or lets it vanish with the image; no test lists or empties a Trash.
- Dependencies: `objc2` 0.6.5 and `objc2-foundation` 0.3.2, both MIT (the design's table gave the latter's older
  licence choice).
