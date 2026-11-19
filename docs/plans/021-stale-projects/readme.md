# 021 — Stale projects

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, engine, scanner, modules,
testing, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The idea [Stale project cleanup](../../ideas/stale-project-cleanup.md): old projects keep gigabytes of what their tools
can make again — `node_modules`, Cargo's `target/`, a Python `.venv`, Gradle's `build/` — and nobody remembers they
are there. Plan [009](../009-system-knowledge/readme.md) already finds projects in the shared walk by their marker files
and lists their artifact folders (`Project`, `ProjectKind`); what is missing is a module that says which of them can go
and why, and a precise "last activity" (deferred item [project-last-activity](../../deferred/project-last-activity.md),
due now).

A second need came from building Fresher itself: an **active** Rust project's `target/` grew to 20–50 GB in a few days.
Most of it is build cache that Cargo makes again on the next build. That part is offered even for an active project.

## Goals

1. A module `stale-projects` (`modules/stale-projects`, crate `fresher-module-stale-projects`, group Development) that
   offers the artifact folders of projects untouched for a set number of days, one finding per artifact folder, named
   by its project, with the project's last activity and the command that makes it again.
2. A project's last activity is the newest change of its own files, outside its artifact folders and outside `.git`,
   or its last commit, whichever is newer — never the folder's own time alone.
3. For any Rust project, active or not, the parts of `target/` that are pure cache are offered on their own: the
   incremental compilation folders and the coverage build.
4. The source code, the project folder, `.git` and anything not listed as an artifact of the project's kind are never
   offered.

## Non-goals

- Running a tool (`cargo clean`, `npm`, `git`): the module reads, the cleaner moves folders to the Trash.
- Deleting single stale files inside `target/deps` (what some tools do by age): a later idea, it needs Cargo's
  fingerprints to be safe.
- Projects inside `~/Library`, app bundles or `/Applications`: an app's own `node_modules` is part of the app (the
  [best practices](../../ideas/best-practices.md), #22).
- Unused toolchain versions: plan 024, which reads this plan's project list.

## Design

### Categories

| Category                              | What                                                                                    | Danger | Offered when                                      |
| ------------------------------------- | --------------------------------------------------------------------------------------- | ------ | ------------------------------------------------- |
| `stale-projects.dependencies`         | `node_modules` of a Node project                                                        | Low    | the project is stale                              |
| `stale-projects.build-output`         | `target/` (Rust, Maven), `build/` and `.gradle` (Gradle), `.build` (Swift), `.dart_tool` and `build` (Dart) | Low | the project is stale                          |
| `stale-projects.python-environments`  | `.venv`, `venv` of a Python project                                                     | Medium | the project is stale                              |
| `stale-projects.rust-build-cache`     | `target/<profile>/incremental` and `target/llvm-cov-target` of any Rust project         | Low    | unchanged for 7 days, whatever the project's age |

- **Stale**: the project's last activity is older than `stale-projects.unused-for-days` (shared `DaySetting`,
  30–3650, 365 by default). A project whose last activity cannot be told is not stale.
- **Low**: a reinstall or a rebuild makes it again; the reason names the command (`npm install`, `cargo build`,
  `gradle build`, `swift build`, `dart pub get`) and that it costs time and the network. **Medium** for a virtual
  environment: it may hold packages installed by hand that no lock file lists.
- A Rust `target/` of a stale project is offered whole, in `build-output`; its incremental folders are then not offered
  a second time (one finding per path).
- The cache category answers the case of an active project: `incremental` is only ever read by the compiler to go
  faster, and the coverage build is a separate full build; both are made again by the next build.

### Last activity

- The walk already knows every folder's modification time and folds files under 1 MiB into their folder. The engine
  computes, per project, the newest time of the project's folders and files, skipping its artifact folders, `.git` and
  nested projects' artifact folders, from the walked tree before the modules' `finish`
  (`Project::last_activity`, made precise).
- A git repository adds its last commit time: the modification time of `.git/logs/HEAD` (written by every commit,
  checkout, merge and pull), read as a file fact, never by running `git`. A `.git` file (a worktree or a submodule)
  points elsewhere: its pointed folder's `logs/HEAD` is read when it is under a walked root, otherwise not used.
