# 007/03 — Autotests

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, fixtures, repo

Every case file becomes a test that goes through the app's own sandbox composition: generate, scan, check every
negative and positive, clean the positives, check what was left, compare the golden report. Then the first two case
files. The design: [`../00-design.md`](../00-design.md), section 5.

After plan 01 and 02/02; in parallel with 02/03 and 02/04.

## Steps

| #   | Step                                          | Status   | Shared files                                                                   |
| --- | --------------------------------------------- | -------- | ------------------------------------------------------------------------------ |
| 01  | [The harness](./01-harness.md)                | ✅ approved | `app/tauri/Cargo.toml` (dev-dependencies), `Cargo.lock`, `package.json`        |
| 02  | [The first case files](./02-first-case-files.md) | ✅ approved | `fixtures/homes/trial*.toml` (kept), `docs/testing.md`                      |

In order.

## Conclusions

**Built:** the autotest of every registered case file and the first two case files, `trial` and `cleanup-guards`, with
golden reports.

**Measured:** `npm run sandbox:test` 11 passed; `fresher-app` 80 after 03/01. Breaking an expectation in either file made
the autotest fail, as intended.

**Left:** the known issue that the scan offers an item inside the Trash; Documents positives wait for plan 006.

**Agent's calls taken during the build (to check):**

- The sandbox avoids its own home's places that prompt (faithful to the real app before plan 006 grants), so the Documents case became a negative and the one-by-one positive moved to `Library/Caches`. Costs, if wrong: Documents positives wait for plan 006.
- The scan offering an item inside `~/.Trash` (then left out as protected) is recorded as a known issue, not fixed here. Costs, if wrong: one confusing finding stays until the product question is decided.
