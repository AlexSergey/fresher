# 032 — Docker

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, engine, testing, sandbox, modules, e2e, app, app/ui, docs

Written on 2026-11-22 from its one-file draft and from a read of the code on main 97d9b314. Approved by the user's
delegation of 2026-11-22: build autonomously, take the recommended decisions and the agent's calls; the user checks
everything later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind." And on this plan (2026-11-22): "Perhaps just describe: your
Docker takes this and this — here is the command to clean it if you need."

[`goal.md`](../../goal.md), section 4, and the README promise Docker; nothing is built. It is the largest single
consumer on many developers' Macs and the most dangerous one: a volume may hold the only copy of a database.

## Goal

1. The user sees what Docker takes, split into build cache, unused images, stopped containers and unused volumes, in
   plain English.
2. For each part the app gives the exact command that frees it, a Copy button, and what running it costs. The app
   never runs it.
3. The disk file card: the largest the file may grow to, what it takes on the disk, what is used inside, and how the
   runtime gives space back.

New in the core: `Insight` (what a module describes and never offers), `FindingSink::describe`, the `insights` of
`ModuleFinished`. New module: `docker` (`fresher-module-docker`, group Development, no category). New in the sandbox:
tool answers (`[[case.tool]]`), other apps' data in the world's access, `written` bytes of a sparse file, insight
expectations; sandbox mode never runs a program but `/usr/bin/open`. New in the UI: the section **Described, not
cleaned** on the scan screen and the insight screen with Copy. The cleaner, the scanner, the store, the rules, the
macOS crates and every other module stay unchanged. No new dependency.

