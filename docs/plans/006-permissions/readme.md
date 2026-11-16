# 006 — Permissions and privileges

**Status:** 🔨 in progress · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, macos, engine, testing, app,
app/ui

## Goal

The app always knows what it may see and do, and says so honestly. A macOS permission is offered only next to the
result it limits, with a plain-English reason and the right System Settings pane one click away; everything works
without any permission, with the reduced coverage shown, never hidden. Modules declare what they need, the engine
resolves it, and only adapters built on `fresher-macos` know what macOS guards. The app never runs as administrator:
items that need one are shown with their size and a Finder instruction, never cleaned. The design:
[`00-design.md`](./00-design.md).

Branch: `feat/006-permissions`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plan [002](../002-base-architecture/readme.md) (ports, events, the engine, IPC); plan
[004](../004-fast-scan/readme.md) (`fresher-macos`, the walk's exclusions and entry facts) for plans 03, 02/04 and 05;
plans [003](../003-base-ui/readme.md) and [005](../005-settings/readme.md) (components, the settings screen) for plan
04 and step 05/02. Plans 01 and 02/01–02/03 need only plan 002 and can start at once.

## Plans

| #   | Plan                                                    | Status         | Depends on                                    | What                                                                                                     |
| --- | ------------------------------------------------------- | -------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| 01  | [Probe the system](./01-probe-the-system/readme.md)     | ✅ approved    | —                                             | throwaway probes in `NO_COMMIT/`: which places prompt, the Full Disk Access check, panes, reason strings |
| 02  | [Capability model](./02-capability-model/readme.md)     | ✔️ done        | — (02/04: 004/03)                             | core types and ports, fakes, availability, tool checks, opt-ins, coverage                                |
| 03  | [macOS adapter and shell](./03-macos-adapter/readme.md) | ✅ approved    | 01, 02/01, 004/02 (03/04: 02)                 | `fresher-macos::privacy`, the app's adapters, commands, focus re-check, `Info.plist`                     |
| 04  | [UI](./04-ui/readme.md)                                 | ✅ approved    | 02, 03/04, 003, 005/03                        | the store, coverage notices, disabled sections, the Permissions group                                    |
| 05  | [Privileges](./05-privileges/readme.md)                 | 🔨 in progress | 02/01, 004/03 (05/02: 03/04, 003; 05/03: all) | removability of findings, "needs an administrator" in the UI, the docs and the version                   |

### Order and parallel work

```
002 ──┬── 01 ───────────────────────┐
      └── 02/01 ─┬─ 02/02 ── 02/03 ─┤
                 │                  ├── 03/01–03/03 ── 03/04 ──┬── 04 ─────┐
004/02 ──────────┼──────────────────┘                          ├── 05/02 ──┼── 05/03 ── conclusions
004/03 ──────────┴── 02/04 , 05/01 ────────────────────────────┘           │
003, 005/03 ───────────────────────────────────────────────────────────────┘
```

- **Wave 1, in parallel** (needs plan 002 only): 01 (probes, by hand, no product code) and 02/01–02/03 in order.
- **Wave 2, in parallel** (needs 01 done and `fresher-macos` from 004/02; 004/03 for the walk): 03/01–03/03 in order;
  02/04 and 05/01 (both on the walk's new inputs and facts).
- **Wave 3**: 03/04 (commands, focus, `Info.plist`, bindings), after 02 and 03/03.
- **Wave 4, in parallel** (needs plans 003 and 005/03): 04 in order, and 05/02.
- **Wave 5**: 05/03 (docs, version), then the conclusions.

Each parallel plan runs in its own worktree branched from `feat/006-permissions` and is merged back when it is done;
two agents never share a worktree. If plan 004 is not merged when wave 2 is due, 006 waits: it never creates
`crates/macos` itself.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                           | Shared files it touches                                                                                                 |
| ---- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| 01   | `NO_COMMIT/006-probes/`, the "Probe results" section of [`00-design.md`](./00-design.md)                 | —                                                                                                                       |
| 02   | `crates/core/src/access.rs`, `crates/core/src/coverage.rs`, `crates/engine/src/access.rs`, `crates/engine/src/coverage.rs` | `crates/core/src/{lib,ports,walk,category,module,event,command,error}.rs`, `crates/core/src/settings/core_schema.rs` (with 005/01), `crates/core/locales/en.json`, `crates/engine/src/{engine,session,registry}.rs`, `crates/testing/src/{fakes,lib}.rs`, `crates/e2e/`, `fixtures/`, `app/tauri/src/{adapters,compose}.rs` (stand-ins), `bindings.generated.ts` (regenerated), `docs/hacks/README.md`, `Cargo.lock` |
| 03   | `crates/macos/src/privacy/`, `app/tauri/src/access.rs`, `app/tauri/Info.plist`                           | `crates/macos/src/lib.rs`, `app/tauri/src/{adapters,compose,commands,shell,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, `bindings.generated.ts` (regenerated, never by hand), `docs/hacks/README.md` |
| 04   | `app/ui/src/screens/access/` (the notices, the cards, the Permissions group)                             | `app/ui/src/state/`, the overview, module and settings screens of plan 003/005, `app/ui/src/gallery/`, the dictionary of `crates/core/locales/en.json` (texts only) |
| 05   | `crates/core/src/removability.rs`, `docs/architecture/permissions.md`                                    | `crates/core/src/{lib,finding,error}.rs`, `crates/core/locales/en.json`, `crates/engine/src/{engine,session}.rs`, `app/tauri/src/{commands,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, the module page, `docs/architecture/{README,modules}.md`, `docs/restrictions/{privacy,deletion-safety}.md`, `docs/testing.md`, the version files |

The core types of 02/01 are frozen once that step is merged: a contract missing later stops the step that needs it, and
the change goes into `crates/core` in the main session. `Cargo.lock` is never merged by hand: on a conflict take either
side, regenerate it and run the checks.

## Ideas this touches

[macOS specifics](../../ideas/architecture/macos.md) (Full Disk Access by probing, re-check on focus),
[scanning](../../ideas/architecture/scanning.md) (errors by kind, "312 folders could not be read"), the
[smart uninstaller](../../ideas/smart-uninstaller.md) (App Management, left to its plan), the
[external drive analyzer](../../ideas/external-drive-analyzer.md) (removable and network volumes as an opt-in), and
honest sizes from [best practices](../../ideas/best-practices.md) (`42 GB+`).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- With every capability faked in every state, the engine's availability, coverage and refusals match the design; no
  test asks macOS for anything or reads the real home folder.
- A scan never enters a guarded place that is not granted or opted into; the result names what was not seen and why.
- `Info.plist` holds exactly the five reason strings of the design and no other usage key.
- By hand, on a real Mac (written in the conclusions): a first scan shows no system prompt; the Full Disk Access card
  opens the right pane; granting it while the app is open updates the notices (or offers "Reopen Fresher", per plan
  01); opting into Desktop prompts once, inside the next scan.

## Conclusions

Not written yet for the plan: 01, 03, 04 and 05/02–05/03 are open.

**Merged in part (2026-11-16)**: at the user's request ("merge everything and push"), the built parts — the capability
model with the coverage of a scan (02/01–02/04) and the removability of findings with the walk's entry facts (05/01) —
went into `main` on 2026-11-16 together with plan [008](../008-safe-removal/readme.md), which was built on top of
them, in its squash-merge. The rest — the probes by the user (01), the macOS adapter and shell (03), the UI (04), and
05/02–05/03 — continues on a new branch `feat/006-permissions` from `main`, squash-merged once when it is done
([`../../workflow.md`](../../workflow.md)). The stand-ins behind `AccessProbe` and `SystemUi` stay in the app until 03
replaces them ([`../../hacks/README.md`](../../hacks/README.md)).
