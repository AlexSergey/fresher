# 004/08/04 — Docs and version

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** docs, app, repo

## Files

- `docs/architecture/README.md` — the layers drawing and the crates table: `fresher-scanner` (the walk, the tree,
  the matcher, the cache codec, the rescan; depends on the core), `fresher-macos` (the adapters and the only `unsafe`;
  depends on the core); the ports table complete; "Data": the scan cache as built.
- `docs/architecture/modules.md` — what a module sees of the walk: matches of its interests, `Match::marker`, the
  three sizes; that its matches come in no fixed order (`finish` must not depend on it); that a disabled module's
  matches are dropped by the engine.
- `docs/testing.md` — the scanner's tests against the reference walk, the large trees, the benches and the
  deterministic bars; the journal's fake.
- `docs/quality.md` — the `unsafe` rule as built (03/01), `npm run bench`.
- `docs/hacks/README.md` — checked: "No disk walk" and "CACHEDIR.TAG read by the rules" are gone; "Places that prompt
  are avoided until plan 006" is there.
- `docs/known-issues/` — anything this plan lives with (a flaky journal test, a bar missed by a little), one file each.
- `docs/ideas/architecture/scanning.md`, `docs/ideas/architecture/snapshots-and-watch-mode.md` — a line each saying
  which proposed decisions plan 004 took, and the FSEvents question answered.
- `docs/plans/README.md` — the row of 004: status and version.
- `Cargo.toml` (`[workspace.package] version`), `app/tauri/tauri.conf.json`, `package.json`, `app/ui/package.json` —
  version `0.3.0` (plan 003 may have moved them to 0.2.0 first; the higher wins).
- `docs/plans/004-fast-scan/readme.md` and each plan's `readme.md` — the conclusions: what was built, the measurements
  (from 03), the deviations from the design (each also an agent's call), the checks by hand for the user.

## Produces

The plan, done and described, ready for the user to look at and for the squash-merge.

## Checks

- `npm run pre-push` passes on the branch's last commit; coverage ≥ 90 % in every crate the plan touched.
- `rg -n "NoWalk|holds_valid_tag" crates app` finds nothing.
- The links in every changed doc resolve (the repository's link check, if one exists; otherwise by hand).
- `cargo clean` at the end (the bench trees under `target/` go with it).

## Conclusions

- **Docs**: `architecture/README.md` (the layers drawing with the OS ports and the cache, the crates and ports tables
  as built, "Data": the scan in memory and the queries next to the cache, rescan and restore), `architecture/modules.md`
  ("What a module sees of the walk"), `testing.md` (the core e2e of the scan, the reference walk and its random trees,
  the fakes of the scan's ports, the bench groups, the footprint test), `quality.md` (the benches by hand, never in a
  hook), the two ideas files (what plan 004 took, the FSEvents question answered), the priority list (004 in
  progress). `CLAUDE.md` and `quality.md` already state the `unsafe` rule as 03/01 built it, and agree.
- **Checked, no change needed**: `docs/hacks/README.md` holds no "No disk walk" and no "CACHEDIR.TAG read by the
  rules"; "Places that prompt are avoided until plan 006" is there. `docs/known-issues/` gains nothing: no flaky test
  was seen, and the one bar missed (the tree heap on a tree of many matched files) is a decision of 08/03, not a defect.
- **Version 0.3.0**: `Cargo.toml` (`[workspace.package]`), `app/tauri/tauri.conf.json`, `package.json`,
  `app/ui/package.json`; `Cargo.lock` and `package-lock.json` regenerated (their version lines only). Plan 003 moves
  `main` to 0.2.0 first; the merge takes 0.3.0.
- **Deviation**: `rg -n "NoWalk|holds_valid_tag" crates app` still finds test code only: two test stubs named `NoWalk`
  (`app/tauri/src/testing.rs`, `crates/e2e/tests/demo.rs`: a walk that reads nothing, for the engine's tests) and the
  reference walk's own tag check (`crates/testing/src/naive_walk.rs`). The rules' read of the tag and the app's stand-in
  walk are gone; the names are left as they are (a rename is not this step's work).
- **Left**: `cargo clean` after the squash-merge of the plan, as `docs/workflow.md` says (the main session's).
