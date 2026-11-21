# 030/03/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/README.md` — the crate list with `fresher-module-app-caches`.
- `docs/architecture/modules.md` — the module `app-caches` in the list of examples: a module that shares another
  module's interest (`Caches/*` shaped to bundle ids, the same as `app-leftovers`') and claims the complementary half
  (the folders an installed app owns), the owner found by `BundleId::names` over the installed apps and their helpers,
  the decision in `finish` (the running apps only when needed), the first real module that empties a folder
  (`Removal::EmptyFolder`); the web-engine, browser and launcher caches deferred and why.
- `docs/categories/app-caches-library-caches.md` — as built (the table's rows, the texts, the decision); its catalogue
  row; the danger table's examples (an installed app's cache under Low, agent's call 4).
- `docs/categories/app-leftovers-caches.md` — under "How the app decides", one sentence: an installed app's caches are
  offered by "Caches of your apps" instead, while the app is closed (the split of the design's section 3).
- `docs/testing.md` — the two case files and the checklist's sentence (02/01), `app_caches.rs` (02/02) and the UI scan
  (02/03), checked.
- `docs/manual-testing.md` — 03/02's runs by hand as items of the consolidated checklist, with links to the step.
- `docs/ideas/installed-app-caches.md` — its line on plan 030: what was built; what is deferred (the deferred item's
  list); the open questions answered. `docs/ideas/README.md`: its row.
- `docs/deferred/installed-app-caches-more-places.md` — checked against what was built (its "what doing it needs").
- `docs/plans/030-installed-app-caches/` — every step's conclusions written as it was built (checked here); the
  design's tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.

## Conclusions

Built: the architecture, categories, testing and manual-testing docs, the idea and the deferred item as built (19 files). The final review folded this step's check in: nine minor findings, all fixed in the first commit of step 02 (`fix(modules): plan 030 — final review minors`).
