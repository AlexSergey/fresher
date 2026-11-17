# 007/02/01 — The reach of a sandbox folder

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** core, cleaner

## Files

- `crates/core/src/cleanup/protection.rs` — `Protection::joined(&self, other: &Protection) -> Result<Protection,
  ProtectionError>`: the union of both lists (never, whole, except, ignored), checked as `new` checks; a path one list
  refuses stays refused.
- `crates/cleaner/src/reach.rs` — `Reach::SandboxFolder(SandboxFolder)`. `SandboxFolder::new(path, user,
  real: &Protection) -> Result<SandboxFolder, SandboxError>`: the canonical folder, reached through no link, holding a
  valid marker of `user`, and not refused by `real` as never, an ancestor or a whole folder. `admits` for an item:
  strictly inside the folder by components, and the marker still valid right then (the same `holds_marker` as
  `Reach::Sandbox`); `admits_tools` is false. `RealDiskToken` is untouched.
- `crates/cleaner/src/lib.rs` — the re-export; `DiskCleaner::new`'s documentation says which protection to pass in
  sandbox mode (the joined one).
- `crates/cleaner/tests/never.rs` — the property test also runs with `Reach::SandboxFolder` over a random tree whose
  sandbox folder is one random folder of it: nothing outside that folder is ever moved or deleted.
- `docs/architecture/removal.md` — "Reach and the sandbox": the third form.

## Produces

The reach and the joined protection that 02 composes.

## Checks

- Negative: `a_folder_without_a_marker_is_not_a_sandbox`, `a_folder_through_a_link_is_not_a_sandbox`,
  `the_real_home_is_not_a_sandbox` and `a_hard_blocked_folder_is_not_a_sandbox` (a real list that makes them whole),
  `an_item_outside_the_folder_is_not_admitted` (a sibling with a shared name prefix included),
  `the_folder_itself_is_not_admitted`, `an_item_is_refused_once_the_marker_is_gone`, `a_tool_is_never_admitted`;
  in `protection.rs`: `a_joined_list_refuses_what_either_refuses` (a property test over both lists).
- Positive: `an_item_inside_the_folder_is_admitted`, `a_marker_above_a_whole_home_still_opens_the_folder` (the
  sandbox's own home whole in the joined list, the marker above it).
- `cargo nextest run -p fresher-cleaner -p fresher-core`; `tests/reach.rs` and `tests/deletion_point.rs` unchanged and
  passing.

## Conclusions

`SandboxFolder` in the cleaner grants the reach of a sandbox; `fresher-core` joins two protection lists, and the one marker
reader lives in `fresher-macos` (`sandbox_marker::holds_marker`, the strictest of the two old copies, with its own tests).
Review findings: a sandbox named through a link with a trailing slash is refused, the joined list keeps its carve-outs per
list, and the guard handles unplaced paths.
