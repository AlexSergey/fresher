# 008/08/01 — The docs

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** docs

## Files

- `docs/restrictions/deletion-safety.md` — every "planned" and "review until then" under **Enforced by** replaced by
  what holds the rule now: the `ConfirmedPlan` and the echo (explicit confirmation), the cleaner's API and
  `PermanentConsent` (Trash first), the guard's checks 5–7 and 11 (re-check before delete), checks 2, 3 and 5
  (links and protected paths), the protection list's `whole` folders (the user's own folders), check 10 (items in
  use), each with its test file.
- `docs/architecture/README.md` — the layers diagram and the crate table (the cleaner built), the ports table
  (`Trash` by `MacTrash`, `OpenFiles`, `Cleaner`, `CleanupJournal`, `Volumes::available`), "Data" (the journal, if
  05/03 has not already written it), "UI" (the Cleanups screen, the cleanup state), a link to `removal.md`.
- `docs/architecture/removal.md` — completed against the code as merged (written in 04/08).
- `docs/architecture/modules.md` — the checklist: a path removal names its subject; a tool removal never forces.
- `docs/testing.md` — the cleaner's tests (fixtures, sandbox, property and compile-fail tests), the journal's, the
  cleanup e2e and its golden file, the UI flows.
- `docs/dependencies.md` — checked: `objc2`, `objc2-foundation`, `trybuild` listed with their versions and licences.
- `docs/hacks/README.md` — "Review cannot remove yet" gone (07/02); "Development builds clean only in a sandbox"
  present (06/04).
- `docs/known-issues/` — what this plan leaves open, each in its own file (for example the residual window of the
  Trash call, if 01 measured it; processes of other users).
- `docs/goal.md` — section 3 links the cleaner's design; nothing else changes.

## Produces

Docs that describe the code as it is.

## Checks

- Every path, type and test file named in the changed docs exists (a `grep` over the names).
- No "planned" is left in `deletion-safety.md`.
- `npm run check:repo` passes.

## Conclusions

Done on 2026-11-16; every doc the step names was checked against the code as merged (008/01–07 with the hardening).

- **`restrictions/deletion-safety.md`**: no "planned" is left; each rule names what holds it now (the `ConfirmedPlan`
  and the echo, `PermanentConsent`, the guard's checks 5–7 and 11, checks 2, 3, 5 and 8, the protection list's
  `whole` folders, check 10) with its test files.
- **`architecture/README.md`**: the layers diagram with the cleaner, its ports and the journal; the crate table (the
  cleaner, the macOS layer, the store and the engine as built); the ports table (`Trash` by `MacTrash`, `OpenFiles`,
  `CommandRunner` by `ProcessCommandRunner`, `Cleaner`, `CleanupJournal`, `Volumes::available`); the sandbox reach;
  "UI" with the cleanup's screens and the quit question. "Data" already held the journal (05/03).
- **`architecture/removal.md`**: matched the code; it now says the trial folders must lie right next to the marker,
  that a mounted volume's root is a whole folder where a marker never counts, that the scan roots are set in
  `settings.json` until plan 005, and that `never.proptest-regressions` is written by the first failure (none
  exists).
- **`architecture/modules.md`**: the checklist's box for a path removal naming its subject and a tool never forcing.
- **`testing.md`**: compile-fail and race tests in the kinds of tests; the cleanup fakes and `Swapper`; the cleaner's,
  the plan types' and the journal's tests; the journal in the footprint test; the UI's cleanup mock, scripts and flows.
- **`dependencies.md`**: every version and licence matches the lock files; "Used by" corrected from `cargo metadata`
  (`thiserror` also in `macos` and `scanner`, `tempfile` in `macos` and `scanner` tests, `proptest` in `macos`
  tests); `Cargo.lock` holds 510 packages (the `trial` module).
- **`hacks/`**: "Review cannot remove yet" was already gone (07/02); the sandbox entry says how to set the roots.
- **`known-issues/`**: two new files, the in-use check seeing only the user's own processes
  (`cleaner-other-users-processes.md`) and the Trash call taking a path, with the permanent delete's last window
  (`cleaner-trash-call-by-path.md`).
- **`goal.md`**: section 3 links the cleaner's design and `removal.md`.

**Found while checking**: the plan builder refuses a tool argument that forces in any form, `f` among short flags
(`-rf`) included (`crates/engine/src/cleanup/plan.rs`), while the cleaner's own re-check (`guard/mod.rs`,
`verify_tool`) refuses only `-f`, `--force` and `--force=…`. Nothing reaches the cleaner without the builder, so no
item can slip through today; the docs describe each check as it is, and the gap is left to 03 to fix or record.
