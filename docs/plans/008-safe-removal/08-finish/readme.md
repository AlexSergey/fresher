# 008/08 — Finish

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** docs, app

The docs say what was built, the first real cleanups happen in a sandbox folder with a development build, and the
version moves. The design: [`../00-design.md`](../00-design.md).

Depends on every other plan of 008.

## Steps

| #   | Step                                                       | Status  | Shared files                                                                                                                                                 |
| --- | ---------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 01  | [The docs](./01-docs.md)                                   | ✔️ done | `docs/architecture/`, `docs/restrictions/deletion-safety.md`, `docs/testing.md`, `docs/dependencies.md`, `docs/hacks/`, `docs/known-issues/`, `docs/goal.md` |
| 02  | [The sandbox run by hand](./02-sandbox-run.md)             | ✔️ done | `docs/plans/008-safe-removal/` (the results)                                                                                                                 |
| 03  | [Version and conclusions](./03-version-and-conclusions.md) | ✔️ done | the version files, `docs/plans/README.md`, the plan's readmes                                                                                                |

01 and 02 may run in parallel; 03 last.

## Conclusions

Done on 2026-11-16. The docs describe the removal as built (01, with one gap found and fixed on the branch: the
cleaner's re-check now refuses a forced tool argument in every form the plan builder does, `-rf` included). The
sandbox run by hand (02) is prepared and [deferred](../../../deferred/removal-sandbox-run.md) by the user's decision of
2026-11-16 to merge without it. The version is 0.5.0 and the conclusions are written (03).
