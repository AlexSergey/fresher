# 031 — Xcode and simulators

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the one-file draft and a read of the code on `main` (97d9b314). Approved by the user's
delegation of 2026-11-22: build autonomously and take the recommended decisions; the user checks everything later. The
design, with its facts and agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Every cleaner aimed at developers handles the whole of Xcode's data; Fresher offers only DerivedData (plan
[018](../018-developer-caches/readme.md)). Archives, device support files and the simulators' caches are usually the
largest part, and [`goal.md`](../../goal.md), section 4, names them.

## Goal

A new module `xcode` (`fresher-module-xcode`, group **Development**) offers four kinds of Xcode's data, each its own
category with its own danger level, one finding per folder:

| Category                 | Level | What                                                                        | Removal                       |
| ------------------------ | ----- | --------------------------------------------------------------------------- | ----------------------------- |
| `xcode.device-support`   | Low   | debug symbols per OS version of a connected device; copied again on connect | Trash, one finding a version  |
| `xcode.previews`         | Safe  | the previews' build cache                                                   | folder emptied                |
| `xcode.simulator-caches` | Safe  | the simulators' shared caches                                               | folder emptied                |
| `xcode.archives`         | High  | archived builds with the symbols of shipped versions                        | Trash, one finding an archive |

Nothing is offered while Xcode or the Simulator runs (left out `app-open`); a folder whose name or sign the module does
not recognise is not claimed. Nothing in the core, the scanner, the cleaner, the engine or the sandbox crate changes.

