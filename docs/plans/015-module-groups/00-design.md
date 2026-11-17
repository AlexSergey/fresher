# 015 — Module groups and ignoring a module: design

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, modules, testing, app, app/ui

Written on 2026-11-17 and approved the same day by delegation: the user asked the agent to work on its own and accept
the recommended decisions. Every open question is answered with the recommended option and recorded under
"Decisions"; the agent's calls wait for the user's check.

## Intent

The user's request of 2026-11-17, translated: "Let's do the JetBrains module after 011. Put into this module's plan
right away the ways to test it in the sandbox and coverage by e2e scenarios. Possibly a separate plan for ignoring a
module (JetBrains, and a general group 'Development' into which JetBrains should be placed)."

The user's addition of the same day, translated: "Ignoring a module or a group must also be editable in Settings,
which gets two tabs for it: one listing the ignored groups, and one for ignoring specific modules. Each tab lists every
group or module with its toggle (ignored or not) and restore, consistent with the Settings 'Ignored' group of plan 005
and its ignore kinds. Include UI tests and e2e scenarios for the tabs: toggle, restore, it persists, the sidebar and
the overview update."

This plan is that separate plan: the small piece of infrastructure the first feature module needs, built **before**
plan [016](../016-jetbrains/readme.md) (JetBrains IDE leftovers), which puts its module in the group Development.

## Goals

1. Every module belongs to one named **group** (Development first); the sidebar shows the modules under their group's
   name, and so does the overview's list of modules during a scan.
2. The user can **ignore a whole module and a whole group**, and restore them, as plan 005 ignores a category and an
   item: the findings leave every answer at once, are never removed, and come back without a rescan.
3. Settings shows the ignore lists in **tabs**: ignored groups, ignored modules, and the categories and items plan 005
   already lists; each group and module has its toggle.
4. The dev-only `demo` and `trial` modules sit in a group of their own, `Debug`, shown only in a development build.

## Non-goals

