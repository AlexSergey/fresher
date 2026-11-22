# 037 — The Trash holds the freed space

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, engine, testing, sandbox, e2e, app, app/tauri, app/ui, docs

Written on 2026-11-22 from its one-file draft and from a read of the code on main 97d9b314. Approved by the user's
delegation of 2026-11-22: build autonomously, take the recommended decisions and the agent's calls; the user checks
everything later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Fresher moves items to the Trash and stops, so after a cleanup the disk is as full as before: macOS frees the space
only when the Trash is emptied. The user checks the Storage settings, sees nothing changed, and stops trusting every
other number in the app. A first release cannot ship that.

## Goal

1. The result of a cleanup says what was moved and what was actually freed, and why they differ (built by plan 008;
   this plan adds the invitation to look at the Trash).
2. The scan screen shows, per Trash folder, what waits there and how much of it Fresher's cleanups put there:
   "Waiting in the Trash: 23 GB, 20 GB of it from cleanups here". The first number is the last scan's measure plus
   what cleanups moved since; the second comes from the cleanup journal, found again in the Trash now. With Finder's
   30-day setting on, the date the app's share leaves by itself.
3. **Open Trash** opens it in Finder, with "Look at what is there and empty it yourself if you no longer need it".
   That is all the app does about the Trash: it never empties it and never offers to (decision 1).

New in the core: the `TrashView` type, the `FinderSettings` port, `SystemUi::open_folder`, the refusal
`UnknownTrash`, and the engine's `trash` and `open_trash`. The cleaner, the scanner, the store, the rules and every
module stay unchanged. No new dependency.

