# 033 — Logs and crash reports

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Drafted on 2026-11-22; designed on 2026-11-22 from a read of the code on `main` (97d9b314). Approved by the user's
delegation of 2026-11-22. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Old logs and crash reports are the first thing any cleaner offers. Fresher offers the logs of removed apps and of
JetBrains IDEs only; the rest of `~/Library/Logs` belongs to nobody.

## Goal

A new module `logs` (`fresher-module-logs`, group **System**), in the shape of `temp`, offers one finding per file:

| Category             | Level | What                                                                       |
| -------------------- | ----- | -------------------------------------------------------------------------- |
| `logs.app-logs`      | Safe  | files under `~/Library/Logs` unchanged for more than the setting's days    |
| `logs.crash-reports` | Safe  | files under `~/Library/Logs/DiagnosticReports`, the same way               |

- The threshold is a module setting, `logs.older-than-days`, 7 days by default and never fewer.
- Files go to the Trash; folders stay, so an app never finds its log folder missing.
- Left out with a reason: a restored scan's file, a special entry, a file not the user's, a locked file, a file
  changed within the days. A file held open at the act is skipped by the cleaner.
- Never claimed: a folder, a link, anything under `Logs/JetBrains` (`jetbrains`) or under a top-level folder shaped as a
  bundle id (`app-leftovers`), an app's state or lock file (`.plist`, `.db`, `.sqlite*`, `.lock`, `.pid`).