**Changed from the draft** (agent's call 1): the unavailable simulator devices are **deferred**
([`xcode-simulator-devices`](../../deferred/xcode-simulator-devices.md)). On the developer's Mac, `xcrun simctl list`
started Xcode's first-launch installer (`xcodebuild -runFirstLaunch`) because the simulator framework was older than
the one Xcode expects: a scan must never do that, and an out-of-date framework's answer may misreport a device's
availability. With them goes the sandbox's tool world, which plan [032](../032-docker/readme.md) builds (its plan 01).

Branch: `feat/031-xcode`, from `main` (97d9b314), rebased onto plans 033, 034 and 037 before it is built (the design's
section "Rebase"), squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)). Depends on plans
[008](../008-safe-removal/readme.md), [009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md),
[015](../015-module-groups/readme.md), [018](../018-developer-caches/readme.md) and
[030](../030-installed-app-caches/readme.md) (the module's shape).

## Plans

| #   | Plan                                    | Status      | Depends on | What                                                                                   |
| --- | --------------------------------------- | ----------- | ---------- | -------------------------------------------------------------------------------------- |
| 01  | [The module](./01-module/readme.md)     | 🔨 awaiting the walk | —          | the crate, the table, the names, the decision, the texts, the category files, fixtures |
| 02  | [End to end](./02-end-to-end/readme.md) | 🔨 awaiting the walk | 01         | the case files and the registration, the core e2e flows, the UI                        |
| 03  | [Finish](./03-finish/readme.md)         | 🔨 awaiting the walk | all        | the docs, the runs by hand, the conclusions, the user's review list                    |

### Order and parallel work

```
01/01 ──┬── 01/02 ── 01/03 ──┬── 02/01 ──┬── 03/01 ── 03/02
        │                    └── 02/02 ──┤
        └── 02/03 ───────────────────────┘
```

- **Wave 1**: 01/01 alone (the crate, its table, texts and category docs).
- **Wave 2, in parallel**: 01/02 (`modules/xcode/src/` only) and 02/03 (`app/ui` only, once 01/01's texts exist).
- **Wave 3**: 01/03 (fixtures, golden, the contract over the fixture runs).
- **Wave 4, in parallel**: 02/01 (case files, goldens, the app's registration) and 02/02 (`crates/e2e`).
- **Wave 5**: 03.

| Plan | Owns                                                                                                                                                                                                               | Shared files it touches                                                                                                                                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `modules/xcode/`, `fixtures/homes/xcode-*.toml`, `fixtures/golden/xcode.json`, `docs/categories/xcode-*.md`                                                                                                        | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                                                                                                                                                      |
| 02   | `fixtures/sandboxes/xcode{,-open}.toml`, `fixtures/golden/sandbox-xcode*.json`, `fixtures/homes/xcode-flow.toml`, `fixtures/golden/xcode-flow.json`, `crates/e2e/tests/{xcode.rs,xcode_home/}`, `app/ui/src/test/xcode-scan.ts` | `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs`, `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 03   | `docs/deferred/xcode-simulator-devices.md` (checked)                                                                                                                                                                       | `docs/` (architecture, testing, manual-testing, categories, deferred, ideas, plans README)                                                                                                                             |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
or a rebase and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/xcode` itself at or above 90 %.
- `git diff $(git merge-base HEAD main) -- crates/core crates/scanner crates/cleaner crates/engine crates/sandbox` is
  empty: no core change.
- The module's contract test passes with `Catalogue::Required`: four category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a `Trash` or `EmptyFolder` of a matched path.
- Nothing but a recognised folder is offered: a name of the wrong shape, a folder without its sign, Xcode's settings
  (`UserData/KeyBindings`), an archive outside a dated folder, a link, a file and an empty folder are never claimed
  (the fixtures' and the case files' negatives); the core e2e shows no folder offered by both `xcode` and `dev-caches`.
- `npm run sandbox:test` passes for both case files and every other one; it fails when a case's expectation (a danger,
  a left-out's items) is changed on purpose (done once by hand per kind, written in 02/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- xcode` and `-- xcode-open`: every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, a dry run): what the module offers checked by hand against
  `~/Library/Developer` (03/02).

## Ideas and deferred items this touches

- [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md): the Xcode line becomes this plan for
  device support, previews, simulator caches and archives; simulator devices and runtimes stay open (deferred).
- [Xcode's simulator devices](../../deferred/xcode-simulator-devices.md): a new deferred item, written with this plan.
- [The world outside the disk in a sandbox](../../deferred/sandbox-world.md): the tools part stays deferred, now for
  plan 032.

## Risks

- **A path or a sign differs on a newer Xcode** (this Mac has no device support, previews or archives): not claimed,
  the safe direction; checked in 03/02.
- **An archive still needed is removed**: High, never selected in bulk, the reason says what it holds, the Trash.
- **Xcode opened between the scan and the cleanup**: only the cleaner's check 10 catches it at the act.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Simulator devices deferred, and the tool world with them (changes the draft).
2. Archives High, never pre-selected, with no code (nothing is pre-selected; only Safe in bulk).
3. Code, not rules.
4. The two newest device support versions of a platform are offered with their own reason.
5. One list of apps (Xcode, the Simulator) keeps every row.
6. The previews' and the simulators' caches are emptied, not moved.
7. The signs and the name shapes, mostly to verify.
8. Every finding in Xcode's line of the map.
9. No scan bench, no performance run.
10. The module after the last feature module in the registration.
11. No new dependency, no new UI component.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken. The question
  the draft left to the facts (listing devices through `simctl`) is settled by agent's call 1, to check.
- No version bump: 0.1.0.

## Conclusions

**Built.** A new module `xcode` (`fresher-module-xcode`, group Development) with four categories, each one finding per
folder: `xcode.device-support` Low (Trash, one finding per OS version of a connected device), `xcode.previews` Safe and
`xcode.simulator-caches` Safe (both emptied), `xcode.archives` High (Trash, one per archive, never pre-selected). The
module is code with a table of places (seven rows) and the decision rows of the design; it runs no command. Nothing is
offered while Xcode or the Simulator runs; a folder whose name or sign is not recognised is not claimed; the two newest
versions of a platform carry their own reason; every finding is counted in Xcode's line of the map. Two case files, nine
core flows, the UI path, the docs and the review list.

**Numbers.** `npm run pre-push` on the branch's last commit: passed. Rust: 4057 tests run, all passed (5 skipped),
workspace line coverage about 97.6 % (floor 90 %); `modules/xcode` 85 tests, 99.29 % of its lines (419 of 422).
`fresher-e2e --test xcode`: 9 passed. `npm run sandbox:test`: 66 of 66; `npm run sandbox -- all`: 242 cases, the same
five files left out as on `main`. UI: vitest 1160 tests, 98.60 % of lines (97.76 % of statements). Playwright, chromium
and webkit: 593 passed and the known flake `category-details-app-leftovers-chosen` (it failed once in webkit and once
in chromium and passed on the other run, with no change to its baseline). By-hand mutations of 02/01: the archive's
danger lowered and a left-out's items changed both fail the autotest (see 02/01). `EXPECTED.md` of both sandboxes was
read once against the goldens.

**No core change.** `git diff $(git merge-base HEAD main) -- crates/core crates/scanner crates/cleaner crates/engine
crates/sandbox` is empty (checked after the rebase onto `main` with plans 033 and 034). No new dependency, no new UI
component.

**Changes from the design.**

- Simulator devices are deferred, and with them the tool world (agent's call 1; see the review list).
- A file held open inside `CoreSimulator/Caches` skips the whole item (`InUse`), not one entry: the cleaner checks per
  item, as it does for `app-caches`. The step 02/02 said otherwise and was corrected.
- `Verdict::LeftOut` carries its reason and its names so a left-out version still counts towards the two newest;
  `Version` compares by its numbers.
- The Simulator caches' cleanup in the UI mock moves nothing: the folder emptied itself between the scan and the
  cleanup (a file held open would skip the item as in use instead).

**Calls taken while building.**

- The planner wrote the steps from a fresh read of the code, with no separate preflight scan.
- Plan 031 was built in its own worktree with its own cargo target directory; it was rebased onto `main` after plans
  033 and 034 merged, and the shared files (the category and architecture lists, `testing.md`, `manual-testing.md`, the
  UI mock) were resolved by keeping both sides; goldens and `Cargo.lock` were regenerated.
- Held-open files and the combined sandbox: with `npm run sandbox -- all`, Xcode open (from `xcode-open`) joins every
  file's world and leaves the positives of `xcode` out; they are walked by name.

**Awaiting the user's walk.** The sandbox boxes and the dry run on the Mac (03/02 above and
[`manual-testing.md`](../../manual-testing.md) 1.18 and 2.16); the unverified facts and the calls worth a second look are
in [`user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md). The plan becomes done only after that.
