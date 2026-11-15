# 001/04/01 — Rust unit and integration tests

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** workspace

## Files

- `.config/nextest.toml` — `default` profile; `ci` profile (`fail-fast = false`, `retries = 0`, JUnit report to
  `target/nextest/ci/junit.xml`), slow-test warning at 10 s.
- `crates/testing` — `FakeClock` (an injected `Clock` trait lives in `fresher-core`), the `Trash` and `CommandRunner`
  traits' fakes (the traits themselves in `fresher-core`), each with its own unit tests in `mod negative` /
  `mod positive`.
- One example unit test per core crate in the `mod tests { mod negative {…} mod positive {…} }` shape, and one
  integration test in `crates/core/tests/` as the pattern; one proptest example (`ByteSize` addition never overflows
  silently).
- Root scripts: `test:rust` (`cargo nextest run --workspace --locked`), `test` (Rust + UI).

## Checks

- `npm run test:rust` passes; `cargo nextest run -p fresher-core` runs one crate.
