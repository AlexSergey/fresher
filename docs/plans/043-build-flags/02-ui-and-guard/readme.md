# 043/02 — The UI, the mock and the guard

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui, e2e, scripts, ci, docs

Largest follows its own flag; the e2e mock takes flags as the app does and every spec names the ones it needs; the
guard `check:flags` and the release text's "This build" section. The design: [`../00-design.md`](../00-design.md),
sections 5 to 7; agent's calls 4, 5 and 7.

## Steps

| #   | Step                                                        | Status      | Shared files |
| --- | ----------------------------------------------------------- | ----------- | ------------ |
| 01  | [The e2e mock takes flags](./01-mock-flags.md)              | ✔️ done     | `app/ui/e2e/core-mock.ts`, `app/ui/e2e/support/withheld.ts`, the e2e specs, `docs/testing.md` |
| 02  | [Largest from its own flag](./02-largest-flag.md)           | ✔️ done     | `app/ui/src/screens/{frame-sidebar,app-frame}.tsx`, `app/ui/src/state/navigation.ts`, their tests, the e2e specs of the sidebar, `docs/architecture/README.md` |
| 03  | [The guard and the release text](./03-guard-and-notes.md)   | ✔️ done     | `scripts/{flags,flags.test,release-version,release-version.test}.ts`, `package.json`, `.github/workflows/ci.yml`, `docs/{quality,release}.md` |

01, then 02: the specs name both flags first, so the UI's change moves no baseline; 03 apart from both.

## Conclusions

To be written when step 03 is done.
