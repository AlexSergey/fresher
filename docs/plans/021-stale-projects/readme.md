# 021 — Stale projects

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Drafted on 2026-11-19 as one file and approved by the user the same day with its ten agent's calls, which became
decisions (below). The design, its decisions and the agent's calls made while writing it from the code and from a
read-only look at this Mac are in [`00-design.md`](./00-design.md); the facts marked "to verify" are checked by hand
in 04/02.

## Goal

Old projects keep gigabytes of what their tools make again, and an active Rust project's `target/` grows by gigabytes
of pure cache in days. The module `stale-projects` (`fresher-module-stale-projects`, group **Development**) offers the
artifact folders of projects nothing has changed in for a year (a setting, 30 to 3650 days), one finding per folder
named by its project, in four categories: packages (`node_modules`, Low), build output (`target`, `build`, `.gradle`,
`.build`, `.dart_tool`, Low), Python environments (`.venv`, `venv`, Medium), and, for any Rust project, the build
cache unused for 7 days (`target/*/incremental`, `target/llvm-cov-target`, Low). Source, the project folder, `.git` and
framework output (`dist`, `.next` and the like) are never offered; projects in `~/Library`, the apps folders, app
bundles, hidden folders of the home and the home itself are not projects here. The idea:
[stale project cleanup](../../ideas/stale-project-cleanup.md).

The core first gains what the code lacks: a question to the walked tree before the commit (`DiskWalk::subtrees`: a
folder's sizes and its newest folder time, skipping named folders), the projects' artifacts and Rust caches with their
sizes, and a precise last activity — the newest folder time outside the artifact folders and `.git`, or the last
commit as the walk saw `.git/refs/heads` (a deferred item, closed here). The project finder moves from the engine to
the core so that the module harness finds the same projects. Nothing is claimed whole: no folder becomes opaque to the
storage map. Tests at every level, as plans 016 to 020 did: unit, reference, fixture, sandbox, core e2e, UI e2e, runs
by hand.

Branch: `feat/021-stale-projects` (from `main` f19e872), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md), [012](../012-storage-map/readme.md),
[015](../015-module-groups/readme.md) and [020](../020-downloads/readme.md) (`DaySetting`), all in `main`.

## Plans

| #   | Plan                                                      | Status      | Depends on | What                                                                                                    |
| --- | --------------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------------------- |
| 01  | [Projects with their facts](./01-project-facts/readme.md) | ✔️ done | —          | the walked tree's subtrees; the finder in the core with sizes, caches and last activity; the engine and the harness; the deferred item closed |
| 02  | [The module](./02-module/readme.md)                       | ✔️ done | 01         | the crate, its table and texts, the decision, fixtures, registration                                    |
| 03  | [End to end](./03-end-to-end/readme.md)                   | ✔️ done | 01, 02     | the case file, the core e2e flows, the UI                                                               |
| 04  | [Finish](./04-finish/readme.md)                           | ✅ approved | all        | the docs, the runs by hand (this repository's own `target/` included), the conclusions                  |

### Order and parallel work

```
01/01 ── 01/02 ──┬── 01/03 ─────────────┬── 02/03 ── 03/01, 03/02, 03/03 ── 04/01 ── 04/02
                 └── 02/01 ── 02/02 ────┘
```

- **Wave 1**: 01/01 (`crates/core/src/walk.rs`, `crates/scanner/src/{subtrees,subtrees_tests,scanner,lib}.rs`,
  `crates/scanner/{tests/reference.rs,benches/scan.rs}`, `crates/testing/src/naive_walk.rs`, the five other
  `impl DiskWalk` sites).
- **Wave 2**: 01/02 (`crates/core/src/system/`, `crates/core/src/lib.rs`, `crates/engine/src/{session,projects,lib,
  testing}.rs`, `crates/e2e/tests/projects.rs`).
- **Wave 3, in parallel**: 01/03 (`crates/testing/src/run_module.rs`, `docs/architecture/`, `docs/deferred/`) and
  02/01 (`modules/stale-projects/` only, beside the catalogue, the root `Cargo.toml` and `docs/dependencies.md`).
- **Wave 4**: 02/02 (`modules/stale-projects/src/` only), then 02/03 after 01/03 and 02/02 (fixtures, golden,
  registration).
- **Wave 5, in parallel**: 03/01 (the case file, `app/tauri/tests/sandboxes.rs`), 03/02 (`crates/e2e`) and 03/03
  (`app/ui`).
- **Wave 6**: 04.

`crates/core/src/lib.rs` takes lines from 01/01 and 01/02, which run one after the other. 01/03 and 02/01 run in
parallel and share no file.

Which files each plan touches:

| Plan | Owns                                                                                                      | Shared files it touches                                                                                                                                                                 |
| ---- | --------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/scanner/src/subtrees{,_tests}.rs`, `crates/core/src/system/finder{,_tests}.rs`                          | `crates/core/src/{walk,lib}.rs`, `crates/core/src/system/{mod,projects,facts}.rs`, `crates/scanner/src/{scanner,lib}.rs`, `crates/scanner/{tests/reference.rs,benches/scan.rs}`, `crates/engine/src/{session,projects,lib,testing}.rs`, `crates/testing/src/{naive_walk,run_module}.rs`, `crates/e2e/tests/{projects,demo}.rs`, `app/tauri/src/testing.rs`, `docs/architecture/`, `docs/deferred/` |
| 02   | `modules/stale-projects/`, `fixtures/homes/stale-projects-*.toml`, `fixtures/golden/stale-projects.json`, `docs/categories/stale-projects-*.md` | root `Cargo.toml` (the member), `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `docs/categories/README.md`, `docs/dependencies.md` |
| 03   | `fixtures/sandboxes/stale-projects.toml`, `fixtures/golden/{sandbox-stale-projects,stale-projects-flow}.json`, `fixtures/homes/stale-projects-flow.toml`, `crates/e2e/tests/{stale_projects.rs,stale_projects_home/}`, `app/ui/src/test/stale-projects-scan.ts` | `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 04   | —                                                                                                         | `docs/` (architecture, testing, categories, ideas, plans README)                                                                                                                        |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/stale-projects` itself at or above 90 %.
- The scanner's `subtrees` equal the reference walk's on generated trees (the reference property test); the scan bench
  and the scan of the developer's home move by no more than 5 % (rule 4), written in 01/01's and 04/02's conclusions.
- The deferred item `project-last-activity` is removed; `Project.last_activity` is the newest folder time outside the
  artifact folders and `.git`, or the newest `.git/refs/heads` folder time.
- No interest of this plan claims anything whole: a folder named `build`, `target` or `incremental` anywhere opens in
  the storage map's browse view (the core e2e's map flow).
