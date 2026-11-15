# 001/01/02 — Rust toolchain and workspace

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace

## Files

- `rust-toolchain.toml` — `channel = "1.99.1"`, components `clippy`, `rustfmt`, `llvm-tools-preview`, profile
  `minimal`, targets `aarch64-apple-darwin`, `x86_64-apple-darwin` (universal build later).
- `rustfmt.toml` — `edition = "2024"`, `max_width = 120`, `newline_style = "Unix"`, `use_field_init_shorthand`,
  `use_try_shorthand`.
- `Cargo.toml` (workspace) — `resolver = "3"`; members `crates/*`, `app/tauri`; `[workspace.package]` (`edition =
"2024"`, `rust-version = "1.99"`, `publish = false`, `license` per open question 3); `[workspace.dependencies]` for
  shared crates (`thiserror`, `tracing`, `tempfile`, `proptest`); the lints are added in plan 02;
  `[profile.release]` `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`;
  `[profile.dev.package."*"] opt-level = 2` (fast scans in debug builds).
- Crates `crates/{core,scanner,rules,store,cleaner,testing,e2e}` — each a `Cargo.toml` taking `edition`,
  `rust-version`, `license`, `publish` from the workspace, and a `lib.rs` with a crate doc comment and nothing else
  (`e2e` has only `tests/`). Dependencies between them as in
  [`../../../architecture/README.md`](../../../architecture/README.md).

## Done when

- `cargo build --workspace --locked` and `cargo fmt --all --check` pass; `Cargo.lock` is committed.
