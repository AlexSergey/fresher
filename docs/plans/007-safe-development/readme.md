# 007 — Safe development: sandboxes, cases and guards

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** testing, sandbox, cleaner, core, app,
app/ui, repo

Expanded on 2026-11-16 from the one-file draft of 2026-11-15, after the user's direction of the same day, and
**approved by the user on 2026-11-16** with the recommended answers to the three open questions and every agent's call
([`00-design.md`](./00-design.md), "Decisions"). The world outside the disk (the former plan 06) is
[deferred](../../deferred/sandbox-world.md). Building started on 2026-11-17, after plans 005 and 009 were merged into
`main` and `main` was merged into this branch.

## Goal

Every feature's positive and negative cases are declared once, in a case file of files and expectations. One generator
writes a case file into a sandbox folder; a development build started on that folder lives entirely inside it — its
home, known folders, scan roots, settings, journal and Trash — and cannot touch anything outside it. The autotests read
the same case files and check every case through the app's own composition: what is offered, with which danger level,
what a cleanup does and what it leaves. The generator also writes the checklist of a run by hand. Around it, the
draft's cheap guards: a separate identity for development builds, the home folder looked up in one place, modules that
start no process.

Branch: `feat/007-safe-development`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [004](../004-fast-scan/readme.md) and [008](../008-safe-removal/readme.md) (done) and on plans
[005](../005-settings/readme.md) (the data folder, the settings) and [009](../009-system-knowledge/readme.md) (known
folders, the system fakes), both merged into `main` on 2026-11-16.

## Plans

| #   | Plan                                          | Status    | Depends on | What                                                                                       |
| --- | --------------------------------------------- | --------- | ---------- | ------------------------------------------------------------------------------------------ |
| 01  | [Case files](./01-case-files/readme.md)       | ✅ approved | —          | the tree vocabulary made reusable, the case format and its checks, the generator           |
| 02  | [Sandbox mode](./02-sandbox-mode/readme.md)   | ✅ approved | 01/03, 04/01 | `Reach::SandboxFolder`, the sandbox composition, the banner, `npm run dev:sandbox`         |
| 03  | [Autotests](./03-autotests/readme.md)         | ✅ approved | 01, 02/02  | one test per case file through the app's sandbox composition; the first case files         |
| 04  | [Guards](./04-guards/readme.md)               | ✅ approved | —          | the development identity, the home lookup in one place, modules without processes          |
| 05  | [Finish](./05-finish/readme.md)               | ✅ approved | all        | the docs and the rule for later plans, a run by hand, the conclusions             |

### Order and parallel work

```
01/01 ── 01/02 ── 01/03 ──┐
02/01 ────────────────────┴── 02/02 ──┬── 02/03 ──────────┐
                                      ├── 02/04 ──────────┤
04/01 ── 04/02 ───────────────────────│──┘               ├── 05
                                      └── 03/01 ── 03/02 ─┘
```

- **Wave 1, in parallel**: 01 (its steps in order), 02/01 (the cleaner's reach and the core's joined protection: no
  file shared with 01) and 04 (separate files).
- **Wave 2**: 02/02 (the sandbox composition needs the generator's layout and 02/01's reach), then in parallel 02/03
  (the banner), 02/04 (the command, also after 04/01) and 03 (the autotests).
- **Wave 3**: 05.
- **The world outside the disk** (the draft's plan 06) is not in this plan: it is
  [deferred](../../deferred/sandbox-world.md) (decision 5); its design stays in section 10.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns                                                                                     | Shared files it touches                                                                                      |
| ---- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| 01   | `crates/sandbox/`, `crates/testing/src/fixture_tree.rs`                                  | `Cargo.toml` (members), `Cargo.lock`, `package.json` (`sandbox`), `.gitignore`, `scripts/check-crate-deps.ts` (the new crate), `docs/testing.md` ("Fixture homes") |
| 02   | `crates/cleaner/src/reach.rs`, `app/tauri/src/sandbox.rs`, `app/ui/src/product/build-banner/`, `scripts/sandbox-dev.ts` | `crates/core/src/cleanup/protection.rs`, `crates/cleaner/src/{lib,guard/mod}.rs`, `crates/cleaner/tests/never.rs`, `app/tauri/src/{compose,lib,commands}.rs`, `app/tauri/Cargo.toml`, `app/ui/src/ipc/bindings.generated.ts`, `app/ui/src/screens/app-frame*`, `package.json`, `docs/hacks/README.md` |
| 03   | `app/tauri/tests/sandboxes.rs`, `fixtures/sandboxes/`, `fixtures/golden/sandbox-*.json`   | `app/tauri/Cargo.toml` (dev-dependencies), `package.json` (`sandbox:test`)                                   |
| 04   | `app/tauri/tauri.dev.conf.json`                                                          | `clippy.toml`, `app/tauri/src/lib.rs`, `package.json` (`dev`), `scripts/check-crate-deps.ts` and its tests, `docs/quality.md`, `docs/architecture/README.md` |
| 05   | —                                                                                        | `docs/` (testing, architecture, restrictions, hacks, rules, plans README, deferred)                         |

`tauri.dev.conf.json` is made by 04/01; 02/04 only names it. `Cargo.lock` and `package-lock.json` are never merged by
hand: on a conflict take either side, regenerate them and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- `npm run sandbox -- trial` writes `sandbox/trial/` with its marker, its `home/` and `EXPECTED.md`; run again, it
  changes nothing it did not make and removes nothing; it refuses a non-empty folder without a marker and the real home.
- `npm run dev:sandbox -- trial`: the banner names the sandbox; the scan sees only `sandbox/trial/home`; the
  `EXPECTED.md` checklist holds by hand — every negative never offered, every positive in its category with its danger
  level, the cleanup puts the positives into `sandbox/trial/home/.Trash`, put back works, the negatives are untouched;
  the settings and the journal are in `sandbox/trial/.fresher-data`; the real Trash and the development build's own data
  folder did not change.
- With `--real-trash` the same cleanup lands in the real Trash and nothing else changes.
- `npm run sandbox:test` runs one test per case file; each fails when a case's expectation is broken (checked by
  editing a case on purpose once); a registered module without a case file fails the app's test.
