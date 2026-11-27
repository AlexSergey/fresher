# 043/03 — High and Largest on for 0.2.0

**Status:** 🔨 in progress · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/tauri (flags only), sandbox,
fixtures, e2e, docs

`flags.toml` turns `high` to `offered` and `largest` to `shown`; what said "High is withheld" says when it is; the High
parts of plan 042's walk become test cases by hand; the user walks them. The design:
[`../00-design.md`](../00-design.md), section 8; agent's call 6.

## Steps

| #   | Step                                                   | Status      | Shared files |
| --- | ------------------------------------------------------ | ----------- | ------------ |
| 01  | [Turn them on](./01-turn-on.md)                        | ✔️ done | `flags.toml`, `crates/sandbox/src/expected.rs`, the `EXPECTED.md` files, `docs/restrictions/deletion-safety.md`, `docs/architecture/README.md`, `docs/changelog.md` |
| 02  | [The test cases](./02-test-cases.md)                   | ✔️ done | `docs/test-cases/{high-and-largest.md,README.md}` |
| 03  | [The user's walk and the conclusions](./03-walk.md)    | ✅ approved | `docs/plans/043-build-flags/`, `docs/plans/README.md`, `docs/test-cases/README.md` |

01, then 02, then 03.

## Conclusions

To be written when step 03 is done.
