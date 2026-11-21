# 015 — Module groups and ignoring a module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, modules, testing, app, app/ui

Written on 2026-11-17 at the user's request and approved the same day by delegation: the user asked the agent to take
the recommended answer to every open question and record it ([`00-design.md`](./00-design.md), "Decisions"). The
agent's calls wait for the user's check.

## Goal

Every module belongs to a named group (Development first; the dev-only modules in Debug), and the sidebar shows the
modules under their group. The user can ignore a whole module or a whole group as plan 005 ignores a category: its
findings leave every answer at once, are never removed, and come back without a rescan. Settings shows the ignore
lists in three tabs — Ignored groups, Ignored modules, Ignored items — with a toggle for every group and module. Plan
[016](../016-jetbrains/readme.md) (JetBrains IDE leftovers) puts the first real module in the group Development.

Branch: `feat/015-module-groups`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
**Built after plan [011](../011-diagnostics/readme.md) is merged into `main`**, on this branch made again from `main`
then (decision 9), and **before plan 016**. Depends on plans [002](../002-base-architecture/readme.md),
[003](../003-base-ui/readme.md), [005](../005-settings/readme.md) and [012](../012-storage-map/readme.md) (done or
merged).

**Since plan 027 (2026-11-20)** the module pages and the sidebar's module groups are gone: a module is ignored from
its card's settings menu on the Scan screen and restored in Settings → Ignored, and a category's details replace its
section of the module page.

## Plans

| #   | Plan                                                  | Status                                  | Depends on | What                                                                                         |
| --- | ----------------------------------------------------- | --------------------------------------- | ---------- | -------------------------------------------------------------------------------------------- |
| 01  | [Groups](./01-groups/readme.md)                       | ✔️ done        | —          | `ModuleGroup` in the descriptor and `ModuleInfo`; the sidebar, chips and settings by group   |
| 02  | [Ignoring a module and a group](./02-ignore/readme.md) | ✔️ done        | 01/01      | the two lists in the core and the engine; the three tabs; the ignored module's look; e2e     |
| 03  | [Finish](./03-finish/readme.md)                       | 🔨 in progress | all        | the docs, a run by hand, the conclusions                                                     |

### Order and parallel work

```
01/01 ──┬── 01/02 ─────────────────────────────┐
        └── 02/01 ──┬── 02/02 ──┬── 02/04 ─────┴── 03/01 ── 03/02
                    └── 02/03 ──┘
```

- **Wave 1**: 01/01 (the core type and the descriptor; every later step leans on it).
- **Wave 2, in parallel**: 01/02 (`app/ui` only: sidebar, chips, settings' module groups) and 02/01 (core and engine
  only).
