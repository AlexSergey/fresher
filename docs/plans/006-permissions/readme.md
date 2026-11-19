# 006 — Permissions and privileges

**Status:** 🔨 in progress (the user's runs by hand are pending) · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, macos, system, engine, testing,
app, app/ui, app-leftovers

## Goal

The app always knows what it may see and do, and says so honestly. A macOS permission is offered only next to the
result it limits, with a plain-English reason and the right System Settings pane one click away; everything works
without any permission, with the reduced coverage shown, never hidden. Modules declare what they need, the engine
resolves it, and only adapters built on `fresher-macos` (in `fresher-system`) know what macOS guards. The app never runs as administrator:
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
| 01  | [Probe the system](./01-probe-the-system/readme.md)     | ✅ approved    | —                                             | the user's run by hand: probes in `NO_COMMIT/` that verify "Probe results" (filled from documentation, 2026-11-19) |
| 02  | [Capability model](./02-capability-model/readme.md)     | ✔️ done        | — (02/04: 004/03)                             | core types and ports, fakes, availability, tool checks, opt-ins, coverage                                |
| 03  | [macOS adapter and shell](./03-macos-adapter/readme.md) | ✔️ done        | 01 (results), 02/01, 004/02 (03/04: 02)       | `fresher-macos::privacy`, the adapters in `fresher-system`, `app-leftovers` containers, commands, focus re-check, `Info.plist` |
| 04  | [UI](./04-ui/readme.md)                                 | ✔️ done        | 02, 03/04, 003, 005/03                        | the store, coverage notices, disabled sections, the Permissions group                                    |
| 05  | [Privileges](./05-privileges/readme.md)                 | ✔️ done        | 02/01, 004/03 (05/02: 03/04, 003; 05/03: all) | removability of findings, "needs an administrator" in the UI, the docs and the version                   |

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

**Remaining order (2026-11-19)**, one step at a time on `feat/006-permissions`: 03/01, 03/02, 03/05, 03/03, 03/04,
04/01, 04/02, 04/03, 04/04, 05/02, 05/03. Plan 01 is the user's run by hand and blocks nothing: plan 03 is built on
the "Probe results" filled from documentation, and its constants change if the run disagrees (agent's call 24).

Each parallel plan runs in its own worktree branched from `feat/006-permissions` and is merged back when it is done;
two agents never share a worktree. If plan 004 is not merged when wave 2 is due, 006 waits: it never creates
`crates/macos` itself.

Which files each plan touches, so parallel plans stay apart:

| Plan | Owns (only this plan edits it during its wave)                                                           | Shared files it touches                                                                                                 |
| ---- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| 01   | `NO_COMMIT/006-probes/`, the "Probe results" section of [`00-design.md`](./00-design.md)                 | —                                                                                                                       |
| 02   | `crates/core/src/access.rs`, `crates/core/src/coverage.rs`, `crates/engine/src/access.rs`, `crates/engine/src/coverage.rs` | `crates/core/src/{lib,ports,walk,category,module,event,command,error}.rs`, `crates/core/src/settings/core_schema.rs` (with 005/01), `crates/core/locales/en.json`, `crates/engine/src/{engine,session,registry}.rs`, `crates/testing/src/{fakes,lib}.rs`, `crates/e2e/`, `fixtures/`, `app/tauri/src/{adapters,compose}.rs` (stand-ins), `bindings.generated.ts` (regenerated), `docs/hacks/README.md`, `Cargo.lock` |
| 03   | `crates/macos/src/privacy/`, `crates/system/src/{access,system_ui,macos_version}.rs`, `app/tauri/Info.plist`, `app/tauri/tests/info_plist.rs` | `crates/macos/src/{lib,volume}.rs` (`places.rs` moves into `privacy/`), `crates/system/src/lib.rs`, `app/tauri/src/{adapters,compose,sandbox,testing,commands,shell,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `crates/scanner/src/scanner.rs` (a doc line), `crates/core/locales/en.json` (one text), `modules/app-leftovers/` (03/05), `crates/e2e/tests/app_leftovers.rs`, `fixtures/`, `docs/categories/app-leftovers-*.md`, `docs/deferred/`, `app/ui/src/ipc/`, `app/ui/src/test/demo-scan.ts`, `app/ui/e2e/core-mock.ts`, `bindings.generated.ts` (regenerated, never by hand), `docs/hacks/README.md`, `docs/architecture/README.md` |
| 04   | `app/ui/src/state/access.ts`, the new files of `app/ui/src/screens/permission/` (the notices, the Permissions group) | `app/ui/src/state/reduce.ts`, `app/ui/src/screens/permission/use-access.ts`, the overview, module and settings screens, `app/ui/src/product/{permission-card,category-card,category-grid,meaning-list}/`, `app/ui/src/locales/en.json` (chrome texts), `app/ui/e2e/core-mock.ts` |
| 05   | `crates/core/src/removability.rs`, `docs/architecture/permissions.md`                                    | `crates/core/src/{lib,finding,error}.rs`, `crates/core/locales/en.json`, `crates/engine/src/{engine,session}.rs`, `app/tauri/src/{commands,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, `app/ui/src/product/{finding-row,finding-list}/`, the module page, `docs/architecture/{README,modules}.md`, `docs/restrictions/{privacy,deletion-safety}.md`, `docs/testing.md`; no version file |

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
- `app-leftovers` never offers a container nor takes one as proof (03/05) once the walk may enter
  `Library/Containers`.
- By hand, on a real Mac (listed under "By hand" below; written in the conclusions when done, together with the
  user's run of plan 01).

## By hand

Open: the user's run on a real Mac. Nothing below was done by an agent. Run it with a build that keeps a stable
signature (see [`architecture/permissions.md`](../../architecture/permissions.md), "Developer builds"), and reset with
`tccutil reset All <bundle id>` between runs, on a development Mac only: `io.github.alexsergey.fresher.dev` for
`npm run dev` (its config sets that identifier), `io.github.alexsergey.fresher` for a built app. Write what happened,
with the macOS version, in the Conclusions below.

**Plan 01, the probes** ([`01-prompts-and-checks.md`](./01-probe-the-system/01-prompts-and-checks.md),
[`02-panes-and-reasons.md`](./01-probe-the-system/02-panes-and-reasons.md)), to turn each "unverified" row of
"Probe results" in the design into a measured one:

- [ ] Which places prompt and which only refuse without Full Disk Access (the table of places).
- [ ] The Full Disk Access check: the first candidate that does not answer "missing" is the right one, and it never
      prompts.
- [ ] A Files and Folders grant applies at once to the running app; a Full Disk Access grant applies at once or only
      after a relaunch (if at once, "Reopen Fresher" goes).
- [ ] The three pane addresses open the right pane, with the network off.
- [ ] The five reason strings appear in the prompts; data from other apps has a fixed text of its own.
- [ ] Under `npm run tauri dev` access is attributed to the terminal app; an ad-hoc rebuild loses a grant.
- [ ] Where a row disagrees, `PLACES`, `FDA_CANDIDATES` and `ADDRESSES` change in the same commit, and the design's
      "Probe results" are updated.

**Acceptance**, on the built app:

- [ ] A first scan shows no system prompt.
- [ ] The Full Disk Access notice opens the right pane of System Settings.
- [ ] Granting Full Disk Access while the app is open updates the notices (or offers "Reopen Fresher", per plan 01).
- [ ] Opting into Desktop in Settings prompts once, inside the next scan, with the reason string; allowing it shows
      Desktop findings; denying it shows a notice.
- [ ] A category that needs a missing permission is limited (a tooltip says what it misses); a category whose tool is
      not installed is unavailable with a plain reason.
- [ ] An item the app may not remove (for example one owned by root) is shown with its badge and no checkbox, and
      "Show in Finder" reveals it.
- [ ] The Permissions group of Settings lists every state truthfully and "Check again" updates it.
- [ ] A volume opt-in: a removable drive, then a network volume, added as a scan root prompts once inside the next
      scan with its reason string; allowing it shows its findings; denying it shows a notice whose "Open System
      Settings" opens Files and Folders.
- [ ] On a Mac before macOS 14, other apps' data is "Not needed on this Mac" in the Permissions group and has no
      notice.

## Conclusions

Everything an agent can build and check is built; what is open is the user's run on a real Mac (see "By hand"
above), so the plan stays 🔨 in progress until it is done and the user has looked at the result.

**What was built**

- The model (02): `Capability`, `Access`, `AccessStatus`, `Need`, `Availability` and the ports `AccessProbe` and
  `SystemUi` in `fresher-core`; `FakeAccess` and `FakeSystemUi`; the engine's availability (a module whose every
  category is unavailable is not called), tool checks and the coverage of a scan; four opt-in switches.
- The macOS half (03): `fresher-macos::privacy` (the table of guarded places, checks that never prompt, volume
  guards, the settings panes); `MacAccess` and `MacSystemUi` in `fresher-system`; the commands `refresh_access`,
  `open_access_settings`, `relaunch` and `reveal_finding`; the re-check on focus; `app/tauri/Info.plist` with exactly
  five reason strings; `app-leftovers` does not read app containers (open question 1, option (b), at the final
  review).
- The UI (04): the access state and the coverage in the store, one notice per capability that limited a scan, limited
  and unavailable categories and sections, the Permissions group of Settings.
- Privileges (05): `Removability` of every finding from the walk's entry facts, items that need an administrator
  shown without a checkbox and with "Show in Finder", and the docs ([`architecture/permissions.md`](../../architecture/permissions.md),
  "No administrator rights" in [`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)).

**Not verified on a real Mac.** The "Probe results" of the design, and so the constants built on them (`PLACES`,
`FDA_CANDIDATES`, `ADDRESSES`), are from documentation. Whether "Reopen Fresher" is needed is assumed. No prompt,
pane or grant was seen by an agent. Plans 03 and 04 were built and tested on fakes and fixtures only.

**Deviations** are the agent's calls 24–32 of the design and the decisions in each step's Conclusions (for example: the
walk's entry facts added to `Match` in 05/01; `NothingToReveal` in 05/02; the `container` row of `app-leftovers`
dropped at the final review, agent's call 33).

**Merged in part (2026-11-16)**: at the user's request ("merge everything and push"), the built parts — the capability
model with the coverage of a scan (02/01–02/04) and the removability of findings with the walk's entry facts (05/01) —
went into `main` on 2026-11-16 together with plan [008](../008-safe-removal/readme.md), which was built on top of
them, in its squash-merge. The rest — the probes by the user (01), the macOS adapter and shell (03), the UI (04), and
05/02–05/03 — continues on a new branch `feat/006-permissions` from `main`, squash-merged once when it is done
([`../../workflow.md`](../../workflow.md)). The stand-ins behind `AccessProbe` and `SystemUi` stay in the app until 03
replaces them ([`../../hacks/README.md`](../../hacks/README.md)).

**Final review fixes (2026-11-19)**: a scan root written in another case or reached through a link is told against
the guarded places as the walk opens it (03/03); "Check again" also runs the tool checks, the focus re-check does not
(03/04); open question 1 switched to option (b): `app-leftovers` has no `container` row, reads no container and
declares no need (agent's call 33, 03/05); a refused opt-in or volume offers "Open System Settings" with the core's new
`core.access.<id>.denied` text, in the notices and the Permissions group, which also shows its heading and switches
before the status is known (04/02, 04/04); dead UI code went (`hasStaleResult`, `useAccess`, 04/01). Recorded, not
fixed: three known issues ([I/O policy in a compose test](../../known-issues/app-compose-test-sets-the-io-policy.md),
[relaunch after a confirmed cleanup](../../known-issues/app-relaunch-after-a-confirmed-cleanup.md),
["Reopen Fresher" state is local](../../known-issues/ui-reopen-button-state-is-local.md)) and one deferred item
([steps as a list](../../deferred/access-steps-as-a-list.md)). Two checks were added to "By hand" (a volume opt-in, and
other apps' data before macOS 14). Playwright baselines regenerated and looked at:
`module-page-app-leftovers{,-chosen}` (2.5 GB, the data category with its support folder alone) and `settings-temp`
(the Permissions group, with the core's texts now in the temporary files mock) in both browsers, the WebKit focus of
`PermissionCard` (its first stop is now the "How" disclosure, which shows its ring) and the new one of
`PermissionRows`, and the gallery's `PermissionRows/all-granted` (no empty description under a tool without a note).
