# 021/01/03 — The harness finds the projects; the docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** testing, docs

`run_module` runs a module "as the engine would", but it finds no projects: a module that reads
`cx.system().projects()` sees `None` in its fixture tests. With the finder in the core (01/02) the harness does what
the engine does. The deferred item is closed and the architecture docs describe the projects' facts. The design:
[`../00-design.md`](../00-design.md), sections 2.1 and 2.3.

## Files

- `crates/testing/src/run_module.rs` —
  - the interest set gains `ProjectFinder::interests()` under `ModuleId::system()`, after the module's own;
  - a match the system owns goes to a `ProjectFinder` (`add(local, &matched)`), never to the module and never into
    `matched_paths` (whatever `ports.order` says: the finder takes it as it comes);
  - after the walk and the held matches, before `finish`: `system.set_projects(finder.finish(walk))`, `walk`
    `Some(&walk)` (the `NaiveWalk` of the run) when the walk returned a summary, `None` when it was cancelled;
  - `run_module`'s doc: "…, finds the projects of the tree as the engine does (`ProjectFinder`), sends each match…".
- `crates/testing/src/run_module.rs`'s tests (or the crate's harness tests where `run_module` is tested) —
  `mod negative` `the_module_never_receives_the_project_finder_s_matches` (a module with one interest of its own, a
  tree with a project: the module's matches and `ModuleRun`'s matched paths hold only its own); `mod positive`
  `a_module_sees_the_projects_with_their_facts_in_finish` (a Node project with dated folders: its `node_modules`
  artifact with sizes, `last_activity` `Known` at the dated time).
- `docs/architecture/modules.md` — the `projects()` row: "the projects the walk found: root, kinds, artifact folders
  and Rust caches with their sizes and last change, and the last activity"; the paragraph "The projects are known in
  `finish`…" rewritten: the last activity is the newest folder time outside the artifact folders and `.git` (and the
  artifact folders and `.git` of the projects inside), or the newest folder time of `.git/refs/heads`; `Known`,
  `AtLeast` (a folder unread) or `Unknown`; status changes and files' times are never used, and why (folders move on
  every new file, rename and atomic save; a migration moves every status change); a project whose artifact was removed
  looks active from that day. `run_module` finds the projects as the engine does.
- `docs/architecture/README.md` — the paragraph on the system's facts: "the engine adds its own interests for the
  projects (`ProjectFinder::interests()`, in the core, so the module harness finds the same projects) under
  `ModuleId::system()` and, after the walk, asks the walk what it saw of them (`DiskWalk::subtrees`) and sets the
  projects before any `finish`".
- `docs/deferred/project-last-activity.md` removed; its row in `docs/deferred/README.md` removed. Every link to it
  (`grep -rn project-last-activity docs`: plan 009's `readme.md`, `00-design.md` and `05-projects/readme.md`, and
  `docs/architecture/modules.md`) keeps its text and points to this plan's design (`../021-stale-projects/00-design.md`
  from plan 009's files) instead; this plan's own `readme.md` and `00-design.md` keep the item's name as plain text
  ("the deferred item *A project's last activity, precisely*, closed by 01/03"), with no link.

## Produces

`run_module` that hands a module the projects, for 02/02 and 02/03.

## Checks

- `cargo nextest run -p fresher-testing`; the modules' fixture tests unchanged (`cargo nextest run -p
  fresher-module-jetbrains -p fresher-module-temp -p fresher-module-dev-caches -p fresher-module-app-leftovers -p
  fresher-module-downloads`: their goldens do not move, as the finder's matches never reach them).
- `npm run lint` (Markdown and links: no link to the removed deferred item is left); clippy; fmt.

## Conclusions

`run_module` now carries `ProjectFinder::interests()` under `ModuleId::system()`, feeds the finder every system match (never the module, never `matched`, whatever the order), and sets the projects after the walk and the held matches with `Some(&walk)` or `None` when cancelled. Architecture docs describe the projects' facts and the last-activity rule; the deferred item is removed and its links repointed.
