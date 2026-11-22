# 031/03/01 — The docs

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten). The design:
[`../00-design.md`](../00-design.md); the facts it found are in its section "Facts found on the developer's Mac".

## Files

- `docs/deferred/xcode-simulator-devices.md` (written with the design commit) — checked against what was built; its
  facts and "what doing it needs" kept; a line added if the build learnt anything about the devices. Its row in
  `docs/deferred/README.md` exists already.
- `docs/deferred/sandbox-world.md` — one sentence under "What is postponed": plan 031 dropped its tool category, so
  plan 032 is the first module that needs `[[world.tool]]`.
- `docs/architecture/README.md` — the crate list with `fresher-module-xcode`.
- `docs/architecture/modules.md` — the module `xcode` among the examples: a table of places across four categories
  with a shape per row (a version-named folder, a fixed folder, a dated archive), a sign held inside each, kept while
  one of two apps runs, two removals (Trash and `EmptyFolder`) in one module; simulator devices deferred and why (one
  sentence and the link).
- `docs/categories/xcode-*.md` (four) and `docs/categories/README.md` — as built (rows, texts, the decision), checked.
- `docs/categories/dev-caches-xcode-derived-data.md` — one sentence: Xcode's other data is in the `xcode` module's
  four categories.
- `docs/goal.md` — section 4 unchanged (it names archives and old simulators as goals; the simulators stay a goal,
  deferred); checked only.
- `docs/testing.md` — the two case files (02/01), `xcode.rs` (02/02) and the UI scan (02/03), checked.
- `docs/manual-testing.md` — 03/02's runs by hand as items of the consolidated checklist (a section "Xcode" in the
  sandbox part and one in the developer's Mac part, as `1.13` and `2.12` for app caches), and the plan's row in the
  table of plans awaiting a walk.
- `docs/ideas/developer-environment-cleaner.md` — its line on plan 031: device support, previews, simulator caches and
  archives built; simulator devices and runtimes deferred (the link). `docs/ideas/README.md`: its row if it lists the
  state.
- `docs/plans/README.md` — the row of 031: status 🔨 in progress, modules `modules, testing, sandbox, e2e, app, app/ui,
  docs` (no `core`: nothing in the core changed).
- `docs/plans/031-xcode/` — every step's conclusions written as it was built (checked here); the design's tables
  brought up to what was built.

## Checks

- `npm run lint:code` (Markdown and links) or the project's doc lint; `npx prettier --check` on the changed docs; every
  relative link of the changed files resolves.
- No other disk-cleaner product or reference project named; no user name or private file name in any doc.

## Conclusions

Written 2026-11-22. The docs of the plan's change were brought up as built: the crate list and `modules.md` (the example
of one table of places across four categories), the sentence in `sandbox-world.md` (plan 032 is the first to need
`[[world.tool]]`), the sentence in the derived data category, the ideas file, the plan's row (🔨 in progress), and the
manual checklist as items `1.18` and `2.16` (plans 033, 034 and 037 took 1.15 to 1.17 and 2.13 to 2.15) with a row in
the table of plans awaiting a walk. `testing.md` says `all` writes every path but leaves `xcode.toml`'s positives out
as `app-open` when `xcode-open.toml`'s Xcode joins the world (walk `xcode` by name); the known issue
`repo-sandbox-all-worlds-interact.md` names the xcode pair and `dev-caches-xcode-open`. The deferred note on simulator
devices, the category docs, `goal.md` and the plan folder were checked and needed no change.
