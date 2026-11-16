# 009/04/02 — Ownership in the cleaner

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** cleaner, testing, engine, app, e2e

## Files

- `crates/cleaner/src/lib.rs` — `CleanerPorts.running: Arc<dyn RunningApps>`.
- `crates/cleaner/src/pipeline.rs` — the running set taken with the open set, at the same moments; a failure refuses
  the cleanup.
- `crates/cleaner/src/guard/in_use.rs` — check 10 reads both: `InUse { app }`.
- `app/tauri/src/compose.rs`, the e2e and the cleaner's tests — the ports.

## Checks

- Negative: an item named after a running app's id, or inside such a folder, is skipped as in use; an app started
  between two snapshots is seen at the next; a failed listing refuses the cleanup.
- Positive: an item of an app that is not running is removed; the other checks are unchanged.

## Conclusions

- Built as written. `CleanerPorts.running`; the pipeline's snapshot is `InUse { open, running }`, both taken at the same
  moments (before the first item, again after two seconds on the clock) and both needed: a failure of either skips
  every item from then on as `CheckFailed`, in the rehearsal too.
- Check 10 asks the open files first, then the running apps, for either form of the path (physical, lexical); a hit
  is `InUse { app }` with the app's name, the same reason and text as an open file.
- The plan builder's rehearsal reads the same check, so an item of an app running while the user reviews is left
  out of the confirmation with "the app is using it".
- The app gives one `MacRunningApps` to the system ports and to the cleaner. The e2e, the app's tests and the
  cleaner's own use `FakeRunningApps`.
- Checks: an item named after a running app's id is skipped, the next ones are removed; an app started after the first
  snapshot is seen two seconds later; a failed listing acts on nothing; an app that is not running changes nothing.
