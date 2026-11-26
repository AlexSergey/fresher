# A compose test sets the real I/O policy of the test process

**Status:** fixed (the compose tests pass a no-op I/O policy; one keeps the real one and notes it needs a process of its own) · **Updated:** 2026-11-26

## Symptom

`the_normal_access_keeps_the_default_root_out_of_every_guarded_place` (and the other tests of `app/tauri/src/compose.rs`
that build the normal parts with `MACOS_POLICY`) applies the app's real process I/O policy (never download cloud files,
leave access times) to the test binary. Every later test in the same process runs with that policy, whatever it
expects.

## Cause

`normal(…, &MACOS_POLICY)` calls the policy's functions, which change a setting of the whole process. nextest runs each
test in a process of its own, so today no other test sees it; `cargo test`, which runs the tests of a binary in one
process, would.

## Fix

The tests of `app/tauri/src/compose.rs` that do not check the policy pass a no-op `IoPolicy` (`NO_POLICY`), as the
failing-policy tests do. `the_normal_engine_withholds_high` keeps `MACOS_POLICY` and says in a comment that it needs a
process of its own, which nextest gives.
