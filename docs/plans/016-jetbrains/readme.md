# 016 — JetBrains IDE leftovers

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, core, sandbox, testing, app, app/ui, docs

Written on 2026-11-17 at the user's request and approved the same day by delegation: the user asked the agent to take
the recommended answer to every open question and record it ([`00-design.md`](./00-design.md), "Decisions"). The
agent's calls wait for the user's check; the IDEs' folder layout and bundle ids are checked by hand on a real Mac.

## Goal

The case that started the project: the settings with their plugins, the caches and the logs that older versions of
JetBrains IDEs (WebStorm, IntelliJ IDEA, PyCharm, …) leave in `~/Library` are found and offered, each with its danger
level, its plain-English texts and its IDE's line in the storage map — never the version in use, a newer one, or
anything of an IDE that is open; those are reported as left out with the reason. The data of an IDE that is no longer
installed is offered in its own category once unchanged for 30 days, a number set in Settings (the user's decision
of 2026-11-17). The first real feature module,
`modules/jetbrains` (`fresher-module-jetbrains`), in the group Development. Its tests at every level are part of the
plan, as the user asked: unit, fixture, sandbox (with the installed and running apps the sandbox now declares), core
e2e, UI e2e, and runs by hand.

Branch: `feat/016-jetbrains`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
**Built after plans [011](../011-diagnostics/readme.md) and [015](../015-module-groups/readme.md) are merged into
`main`**, on this branch made again from `main` then (decision 9). Depends on plans
[007](../007-safe-development/readme.md), [009](../009-system-knowledge/readme.md) and
[012](../012-storage-map/readme.md) (merged).

## Plans

| #   | Plan                                                    | Status                                  | Depends on    | What                                                                                       |
| --- | ------------------------------------------------------- | --------------------------------------- | ------------- | ------------------------------------------------------------------------------------------ |
| 01  | [The sandbox world: apps](./01-sandbox-world/readme.md) | ✔️ done | —             | installed apps as bundles, running apps, `world.toml`, the left-out expectation            |
| 02  | [The module](./02-module/readme.md)                     | ✔️ done | —             | the crate, products and names, the decision, the data of a removed IDE (04, approved by the user), fixture tests, categories, registration |
| 03  | [End to end](./03-end-to-end/readme.md)                 | ✔️ done | 01, 02        | the case file, the core e2e flows, the UI e2e scenarios                                    |
| 04  | [Finish](./04-finish/readme.md)                         | 🔨 in progress | all           | the docs, the runs by hand, the conclusions                                                |

### Order and parallel work

```
01/01 ── 01/02 ── 01/03 ──────────────────────────────┐
02/01 ──┬── 02/02 ── 02/04 ── 02/03 ──┬───────────────┴── 03/01 ──┐
        │                             └── 03/02 ──────────────────┼── 04/01 ── 04/02
        └── 03/03 ────────────────────────────────────────────────┘
```

