# 001/02/01 — Rust lints

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace

## Files

- `Cargo.toml` — `[workspace.lints.rust]` and `[workspace.lints.clippy]` as listed in
  [`../00-design.md`](../00-design.md), "The quality layers"; every crate gets `[lints] workspace = true`.
- `clippy.toml` — `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-print-in-tests`,
  `avoid-breaking-exported-api = false`, `disallowed-methods` for the deleting `std::fs` / `tokio::fs` functions with
  the reason "only fresher-cleaner deletes (docs/restrictions/deletion-safety.md)".
- `deny.toml` — `[advisories]`, `[licenses]` (the allowlist, `confidence-threshold = 0.93`, private crates ignored),
  `[bans]` (`multiple-versions = "warn"`, `wildcards = "deny"`, `allow-wildcard-paths = true`, HTTP clients denied),
  `[sources]` (unknown registries and git denied).
- Root scripts: `lint:rust` (`cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D
warnings`), `check:deny`, `check:machete`.

## Checks

- A probe file with `unwrap()`, `println!`, a bare `#[allow]` and a `std::fs::remove_file` call in `fresher-scanner`
  fails `lint:rust` with four errors; the probe is removed.
- `cargo deny check` and `cargo machete` pass.
