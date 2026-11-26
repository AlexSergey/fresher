# 026 — The development run record

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** app, app/tauri, app/ui, core, cleaner, store, scripts, docs

Drafted on 2026-11-19 as one file and approved by the user the same day with its six agent's calls, which became
decisions. The design, its decisions and the agent's calls made while writing it from the code are in
[`00-design.md`](./00-design.md).

## Goal

The runs by hand on the real Mac ([`../../manual-testing.md`](../../manual-testing.md), section 2) ask the user to
read the window line by line against shell commands. This plan turns them into: start the development build, scan,
press clean, hand over one file.

- **The dry run**: a development build on the real Mac never moves, empties, deletes or runs anything. Its cleaner
  gets a new reach, `Reach::DryRun` (debug builds only), under which every check of a release build runs right before
  each item and the act is replaced by the outcome "skipped: a dry run" (`SkipReason::DryRun`); a refusing Trash port
  is a second wall. Sandbox mode keeps its real cleanups.
- **The record**: when the launcher names a folder (`FRESHER_DEV_RUNS`), a development build writes each run to
  `NO_COMMIT/dev-runs/<YYYY-MM-DD_HH-MM-SS>_<version>+<commit>[-dirty]_<mac|sandbox-<name>>.json`, never overwritten,
  with an append-only `.events.jsonl` beside it: the session, the settings, the access, every action the UI sent to
  the core, every scan's findings, left-outs and coverage, and every cleanup by module group with each item's
  would-be outcome. `npm run dev` becomes the launcher that sets the folder and the build's commit.
- **The checker**: `npm run dev-runs:check -- <file>` checks every finding and every would-be removal against the
  disk, read-only, and writes `<file>.check.md`.

A release build compiles neither the recorder nor the dry run, and reads none of the variables.

Branch: `feat/026-dev-run-record` (from `main` b78d2cd), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [003](../003-base-ui/readme.md) (commands),
[006](../006-permissions/readme.md) (access), [007](../007-safe-development/readme.md) (sandbox mode) and
[008](../008-safe-removal/readme.md) (the cleaner, the journal), all in `main`.

## Plans

| #   | Plan                                                    | Status      | Depends on | What                                                                                                                           |
| --- | ------------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 01  | [The dry run](./01-dry-run/readme.md)                   | ✅ approved | —          | `SkipReason::DryRun`; `Reach::DryRun` in the cleaner; the development composition and its refusing Trash; the docs of removal |
| 02  | [The recorder](./02-recorder/readme.md)                 | ✅ approved | 01         | the record's files and shape; the recorder in the shell; the launcher                                                          |
| 03  | [Checker and finish](./03-checker-and-finish/readme.md) | ✅ approved | 01, 02     | the checker script; `docs/manual-testing.md` section 2 rewritten around the record; the run by hand                            |

### Order and parallel work

```
01/01 ── 01/02 ── 02/01 ── 02/02 ──┬── 03/02 ── 03/03
                  02/03 ───────────┤
                  03/01 ───────────┘
```

- **Wave 1**: 01/01 (`crates/core`, `crates/store`, `crates/cleaner`, the regenerated bindings).
- **Wave 2**: 01/02 (`app/tauri/src/{compose,dry_run,lib,testing,commands}.rs`, the removal docs).
- **Wave 3, in parallel**: 02/01 (`app/tauri/src/dev_runs/` only, beside `lib.rs`'s module line and
  `app/tauri/Cargo.toml`), 02/03 (`scripts/dev.ts`, `scripts/dev.test.ts`, `scripts/sandbox-dev.ts`, `package.json`, docs) and 03/01
  (`scripts/dev-runs-check.ts`, its test, `package.json`). 02/03 and 03/01 both add a line to `package.json`'s
  `scripts`: the second to merge rebases.
- **Wave 4**: 02/02 (`app/tauri/src/dev_runs/`, `compose.rs`, `lib.rs`), after 02/01.
- **Wave 5**: 03/02, then 03/03.

Which files each plan touches:

