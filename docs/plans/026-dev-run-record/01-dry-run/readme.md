# 026/01 — The dry run

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, store, cleaner, app, app/tauri, app/ui, docs

A development build on the real Mac checks every item of a confirmed cleanup as a release build does and acts on
none: the cleaner's new reach `Reach::DryRun` admits the real disk, `act_on` and put back stop before any act, the
item ends `SkipReason::DryRun`, and the composition gives the cleaner a Trash that refuses. The design:
[`../00-design.md`](../00-design.md), section 1 (agent's calls 1 to 4).

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                                                                                                  |
| --- | ------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01  | [The dry-run reach and its outcome](./01-reach.md)            | ✔️ done     | `crates/core/src/cleanup/outcome.rs`, `crates/core/locales/en.json`, `crates/store/src/journal/records.rs`, `crates/cleaner/src/{reach,pipeline,put_back,lib,test_support}.rs`, `crates/cleaner/tests/{reach,never}.rs`, `app/ui/src/ipc/bindings.generated.ts` |
| 02  | [The development composition and the docs](./02-composition.md) | ✔️ done     | `app/tauri/src/{compose,dry_run,lib,testing,commands}.rs`, `docs/architecture/{README,removal}.md`, `docs/restrictions/deletion-safety.md` |

In order: 02 makes the token 01 defines.

## Conclusions

Not written yet.
