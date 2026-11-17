# 012/05 — The sandbox case file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
sandbox, app, fixtures

The storage map's case file under plan [007](../../007-safe-development/readme.md)'s rule: every feature ships
`fixtures/sandboxes/<feature>.toml`, negative cases first, with its golden report. The map is a core feature, like
`cleanup-guards`; its cases need one optional expectation more in 007's format. The design:
[`../00-design.md`](../00-design.md), section 8; the format: 007's step 01/02, "The case format"
(`docs/plans/007-safe-development/01-case-files/02-case-format.md`, in `main` once 007 is merged).

**Built after plan 007 is merged into `main`** and `main` into this branch (decision 8): the files below are 007's.

## Steps

| #   | Step                                                  | Status                                   | Shared files                                                                                |
| --- | ----------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| 01  | [Map expectations](./01-map-expectations.md)          | ✔️ done | `crates/sandbox/src/{case,error,expected}.rs`, `app/tauri/tests/sandboxes.rs`                |
| 02  | [The storage map's case file](./02-case-file.md)      | ✔️ done | `docs/testing.md` ("Sandboxes")                                                             |

In order, after plan 03 and plan 007.

## Conclusions

Plan 05 is built on `feat/012-storage-map` after plan 007 was merged into `main` and `main` into the branch: the map
expectation in the case format (01) and the storage map's case file with its golden report (02). Deviations and the
reasons are in the two steps' conclusions.

- Tests: `npm run sandbox:test` 21/21, stable 20/20; the goldens add up to the fake disk's 1 GiB.
- Concerns left in the harness: the goldens assume an APFS volume (guarded with a message).
