# 008/04/01 — The crate, reach and the sandbox

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** cleaner

## Files

- `crates/cleaner/Cargo.toml` — `fresher-core`, `thiserror`, `tracing`; dev: `fresher-testing`, `proptest`,
  `tempfile`, `trybuild`.
- `crates/cleaner/src/lib.rs` — the crate's doc (what it is, that it is the only code that removes, the link to
  `docs/architecture/removal.md`), `#![forbid(unsafe_code)]`, the modules, and `DiskCleaner::new(reach, user,
  protection, ports)` with `CleanerPorts { trash, open_files, commands, clock }`; the `Cleaner` methods answer
  "not built" until 06 (a `todo!` is not allowed: they return every item skipped with `CheckFailed`).
- `crates/cleaner/src/reach.rs` (new):

  ```rust
  pub enum Reach { Sandbox, RealDisk(RealDiskToken) }
  pub struct RealDiskToken(());
  impl RealDiskToken {
      #[cfg(not(debug_assertions))]
      pub fn new() -> Self;
  }
  ```

  and check 1: in `Sandbox`, the item must have, at its root or between its root and itself, a folder holding the
  core's `SANDBOX_MARKER` — a regular file (not a link), owned by the user, whose first line is `SANDBOX_SIGNATURE` —
  and that folder must not be a `whole` folder of the protection (never the home). The marker is read with the same
  no-link open as the items (02).
- `crates/cleaner/tests/reach.rs` (new) — reads every `.rs` file of the workspace (`crates/`, `modules/`, `app/`) and
  fails when `RealDiskToken::new` appears anywhere but `app/tauri/src/compose.rs` and this crate's `reach.rs`.

## Produces

The cleaner's skeleton and the last line of defence that makes a debug build or a test unable to remove anything
outside a sandbox.

## Checks

- Negative (first): `an_item_with_no_marker_above_it_is_not_in_the_sandbox`, `a_marker_that_is_a_link_does_not_count`,
  `a_marker_with_the_wrong_signature_does_not_count`, `a_marker_above_the_root_does_not_count`,
  `a_marker_in_a_whole_folder_does_not_count` (a fake protection names the fixture's `home` as whole),
  `the_token_appears_only_in_the_composition` (the source test, against a planted offender in a temporary copy).
- Positive: `an_item_under_the_fixture_root_is_in_the_sandbox`, `a_marker_in_a_subfolder_opens_only_that_subfolder`.
- `cargo nextest run -p fresher-cleaner`; Clippy; the crate guard (`npm run check:crate-deps`) passes with the core as
  the only normal dependency.

## Conclusions

- Built as written: `Reach`, `RealDiskToken` (made only under `cfg(not(debug_assertions))`, no `Default`), check 1,
  `DiskCleaner::new(reach, user, protection, CleanerPorts { trash, open_files, commands, clock })` with the ports as
  `Arc<dyn …>` like the engine's, and the `Cleaner` methods answering "not built" (every item `CheckFailed`) until 06.
- Taken the safest simple way where the step is silent:
  - **Which protection decides where a marker counts**: the cleaner's own (the system's list given to `new`), not the
    context's, which may name places inside a sandbox for the tests of check 3. A marker counts only in a folder that
    list does not refuse as a protected place, an ancestor of one or a `whole` folder (or an ancestor of one): the
    core's `Protection::check` cannot tell a `whole` folder from its ancestors, and the real home answers `Ancestor`,
    so both are refused. The ignored paths do not matter here.
  - **"Between the root and the item"** excludes the item: a marker inside a folder opens what is below it, never the
    folder itself.
  - The marker is opened with `O_NOFOLLOW_ANY` (a link as the marker fails) and `O_NONBLOCK` (a named pipe does not
    wait); it must be a regular file of the user; its first line is the signature exactly (with or without a line
    end), read from at most the signature's length and one byte.
  - **A tool's item is never in the sandbox** (`Reach::admits_tools`): a tool acts on the real system, so a debug
    build or a test never runs one through the cleaner.
  - Off macOS the no-link open answers `Unsupported`, so nothing is ever acted on there.
  - **The source test is stricter**: besides `RealDiskToken::new` (whitespace ignored) outside `reach.rs` and
    `app/tauri/src/compose.rs`, it refuses any mention of the token's name outside those two files and the cleaner's
    `lib.rs` (the re-export). It is tested on a temporary tree with planted offenders, then on the workspace.
- Dependencies: `fresher-core` only; `fresher-testing` and `tempfile` for the tests. `thiserror` and `tracing` are
  added by the step that first uses them (`cargo machete` refuses an unused one). The crate guard passes.