- The module's contract test passes with `Catalogue::Required`: four category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing but a listed artifact or cache of a project's kind can be offered: source, the project folder, `.git`,
  `dist`, `.next`, `.turbo`, a `node_modules` without a project, a link, the home's own `node_modules`, a project in
  `~/Library`, an app bundle or a hidden home folder are never claimed (the fixtures' and the case file's negatives).
- `npm run sandbox:test` passes for `stale-projects.toml`; it fails when a case's expectation (a danger, a left-out's
  items) is changed on purpose (done once by hand per kind, written in 03/01).
- `npm run test:e2e` passes with the ten core flows and the UI scenarios.
- `npm run dev:sandbox -- stale-projects`: every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, scan only): every offered item checked by hand, this repository's `target/`
  caches among them (04/02).

## Ideas and deferred items this touches

- [Stale project cleanup](../../ideas/stale-project-cleanup.md): becomes this plan; framework output (`dist` and the
  like) is out for good (decision 1); `Pods` and other kinds stay ideas.
- [Unused toolchain versions](../../ideas/unused-toolchain-versions.md): plan 024 reads this plan's project list and
  its `LastChange`; `nvm` is off by default there (decision 2).
- [Developer environment cleaner](../../ideas/developer-environment-cleaner.md),
  [best practices](../../ideas/best-practices.md) (#21, #22): touched, not changed.
- *A project's last activity, precisely*: closed by 01/03.
- [Why an item matched](../../deferred/match-explanations.md): stays deferred (the reason carries the date and the
  command).

## Risks

- **The worst false positive is a project the user still works on, judged stale**: its folders must not have changed
  for a year and its branches not moved; only folders its tools make again are offered (Low, or Medium for an
  environment), and they go to the Trash. Editors that save in place are checked in 04/02.
- **Cleaning a project makes it look active**: removing an artifact moves the project folder's time, so the artifacts
  left wait for the days again. The category docs say so.
- **The Rust cache of a project built every day is never 7 days old** (decision 3.4): for this repository, built daily
  and covered on every push, the cache category may offer little; 04/02 measures what it offers and which part of
  `target/` grows, for a later decision.
- **The post-walk queries** (01/01) run on every scan: bounded by the projects' own folders (artifacts skipped); the
  bench and the home scan say.
- **The finder's move** (01/02, 01/03) touches every scan: the engine's and the e2e's project tests keep their meaning,
  plus the new ones.

## Decisions

The user's, of 2026-11-19 (translated):

1. "`dist` and the rest must definitely not be touched": `dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` and other
   framework output are never offered by this module, not even later.
2. "Maybe think about cleaning nvm too, but it must definitely not be on by default": recorded in
   [unused toolchain versions](../../ideas/unused-toolchain-versions.md) for plan 024; not part of this plan.
3. "The rest seems fine; work autonomously, take the recommended decisions": the draft's ten agent's calls are
   approved; they are listed as decisions 3.1 to 3.10 in [`00-design.md`](./00-design.md).

## Conclusions

Not written yet.
