# 008 — Safe removal and the cleanup journal

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core, cleaner, macos, store, engine,
testing, app, app/ui

Approved by the user on 2026-11-16 with the recommended answers ([`00-design.md`](./00-design.md), "Decisions"): the
minimum of plan 007, a 10-minute expiry, built on top of plan 006 and merged after it.

## Goal

The one path by which anything is ever removed. The user's selection becomes a frozen plan the core checked item by
item; the user confirms that exact list, and the core checks the summary the UI sends back; only a confirmed plan
reaches the cleaner, which runs every rule of [`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)
again right before each item — links, roots, protected places, identity, removability, capabilities, items in use,
growth — and fails closed. Items go to the Trash by default; a permanent delete is its own consent. A journal records
where everything went, so items still in the Trash can be put back, and the result is honest: "moved to the Trash", the
space measured on the volume, every skip with its reason. Debug builds and tests cannot remove anything outside a
sandbox folder.

Branch: `feat/008-safe-removal`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [002](../002-base-architecture/readme.md), [003](../003-base-ui/readme.md) and
[004](../004-fast-scan/readme.md) (done) and on plan [006](../006-permissions/readme.md) (removability, capabilities,
guarded places, `SystemUi`, the walk's entry facts): **006 is merged first**, and this branch is rebased on `main`
then. It takes from plan [007](../007-safe-development/readme.md) only the guards it cannot be tested without (design,
section 14; open question 1).

## Plans

| #   | Plan                                            | Status  | Depends on             | What                                                                                                  |
| --- | ----------------------------------------------- | ------- | ---------------------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [Measure first](./01-measure-first/readme.md)   | ✔️ done | —                      | throwaway probes: the Trash call, put back, the Trash's privacy, links, open files, freed space       |
| 02  | [Contracts](./02-contracts/readme.md)           | ✔️ done | —                      | the core's plan, confirmation and outcome types, protection, ports, events, fakes, the sandbox marker |
| 03  | [macOS layer](./03-macos-layer/readme.md)       | ✔️ done | 01, 02                 | `MacTrash` and put back, `MacOpenFiles`, the protection list, the current user                        |
| 04  | [The cleaner](./04-cleaner/readme.md)           | ✔️ done | 02                     | reach and paths, the guard, the folder walk, the removals, rehearse and execute, put back, proofs     |
| 05  | [The journal](./05-journal/readme.md)           | ✔️ done | 02                     | `FileCleanupJournal`: records, reading, versions, rotation, the footprint                             |
| 06  | [Engine and app](./06-engine-and-app/readme.md) | ✔️ done | 02 (06/04: 03, 04, 05) | the plan builder, the clean operation, history, the commands, the `trial` module, the core e2e        |
| 07  | [Interface](./07-interface/readme.md)           | ✔️ done | 06/04                  | the state, the confirmation, progress and result, the Cleanups screen, flows and screenshots          |
| 08  | [Finish](./08-finish/readme.md)                 | ✔️ done | all                    | the docs, the sandbox run by hand, version 0.5.0, the conclusions                                     |

### Order and parallel work

```
01 ────────────────────┐
02 ──┬─────────────────┴── 03 ──┐
     ├── 04 ────────────────────┤
     ├── 05 ────────────────────┼── 06/04 ──┬── 06/06 ──┐
     └── 06/01–06/03, 06/05 ────┘           └── 07 ─────┴── 08 ── conclusions
```

- **Wave 1, in parallel**: 01 (probes, by hand, no product code) and 02 (its steps in order). The core is frozen after
  02, as in plans 002 and 004: a contract missing later stops the plan that needs it, and the change goes into
  `crates/core` in the main session.
- **Wave 2, in parallel** (after 02; 03 also after 01): 03, 04, 05, 06/01–06/03 on the fakes of 02, and 06/05 (the
  dev-only `trial` module).
- **Wave 3**: 06/04 (the app wired to the real adapters, the commands, the bindings), then 06/06 (the core e2e).
- **Wave 4**: 07, its steps in order (07/04 may run beside 07/02–07/03: separate folders).
- **Wave 5**: 08, then the conclusions.

Each parallel plan runs in its own worktree branched from `feat/008-safe-removal` and is merged back when it is done;
two agents never share a worktree.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                                                                                 | Shared files it touches                                                                                                                                                                                                                                                                                                                                                                       |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `NO_COMMIT/probes/008/` (never committed)                                                                                                                      | `00-design.md`, section "Measured (01)" (01/03 only)                                                                                                                                                                                                                                                                                                                                          |
| 02   | `crates/core/src/cleanup/`, `crates/core/tests/compile_fail/`, `crates/core/tests/types.rs`                                                                    | root `Cargo.toml` (pins), `docs/dependencies.md`, `crates/core/{Cargo.toml,locales/en.json}`, `crates/core/src/{lib,ports,volume,event,command,error,texts}.rs`, `crates/macos/src/volume.rs` (one method), `crates/testing/src/`, `crates/engine/` and `crates/scanner/` (adapt to the port changes only), `app/tauri/src/adapters.rs`, `app/ui/src/ipc/bindings.generated.ts`, `Cargo.lock` |
| 03   | `crates/macos/src/{trash,open_files,protection}.rs`, `crates/macos/src/ffi/{trash,libproc,rename}.rs`                                                          | `crates/macos/{Cargo.toml,src/lib.rs,src/ffi/mod.rs,src/process.rs}`, `docs/dependencies.md`, `Cargo.lock`                                                                                                                                                                                                                                                                                    |
| 04   | `crates/cleaner/`, `docs/architecture/removal.md`                                                                                                              | `Cargo.lock`, `clippy.toml` (the reason text of the deletion lint)                                                                                                                                                                                                                                                                                                                            |
| 05   | `crates/store/src/journal/`, `fixtures/journal/`                                                                                                               | `crates/store/src/lib.rs`, `crates/e2e/tests/footprint.rs`, `docs/architecture/README.md` ("Data")                                                                                                                                                                                                                                                                                            |
| 06   | `crates/engine/src/cleanup/`, `modules/trial/`, `crates/e2e/tests/cleanup.rs`, `fixtures/homes/{cleanup,trial-*}.toml`, `fixtures/golden/{cleanup,trial}.json` | `crates/engine/src/{engine,state,results,session,lib}.rs`, `crates/engine/Cargo.toml`, `app/tauri/` (commands, composition, shell, capability, build, tests), `app/ui/src/ipc/bindings.generated.ts`, `crates/e2e/Cargo.toml`, `docs/hacks/README.md`, `Cargo.lock`                                                                                                                           |
| 07   | `app/ui/src/state/cleanup.ts`, `app/ui/src/product/{cleanup-progress,cleanup-row}/`, `app/ui/src/screens/cleanups/`                                            | `app/ui/src/ipc/`, `app/ui/src/state/{store,reduce,selection,findings,navigation}.ts`, `app/ui/src/product/{confirm-dialog,result-summary}/`, `app/ui/src/screens/{review,app-frame,frame-sidebar}*`, `app/ui/src/locales/en.json` (`ui.cleanup.*`, `ui.screen.cleanups.*`), `crates/core/locales/en.json` (texts only), `app/ui/e2e/`, `docs/hacks/README.md`                                |
| 08   | —                                                                                                                                                              | `docs/architecture/`, `docs/restrictions/deletion-safety.md`, `docs/testing.md`, `docs/dependencies.md`, `docs/hacks/README.md`, `docs/known-issues/`, `docs/plans/README.md` (the status row), the version files                                                                                                                                                                             |

`Cargo.lock` and `package-lock.json` are never merged by hand: on a conflict take either side, regenerate them and run
the checks. The core's dictionary (`crates/core/locales/en.json`) gets its keys in 02; later plans only edit texts.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- The property tests hold on every run: no plan, method or depth ever moves or deletes anything outside its roots,
  inside or above a protected or ignored place, or a hard-blocked folder; a rehearsal changes nothing.
- The compile-fail tests hold: no `ConfirmedPlan`, `PermanentConsent`, `Confirmation` or `Act` can be made outside the
  confirmation; a confirmed plan executes once; a `Verified` is used once.
- `fresher-cleaner` holds the only `expect` of the deletion lint in the workspace, and `RealDiskToken::new` appears only
  in the app's composition (both checked by tests).
- The core e2e: scan → prepare → confirm → execute → journal → put back on a fixture home gives the golden report; a
  cancelled and a refused cleanup leave the disk and the journal as the golden says.
- In the app (`npm run tauri dev`, a sandbox folder of `trial` folders as the scan root): select, review, confirm,
  watch the progress, see the result with the measured space, open the Cleanups screen, put an item back. Against the
  real home folder the same build leaves every item out ("Development build").
- By hand (08/02, written in the conclusions), in that sandbox: the real Trash, put back, Finder's own Put Back, a held
  folder skipped, a disk image without a Trash skipped, quitting mid-cleanup.

## Ideas this touches

- [Cleanup preview and safe removal](../../ideas/cleanup-preview-and-safe-removal.md) and the
  [cleanup journal](../../ideas/cleanup-journal.md): both become this plan; their open questions are answered in the
  design (put back from the history; no quarantine folder of our own).
- The [cleanup simulator](../../ideas/cleanup-simulator.md): the rehearsal and the dry run.
- [The Trash holds the freed space](../../ideas/trash-holds-freed-space.md): "moved", the measured space, the Trash
  sentence in the confirmation, "waiting in the Trash from cleanups".
- [Safety in the types](../../ideas/architecture/safety-in-types.md): the confirmation, `Act`, `Verified`,
  `PermanentConsent`, the echo, the compile-fail tests. `Evidence` instead of a free-text reason is left to the module
  plans: today the identity and the entry facts are what can be checked again.
- [macOS specifics](../../ideas/architecture/macos.md), "Moving to the Trash".
- The [ignore list](../../ideas/ignore-list.md): an input of the guard, fail closed.
- The [cleanup guardian](../../ideas/cleanup-guardian.md) and [storage history](../../ideas/storage-history.md): they
  read the journal later.
- [Best practices](../../ideas/best-practices.md): reasons for every skip (5), fail closed (6), allowed roots and a
  re-check right before removal (7), Trash only and put back (9), the exact action in the confirmation (10), empty a
  cache folder and keep it (25), hard-blocked user folders (26).

## Conclusions

Done on 2026-11-16; version 0.5.0. Each plan's readme has its own conclusions; this is the whole.

**The user's decision of 2026-11-16**: "merge everything and push". This plan goes into `main` together with the built
parts of plan [006](../006-permissions/readme.md) (the capability model with the coverage of a scan, 02/01–02/04, and
the removability of findings with the walk's entry facts, 05/01), on which it was built; 006 was **not** merged first,
as the header above planned, and its rest continues on a new branch from `main`
([`../../workflow.md`](../../workflow.md)). The sandbox run by hand (08/02) is prepared and
[deferred](../../deferred/removal-sandbox-run.md) to before the first release.

### What was built

- **The contracts** (02, `fresher-core`): the plan, the confirmation and the outcome types, the protection check, the
  ports `Trash`, `OpenFiles`, `Cleaner`, `CleanupJournal` and `Volumes::available`, the events and errors, the sandbox
  marker and their fakes in `fresher-testing`.
- **The macOS layer** (03, `fresher-macos`): `MacTrash` (the system's Trash call, the landed path from the held handle,
  put back with `renamex_np`), `MacOpenFiles` (descriptors, working folders and mapped files of the user's own
  processes), the protection list, the current user; later the `beneath` module that acts below held folders.
- **The cleaner** (04, `fresher-cleaner`): reach and paths, the guard's eleven checks right before each item, the
  folder walk, the removals (Trash, emptying a folder, permanent, a tool), rehearse and execute, put back.
- **The journal** (05, `fresher-store`): `FileCleanupJournal`, an append-only JSON-lines file written ahead, two
  generations rotated by rename at 4 MiB.
- **The engine and the app** (06): the plan builder and the pending plan, the `Clean` operation, history and put back,
  the commands, the app wired to the real adapters with the close and quit guard, the dev-only `trial` module and the
  core e2e.
- **The interface** (07): Review → **Move to Trash…** → the core's plan in the confirmation (the permanent choice off
  and red when ticked, what will not be removed and why) → progress with Stop → the result with the measured space →
  the Cleanups screen with put back; ⌘Q and the app menu's Quit ask during a cleanup.
- **The docs** (08/01): `restrictions/deletion-safety.md` names what holds each rule, `architecture/removal.md`, the
  architecture, testing, modules and dependency docs as built.

### The guarantees, and what holds them

| Guarantee                                              | Held by                                                                                       | Tested by                                                         |
| ------------------------------------------------------ | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| nothing is removed without a confirmation of that list | `ConfirmedPlan` made only by `CleanupPlan::confirm`; the UI's echo of the summary must match  | `crates/core/tests/compile_fail/`, the echo's property test       |
| a confirmed plan runs once; an old plan is refused     | `ConfirmedPlan` not `Clone`, consumed by `execute`; a 10-minute lifetime on the injected time | `confirmed_plan_used_twice`, `confirmed_plan_clone`, the core e2e |
| every removal acts on a checked item, once             | `Verified` consumed by each removal, `Act` lent only by a `ConfirmedPlan`                     | `crates/cleaner/tests/compile_fail/`                              |
| the Trash first; permanent is its own consent          | `PermanentConsent` made only by `confirm` for `Method::Permanent`; the checkbox off each time | `permanent_consent_literal`, the UI flows                         |
| the checks run again right before each item            | the guard's eleven checks (links, roots, protection, identity, removability, in use, growth)  | one negative test per skip reason; `never.rs`                     |
| nothing outside the roots or in a protected place      | `Protection::check` by components; the system's list always applies                           | `never.rs` (property test), `protection.rs`'s property tests      |
| one deletion point                                     | the only `expect` of the deletion lint, `remove/permanent.rs`, the only caller of `remove_at` | `crates/cleaner/tests/deletion_point.rs`                          |
| development builds and tests reach only a sandbox      | `Reach::Sandbox` in debug builds; `RealDiskToken` only in the app's release composition       | `crates/cleaner/tests/reach.rs`                                   |
| a record of where everything went                      | the journal: `begin` synced before the first item; a failed write stops the cleanup           | the journal's torn-record test, the core e2e's golden files       |
| put back never replaces anything                       | `renamex_np` with `RENAME_EXCL` and `RENAME_NOFOLLOW_ANY`                                     | the macOS layer's tests on mounted APFS images                    |
| the result is honest                                   | "moved to the Trash" never "freed"; the space measured on each volume; every skip's reason    | the core e2e (`fixtures/golden/cleanup-*.json`), the UI flows     |

### Measured

- **Plan 01** (Apple M3 Pro, macOS 27.2.1, APFS, no Full Disk Access): the Trash call takes 0.4–2 ms per item after a
  first call of 17–311 ms, follows no link inside the item, keeps its identity, never deletes when it refuses
  (`NoTrash` is code 3328) and needs no permission; a parent swapped for a link makes it move the item behind the link
  and sometimes return no path, while the held handle named the right place in 2 000 of 2 000 calls (call 37); the
  open-files snapshot takes 7–19 ms, and the walk of mapped regions 0.4–0.9 s (call 38); freed space shows at once,
  so no delay before measuring; `remove_dir_all` crosses mount points, so the folder walk compares devices.
- **The macOS layer** (03): a snapshot of open files in 11 ms in a release build, 0.56 s with the region walk (reused
  for 30 s).
- **The cleaner** (04): the property test `never.rs` ran 10 000 cases once by hand in 54 s, all passing (256 in every
  run); coverage of `fresher-cleaner` 96 to 100 % of lines in every file.
- **The journal** (05): a hundred cleanups of a hundred items take 6.9 MB with the scans' cache; the data folder stays
  at 7.0 MB of its 50 MB.

### Decisions

The user's, of 2026-11-16: the design approved with the agent's calls to check; from plan 007 only the minimum; an
unconfirmed plan expires after 10 minutes; built on top of plan 006; and, at the end, merged into `main` with 006's
built parts, without the sandbox run (deferred).

The agent's calls still to check ([`00-design.md`](./00-design.md), "Agent's calls"; all 38 are taken as recommended),
the most important ones:

- **37 — the Trash call through the held handle**: `Trash::move_to_trash` takes the verified item's open handle and
  answers `Landed { at, reported }`; a move that landed something else is `Failed`, journalled when it can be named.
- **38 — mapped files count as in use**: `MacOpenFiles` also walks mapped regions, reused for 30 s.
- **Fresher's own process is left out of the open files** (03): the cleaner holds the item it checks; the pipeline
  holds no handle while it takes a snapshot.
- **`/Users` is never touched, with carve-outs** (03): the user's own home and `/Users/Shared` are carved out of a
  `never` `/Users`, so other users' homes stay protected without listing them.
- **The sandbox marker is opt-in in fixtures** (02): `FixtureTree::with_sandbox`, so no scan test or golden file
  changed.
- **The journal's cap is a threshold, not a split** (05/03): a file past 4 MiB rotates at the next `begin`, never
  inside a cleanup.
- **Group-writable means "needs an administrator"** (006/05/01, used by the guard): write access comes from the
  owner's or the others' bits; a group never counts, so the check fails closed.
- **The Dock's Quit is not held** (07): the app menu's Quit and ⌘Q are replaced by an item that asks; the Dock's Quit
  cannot be held and stops after the item in flight, the history showing the cleanup as interrupted.
- **To check from 07**: the "how" lines are the UI's texts; a mixed plan of Trash and tool items is titled "Move N
  items to the Trash?"; a tool-only plan's "Remove" button is red like the permanent one.

### Deviations from the design

- The landed path from the held handle (call 37) changed plan 02's `Trash` port; put back adds
  `RENAME_NOFOLLOW_ANY`; the process's name comes from its app bundle (call 35).
- The permanent delete, the emptying of a folder and check 11's walk act beneath held folders through
  `fresher-macos`'s `beneath` module (`openat`, `fstatat`, `unlinkat`), not `std::fs`; the cleaner depends on
  `fresher-macos` (the known issue `cleaner-act-by-path` fixed).
- The core e2e keeps one golden file per case (`fixtures/golden/cleanup-<case>.json`).
- Found while writing the docs (08/01) and fixed: the cleaner's re-check refused fewer forced tool arguments than the
  plan builder; both now share one rule, `-rf` included.

### Known issues left

- [`app-quit-during-cleanup.md`](../../known-issues/app-quit-during-cleanup.md): only the Dock's Quit does not ask.
- [`cleaner-other-users-processes.md`](../../known-issues/cleaner-other-users-processes.md): the in-use check sees only
  the user's own processes, and not a file opened after the last snapshot.
- [`cleaner-trash-call-by-path.md`](../../known-issues/cleaner-trash-call-by-path.md): the system's Trash call takes a
  path; a swapped parent is detected after the move.
- [`core-fifo-read-as-file.md`](../../known-issues/core-fifo-read-as-file.md): a FIFO in a scanned folder is read as a
  regular file.

### Left to later plans

- The sandbox run by hand: [deferred](../../deferred/removal-sandbox-run.md), before the first release.
- Plan [006](../006-permissions/readme.md): its probes by the user (01); the macOS adapter (03), whose `SystemUi`
  replaces the stand-in behind the `reveal_in_trash` command; the UI (04); "needs an administrator" in the UI and its
  docs (05/02–05/03).
- Plan [005](../005-settings/readme.md): the settings screen, so the scan roots stop being set in `settings.json`.
- Plan [007](../007-safe-development/readme.md): its scenarios, the separate identity, the debug panel and the CLI.
- Plan [009](../009-system-knowledge/readme.md): app ownership in the in-use check; plan
  [010](../010-history/readme.md): reading the journal for history; plan [012](../012-storage-map/readme.md): the
  storage map's nodes.
