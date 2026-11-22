# 017 — Temporary files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 ·
**Modules:** core, macos, system, scanner, engine, cleaner, testing, sandbox, modules, app, app/ui, docs

Written on 2026-11-17 from the agent's research and the user's four decisions of the same day; every other open
question was decided by delegation (the user's instruction: accept the recommended decisions), recorded in
[`00-design.md`](./00-design.md), "Decisions". The agent's calls wait for the user's check; the facts about macOS that
the research could not verify are checked by hand, read-only, in 04/02.

## Goal

The old working files that apps and tools leave in the user's own temporary folder (`$TMPDIR`,
`/private/var/folders/<xx>/<hash>/T`) and the user's own items in `/private/tmp` are offered in one Safe category,
"Old temporary files", once nothing changed them for more than 7 days (a number the user may only raise) — and the
module says plainly what macOS already clears on its own. What is young, in use by an open app, the system's or
another user's, locked, a socket, a pipe or a lock is never offered: it is reported as left out with its reason. The
module is `modules/temp` (`fresher-module-temp`), in a new group, System.

The core first gains what the research found missing: the two places as known folders with their protection, walked
although they lie outside the home; a fourth entry kind for sockets and pipes (the
[known issue](../../known-issues/core-fifo-read-as-file.md) fixed); the status-change time and the newest change of a
whole folder; a sandbox that states the facts it cannot make on disk. Tests at every level, as plan
[016](../016-jetbrains/readme.md) did: unit, fixture, sandbox, core e2e, UI e2e, runs by hand.

Branch: `feat/017-temp-files`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
**Built after plan [016](../016-jetbrains/readme.md) is merged into `main`**, on this branch rebased onto `main` then
(decision 20). Depends on plans [008](../008-safe-removal/readme.md), [009](../009-system-knowledge/readme.md),
[011](../011-diagnostics/readme.md), [012](../012-storage-map/readme.md), [015](../015-module-groups/readme.md) and
016.

## Plans

| #   | Plan                                                        | Status                                  | Depends on | What                                                                                                     |
| --- | ----------------------------------------------------------- | --------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------- |
| 01  | [Core prerequisites](./01-core-prerequisites/readme.md)     | ✔️ done                                | —          | the two places and their protection, a fourth entry kind, the times of a subtree, places outside the roots, the sandbox's stated facts |
| 02  | [The module](./02-module/readme.md)                         | ✔️ done                                | 01         | the crate, the group System, the decision and its left-outs, the setting, fixture tests, registration   |
| 03  | [End to end](./03-end-to-end/readme.md)                     | ✔️ done                                | 01, 02     | the case file, the core e2e flows, the UI e2e scenarios                                                  |
| 04  | [Finish](./04-finish/readme.md)                             | 🔨 in progress                          | all        | the docs, the runs by hand, the conclusions                                                              |

### Order and parallel work

```
01/01 ──┬── 01/04 ─────────────────────┐
        └── 02/01 ──┬── 03/03 ─────────┼──────────────────────────────────┐
                    │                  │                                  │
01/02 ── 01/03 ─────┴── 02/02 ──┐      │                                  │
             └──────────────────┼── 01/05 (after 01/03, 01/04)            │
                                └── 02/03 (after 02/02, 01/05) ──┬── 03/01 ──┼── 04/01 ── 04/02
                                                                 └── 03/02 ──┘
```

- **Wave 1, in parallel**: 01/01 (known folders, `crates/system`, `crates/macos/src/{protection,ffi}`) and 01/02 (the
  entry kind: `crates/core/src/{interest,folder}.rs`, the bulk reader, the matcher, the guard).