| Plan | Owns                                                                | Shared files it touches                                                                                                                                                                                                                                                                                                          |
| ---- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `app/tauri/src/dry_run.rs`                                          | `crates/core/src/cleanup/outcome.rs`, `crates/core/locales/en.json`, `crates/store/src/journal/records.rs`, `crates/cleaner/src/{reach,pipeline,put_back,lib,test_support}.rs`, `crates/cleaner/tests/{reach,never}.rs`, `app/tauri/src/{compose,lib,testing,commands}.rs`, `app/ui/src/ipc/bindings.generated.ts`, `docs/architecture/{README,removal}.md`, `docs/restrictions/deletion-safety.md` |
| 02   | `app/tauri/src/dev_runs/`, `scripts/dev.ts`, `scripts/dev.test.ts` | `app/tauri/src/{compose,lib}.rs`, `app/tauri/Cargo.toml`, `Cargo.lock`, `scripts/sandbox-dev.ts`, `package.json`, `docs/{testing,dependencies}.md`, `docs/restrictions/privacy.md`, `docs/architecture/README.md`, `docs/deferred/`                                                                                                                       |
| 03   | `scripts/dev-runs-check.ts`, `scripts/dev-runs-check.test.ts`       | `package.json`, `docs/manual-testing.md`, `docs/testing.md`, `docs/plans/`                                                                                                                                                                                                                                                       |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- **A development build on the real Mac moves nothing**: the composition test (`compose::engine` on a temporary home,
  a marked folder included) and the cleaner's property test (`never.rs`, random trees, every method and action) leave
  every entry in place and every acted item `dry-run`; the user's run in 03/03 confirms it on this Mac (the offered
  folders are all still there after the clean button).
- **Every check still runs in a dry run**: a link, an ignored path, an item in use and a path outside the roots end
  with their own skip reasons, not `dry-run` (`pipeline.rs`).
- **A release build has neither**: `cargo clippy -p fresher-app -p fresher-cleaner --release --all-targets -- -D
  warnings` passes, and `cargo nextest run --release -p fresher-app` passes
  `a_release_build_ignores_the_dev_runs_variable` (nothing is written in a temporary folder named by the variable).
- **Nothing is written without `FRESHER_DEV_RUNS`**, and a record is never overwritten (the `dev_runs` tests).
- `npm run dev` on this Mac writes one record pair under `NO_COMMIT/dev-runs/` named with the date, the build and
  `mac`; `npm run dev:sandbox -- trial` writes one named `sandbox-trial`.
- `npm run dev-runs:check -- <the user's record>` writes its report; every difference it lists is explained in
  03/03's conclusions (a bug fixed, a rule of the checker corrected, or the disk changed since the scan).

## Ideas and deferred items this touches

- [A log file of a session](../../deferred/session-log-file.md): stays deferred; this plan's writer (a new file per
  run, an append-only event file, an atomic snapshot) and its English texts are there to reuse (02/03 writes it in).
- [The debug panel and the developer command line](../../deferred/dev-debug-panel.md): its "print a dry run" is
  answered on the real Mac by the dry run and the record; the panel and the rest stay deferred (02/03 writes it in).
- [The first real cleanups, in a sandbox](../../test-cases/sandbox-cleanup.md): unchanged; real cleanups in a
  development build stay in sandbox mode.

## Risks

- **The dry run is the only thing between a development build and the real disk.** Before this plan, check 1 refused
  every real-Mac item; after it, every check admits them and `act_on` stops the act. The stop is one branch after the
  checks, a refusing Trash behind it, and three layers of tests (unit, property, composition); 03/03's run confirms it
  by hand.
- **The record holds real paths of the user's Mac**: it is written only where the launcher says, under `NO_COMMIT/`,
  which git ignores; the privacy restriction names it (02/03). It is never sent anywhere.
- **The invoke hook runs before every command**: it must never block or fail the command. It takes the run's lock for
  one append; a failed write is logged.
- **A long scan's record is large** (every finding twice: the events and the snapshot). Measured in 03/03 on this
  Mac's home; a later change may drop findings from the events if it matters.

## Decisions

The user's, of 2026-11-19 (translated): the idea above; "add the build number to every run, and include the build
number and the date in the file name, so we know when and how it was run and nothing is overwritten"; work
autonomously with the recommended decisions — the draft's six agent's calls are approved and listed as decisions 3.1
to 3.6 in [`00-design.md`](./00-design.md). No version bump: 0.1.0.

## Conclusions

Not written yet.