Branch: `feat/037-trash`, from main 97d9b314, rebased over plans 033 and 034 before its squash-merge into `main`
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md) (the journal,
`Located`), [012](../012-storage-map/readme.md) (the Trash's node in the scan) and [027](../027-scan-screen/readme.md)
(the scan screen).

## Plans

| #   | Plan                                    | Status      | Depends on | What                                                                        |
| --- | --------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------- |
| 01  | [Numbers](./01-numbers/readme.md)       | 🔨 in progress | —          | the types and ports, the engine's view and Open Trash, the commands         |
| 02  | [Screens](./02-screens/readme.md)       | 🔨 in progress | 01         | the Trash line on the scan screen, the result's invitation                  |
| 03  | [End to end](./03-end-to-end/readme.md) | 🔨 in progress | 01, 02     | the sandbox's Trash invariants and two case files, the core e2e, the UI e2e |
| 04  | [Finish](./04-finish/readme.md)         | 🔨 in progress | all        | the docs, the walks, the conclusions, the review list                       |

### Order and parallel work

```
01/01 ── 01/02 ── 01/03 ──┬── 02/01 ── 03/03 ──┬── 04/01
                          ├── 03/01 ───────────┤
                          └── 03/02 ───────────┘
```

- **Waves 1 to 3**: 01/01, 01/02, 01/03 in order (each needs the one before).
- **Wave 4, in parallel**: 02/01 (`app/ui/src` only), 03/01 (`app/tauri/tests/sandboxes*`, `fixtures/`), 03/02
  (`crates/e2e`).
- **Wave 5**: 03/03 (`app/ui/e2e`), after 02/01.
- **Wave 6**: 04/01.

Which files each plan touches:

| Plan | Owns | Shared files it touches |
| ---- | ---- | ----------------------- |
| 01   | `crates/core/src/cleanup/trash.rs`, `crates/system/src/finder_settings.rs`, `crates/engine/src/cleanup/trash{,_tests}.rs` | `crates/core/src/{ports.rs,error.rs,lib.rs,cleanup/mod.rs}`, `crates/core/locales/en.json`, `crates/testing/src/`, `crates/system/src/{lib.rs,system_ui.rs}`, `crates/macos/src/privacy/`, `crates/engine/src/`, every `CleanupPorts` site (`crates/e2e/tests/`, `app/tauri/src/{compose,sandbox,testing}.rs`), `app/tauri/` (commands, lib, build.rs, capabilities, dev runs), `app/ui/src/ipc/` |
| 02   | `app/ui/src/product/trash-line/`, `app/ui/src/screens/scan/use-trash.ts` | `app/ui/src/screens/scan/scan-screen.tsx`, `app/ui/src/product/result-summary/`, `app/ui/src/locales/en.json` |
| 03   | `fixtures/sandboxes/trash{,-empty}.toml`, `fixtures/golden/sandbox-trash{,-empty}.json`, `crates/e2e/tests/trash.rs`, `fixtures/homes/trash.toml` | `app/tauri/tests/sandboxes{.rs,/harness.rs}`, every `fixtures/golden/sandbox-*.json`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/` |
| 04   | `docs/known-issues/app-trash-waiting-stale-after-emptying.md` | `docs/` (architecture, testing, manual-testing, plans README, deferred, known issues, plan 010's draft) |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
or a rebase and run the checks. So are the sandbox goldens (`UPDATE_GOLDEN=1 npm run sandbox:test`, the diff
reviewed).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- `git diff 97d9b314 -- crates/cleaner crates/scanner crates/store crates/rules modules` is empty.
- No code path, port or command of the plan empties the Trash or deletes anything; no button of the UI names emptying
  (unit tests and an e2e scenario).
- `cargo nextest run -p fresher-engine` holds the Trash view's negatives (nothing counted for a gone, replaced, put
  back, deleted, tool or skipped item; no line for an empty Trash) before its positives.
- `npm run sandbox:test` passes for every case file, with the two Trash invariants and the `trash` section of each
  golden; it fails when a golden's Trash numbers are changed by hand (done once, written in 03/01).
- `cargo nextest run -p fresher-e2e --test trash` and `npm run test:e2e` pass.
- The walks of 04/01 are written for the user: `npm run dev:sandbox -- trash` and `-- trash-empty`, with and without
  `--real-trash`, and a dry run on the real Mac.

## Ideas this touches

This plan replaces the idea "The Trash holds the freed space" (2026-11-15, the agent's proposal); everything it held
is above. [External Drive Analyzer](../../ideas/external-drive-analyzer.md): the numbers per Trash folder, a second
volume's Trash measured only when a scan root holds it.

## Open questions

None.

## Decisions

1. **The app does not let the user empty the Trash** (the user, 2026-11-22): it only says that something waits
   there and invites the user to look and, if they want, empty it themselves. In line with "Trash only, never empty
   it" ([best practices](../../ideas/best-practices.md), 9).
2. **2026-11-22 — the user delegated**: build autonomously, the recommended decisions and agent's calls taken.

## Agent's calls (to check)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. "Moved", not "freed", with both numbers on the result (already built; only the invitation is added).
2. The Trash is information, never a category.
3. Open Trash is the only action, on the Trash line; the result keeps Show in the Trash with the invitation.
4. The Trash's size is the last scan's plus what cleanups moved since; no walk of its own.
5. Finder's 30-day setting read through `defaults`; the date given for the app's share only.
6. The confirmation is not changed: its sentences are already the draft's.
7. The sandbox proves the numbers through every golden and two invariants, with no new case-file table; an emptied
   journal entry is proven in the engine tests and the core e2e.
8. One line per Trash folder that holds something; the result keeps one measured number.
9. The store is not touched (the draft listed it); `fresher-macos` gains one argument helper only.
10. The line is refreshed when the window gains focus.
11. The history screen's sum stays per page.

## Conclusions

Built: the Trash line on the scan screen (what waits, how much of it came from cleanups here, the date Finder's
30-day setting gives, Open Trash), the invitation on the result, the core's `TrashView`, the `FinderSettings` port,
`SystemUi::open_folder`, the `unknown-trash` refusal and the engine's `trash` and `open_trash`. The app never empties a
Trash and nothing in the core can. The cleaner, the scanner, the store, the rules and every module are unchanged; no
new dependency.

Numbers (the last runs of each step): Rust `fresher-engine` 452, `fresher-app` 247, core, macos, system and testing
1093, `fresher-e2e` 171 (2 skipped); the engine's `cleanup/trash.rs` 91.97 % of lines; the sandbox 76; UI unit tests
1189; Playwright 624 passed with 1 known flaky chromium baseline. Pre-push on the last commit: lint, crate guard, knip, machete, the script tests and both coverage runs pass (Rust: 4123
tests with the 90 % bar met; UI: 98.62 % of lines, 92.66 % of branches); `check:deny` fails on the advisory
RUSTSEC-2026-0592 (`url`, "stack overflow in `Url::origin()` on nested `blob:` URLs", fixed in 2.5.9), which this plan
does not touch: `Cargo.lock` is the same as on main, so main fails it too. Updating `url` is a dependency change for
its own commit and the user's call. The core-change rule holds against main (empty diff).

Known issues written: [the stale first number](../../known-issues/app-trash-waiting-stale-after-emptying.md) and
[the sandbox clock](../../known-issues/app-sandbox-clock-restarts-each-launch.md). Awaiting the user's walk: items 1.17
and 2.15 of the manual testing doc and the plan's section of the review list.
