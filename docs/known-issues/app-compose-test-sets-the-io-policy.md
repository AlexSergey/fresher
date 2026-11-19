# A compose test sets the real I/O policy of the test process

**Status:** open · **Updated:** 2026-11-19

## Symptom

`the_normal_access_keeps_the_default_root_out_of_every_guarded_place` (and the other tests of `app/tauri/src/compose.rs`
that build the normal parts with `MACOS_POLICY`) applies the app's real process I/O policy (never download cloud files,
leave access times) to the test binary. Every later test in the same process runs with that policy, whatever it
expects.

## Cause

`normal(…, &MACOS_POLICY)` calls the policy's functions, which change a setting of the whole process. nextest runs each
test in a process of its own, so today no other test sees it; `cargo test`, which runs the tests of a binary in one
process, would.

## What was tried

Nothing yet: the test needs the normal parts (the real `MacAccess` on a temporary home), and `normal` takes the policy
as a parameter already.

## Options left

- Pass a no-op `IoPolicy` (as the compose tests of a failing policy do) where the test only checks the access.
- Keep `MACOS_POLICY` for the one test that checks the policy is applied, and mark it as needing a process of its own.