Branch: `feat/032-docker`, from main 97d9b314, rebased over plans 031, 033, 034 and 037 before its squash-merge into
`main` ([`../../workflow.md`](../../workflow.md)). Depends on plans [009](../009-system-knowledge/readme.md),
[011](../011-diagnostics/readme.md), [012](../012-storage-map/readme.md), [015](../015-module-groups/readme.md),
[017](../017-temp-files/readme.md) (stated sockets), [020](../020-downloads/readme.md) (the world's access) and
[027](../027-scan-screen/readme.md) (the scan screen).

## Plans

| #   | Plan                                    | Status      | Depends on  | What                                                                         |
| --- | --------------------------------------- | ----------- | ----------- | ---------------------------------------------------------------------------- |
| 01  | [Insights](./01-insights/readme.md)     | 🔨 awaiting the user's walk | —           | the core type, the sink, the event, the engine, the UI state; the test kit   |
| 02  | [The module](./02-module/readme.md)     | 🔨 awaiting the user's walk | 01          | the crate and its table, the answer, the decision, the insight, fixtures     |
| 03  | [The screen](./03-screen/readme.md)     | 🔨 awaiting the user's walk | 01/01       | the section on the scan screen, the card, the insight screen, Copy           |
| 04  | [The sandbox](./04-sandbox/readme.md)   | 🔨 awaiting the user's walk | 01, 02      | tool answers, other apps' data, `written`, insight expectations, case files, registration |
| 05  | [End to end](./05-end-to-end/readme.md) | 🔨 awaiting the user's walk | 02, 03, 04  | the core e2e, the UI e2e                                                     |
| 06  | [Finish](./06-finish/readme.md)         | 🔨 awaiting the user's walk | all         | the docs, the walks, the conclusions, the review list                        |

### Order and parallel work

```
01/01 ──┬── 01/02 ── 02/01 ── 02/02 ── 02/03 ──┐
        ├── 03/01 ─────────────────────────────┼── 04/03 ──┬── 05/01 ──┬── 06/01
        └── 04/01 ── 04/02 ────────────────────┘           └── 05/02 ──┘
```

- **Wave 1**: 01/01 alone (core, engine, bindings, UI state).
- **Wave 2, in parallel**: 01/02 (`crates/testing`), 03/01 (`app/ui/src` only), 04/01 (`crates/sandbox`,
  `app/tauri/src/sandbox*`, `app/tauri/tests/sandboxes*`, `crates/core/src/app_info.rs`).
- **Wave 3**: 02/01 (`modules/docker`), then 02/02, then 02/03; in parallel, 04/02 after 04/01 (`crates/sandbox`,
  `crates/testing/src/tree_description.rs`, `app/tauri/tests/sandboxes*`).
- **Wave 4**: 04/03 (case files, goldens, the app's registration), after 02/03 and 04/02.
- **Wave 5, in parallel**: 05/01 (`crates/e2e`), 05/02 (`app/ui/e2e`).
- **Wave 6**: 06/01.

Which files each plan touches:

| Plan | Owns | Shared files it touches |
| ---- | ---- | ----------------------- |
| 01   | `crates/core/src/insight.rs`, `crates/engine/src/insight_tests.rs` | `crates/core/src/{lib.rs,module.rs,event.rs}`, `crates/engine/src/session.rs`, every `Event::ModuleFinished` literal (`crates/engine`, `crates/e2e`, `app/tauri`), `app/ui/src/ipc/bindings.generated.ts`, `app/ui/src/state/reduce{,.test}.ts`, `app/ui/src/ipc/mock.ts`, `app/ui/e2e/core-mock.ts`, `crates/testing/src/{run_module.rs,contract.rs}` |
| 02   | `modules/docker/`, `fixtures/homes/docker-*.toml`, `fixtures/golden/docker.json` | `Cargo.lock`, `docs/dependencies.md` |
| 03   | `app/ui/src/product/insight-card/`, `app/ui/src/screens/insight/`, `app/ui/src/screens/scan/described-only*` | `app/ui/src/screens/scan/scan-screen.tsx`, `app/ui/src/state/navigation.ts`, `app/ui/src/screens/screen-stack.tsx`, the place that renders a screen by its kind, `app/ui/src/locales/en.json`, `app/ui/src/gallery/registry.ts` |
| 04   | `fixtures/sandboxes/docker*.toml`, `fixtures/golden/sandbox-docker*.json` | `crates/sandbox/src/{case,world,expected,generate,error}.rs`, `crates/testing/src/tree_description.rs`, `crates/core/src/app_info.rs`, `crates/core/locales/en.json`, `app/tauri/src/{sandbox.rs,sandbox/tests.rs,compose.rs,commands.rs}`, `app/tauri/Cargo.toml`, `app/tauri/tests/{sandboxes.rs,sandboxes/harness.rs}`, `app/ui/src/ipc/bindings.generated.ts` |
| 05   | `crates/e2e/tests/docker.rs` | `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/` |
| 06   | `docs/deferred/docker-other-runtimes.md`, `docs/deferred/user-review-2026-11-22.md` (created or appended) | `docs/` (architecture, testing, manual-testing, categories README, ideas, deferred, plans README) |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
or a rebase and run the checks. So are the sandbox goldens (`UPDATE_GOLDEN=1 npm run sandbox:test`, the diff
reviewed).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/docker` itself at or above 90 %.
- `git diff 97d9b314 -- crates/cleaner crates/scanner crates/store crates/rules crates/macos crates/system` and every
  other module's folder are empty.
- No code path of the plan runs a command that changes Docker: the module's only call is
  `<bundle>/<cli> --context <context> system df --format json` (unit tests); the core e2e and every sandbox case file
  fail on any other call; no finding, total or plan item is Docker's.
- A runtime whose app is closed, or whose socket is absent, or a restored scan, runs no program (unit, e2e, sandbox).
- `npm run sandbox:test` passes for the four case files and every other one; it fails when an insight expectation is
  changed on purpose and when a case runs a program it does not state (done once by hand each, written in 04/03).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- docker` (and the three others): every line of `EXPECTED.md` holds by hand, Copy included.
- On the developer's Mac (`npm run dev`) with a runtime running, the card's numbers match `docker system df` run by
  hand (06/01, the user's).

## Ideas and deferred items this touches

- [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md): the Docker line, built as a
  description; this plan replaces the idea "Why Docker does not give the space back" (2026-11-15, the agent's
  proposal); everything it held is in the design.
- [The world outside the disk in a sandbox](../../deferred/sandbox-world.md): the tools part is taken (stated answers,
  every other call refused and recorded); open files and the steps between scans stay deferred.
- New deferred item [Docker's other runtimes](../../deferred/docker-other-runtimes.md), written in 06/01.

## Risks

See [`00-design.md`](./00-design.md), "Risks": the answer's format (unverified here: Docker not running, and its CLI
may start it), a CLI that starts its engine, a copied command that removes more than counted, the guarded disk files,
the UI missing a restore's events, the rebase over plans 031, 033, 034 and 037.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Whole-kind `prune` commands with `docker system df`'s four numbers.
2. Only runtimes with an app bundle (Docker Desktop, OrbStack); the others deferred.
3. The commands shown, each with `--context`; `volume prune` in its unnamed-only form.
4. No category, no category file; the texts in the module's own dictionary.
5. No storage-map change.
6. Asked only when its app runs and its socket is present; never in a restored scan.
7. Copy through the web view's clipboard, selectable text as the fallback.
8. A generic core output, `Insight`, sent with `ModuleFinished`.
9. Sandbox mode never runs any program but `/usr/bin/open`.
10. Tool answers inline in the case file.
11. Group Development, module `docker`, insight identities = row ids.
12. The parts' danger levels colour the wording only.

## Open questions

None.

## Decisions

1. **Docker is described, not cleaned** (the user, 2026-11-22): the app shows what Docker takes and the command that
   frees each part; the user runs it if they want. This also settles how a removal that cannot go to the Trash would
   have met rule 2: there is none.
2. **2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken.** The open
   question of the draft (how an information-only module sits on the scan screen) was settled by the agent and is
   recorded as agent's call 8 to check: a generic `Insight` output, shown in its own section under the danger zones,
   with its own screen.

## Conclusions

Built on `feat/032-docker`, rebased over plans 031, 033, 034 and 037 (main 9e7fde8a), **awaiting the user's walk**: the
plan is not marked done until the user has looked at the result (the list is
[`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md), section "032", and items 1.19
and 2.17 of [`../../manual-testing.md`](../../manual-testing.md)).

**What was built.** Docker is described and never cleaned. The core gained `Insight` (an identity, a title, a summary,
up to 4 figures, 8 parts and 4 notes; a part has a size, a detail, an effect, a danger level as wording and an optional
`ShownCommand`), `FindingSink::describe` (a no-op by default) and `ModuleFinished.insights`; the engine drops a repeated
identity with a warning. The module `docker` (group Development, no category, no setting) has a table of two runtimes,
Docker Desktop and OrbStack, and runs the runtime's own CLI, `<app>/<cli> --context <ctx> system df --format json`,
only when its app is installed and running, the walk saw its socket and the scan is not restored; it describes the
four parts (build cache Safe, images Low, stopped containers Medium, volumes High) with the command to copy, and the
disk file's figures. The UI shows the section "Described, not cleaned" under the danger zones and an insight screen
with Copy. The sandbox gained tool answers (`[[case.tool]]`), other apps' data in the access, `written` bytes of a
sparse file and insight expectations; sandbox mode runs no program but `/usr/bin/open`. Four case files, eight core e2e
flows, the UI scenarios and baselines, and the docs as built. The cleaner, the scanner, the store, the rules, the macOS
crates, the system crate and every other module are unchanged; no new dependency.

**Numbers on the last code commit (718c8954), on the reference Mac.**

| Check                           | Result                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------- |
| `npm run pre-push`              | green: lint, guards, knip, machete, deny, script tests (78), tests with coverage            |
| Rust tests and coverage         | 4,296 passed (5 skipped); 97.62 % of lines (floor 90 %)     |
| `modules/docker` alone          | 92 tests; 99.76 % of lines (424 of 425; floor 90 %)                                         |
| `fresher-e2e`                   | 179 passed (2 skipped), 8 of them `docker.rs`                                               |
| `npm run sandbox:test`          | 90 passed                                                                                   |
| UI unit tests and coverage      | 1,214 passed in 137 files; 98.45 % of lines (floor 90 %)                                    |
| Playwright, chromium and WebKit | 655 passed, 1 skipped (the WebKit clipboard-read test, by design), 1 failed: the known flaky chromium baseline `category-details-app-leftovers-chosen` (step 05/02) |
| No change elsewhere             | `git diff 9e7fde8a -- crates/cleaner crates/scanner crates/store crates/rules crates/macos crates/system` and every other module's folder: empty (against the old base 97d9b314 the diff holds main's own plans 031, 033, 034 and 037) |
| Perf                            | not run: no walk, scanner or scan-time path changed, and the user's decision of 2026-11-21 is no performance runs unless a step says so |

**What changed from the design and why.**

1. **The `left` card only on a complete installed-apps listing** (review of 02/02): a partial listing could read an
   installed runtime as removed and show a false "not installed any more" card. The design's decision table (row 3 and
   section 2) was amended; the cost is that a truly left-behind disk file is not described when the listing is partial.
2. **The answer is capped at 64 KiB** before parsing (a longer one is unreadable), and a line of an unknown `Type` is
   ignored whatever its fields.
3. **The unreadable reason is not logged**: the module has no logging dependency (no new edge); the state shown is
   `unreadable`. A gap, listed in the user's review.
4. **A restored scan reads a disk place as absent** (`PlaceSeen::Unknown` maps to `Disk::Absent`), so no "allow access"
   note appears in a restored scan.
5. **The sandbox autotest exempts the trial module's two reads** (`fresher-trial-tool version` and `list`): they are a
   bare-name test fixture a case file cannot state; its remove is never let through. The brief's "no existing case
   file runs a program" was wrong.
6. **`SandboxCommands` lives in its own file** (`app/tauri/src/sandbox/commands.rs`) to keep the rebase over plan 037
   mechanical; `Extras.commands` was added to the e2e harness so a test can read the calls.
7. **Sparse disk files are written 1 MiB at their head** in fixtures and case files (APFS allocates a written head of a
   sparse file of 16 MiB or less whole), so their on-disk size is exact; the goldens assume APFS.
8. **The `EXPECTED.md` line of an insight** names Copy only when the card has a command.

**By-hand mutation checks of 04/03** (each reverted): `commands = 3` changed to 4 fails the autotest ("expected 4,
described 3"); the OrbStack tool's `json` changed to `table` fails it ("a program the case file does not state was
run"); `[[case.insight]]` removed fails it ("described but no case expects it"). The e2e gate was not mutation-checked
by hand; the module's own unit tests cover every row of the decision.

**The calls taken while building (for the user to look at).** The generic `Insight`; only runtimes with an app; the
four whole-kind commands with `--context`; no storage-map change; nothing asked in a restored scan; sandbox mode
running no program but `/usr/bin/open`; the parts' levels colour the wording only; the OrbStack disk file's length
hidden. The full list with costs: [`00-design.md`](./00-design.md), "Agent's calls (to check)".

**The user's walk.** Pending: the four sandboxes, Copy in the built app (the clipboard in the built web view is
unverified) and the facts to verify on a Mac with a runtime running (the answer's format, taken from the documentation
and mocked; the CLI's place and context for Docker Desktop; the disk files' places; whether a CLI starts a stopped
runtime; how each runtime gives space back; `volume prune` unnamed-only; `Reclaimable` against what `--all` frees). A
difference is a correction in `runtimes.toml`, the texts and the category doc. This plan becomes ✔️ done when it passes.
