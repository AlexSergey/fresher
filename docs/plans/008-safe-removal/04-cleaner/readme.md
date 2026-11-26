# 008/04 — The cleaner

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner, docs

`fresher-cleaner` becomes the only code that removes anything: `DiskCleaner` implements the core's `Cleaner` port with
the reach and the sandbox, the eleven checks of the guard, the four removals and the single deletion point, the
rehearsal and the execution, and put back. It depends on the core alone and reaches the disk through `std` and the
ports it is given, so every test runs on a `FixtureTree` (with its sandbox marker) and the fakes of 02/04. The design:
[`../00-design.md`](../00-design.md), sections 2, 5, 6, 7 and 8.

Depends on 02. Runs in parallel with 03, 05 and 06/01–06/03. Nothing here runs on the real disk: in tests the reach
is always `Sandbox`.

## Steps

| #   | Step                                                                     | Status   | Shared files                              |
| --- | ------------------------------------------------------------------------ | -------- | ----------------------------------------- |
| 01  | [The crate, reach and the sandbox](./01-crate-and-reach.md)              | ✔️ done    | `crates/cleaner/Cargo.toml`, `Cargo.lock` |
| 02  | [Paths: roots, protection, links](./02-paths.md)                         | ✔️ done    | —                                         |
| 03  | [The item: identity, removability, capability, use](./03-item-checks.md) | ✔️ done    | —                                         |
| 04  | [The folder walk](./04-folder-walk.md)                                   | ✔️ done    | —                                         |
| 05  | [The removals and the deletion point](./05-removals.md)                  | ✔️ done    | `clippy.toml` (the lint's reason)         |
| 06  | [Rehearse and execute](./06-rehearse-and-execute.md)                     | ✔️ done    | —                                         |
| 07  | [Locate and put back](./07-put-back.md)                                  | ✔️ done    | —                                         |
| 08  | [Proofs and the threat model](./08-proofs.md)                            | ✔️ done    | `docs/architecture/removal.md` (new)      |

In order: each step adds checks or acts on the ones before. 04 and 07 may run beside 05–06 (separate files).

## Conclusions

The eight steps are built in order, one commit each; `npm run pre-push` passes on the last one. `fresher-cleaner`
depends on the core alone; its tests run only on temporary trees with the sandbox marker and the fakes of 02/04 (and
disk images they mount inside them, skipped without `hdiutil`); nothing touches the home folder or the real Trash.

- **Changed from the steps** (each the safer or the simpler way, with the reason in its step):
  - `DiskCleaner::new(reach, user, home, protection, ports)` takes **the home folder**: the scan roots reach the
    cleaner as the settings write them (`~`, `~/…`), and are read under it before any check (06).
  - `verify(item, cx, depth)` answers `Unverified` (`Skipped(reason)` or `Cancelled`), so a walk stopped by the cancel
    token is `NotReached` (04); the removals take `Act` by value (it is `Copy`; Clippy refuses `&Act`) and return a
    `Removed` the pipeline makes the `ItemDone` from (05).
  - `Verified` is public (re-exported) so the compile-fail cases can name it; it has no constructor or accessor
    outside the crate (03, 08).
  - `thiserror` and `tracing` are not dependencies: nothing uses them (`cargo machete`); `serde_json`, `proptest` and
    `trybuild` are dev-dependencies.
  - The source tests are stricter than asked: the token's name anywhere but its three places, and the groups that
    hold the deletion lint (`clippy::style`, `clippy::all`).
- **Safer than asked** (the steps were silent): a tool never runs in the sandbox; a tool must be `Granted`; forced
  tool arguments are refused again; the sandbox marker counts only where the system's protection refuses nothing;
  neither a root nor an ancestor of one is an item; checks 3, 4, 9 and 10 look at both forms of the path; check 9
  asks for guarded places inside the item too; check 11 measures files too; put back runs check 1; an act on an item
  without an identity, a root that cannot be made canonical or a relative root acts on nothing.
- **What the other plans said, and how it is met**: 03's `MacTrash` gives an empty `at` when it cannot read the held
  handle's path: an item that left its place is then `Moved` with nothing to put back, never "failed and still
  there" (05); its put-back answers map as 07 says; 03's `MacOpenFiles` leaves Fresher's own process out, and the
  pipeline holds no handle while it takes a snapshot; 03's protection list (`/Users` `never` with the home and
  `/Users/Shared` carved out) refuses a marker in the home (`Ancestor`) and allows one in a folder of it. The engine
  (06/01–06/03) passes an empty context protection, `~` roots and no open-files check: the system's list always
  applies, `~` is read under the home and an unusable root holds nothing, and a failed snapshot skips every item —
  each with a negative test (06).
- **Left, as the threat model says**: the system's Trash call and the two deletes take a path, so a parent folder
  swapped for a link in the microseconds between the check and the act can make them act behind the link; after a
  move this is detected and reported (and what can be named is journalled), after a delete it is not. Processes of
  other users and a file opened after the last snapshot are not seen.
- **Hardened afterwards** (the known issue [`cleaner-act-by-path`](../../../known-issues/fixed/cleaner-act-by-path.md), now
  fixed): a permanent delete, the emptying of a folder and check 11's walk act beneath held folders through
  `fresher-macos`'s new `beneath` module (`open_folder`, `open_folder_at`, `identity_at`, `read_folder`, `remove_at`;
  `openat`/`fstatat`/`unlinkat` relative to a descriptor, `O_NOFOLLOW_ANY`). The cleaner now depends on
  `fresher-macos` (macOS only), as the crate table allows; no port and no change to the core. `std::fs`'s removals are
  no longer called: the deletion point stays `remove/permanent.rs`, now the only caller of `remove_at` (in
  `clippy.toml`), and `tests/deletion_point.rs` also keeps the system's deleting calls in `ffi/beneath.rs`. A folder
  is removed depth first, never across a volume, and stops at the first refusal (`Failed`; what went before it stays
  gone). The walk is `HeldWalk` (was `StdWalk`). `fresher-testing` gains `Swapper` for the race tests. Left: an entry
  replaced inside a held folder between its check and `unlinkat`, which stays in that folder on its volume; the Trash
  call still takes a path (detected after the move).

