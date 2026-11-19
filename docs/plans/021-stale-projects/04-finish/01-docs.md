# 021/04/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/README.md` — the crate list with `fresher-module-stale-projects`; the `DiskWalk` port's
  `subtrees` (01/01) and the system's facts paragraph (01/03), checked.
- `docs/architecture/modules.md` — the projects' facts (01/03), checked; the module `stale-projects` as the example of
  a module that decides from `cx.system().projects()` in `finish`, offers only the folders it matched with `Item`
  interests, and takes the sizes from the projects rather than claiming anything whole.
- `docs/testing.md` — the case file (03/01) and `stale_projects.rs` (03/02), checked; a line under the fixture homes:
  a project that must look stale dates every folder.
- `docs/categories/stale-projects-*.md` — as built (the table's rows, the texts, the decision); the catalogue's rows;
  the danger table's examples of `docs/categories/README.md` gain "an old project's `node_modules`" (Low) and "an old
  project's Python environment" (Medium) if they fit.
- `docs/ideas/stale-project-cleanup.md` — its line on plan 021: what was built; what stays an idea (`Pods` and other
  kinds; framework output never, by the user's decision of 2026-11-19; single stale files in `target/deps`; the
  editors that save in place, if 04/02 finds any). `docs/ideas/README.md`: its row.
  `docs/ideas/unused-toolchain-versions.md` — a line: the project list with `LastChange` exists for plan 024.
- `docs/deferred/` — the last-activity item removed (01/03), checked.
- `docs/plans/021-stale-projects/` — every step's conclusions written as it was built (checked here); the design's
  tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.

## Conclusions

Checked against the code, not only the plan: the crate list and the module's paragraph in `docs/architecture/`
(the `DiskWalk` row and the facts paragraph were written by 01/01 and 01/03), the case file and the golden files in
`docs/testing.md` (a line under the fixture homes: a project that must look stale dates every folder), the four
category files and their rows, the danger table's examples (an old project's `node_modules` Low, its Python
environment Medium), the idea (built; framework output never, the user's decision of 2026-11-19; `Pods` and other
kinds, single files in `target/deps` and the editors that save in place stay ideas), `unused-toolchain-versions.md`,
the plan's statuses, and the design's agent's calls 19 and 20.

Two known risks are recorded in `docs/known-issues/` with what was tried: a file edited in place moves no folder's
time, and a project cleaned once looks active for the set days (core e2e flow 10 pins it). The deferred item on the
last activity was already removed by 01/03; no link to it is left.

Parked nits fixed: three long lines of the design, the comments of `fixtures/sandboxes/stale-projects.toml` and
`fixtures/homes/stale-projects-flow.toml`.

`docs/manual-testing.md` (the consolidated checklist of runs by hand) is not on this branch: step 04/02's items must be
added to it after the merge into `main`; no copy was made here.