- `npm run dev` runs with the development identifier; a debug build with the release identifier refuses to start.
- A release build ignores `FRESHER_SANDBOX`.

## Ideas this touches

- [Best practices](../../ideas/best-practices.md): the demo mode and automatic screenshots (a sandbox is a stable world
  to photograph by hand; the screenshots of the UI e2e stay on the mocked core), the dry run.
- The [cleanup simulator](../../ideas/cleanup-simulator.md): a sandbox is a place to rehearse a cleanup for real.
- Plan [014](../014-onboarding/readme.md)'s sample data may reuse the case format; it runs in release builds, where this
  plan's mode does not exist.

## Conclusions

**Merged into `main` on 2026-11-17 at the user's request, before the run by hand** (05/02): the plan stays 🔨 in
progress until the user has run its checklist; the conclusions of 05/02 are written then, on `main`.

Built on `feat/007-safe-development` on 2026-11-16 and 2026-11-17, task by task, each commit through the hooks and each task
reviewed; a whole-branch review then led to one fix wave and a re-review.

- **What is there now**: the case-file vocabulary and generator (`fresher-sandbox`, `npm run sandbox`), the sandbox's reach
  in the cleaner with one shared marker reader in `fresher-macos`, sandbox mode and the banner of a development build
  (`AppInfo`/`Build` in `fresher-core`, the `app_info` command), `npm run dev:sandbox`, the autotest of every case file with
  the first two (`trial`, `cleanup-guards`), the development build's own identity, and the lints (one home lookup, no
  process from a module).
- **Changed from the design while building** (each in its step's conclusions): `sandbox_engine` takes `real_data_folder`
  (02/02); `AppInfo` lives in the core (02/03); the Documents positive became a negative (03/02); the home lookup needed
  a named exception in the cleaner's deletion-point test (04/02); the fake Trash never overwrites (final review); the
  command `app_version` became `app_info` (final review).
- **Measured**: `npm run pre-push` passes on the final commit with coverage at or above 90 % (1694 Rust tests, 4 skipped;
  UI lines 98.36 %, branches 92.05 %); `npm run test:e2e` passes (core 34, 1 skipped; UI 225, no baseline changed by the fix
  wave); `npm run sandbox:test` 11 passed.
- **Left**: the run by hand (05/02) is waiting on the user. The known issue
  [the scan offers an item in the Trash](../../known-issues/app-scan-offers-an-item-in-the-trash.md); the deferred
  [development debug panel](../../deferred/dev-debug-panel.md); the [sandbox world](../../deferred/sandbox-world.md) stays
  deferred. Documents positives wait for plan 006. No version bump (0.1.0).
- **For the user to check**: the design's agent's calls, and the list below.

**Agent's calls taken during the build (to check):**

- Adapt the step text to the code as it is in `main` (plans 005 and 009 merged after this plan was written). Costs, if wrong: a reviewer flags a drift between text and code.
- No version bump anywhere; the version stays 0.1.0 (the user's decision of 2026-11-16 overrides any older number left in a step). Costs, if wrong: nothing, a version is set later.
- `EXPECTED.md` names categories by id, not title: the sandbox crate cannot read the modules' texts (crate guard). Costs, if wrong: a less friendly checklist; revisit with the run in the app.
- The one marker reader moved to `fresher-macos` in 02/01 (which owns the reach), replacing the cleaner's and the generator's copies. Costs, if wrong: none; if skipped, two safety checks drift apart.
- 05/02 runs only the release-gating tests under `--release`: the demo and trial modules exist in debug builds only, so 12 older tests of `fresher-app` cannot pass in release. Costs, if wrong: none, they are debug tests.
- The sandbox avoids its own home's places that prompt (faithful to the real app before plan 006 grants), so the Documents case became a negative and the one-by-one positive moved to `Library/Caches`. Costs, if wrong: Documents positives wait for plan 006.
- The scan offering an item inside `~/.Trash` (then left out as protected) is recorded as a known issue, not fixed here. Costs, if wrong: one confusing finding stays until the product question is decided.
- The home lookup in the deletion-point guard is allowed in exactly two files, and only as `let home = <path>home_dir()` (tightened after the re-review). Costs, if wrong: a third legitimate lookup needs the guard edited, on purpose.