- **Wave 3, in parallel**: 02/02 (the Settings tabs: `screens/settings/`, `design/layout/tabs/`) and 02/03 (the
  ignored module's look: `screens/module/`, `screens/frame-sidebar.tsx`). 02/03 starts after 01/02, which edits the
  sidebar first. Both add keys to `app/ui/src/locales/en.json`: the second rebases on the first.
- **Wave 4**: 02/04 (the core e2e and the Playwright scenarios over everything above), then 03.

Which files each plan touches, so parallel work stays apart:

| Plan | Owns                                                                                   | Shared files it touches                                                                                                                                                                                              |
| ---- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/group.rs`                                                             | `crates/core/src/{lib,module,command,texts}.rs`, `crates/core/locales/en.json`, `crates/core/tests/typescript.rs`, `crates/engine/src/registry.rs`, `modules/{demo,trial}/src/lib.rs`, every test that builds a descriptor, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/screens/{frame-sidebar.tsx,app-frame.module.css}`, `app/ui/src/screens/overview/module-progress.ts`, `app/ui/src/product/scan-progress/`, `app/ui/src/screens/settings/settings-groups.ts`, `app/ui/src/test/demo-scan.ts`, `app/ui/src/locales/en.json` |
| 02   | `app/ui/src/design/layout/tabs/`, `app/ui/src/screens/settings/ignored-{groups,modules,items}.tsx` | `crates/core/src/{settings,command,error}.rs`, `crates/core/src/settings/core_schema.rs`, `crates/core/locales/en.json`, `crates/engine/src/{ignore,settings,results}.rs`, `crates/engine/src/storage/*_tests.rs`, `crates/e2e/tests/ignore.rs`, `fixtures/golden/ignore.json`, `app/ui/src/ipc/{index,mock}.ts`, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/screens/settings/{ignored-group.tsx,settings-screen.tsx}`, `app/ui/src/screens/module/`, `app/ui/src/screens/frame-sidebar.tsx`, `app/ui/src/screens/overview/overview-screen.tsx`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts,keyboard.spec.ts}`, `app/ui/e2e/screenshots/` |
| 03   | —                                                                                      | `docs/` (architecture, testing, ideas, plans README)                                                                                                                                                                 |

**Overlap with plan 016** (built after this one): 016 owns `modules/jetbrains/` and the sandbox's app vocabulary; it
uses `ModuleGroup::Development` from 01/01 and the ignore kinds from 02/01. Both touch `app/ui/e2e/core-mock.ts` and
`flows.spec.ts` (016 after this plan is in `main`). `bindings.generated.ts`, `Cargo.lock` and `package-lock.json` are
never merged by hand: regenerate them after a merge and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- The sidebar shows "Debug" with `demo` and `trial` under it in a development build; a release build registers no
  `Debug` module (the registry's check is tested).
- Ignoring the module `trial`, then the group `Debug`: their findings leave the overview, the module pages, the review
  and the storage map's explained part at once; Restore brings them back without a rescan; the lists survive a
  restart (`crates/e2e/tests/ignore.rs`).
- Settings → Ignored has three tabs; each group and module has its switch; a module of an ignored group is disabled
  with its note; plan 005's categories and paths are in "Ignored items", unchanged.
- `npm run test:e2e` passes with the six new scenarios; the screenshots of the tabs and the ignored module are
  reviewed.
- `npm run sandbox:test` passes with no golden changed.

## Ideas this touches

- The [ignore list](../../ideas/ignore-list.md): "an Ignore button on a section" becomes a module and a group; the
  item's button stays [deferred](../../deferred/ignore-an-item-button.md).
- The [developer environment cleaner](../../ideas/developer-environment-cleaner.md): its "one section per tool" sits
  in the group Development.

## Conclusions

**Merged into `main` on 2026-11-17 at the user's request, before the run by hand** (03/02): the plan stays 🔨 in
progress until the user has run its checklist; the results are written then, on `main`.

**Built.** `ModuleGroup` (Development, Debug) is part of every descriptor and of `ModuleInfo`; the registry refuses a
`Debug` module in a release build. `core.ignore.modules` and `core.ignore.groups` sit beside plan 005's two lists; a
group hides its modules, a module restored inside an ignored group stays hidden, restoring needs no rescan, the lists
survive a restart and a pending cleanup plan is killed by any change. The sidebar, the scan's chips and Settings show
modules by group; an ignored module is dimmed with "Ignored" and its page has "Ignore this module" or the note with
Restore. Settings → Ignored has three tabs (the new `Tabs` component): "Ignored groups", "Ignored modules", "Ignored
items". No version bump (0.1.0).

**The user's decision of 2026-11-17 on the Settings tabs**: Settings gets two tabs for ignoring, one for groups and one
for specific modules, each listing every group or module with its toggle and restore, consistent with plan 005's
Ignored group, with UI tests and e2e scenarios. Plan 005's lists went to a third tab.

**Changed from the design.** The overview carries `hiddenModules` (`{ module, by }`), computed in the engine, instead
of the UI reading the ignore lists. The golden reports gained `"hiddenModules": []` (and `groups`, `modules` in the
ignored counts).

**Measured.** Workspace nextest 1938 passed (after 02/01); vitest 995 passed; Playwright 265 passed (02/04); the
sandbox 21/21 with no golden changed (01/01). Coverage: the pre-push run on the last commit measures it; not written
here until then.

### Agent's calls taken during the build (to check)

- No contract check of the group texts in `fresher-testing`: the core's texts test already requires every group's
  name and description. Cost, if wrong: one check in `contract.rs`.
- `PageHeader` is the only title of a module page; the module section keeps an aria-label and shows the size and the
  scan state. Cost, if wrong: a `nameless` prop and a heading less on the page.
- Hidden modules come from the core (`Overview.hiddenModules`), not from the UI. Cost, if wrong: a field in the
  overview and the engine's `hidden_modules`.
- `check_group` takes a `bool` build flag, private to the registry. Cost, if wrong: an enum in one function.
- `Ignored.modules` is a `Vec<String>`, so an id no longer registered is still counted and can be restored; it hides
  nothing. Cost, if wrong: a stale entry cannot be restored, or the count drops it.
- `restore_ignored` refuses nothing (restoring what is not listed is a no-op); `ignore` refuses an unregistered
  module or a group with no registered module. Cost, if wrong: a refusal in one command.
- Test modules other than `demo` and `trial` are `Development`, so the engine and e2e suites also pass in a release
  build. Cost, if wrong: none.
- The UI mock has only `demo`; an unknown group is covered by the engine's unit tests, not e2e. Cost, if wrong: one
  e2e scenario and a second mock module.

The agent's calls of the design (section "Agent's calls") stay to check as written there.

### Left for later

The Debug heading sits tight under the description in the Ignored modules tab; parked minors (a fallback name for an
unregistered group, the tab panel's tab stop, a few tests) went to the build's final wave.
