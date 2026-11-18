# 019/04/01 — The docs

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/modules.md` — `only_bundle_ids` (01/01), `place_seen` (01/02), `find_apps` after the search
  check (01/03), `for_app_named`, the group Apps and the choices (01/04), checked; the module `app-leftovers` as the
  example of a module that decides per app from every leftover of its id, with a proof, the places seen and Spotlight
  last; the checklist gains "a module with choices gives each its two texts".
- `docs/architecture/README.md` — the crate list with `fresher-module-app-leftovers`; the system's facts with
  Spotlight's reach; the IPC's `ModuleInfo.choices`; the UI's selection `choose`.
- `docs/testing.md` — `[[case.elsewhere]]` and `spotlight = "off"`, the two case files (03/01), the shared harness
  and `app_leftovers.rs` (03/02), checked; the rule that a leftover's entries state `age` and `changed`, in the fixture
  homes' section.
- `docs/categories/app-leftovers-*.md` — as built (rows, texts, the decision); the catalogue's rows; the danger table's
  examples of `docs/categories/README.md` gain "a removed app's caches" (Safe) and "a removed app's support folder"
  (High) if they fit.
- `docs/categories/jetbrains-uninstalled.md` — the Spotlight reason (01/03), checked.
- `docs/ideas/uninstalled-app-leftovers.md` — its line on plan 019: what was built; what stays an idea (group
  containers, launch agents, folders named after the app's name, a per-app choice, a later proof). `docs/ideas/README.md`:
  its row.
- `docs/deferred/` — the removed app's name (01/04) and the shared harness (03/02) removed, the sandbox world updated
  (03/01), checked; `docs/known-issues/` — the Spotlight issue closed (01/03), checked.
- `docs/hacks/README.md` — "Places that prompt are avoided until plan 006": what it costs `app-leftovers` (no
  container is offered until then).
- `docs/plans/019-uninstalled-app-leftovers/` — every step's conclusions written as it was built (checked here); the
  design's tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); every link of the changed docs resolves.

## Conclusions

Built (the docs commit): checked what the steps already wrote and filled the gaps.

- `docs/architecture/README.md`: `fresher-module-app-leftovers` in the crate list, `ModuleInfo.choices` in the IPC, the
  selection action `choose` in the UI; Spotlight's reach was already in the `InstalledApps` row (01/03).
- `docs/architecture/modules.md`: the module `app-leftovers` as the example of a module that decides per app, with a
  proof, the places seen and Spotlight last; the checklist's "a module with choices gives each its two texts".
  `only_bundle_ids`, `place_seen`, `search_complete`, `for_app_named`, the group Apps and the choices were checked, as
  01/01 to 01/04 wrote them.
- `docs/testing.md`: the rule that a leftover's entries state `age` and `changed`; `[[case.elsewhere]]`,
  `spotlight = "off"`, the case files, the shared harness and `app_leftovers.rs` were checked.
- `docs/categories/README.md`: "a removed app's caches" (Safe) and "a removed app's support folder" (High) in the
  danger table; the three category docs and `jetbrains-uninstalled.md` were checked.
- `docs/ideas/uninstalled-app-leftovers.md` and its row: what is built, what stays an idea (group containers, launch
  agents, folders named after the app's name, a choice per app, a later proof).
- `docs/hacks/README.md`: what the avoided places cost `app-leftovers` (no container is offered until plan 006).
- Known issues, new: a link at a module's place reads `Absent`; a mounted unindexed volume turns off removed-app
  detection; Spotlight misses a nested app and a privacy-list folder; the reference walk and the scanner compare avoided
  paths differently. Deferred, new: containers of an app's extensions, due at plan 006. The Spotlight known issue and the
  sandbox world's deferred item were checked.
- The links to the removed deferred items (the removed app's name, the shared e2e harness) are gone from this plan, from
  plans 016 and 018 (now "done by plan 019/NN/NN"); every step's and sub-plan's Conclusions are written, and 04/02's
  checklist gained the bench retake and a backup disk.

Checks: prettier on the changed docs; every relative link of the changed files and of this plan resolves.
