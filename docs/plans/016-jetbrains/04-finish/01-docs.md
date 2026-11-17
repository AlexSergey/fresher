# 016/04/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## Files

- `docs/architecture/modules.md` — `modules/jetbrains` named as the first real module beside `demo`; "code, not rules"
  with its reason (the answer needs installed and running apps across places); left-outs in practice.
- `docs/architecture/README.md` — the crate in the crates list; the sandbox's `world.toml`.
- `docs/testing.md` — "Sandboxes": the `jetbrains.toml` case file in the list; `crates/e2e/tests/jetbrains.rs` in the
  core e2e; the UI e2e's JetBrains scenarios.
- `docs/categories/` — checked against what was built (levels, texts, the decision table).
- `docs/deferred/sandbox-world.md` and its row — already marked "partly taken" when this plan was written; checked
  against what was built, and "Until then, sandbox mode keeps … running apps … real" corrected.
- `docs/deferred/dev-debug-panel.md` — its "due" re-read: the world is only partly built; unchanged unless it says
  otherwise.
- `docs/ideas/developer-environment-cleaner.md`, `uninstalled-app-leftovers.md`, `application-footprint.md` — each
  links here for the part this plan took and says what stays an idea; a new idea file and row for the layout before
  2020 (decision 5), if the run by hand finds such folders.
- `docs/dependencies.md` — the new crate, if the file lists workspace crates.

## Produces

The docs as built.

## Checks

- Every link resolves; `npm run lint` passes; no product or project is named but the tools the app cleans (the
  JetBrains IDEs and Toolbox) — clean room; there is no "Other" anywhere.

## Conclusions

Docs as built: `docs/architecture/modules.md` and `README.md` (the first real module, code not rules because the
answer needs the installed and running apps across three places; left-outs in practice; `world.toml`), `docs/testing.md`
(case files, core e2e, UI e2e), `docs/categories/` (four JetBrains categories), `docs/deferred/sandbox-world.md`
(corrected; the clock and apps taken), `docs/known-issues/` and `docs/deferred/` entries added during the build, and
the ideas that link here. The dev debug panel's deferral is unchanged: the world is only partly built.