- **Wave 1, in parallel**: plan 01 (`crates/sandbox`, `crates/testing`'s tree vocabulary, the harness, sandbox mode)
  and 02/01 (`modules/jetbrains/` and `docs/categories/` only).
- **Wave 2**: 02/02, 02/04 and 02/03 in order (the module's decision, the data of an IDE that is no longer installed,
  then its fixture tests and registration); 03/03 (the UI's mock scan and Playwright: `app/ui` only) may start once
  02/01's texts exist.
- **Wave 3**: 03/01 (needs plan 01 and 02/03) and 03/02 (needs 02/03), in parallel: different files.
- **Wave 4**: 04.

Which files each plan touches:

| Plan | Owns                                                                                                         | Shared files it touches                                                                                                                                                         |
| ---- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/sandbox/src/{apps,world}.rs`, `crates/testing/src/app_bundle.rs`                                     | `crates/sandbox/src/{case,error,expected,generate,layout,lib}.rs`, `crates/testing/src/{lib,tree_description,fixture_tree}.rs`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/sandboxes.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/ui/src/product/build-banner/`, `app/ui/src/locales/en.json`, `docs/testing.md` ("Sandboxes") |
| 02   | `modules/jetbrains/`, `fixtures/homes/jetbrains-*.toml`, `fixtures/golden/jetbrains.json`, `docs/categories/jetbrains-*.md` | root `Cargo.toml` (the member), `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`modules()`), `docs/categories/README.md`, `docs/dependencies.md` (the crate's row, if the file lists workspace crates); 02/04: `crates/core/src/{cancel.rs,system/}`, `crates/system/src/{apps,clock}.rs`, `crates/sandbox/src/{world,generate}.rs`, `crates/testing/src/{system_fakes,tree_description}.rs`, `app/tauri/src/sandbox.rs`, the harness, the sandboxes' goldens |
| 03   | `fixtures/sandboxes/jetbrains.toml`, `fixtures/golden/{sandbox-jetbrains,jetbrains-flow}.json`, `fixtures/homes/jetbrains-flow.toml`, `crates/e2e/tests/jetbrains.rs`, `app/ui/src/test/jetbrains-scan.ts` | `crates/e2e/Cargo.toml`, `app/tauri/tests/sandboxes.rs` (`case_files!`), `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/` |
| 04   | —                                                                                                            | `docs/` (architecture, testing, deferred, ideas, categories, plans README)                                                                                                       |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/jetbrains` itself at or above 90 %.
- The module's contract test passes with `Catalogue::Required`: three category files, the same danger levels, every
  text and every declared left-out reason used by a fixture run.
- `npm run sandbox:test -- jetbrains` passes; it fails when a case's expectation (an item, a danger, a left-out count,
  a map part) is changed on purpose (done once by hand per kind, written in the conclusions).
- `npm run test:e2e` passes with the six core flows and the seven UI scenarios.
- `npm run dev:sandbox -- jetbrains`: every line of `EXPECTED.md` holds by hand; a real open IDE changes nothing in the
  sandbox.
- On the developer's Mac (`npm run dev`, scan only): every offered folder is an older version of an installed IDE that
  is not open; the version in use and an open IDE's folders are left out with their reason; no folder outside
  `~/Library/{Application Support,Caches,Logs}/JetBrains/` is claimed.

## Ideas and deferred items this touches

- The [developer environment cleaner](../../ideas/developer-environment-cleaner.md): its "IDEs: stale versions of
  JetBrains IDE data" becomes this plan.
- The [uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md): an IDE that is no longer installed is
  offered here, in its own category, after the setting's days (decision 10, which replaces decision 3); the idea's
  own module would take every other app's.
- The [application footprint](../036-uninstall/readme.md): the IDE's line in the storage map now holds its
  older versions' data.
- The deferred [sandbox world](../../deferred/sandbox-world.md): its installed and running apps are built here; the
  rest stays deferred (updated in this plan's commit).

## Conclusions

**Merged into `main` on 2026-11-17 at the user's request, before the run by hand** (04/02): the plan stays 🔨 in
progress until the user has run its checklist; the results are written then, on `main`.

What was built: the module `modules/jetbrains` (the first real module, group Development, four categories:
`jetbrains.caches` and `jetbrains.logs` Safe, `jetbrains.settings` and `jetbrains.uninstalled` Medium; eleven left-out
reasons, three of them added by the final review; one setting, the days after which a removed IDE's data is offered); the system reader's Toolbox folders,
links read through, `unread` and the Spotlight search; the sandbox world (`[[case.app]]`, `world.toml` with `now`
and the running apps, `[[case.left-out]]`, the sandbox's clock, `[[dir]] age`) and sandbox mode taking its running
apps from it; three case files; ten core e2e tests; nine UI e2e scenarios.

Changed from the design: the data of a removed IDE (decision 10 replaced decision 3, built in step 02/04); the
decision is made in `on_match`; the Toolbox case and the unknown-app cases moved (one world per case file); the
version in use is not in the IDE's line of the storage map. The agent's calls and the rulings are in
[`00-design.md`](./00-design.md), "Conclusions".

The final review's fix wave (2026-11-17, rulings 29 to 35 of the design): a module page's subtitle is the module's
description, with what ignoring does in the ignore button's tooltip; a partial app list, Spotlight and an unknown
vendor app open each have their own reason; Spotlight is asked for the Finder first (an index that is off keeps
everything) and read NUL-separated; a walk that could not read every folder never offers a removed IDE's data
(`ModuleContext::is_walk_complete`); the sandbox autotests run one at a time with synced fills and stricter case
files and `world.toml`; two known issues recorded (a volume Spotlight does not index, a sandbox scan dated at the
generation). Each of the 44 new JetBrains assertions of the UI e2e was flipped once, then put back: 39 failed; the
other 5 are waits for a state an action brings (the totals loaded, "You ignore this module.", the group's note), which
a flipped wait passes before the state arrives; the assertions after each wait check that state.

Numbers: `modules/jetbrains` 98.0 % of lines, 100 % of functions; module tests 88+; sandbox autotests 33; core e2e 48
in the crate, the JetBrains ones stable over 150 runs; Playwright 295 passed. `npm run pre-push` and the scan's time
are for the run by hand.

Open: everything in [04/02](./04-finish/02-run-and-conclusions.md), the sandbox walk of `EXPECTED.md`, the `mdls` check of
every bundle id and the Toolbox paths on the real Mac, the scan's time against `main`, the "change an expectation on
purpose, it fails" check once per kind, and the parked small items in the design's conclusions.