- Last activity = the newer of the two. Unknown when neither is known.

### Which projects

- Projects as plan 009 finds them, in the walked roots, minus those under `~/Library`, `/Applications`, an app bundle or
  a hidden folder of the home (`~/.cargo`, `~/.npm`: tools' own stores, covered by plan 018).
- A project nested in another one (a monorepo's package) is stale only when its outermost project is stale too; its
  artifacts are offered inside the same project name, so a monorepo reads as one line per artifact folder.
- A project with an artifact folder that is a link is left out (`artifact-is-a-link`); a link is never followed.

### What this asks of the core

- The engine's project collector gets the newest time per project from the walked tree (the deferred query), and
  reads `.git/logs/HEAD`'s time through the walk (an interest on `.git/logs/HEAD` under `ModuleId::system()`).
- `ProjectKind` gains nothing new; the artifact folders are the ones plan 009 lists, plus the Rust cache paths, which
  are this module's own interests.
- The sandbox and the fixtures state project trees with ages and a `.git/logs/HEAD`; no new port.

### In the UI

Nothing new: the module's page lists its categories; a finding's title is the project's name and the artifact
(`my-app · node_modules`), its detail line the last activity ("last changed 14 months ago") and the restore command.

## Plans (after approval)

| #   | Plan                | What                                                                                              |
| --- | ------------------- | ------------------------------------------------------------------------------------------------- |
| 01  | Last activity       | the newest time per project from the walked tree, `.git/logs/HEAD`, the deferred item closed      |
| 02  | The module          | the crate, the four categories, the day setting, the decision, texts, category docs, fixtures     |
| 03  | End to end          | case files (stale, active with a big `target/`, monorepo, a link), core e2e, UI e2e and baselines |
| 04  | Finish              | the run by hand on the user's Mac (this repository's own `target/`), conclusions                  |

Depends on plans 008 (removal), 009 (projects), 015 (groups), 020 (`DaySetting` in the core).

## Open questions

None for the user; the calls below are waiting for a confirm or an overrule.

## Agent's calls (to check)

1. **One module, four categories by what the folder is** (dependencies, build output, environments, Rust cache), not
   one category per language: the danger and the restore cost follow what the folder is.
2. **365 days by default**, 30–3650, one setting for the module.
3. **Python environments are Medium**, every other artifact Low.
4. **The Rust build cache is offered for active projects too**, after 7 days unchanged, because it answers the real
   case of this repository; the rest of an active project's `target/` is not offered.
5. **The last commit is `.git/logs/HEAD`'s modification time**, read as a file fact; `git` is never run.
6. **Uncommitted changes do not matter**: only regenerable folders are offered, never source; no git status is read.
7. **Projects under `~/Library`, `/Applications`, app bundles and hidden home folders are not projects here.**
8. **A nested project follows its outermost project's staleness.**
9. **`dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` are never offered** — the user's decision (below).
10. **No running-process check**: a stale project has no build running; the Rust cache's 7 days unchanged and the
    cleaner's in-use check cover an active one.

## Decisions

The user's, of 2026-11-19 (translated):

1. "`dist` and the rest must definitely not be touched": `dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` and other
   framework output are never offered by this module, not even later.
2. "Maybe think about cleaning nvm too, but it must definitely not be on by default": recorded in
   [unused toolchain versions](../../ideas/unused-toolchain-versions.md) for plan 024; not part of this plan.
3. "The rest seems fine; work autonomously, take the recommended decisions": the agent's calls above are approved.

## Deferred items this takes

[A project's last activity, precisely](../../deferred/project-last-activity.md) (closed by plan 01). Possibly
[why an item matched](../../deferred/match-explanations.md) — due at "the first rule-based module that needs more than
its reason": the last activity and the restore command fit in the reason, so it stays deferred.

## Ideas this touches

[Stale project cleanup](../../ideas/stale-project-cleanup.md) (becomes this plan),
[unused toolchain versions](../../ideas/unused-toolchain-versions.md) (plan 024 reads the project list),
[developer environment cleaner](../../ideas/developer-environment-cleaner.md),
[best practices](../../ideas/best-practices.md) (#21, #22).