- The place's contract, not a file's name, says what it is: the second exception of "Unknown is not junk"
  ([`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)), written by 01/02.

Nothing in the core, the scanner, the cleaner or the engine changes. The walk does not change shape (six `Claim::Item`
interests under `Logs`, nothing made opaque), so no scan bench is due (agent's call 6).

Branch: `feat/033-logs`, from `main` (97d9b314), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[011](../011-diagnostics/readme.md), [015](../015-module-groups/readme.md), [017](../017-temp-files/readme.md) and
[019](../019-uninstalled-app-leftovers/readme.md).

## Plans

| #   | Plan                                    | Status      | Depends on | What                                                                          |
| --- | --------------------------------------- | ----------- | ---------- | ----------------------------------------------------------------------------- |
| 01  | [The module](./01-module/readme.md)     | 🔨 in progress | —          | the crate, texts and category files; the decision; fixtures, golden, contract |
| 02  | [End to end](./02-end-to-end/readme.md) | 🔨 in progress | 01         | the case file and the registration, the core e2e, the UI                      |
| 03  | [Finish](./03-finish/readme.md)         | 🔨 in progress | all        | the docs, the conclusions, the user's review list                             |

### Order and parallel work

```
01/01 ──┬── 01/02 ── 01/03 ──┬── 02/01 ──┬── 03/01
        │                    └── 02/02 ──┤
        └── 02/03 ───────────────────────┘
```

- **Wave 1**: 01/01. **Wave 2, in parallel**: 01/02 (`modules/logs/` and `deletion-safety.md`) and 02/03 (`app/ui`
  only, once 01/01's texts exist). **Wave 3**: 01/03. **Wave 4, in parallel**: 02/01 and 02/02 (both add a line to
  `docs/testing.md`: the second to finish rebases). **Wave 5**: 03/01.

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them and run the
checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/logs` itself at or above 90 %.
- `git diff 97d9b314 -- crates/core crates/scanner crates/cleaner crates/engine` is empty: no core change.
- The contract test passes with `Catalogue::Required`; every text and every declared left-out is used by a fixture run;
  every removal is a `MoveToTrash` of a matched file.
- Never offered: a folder, a link, a file under `JetBrains` or a bundle-id-shaped top folder, a state or lock file, a
  file outside `~/Library/Logs` (the fixtures' and the case file's negatives); the core e2e shows no path offered by two
  modules and every log folder still there after the cleanup.
- `npm run sandbox:test` passes; it fails when a case's danger or a left-out's items is changed on purpose (done once
  by hand per kind, written in 02/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- The user's walk (`npm run dev:sandbox -- logs`, `EXPECTED.md`) and the dry run on the developer's Mac are listed in
  `docs/deferred/user-review-2026-11-22.md`.

## Ideas and deferred items this touches

- [System Junk Cleaner](../../ideas/system-junk-cleaner.md): its logs and crash reports become this plan.
- More places of logs (`docs/deferred/logs-more-places.md`): a new deferred item written in 03/01 (an installed app's
  logs under its bundle id, files deeper than six names).

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken.
- The draft's agent's calls stand: only the user's Library; older than 7 days, a setting; files go, folders stay; a
  module of its own rather than a category of `temp`.

## Agent's calls (to check)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)". The ones that change the
draft or need a second look:

1. **The place's contract** claims every file of `~/Library/Logs` whatever its name: the second exception of "Unknown
   is not junk", bounded by the decision's rows.
2. **An installed app's logs under its bundle id are not offered** (deferred): the draft expected them, but
   `app-leftovers`' whole claim of every bundle-id folder of `Logs` hides their files from any other module.
3. **A top-level folder shaped as a bundle id is refused**; a top-level file of that shape is offered like any log.
4. **One finding per file.**
5. **No app attribution, no running-apps check**: the reason names the folder; the cleaner's open-file check protects a
   log in use.
6. **No scan bench.**
7. **State and lock suffixes are not claimed.**
8. **Six names deep.**

## Conclusions

Built on `feat/033-logs` over `main` (97d9b314), **awaiting the user's walk**: the plan is not marked done until the user
has looked at the result (the list is
[`docs/deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md), section 033).

**What was built.** The module `logs` (`modules/logs`, group System) offers one finding per file: the old files of the
user's `~/Library/Logs` as "Old logs of your apps" (`logs.app-logs`) and those under `DiagnosticReports` as "Old crash
reports" (`logs.crash-reports`), both Safe, to the Trash. A file is offered when nothing changed in it for more than the
setting's days (`logs.older-than-days`, 7 to 90, 7 by default). Never claimed: a folder, a link, anything under
`JetBrains`, a folder shaped as a bundle id, an app's state or lock file. Left out with a reason: a restored scan's file,
a special entry, a file not the user's, a locked file, a file changed within the days. Folders always stay. Around it:
three fixture homes and a golden, a case file, seven core e2e flows, the UI scenarios and baselines, and the docs as
built. The second exception of "Unknown is not junk" is written in `deletion-safety.md`. No core change
(`git diff 97d9b314 HEAD -- crates/core crates/scanner crates/cleaner crates/engine` is empty), no new dependency and
no new UI component.

**Numbers on the last code commit (a63e6811), on the reference Mac.**

| Check                           | Result                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------- |
| `npm run pre-push`              | green: lint, guards, knip, machete, deny, script tests, tests with coverage           |
| Rust tests and coverage         | 3,899 passed (5 skipped); 97.55 % of lines (floor 90 %)                               |
| `modules/logs` alone            | 66 tests; 99.64 % of lines (280 of 281; floor 90 %)                                   |
| Core e2e (`test:e2e:core`)      | 145 passed (2 skipped); 7 flows are the module's                                      |
| UI unit tests and coverage      | 1,160 passed in 131 files; 98.6 % of lines (floor 90 %)                               |
| Playwright, chromium and WebKit | 552 passed; in the `npm run test:e2e` run the watched chromium baseline               |
|                                 | `category-details-app-leftovers-chosen` failed once (a known flake; passed alone)     |
| No core change                  | the diff above is empty                                                               |
| Perf                            | not run: the walk's shape does not change (agent's call 6)                            |
| What the module offered on Mac  | pending: the dry run is the user's                                                    |

**What changed from the design and why.**

1. **A top-level file shaped as a bundle id is offered** (call 3, 01/02's fix round): the shape needs one dot, so
   refusing every such name hid `system.log`, `install.log` and the like from every module. Only a folder of that shape
   (what `app-leftovers` claims) is refused.
2. **`serde_json` is a dev-dependency** of the crate (the setting test builds `StoredSettings`); an allowed dev
   dependency, no new row beyond the used-by column.
3. **The registration came with the case file** (02/01), so `fresher-app` stayed green between steps.
4. **The golden of the core e2e drops the findings' ids and is sorted by identity**: the ids follow the walk's order,
   which uses two threads.
5. **The cleanup in the UI mock moves two logs** ("Moved 2 items"), the selection of the review.
6. **A restored-file check in the contract test** stands in for the file kind, since `ModuleRun` has no kinds.

**Calls taken while building.**

- No separate preflight scan: the steps were written from a fresh read of the code. The cost, a stale step, did not
  happen.
- The draft's agent's calls were approved by the user's delegation of 2026-11-22; the ones that need a second look are
  in the review list: the second exception of "Unknown is not junk" (call 1), the top-level files (call 3), no app
  attribution and no running-app check (call 5).
- Reviews: tasks 1 to 5 were approved at once (task 2 after one fix round for the top-level files); task 6 needed one
  fix round (four unrelated scenarios regressed by a blanket replace).

**Left to do.** The user's walk and dry run ([`user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md));
the deferred [More places of logs](../../deferred/logs-more-places.md).