- **Wave 2**: 01/03 after 01/02 (the same reader and codec files); 02/01 after 01/01 (`modules/temp/`,
  `crates/core/src/group.rs`, the docs' category file); 01/04 after 01/01 (`crates/engine`, the walk request, the
  cleaner's roots, the map).
- **Wave 3**: 01/05 after 01/03 and 01/04 (`crates/testing`, `crates/sandbox`, the sandbox composition); 02/02 after
  02/01 and 01/03 (`modules/temp/src/` only); 03/03 may start once 02/01's texts exist (`app/ui` only).
- **Wave 4**: 02/03 (needs 01/05's lens and 02/02), then 03/01 and 03/02 in parallel (different files).
- **Wave 5**: 04.

Which files each plan touches:

| Plan | Owns                                                                                                         | Shared files it touches                                                                                                                                                                                       |
| ---- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/inside.rs`, `crates/macos/src/ffi/confstr.rs`, `crates/testing/src/facts_lens.rs`           | `crates/core/src/{known_folder,interest,folder,walk,lib}.rs`, `crates/core/src/settings/folders.rs`, `crates/core/locales/en.json`, `crates/macos/src/{protection.rs,bulk/,ffi/mod.rs,lib.rs}`, `crates/system/src/known_folders.rs`, `crates/scanner/src/{scanner.rs,matcher/,tree/,codec/,rescan/,naive*}`, `crates/engine/src/{engine.rs,scan*,cleanup/context.rs,storage/}`, `crates/cleaner/src/{guard/,paths.rs}`, `crates/testing/src/{known_folders,tree_description,fixture_tree,run_module,lib}.rs`, `crates/sandbox/src/{case,world,layout,generate,expected}.rs`, `app/tauri/src/{compose,sandbox}.rs`, `app/tauri/tests/sandboxes/harness.rs`, the sandboxes' and the scanner's goldens, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/locales/en.json`, `docs/known-issues/`, `docs/testing.md` |
| 02   | `modules/temp/`, `fixtures/homes/temp-*.toml`, `fixtures/golden/temp.json`, `docs/categories/temp-old-files.md` | root `Cargo.toml` (the member), `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`modules()`), `crates/core/src/group.rs`, `crates/core/locales/en.json`, `crates/core/tests/typescript.rs`, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/locales/en.json`, the UI's group stories and tests, `docs/categories/README.md` |
| 03   | `fixtures/sandboxes/temp.toml`, `fixtures/golden/{sandbox-temp,temp-flow}.json`, `fixtures/homes/temp-flow.toml`, `crates/e2e/tests/temp.rs`, `app/ui/src/test/temp-scan.ts` | `crates/e2e/Cargo.toml`, `app/tauri/tests/sandboxes.rs` (`case_files!`), `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/` |
| 04   | —                                                                                                            | `docs/` (architecture, restrictions, testing, categories, known issues, ideas, deferred, plans README)                                                                                                        |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/temp` itself at or above 90 %.
- The module's contract test passes with `Catalogue::Required`: one category file, the same danger level, every text
  and every declared left-out reason used by a fixture run.
- No path outside the user's own `T` (but `TemporaryItems`) and `/private/tmp` can be offered: the protection list's
  tests, the cleaner's `tests/never.rs` with the new places, and the sandbox's negatives prove it.
- The [FIFO known issue](../../known-issues/core-fifo-read-as-file.md) is fixed: a pipe is `Other`, never a marker, and
  the guard opens one without blocking (a test).
- `npm run sandbox:test -- temp` passes; it fails when a case's expectation (an item, a danger, a left-out count, a
  stated fact) is changed on purpose (done once by hand per kind, written in the conclusions).
- `npm run test:e2e` passes with the nine core flows and the six UI scenarios.
- `npm run dev:sandbox -- temp`: every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, scan only): every offered item is in the user's own `T` or `/private/tmp`,
  owned by the user and unchanged for more than 7 days; the locked updater package the research found is left out
  `system`; no item of `C/`, `0/`, `X/`, `TemporaryItems` or another user is claimed; the scan's time stays within
  5 % of `main`'s (rule 4).

## Ideas and deferred items this touches

- The [System Junk Cleaner](../../ideas/system-junk-cleaner.md): its "temporary files of the user" becomes this plan;
  crash reports, caches and logs stay there.
- [Downloads and installers](../../ideas/downloads-and-installers.md) (new, the user's decision 2): partial downloads
  and old installers, out of this module.
- [The Trash holds the freed space](../037-trash/readme.md): the category's "if removed" says the space
  comes back when the Trash is emptied.
- The [Storage Leak Detector](../../ideas/storage-leak-detector.md): temporary folders that keep growing are its signal;
  this plan's places and subtree times are what it would read.
- The deferred [sandbox world](../../deferred/sandbox-world.md): the file facts a case states (times, owners, flags,
  kinds) are built here (01/05), beside plan 016's apps and clock; open files, tools, access and the steps between
  scans stay deferred (the item is updated in 04/01).

## Conclusions

**Built on 2026-11-17 and 2026-11-18, before the run by hand** (04/02): the plan stays 🔨 in progress until the user has
run its checklist and looked at the result; the numbers of the real Mac are written then.

What was built: the core's prerequisites (the two places as known folders with their protection, `EntryKind::Other`, the
status change, the access and the inside of a whole folder, the places walked outside the scan roots and always read
from the disk, the sandbox's stated facts through `FactsLens`); the module `modules/temp` in the new group System (one
Safe category, ten rows, eight left-out reasons, the setting from 7 to 30 days), registered in every build; five fixture
homes, a case file of 26 cases, nine core e2e flows, nine UI scenarios.

Changed from the design: decision 21 (the places always walked, a restore emitting nothing inside them) and the rulings
R1 to R9 of [`00-design.md`](./00-design.md), "Conclusions"; each step says what changed from its brief.

Numbers: the scan bench moved by −1 % to +4.6 % (within 5 %); the tree's memory on the bench 2.0 to 2.7 MB;
`modules/temp` 100 % of lines; sandbox autotests 37; the temp core e2e 13 tests, stable over 150 runs; Playwright 320 of
321 (a flake that was there before). `npm run pre-push` and the scan's time on the real Mac are for the run by hand.

Open: everything in [04/02](./04-finish/02-run-and-conclusions.md); two deferred items (the e2e homes'
helpers, done by plan 018/03/02, [a special entry in the browse
view](../../deferred/special-entry-in-browse-view.md)); a flake under load seen before this plan, the accessibility
gallery's part 3 in WebKit ([known issue](../../known-issues/ui-axe-gallery-part-3-webkit-timeout.md)); the parked small
points of the design's conclusions. Fixed in the final review's wave: the rescan property test's access time ([known
issue](../../known-issues/scanner-rescan-property-access-time.md)), its changes settling the files they write; the
sandbox autotests' allocated sizes under load (`storage_map`, `trial`; [known
issue](../../known-issues/app-sandbox-storage-map-allocated-size.md)), a data-filled fixture file sized before it is
written.