- The **Ignore** button on a single finding row: still [deferred](../../deferred/ignore-an-item-button.md) (it waits
  for the row's actions); not due here, since this plan adds no row action.
- Groups chosen or renamed by the user, nested groups, or a module in two groups.
- A group page of its own (a screen listing a group's findings): the sidebar's heading is enough for now.
- Changing what a module offers: no module, category or danger level changes, so no case file changes (section 7).
- Turning a module off: the `<module>.enabled` switch of plan 003 stays as it is (section 3 says how the two differ).

## What already exists (plans 002, 003, 005, 012)

| Need                        | What exists today                                                                                         | This plan                                                       |
| --------------------------- | --------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| Who a module is             | `ModuleDescriptor` (`crates/core/src/module.rs`): id, categories, settings, locales, tool checks (011: problems) | a required `group`                                              |
| What the UI knows of it     | `ModuleInfo` (`crates/core/src/command.rs`): id, categories, settings keys; `get_modules` in registration order | `group`                                                         |
| The ignore lists            | `core.ignore.categories` (`Ids`) and `core.ignore.paths` (`Paths`), group Ignored, effect Now (plan 005)  | `core.ignore.modules` and `core.ignore.groups`                  |
| Ignoring and restoring      | `ignore(IgnoreTarget)`, `restore_ignored(IgnoredEntry)` (`crates/engine/src/ignore.rs`), kinds `category`, `finding`/`path` | kinds `module` and `group`                                      |
| Left out of every answer    | `Ignored::hides` (`crates/engine/src/settings.rs`): a finding of an ignored category or at or below an ignored path | also a finding of an ignored module or of a module of an ignored group |
| The quiet note              | `Overview.ignored: IgnoredCount { categories, paths }`; "N ignored" on the overview                       | `modules`, `groups` counted too                                 |
| The pending plan            | a change of an ignore list kills the pending cleanup plan (005, decision 10; `is_ignore_list`)            | the two new lists too                                           |
| The storage map             | an ignored finding's bytes are your folders, or stay in its app's data when it names an app (012, decision 7) | the same for a finding hidden by its module or group             |
| Turning a module off        | `<module>.enabled` (Module group, NextScan): a disabled module does not run; its matches are dropped       | unchanged                                                       |
| Settings' Ignored group     | `screens/settings/ignored-group.tsx`: categories by name and paths, each with Restore                     | becomes three tabs                                              |
| The sidebar                 | `screens/frame-sidebar.tsx`: one "Modules" section, one item per module with what it found               | one section per group                                           |

## Design

### 1. Groups

```rust
/// The named group a module belongs to; the sidebar and the overview show modules under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleGroup {
    /// Tools for building software: IDEs, package managers, containers, project folders.
    Development,
    /// The dev-only modules of a development build (`demo`, `trial`).
    Debug,
}
```

- **A closed enum in the core** (agent's call 1): a group is a fact the core shows and ignores, with texts and an
  order, not a module's free string. Only the groups some module uses exist; the plan of the first module of another
  kind adds its group (for example `Apps`, `System`) with its texts.
- **Required in the descriptor**: `ModuleDescriptor::new(id, group, categories, settings, locales)`. Every module says
  its group explicitly; there is no default (agent's call 2).
- **Texts in the core's dictionary**: `core.group.<group>.name` and `core.group.<group>.description`, for example
  "Development" and "Tools for building software: IDEs, package managers, containers and project folders." The core's
  texts test checks every variant has both.
- **Order**: the enum's order is the screen's order, `Debug` always last. Inside a group, the modules keep their
  registration order.
- **`Debug` is a development build's**: the registry refuses a `Debug` module in a release build
  (`RegistryError::DebugModule`), so the group can never appear in the app users get; `demo` and `trial` are already
  registered only with `debug_assertions` (`app/tauri/src/compose.rs`).
- `ModuleInfo` carries `group`; the bindings are regenerated.

### 2. Where groups show

- **The sidebar**: one `NavSection` per group that has a registered module, titled by the group's name, in the enum's
  order, replacing today's single "Modules" section. Each item is as today (the module's name, what it found).
- **The overview**: during a scan, the module chips (`moduleProgress`) are grouped the same way, with the group's name
  before its chips. After the scan the overview is unchanged: its cards are categories and apps (plan 012), not
  modules.
- **Settings**: the modules' own settings (their switch and settings, plan 003) are shown under their group's name too.
- **An ignored module** stays in the sidebar under its group, dimmed, with "Ignored" in place of its size; its page
  opens on a short note ("You ignore this module. Its findings are not shown or cleaned.") with **Restore**. **An
  ignored group**'s heading says "Ignored" and every module under it shows as ignored.

### 3. Ignoring a module and a group: the semantics

**Recommended and taken (decision 2): ignoring hides at query time, exactly as an ignored category does.**

| Option                                                                | What happens                                                                                                     | Verdict           |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------- |
| (a) Hide its findings, as plan 005's kinds do — **recommended**       | the module still runs; its findings stay in the engine and leave every answer at once; Restore shows them at once | consistent        |
| (b) Do not run it (the `<module>.enabled` switch)                     | applies at the next scan; Restore needs a rescan; the map can only say "your folders"                            | exists already    |

Why (a): plan 005's ignore kinds all apply **Now** and restore without a rescan; plan 012 already counts an ignored
finding's bytes as your folders (or in its app's data when it names an app), so a hidden module's bytes land where the
user expects; and the cleaner's safety holds unchanged, since a cleanup is prepared from the visible findings only. The
`<module>.enabled` switch stays what it is, "do not look for this", which saves the module's work; ignoring is "I have
seen this; stop suggesting it". Both are explained in plain words where they are offered (rule 7):

- The module's switch: "Looks for these files when you scan. Off: the next scan skips them."
- Ignore a module: "Hides everything this module finds and never cleans it. Your files stay as they are; you can
  restore it at any time."
- Ignore a group: "Hides everything the modules of this group find and never cleans it. Your files stay as they are;
  you can restore it at any time."

The rules, in the engine:

- `Ignored` gains `modules: Vec<ModuleId>` and `groups: Vec<ModuleGroup>`, and the set of modules the groups cover,
  resolved through the registry when the lists are taken. `hides(finding)` is true for an ignored category, path,
  module, or a module of an ignored group.
- **A module inside an ignored group stays hidden while the group is ignored**, whatever its own entry says; restoring
  the group restores every module of it that is not ignored on its own (decision 4).
- `ignore` refuses a module or a group nobody registered (`UnknownModule`, `UnknownGroup`, new `CommandError` codes
  with their texts); ignoring what is ignored, or restoring what is not, changes nothing and publishes nothing.
- A change of either list kills the pending cleanup plan (`is_ignore_list`), as plan 005's lists do.
- `Overview.ignored` gains `modules` and `groups`; the overview's "N ignored" adds them.
- **Reset all settings keeps the four lists**, as it keeps the two today (plan 005, decision 2).
- **Watch mode** (plan 010, not built): an ignored module or group is not tracked, as the ignore idea says for every
  kind; nothing to build here.

### 4. The settings

| Key                     | Kind                       | Default | Group   | Effect |
| ----------------------- | -------------------------- | ------- | ------- | ------ |
| `core.ignore.modules`   | ids of modules             | `[]`    | Ignored | Now    |
| `core.ignore.groups`    | ids of groups              | `[]`    | Ignored | Now    |

- `SettingKind::Ids` becomes `Ids { of: IdOf }` with `IdOf::{Category, Module, Group}` (agent's call 3): one kind
  of list, checked by what it holds — a category id (two or more segments, as today), a module id (one kebab-case
  segment), or a group's serialized name. A stored value of another kind falls back to `[]` with the row's notice, as
  any broken value does (plan 003). The existing `core.ignore.categories` is `Ids { of: Category }`; its stored form
  does not change, so no migration is needed.
- `IgnoreTarget` and `IgnoredEntry` gain `{ kind: "module", module }` and `{ kind: "group", group }`.

### 5. Settings: the Ignored group in three tabs

The user's addition. The Ignored group of plan 005 becomes one section with three tabs (decision 5, agent's call 4):

| Tab                     | What it lists                                                                                                       |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **Ignored groups**      | every group that has a registered module, with its name, its description and an **Ignore** switch                    |
| **Ignored modules**     | every module, under its group's name, with its name and an **Ignore** switch                                         |
| **Ignored items**       | plan 005's lists as today: every ignored category (its name) and path, each with **Restore**; "Nothing ignored" when empty |

- **The switch is the toggle and the restore**: on, the group or module is ignored (`ignore`); off, it is restored
  (`restore_ignored`). An ignored row also shows a **Restore** button that does the same as turning the switch off, so
  every tab restores in the same words (the user's "toggle and restore").
- A module whose group is ignored shows its switch on and disabled, with "Ignored with its group: Development".
- Each tab's label carries its count when it is not zero ("Ignored modules · 1"); the tabs open on the first one with
  an entry, else on **Ignored groups**.
- While the settings file is read-only, every switch and button is disabled, as today.
- The three tabs are one `Tabs` component (`app/ui/src/design/layout/tabs/`): an ARIA tab list, arrow keys between the
  tabs, one tab stop, with its stories in the gallery (agent's call 5). The overview's "N ignored" link opens the
  Ignored group on the tab of the first kind it counts.

**Where plan 005's lists live (decision 6)**: in the third tab, **Ignored items**, unchanged; groups and modules come
first because they hide the most. A single list mixing all four kinds was weighed and left: a module's row needs a
switch (every module is listed), an item's only a Restore (only ignored items are listed).

### 6. Where the code lives

| Crate or folder                         | What it gains                                                                                            |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `fresher-core`                          | `ModuleGroup`, the descriptor's `group`, `ModuleInfo.group`, `IdOf`, the two settings, the two ignore kinds, `IgnoredCount`'s two fields, two error codes, the texts |
| `fresher-engine`                        | `Ignored`'s two lists and the group → modules resolution, `ignore` and `restore_ignored`, the registry's `Debug` check |
| `fresher-testing`                       | the contract checks the group's texts exist (through the core's dictionary)                              |
| `modules/demo`, `modules/trial`         | `ModuleGroup::Debug`                                                                                     |
| `app/tauri`                             | the bindings regenerated; nothing else (the commands exist)                                              |
| `app/ui`                                | the grouped sidebar and chips, the ignored module's look and page note, the `Tabs` component, the three tabs |

No new crate, no new dependency.

### 7. Tests

- **Unit, core** (negatives first): `ModuleGroup` serializes kebab-case and every variant has its two texts; an `Ids`
  list of modules refuses a category id, an empty name and an upper-case name; an `Ids` list of groups refuses an
  unknown group; `IgnoreTarget` and `IgnoredEntry` of the new kinds round-trip.
- **Unit, engine** (`crates/engine/src/ignore.rs`, `settings.rs`): an unknown module or group is refused and nothing
  is saved; an ignored module's findings are in no answer and cannot be selected; an ignored group hides every module
  of it and only those; a module restored inside an ignored group stays hidden; restoring the group brings back what
  is not ignored on its own, without a rescan; ignoring twice or restoring what is not ignored publishes nothing;
  reset keeps the four lists; a change of either list kills the pending plan; the overview counts both lists; the
  registry refuses a `Debug` module when built for release (a test of the check's function with the build flag as an
  argument).
- **Storage map** (`crates/engine/src/storage/*_tests.rs`): a hidden module's finding is your folders, or stays in its
  app's data when it names an app, as an ignored category's is.
- **Core e2e** (`crates/e2e/tests/ignore.rs`, extended; `fixtures/golden/ignore.json` updated): with the real settings
  file, ignoring the module `trial`, then the group `Debug`, leaves every answer; restore brings them back without a
  rescan; the lists survive a restart.
- **UI unit** (Vitest, negatives first): the sidebar renders a section per group in order and none for an empty
  group; an ignored module shows "Ignored" and no size; the module page shows the note and Restore; the chips are
  grouped; the `Tabs` component (keys, focus, one tab stop); each tab lists what it should, the switch calls `ignore`
  or `restore_ignored` with the right target, a module of an ignored group is disabled with its note, a read-only file
  disables everything, the counts on the tabs.
- **UI e2e** (Playwright, mocked core; `e2e/flows.spec.ts` and `e2e/core-mock.ts`, which learns the two kinds and keeps
  the lists across a reload of the mock's settings):
  1. Settings → Ignored groups → turn on **Debug**: the sidebar's Debug heading says "Ignored", the overview's
     totals drop the demo's findings and "1 ignored" appears; turn it off: everything is back.
  2. Settings → Ignored modules → turn on **trial**: the trial item in the sidebar is dimmed with "Ignored"; its page
     shows the note; **Restore** on the page brings it back.
  3. A module of an ignored group: its switch is disabled with the note; restoring the group re-enables it.
  4. It persists: after ignoring a module, the page is reloaded (the mock keeps its settings) and the switch is still
     on.
  5. The overview's "N ignored" link opens Settings on the right tab.
  6. Keyboard: the tabs are reached by Tab and switched with the arrow keys; the switches answer Space.
  Visual and accessibility: `visual.spec.ts` and `axe.spec.ts` gain the three tabs, the grouped sidebar and the
  ignored module's page; the screenshots are reviewed.
- **Sandbox case file: none** (decision 7). Rule 8 asks one of a plan that adds or changes a module or a category; this
  plan changes neither what is offered nor any danger level, and an ignore is the user's setting, which the case files
  do not declare. The existing case files and goldens must not change; `npm run sandbox:test` proves it. Plan 016's
  case file is the first to run under a group.

## Interfaces with other plans

- **Plan 011** (merged before this plan is built): `ModuleDescriptor::with_problems` stays a builder after `new`;
  this plan's new parameter of `new` is added beside it.
- **Plan 016** (built after this one): `ModuleDescriptor::new(id, ModuleGroup::Development, …)`; its e2e scenarios
  ignore the module `jetbrains` and the group Development.
- **Plan 012**: the map reads `Ignored` through the results; nothing else changes.
- **Plan 010** (draft): an ignored module or group is not tracked.

## Open questions

None left: every question was delegated and decided below.

## Decisions

The user's, of 2026-11-17 (translated): the request and the addition quoted in the intent — a separate plan for
ignoring a module and a group, a group "Development" with JetBrains in it, and two tabs in Settings for ignored groups
and ignored modules, each with its toggle and restore, with UI tests and e2e scenarios.

Taken by delegation on 2026-11-17 (the user's instruction: accept the recommended decisions):

1. **The plan is approved** as designed here, with the agent's calls below still to check.
2. **Ignoring a module or a group hides its findings at query time** (section 3, option a): consistent with plan 005's
   kinds (Now, restore without a rescan, never removed) and with plan 012 (hidden bytes are your folders or stay in
   the app's data). The `<module>.enabled` switch keeps its meaning, "do not look".
3. **An ignored module stays visible in the sidebar**, dimmed with "Ignored", so the user can find it and restore it.
4. **A group's ignore covers its modules**; a module restored inside an ignored group stays hidden until the group is
   restored.
5. **Settings shows the ignore lists in three tabs** of the Ignored group: Ignored groups, Ignored modules, Ignored
   items.
6. **Plan 005's category and path lists live in the third tab**, "Ignored items", unchanged.
7. **No sandbox case file** in this plan: nothing offered changes (section 7).
8. **No version bump**: the version stays 0.1.0 ([`../README.md`](../README.md), "Versions").
9. **Built before plan 016 and after plan 011 is merged into `main`**, on this branch made again from `main` then
   (both 011 and this plan edit `crates/core/src/module.rs`, the core's dictionary and `modules/{demo,trial}`).

## Agent's calls (to check)

1. **`ModuleGroup` is a closed enum of the core**, with only `Development` and `Debug` now. Costs, if wrong: groups as
   module data later, one type and its texts.
2. **The group is a required parameter of `ModuleDescriptor::new`**, not a builder with a default. Costs, if wrong: a
   one-line change in every module.
3. **`SettingKind::Ids { of: IdOf }`** rather than two new kinds. Costs, if wrong: the kind split in the core, the
   bindings and the settings screen's row for lists.
4. **The tabs' names**: "Ignored groups", "Ignored modules" and "Ignored items" (the user asked for the first two
   names to be recommended). Costs, if wrong: three texts.
5. **A new `Tabs` component** in the design system, not the `SegmentedControl` reused: tabs need the ARIA tab list and
   its keys. Costs, if wrong: one component less.
6. **A module of the `Debug` group is refused in a release build** by the registry, beside the `cfg` that already keeps
   `demo` and `trial` out. Costs, if wrong: one check less.
7. **The overview after a scan does not show groups**: its cards are categories and apps (plan 012). Costs, if wrong: a
   grouping of the cards later.
8. **Ignoring a group from anywhere but Settings is not offered** (a module can be ignored from its page too). Costs,
   if wrong: a menu on the sidebar's heading later.

## Conclusions

Not written yet.
