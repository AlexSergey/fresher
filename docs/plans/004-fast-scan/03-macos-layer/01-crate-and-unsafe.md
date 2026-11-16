# 004/03/01 — The crate and the unsafe boundary

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos, workspace, scripts

## Files

- Root `Cargo.toml` — `[workspace.lints.rust] unsafe_code = "deny"` (was `forbid`, which no crate can relax); a
  comment points to this step. `[workspace.dependencies]` pins nothing new (`libc` is already there).
- Every crate root except `crates/macos` (`crates/*/src/lib.rs`, `modules/*/src/lib.rs`, `app/tauri/src/main.rs` and
  `lib.rs`) — `#![forbid(unsafe_code)]` as its first attribute. The core crates stay exactly as strict as before.
- `scripts/check-crate-deps.ts` and its test — the table: `fresher-macos` may depend on `fresher-core`; a new check:
  every workspace crate's root file except `fresher-macos`'s holds `#![forbid(unsafe_code)]` (read from the target's
  `src_path` in `cargo metadata`). Negative tests first: a root without the line fails; `fresher-macos` without it
  passes.
- `crates/macos/Cargo.toml` — `fresher-macos`, "The macOS adapters of Fresher: the only crate with unsafe code";
  `[lints] workspace = true`; dependencies `fresher-core`, `libc`, `thiserror`, `tracing`; a feature `fixtures`
  (off by default) for test helpers that write (see below).
- `crates/macos/src/lib.rs` — `#![cfg(target_os = "macos")]` contents, so the crate is empty elsewhere;
  `#[expect(unsafe_code, reason = "every FFI call of the app lives here, each block with its SAFETY comment")] mod ffi;`
  and nothing else may use `unsafe` (a test greps the crate: `unsafe` appears only under `src/ffi/`).
- `crates/macos/src/ffi/mod.rs`, `ffi/iopolicy.rs` — `setiopolicy_np` (from `libc`, or declared here if `libc` lacks
  it) for `IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES` and, if 01 confirmed it, `IOPOL_TYPE_VFS_ATIME_UPDATES`.
- `crates/macos/src/io_policy.rs` — `pub fn never_download_cloud_files() -> Result<(), IoPolicyError>` and
  `pub fn leave_access_times() -> Result<(), IoPolicyError>`: idempotent, process-wide, logged once.
- `crates/macos/src/ffi/rusage.rs`, `crates/macos/src/process.rs` — `pub fn peak_memory() -> ByteSize` from
  `getrusage(RUSAGE_SELF)` (`ru_maxrss` is in bytes on macOS, as 01 checked), for the benches and later diagnostics.
- `crates/macos/src/ffi/clone.rs`, `crates/macos/src/fixtures.rs` — behind the `fixtures` feature only:
  `clone_file(from, to)` (`clonefile`), so tests can build trees with APFS clones. It creates a file, never removes
  one.
- `CLAUDE.md` ("Rust Rules") and `docs/quality.md` (the lints table) — "`unsafe_code = "deny"` in the workspace;
  `#![forbid(unsafe_code)]` in every crate but `fresher-macos`, checked by the crate guard; in `fresher-macos` only the
  `ffi` module".
- `docs/architecture/README.md` — `fresher-macos` "may depend on: core".

## Produces

The crate the next steps fill; `io_policy` for the composition root (05/06); `peak_memory` for the benches (05/01);
`clone_file` for the scanner's clone tests (04/03, 05/04).

## Checks

- Negative: `a_crate_root_without_forbid_unsafe_fails_the_guard`, `no_unsafe_outside_the_ffi_module` (a test in
  `fresher-macos` that reads its own sources), `setting_the_policy_twice_is_not_an_error`.
- Positive: `the_policy_is_set_for_the_process` (reads it back with `getiopolicy_np`), `peak_memory_grows_after_a_large_allocation`,
  `a_cloned_file_has_a_clone_id` (`fixtures` feature, in a temporary folder).
- `npm run check:crate-deps`; `cargo clippy --workspace --all-targets -- -D warnings`; Clippy's
  `undocumented_unsafe_blocks` and `multiple_unsafe_ops_per_block` report nothing.
- `cargo nextest run -p fresher-macos`; `cargo llvm-cov` ≥ 90 % of the crate's lines.

## Conclusions

- `unsafe_code` is `deny` in the workspace; every other crate root (`crates/*/src/lib.rs`, `modules/demo/src/lib.rs`,
  `app/tauri/src/lib.rs` and `main.rs`) starts with `#![forbid(unsafe_code)]` after its doc comment.
- The crate guard reads each package's targets from `cargo metadata` and checks the crate roots only (kinds `lib`,
  `rlib`, `cdylib`, `staticlib`, `dylib`, `proc-macro`, `bin`; tests, benches, examples and build scripts are not
  roots). The line must stand on its own line, so a comment that quotes it does not count. `fresher-macos` may depend
  on `fresher-core`.
- `fresher-macos` holds `ffi` (`iopolicy`, `rusage`, `clone`), `io_policy`, `process` (`peak_memory`) and `fixtures`.
  `libc` lacks `setiopolicy_np`, `getiopolicy_np`, the `IOPOL_*` values and `CLONE_NOFOLLOW`, so `ffi` declares them
  from the system headers. Both IO policies exist and are set (01 confirmed access-time updates too).
- Unspecified, taken the simplest way: `IoPolicyError` wraps the `io::Error` of the refusal; the policy is set on
  every call (the system call is idempotent) and logged once per policy; `peak_memory` answers zero, with a warning,
  if `getrusage` fails. `fixtures` is compiled for the crate's own tests as well (`cfg(any(test, feature =
  "fixtures"))`), so its unit tests need no self dev-dependency (which the crate guard would refuse).
- The guard on `unsafe` in the crate's own sources looks for the forms `unsafe {`, `unsafe fn`, `unsafe impl`,
  `unsafe extern` and `unsafe(` outside `src/ffi/`.
- Checks: 6 tests in `fresher-macos`, 18 in the guard's test; Clippy clean (with and without `fixtures`);
  `cargo llvm-cov` on the crate: 96.4 % of lines.

