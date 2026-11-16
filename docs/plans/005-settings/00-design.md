# 005 — Settings: design

**Status:** 🔨 in progress · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app, app/ui

## Intent

The user's task (translated): "We should probably also draft a plan for settings right away."

Plan [002](../002-base-architecture/readme.md) gave the mechanism: a typed schema (`SettingDescriptor`, four kinds,
the check of a value), `settings.json` through the `SettingsStore` port, `get_settings`, `update_setting` and the
`SettingChanged` event, and a settings file that is read-only when it is newer or unreadable. Plan
[003](../003-base-ui/readme.md) gave `SettingsRow` and a settings screen that renders every setting generically, the
folders read-only. Plan [006](../006-permissions/readme.md) added the `core.access.*` opt-ins; plan
[008](../008-safe-removal/readme.md) made cleaning fail closed when the settings file is not loaded, and gave the
cleaner's `Protection::with_ignored` and the skip reason "ignored". This plan is the feature on top of them: which
settings exist, how they behave, the ignore list, and the screen.

## Goals

1. One settings screen in groups, each answering one question: General, Scan, Modules, Ignored, Permissions, Your
   Fresher data, About.
2. Every setting has a plain-English label, a one- or two-sentence description, a default and a **Reset**
   (rule 7 of [`rules.md`](../../rules.md)).
3. The core says which group a setting belongs to and when it takes effect; the UI renders, it decides nothing.
4. A module brings its own settings; the screen renders them with no module-specific code (already true since 003).
5. The ignore list: a category or a single item can be ignored and restored; ignored things are left out of the
   results and can never be removed while ignored.
6. A setting can hide or filter, but never makes deletion less safe.

## Non-goals

- Syncing settings between Macs; accounts.
- Settings for features not built yet: each arrives with its module ("folders that hold projects" with the stale
  project module, a language choice with the second language).
- Export and import with a preview of the changes (deferred, see "Decisions", 3).
- Clearing the scan cache and the stale temporary files from the app (deferred: only `fresher-cleaner` may delete,
  see "Decisions", 5).
- The licences of the dependencies in "About" (stays [deferred](../../deferred/licence-notices.md)).

## Design

### 1. Where a setting goes and when it applies

`SettingDescriptor` gets two fields the core sets, serialized to the UI:

```rust
pub enum SettingGroup { General, Scan, Ignored, Permissions, Module }
pub enum SettingEffect { Now, NextScan }
```

- A module's settings (and its `<module>.enabled` switch) are `Module` and `NextScan`; `SettingDescriptor::new` keeps
  that for modules, so a module cannot place a setting in a core group.
- The core's settings: `core.motion` (General, Now), `core.scan.roots` (Scan, NextScan), `core.access.*`
  (Permissions, NextScan), `core.ignore.*` (Ignored, Now).
- The screen renders the groups in the order above; inside a group, in the schema's order.

### 2. The catalogue of the core's settings

| Key                      | Kind                      | Default    | Group       | Effect    |
| ------------------------ | ------------------------- | ---------- | ----------- | --------- |
| `core.motion`            | choice `system`/`reduced` | `system`   | General     | Now       |
| `core.scan.roots`        | folders                   | `["~"]`    | Scan        | NextScan  |
| `core.access.*` (four)   | switch                    | off        | Permissions | NextScan  |
| `core.ignore.categories` | ids (new kind)            | `[]`       | Ignored     | Now       |
| `core.ignore.paths`      | paths (new kind)          | `[]`       | Ignored     | Now       |

- **Two new kinds**: `Ids` — a list of category ids, each two or more dot-separated kebab-case segments; `Paths` — a
  list of absolute paths of plain names (no `~`, `.`, `..`, empty names), anywhere: ignoring a protected place only
  adds protection, so the protected-folder refusal of `Folders` does not apply.
- **Guard rails, by construction**: no key turns off the confirmation, lowers a danger level, or chooses how items are
  removed. A test lists the core's keys and fails on any that names the confirmation, a danger level, the Trash or a
  permanent delete. A module's settings are read only by that module; the engine reads none of them for a cleanup.
- `core.motion`: `system` follows macOS's "Reduce motion"; `reduced` keeps only fades. The UI sets the `data-motion`
  attribute plan 003 already reads.

### 3. Behaviour

- **Applied at once**: no "Save" button. A change is checked, written atomically and announced with
  `SettingChanged` (as since 002).
- **"Rescan to apply"**: a change of a `NextScan` setting after a scan sets `LastScan.settingsChanged`; the overview
  and the settings screen show a quiet "Rescan to apply" with a **Rescan** button. The app never rescans on its own.
  The flag is cleared by the next scan. A change of a `Now` setting does not set it.
- **Reset**: a row whose value differs from its default shows **Reset**, which sends the default through
  `update_setting`. **Reset all settings** (`reset_settings`) puts every setting but the ignore list back to its
  default, saves once, and publishes one `SettingChanged` per value that changed. Refused while the file is read-only.
- A broken or unknown stored value falls back to its default and the row says so (since 003).

### 4. The ignore list

- **What is ignored**: a category, by its id (`core.ignore.categories`); an item, by its path (`core.ignore.paths`).
  An item of a tool (`Subject::External`, for example a Docker image) cannot be ignored yet: its module defines its
  identity when it is built.
- **Commands**: `ignore(IgnoreTarget)` with `{ kind: "category", category }` or `{ kind: "finding", finding }` (the
  engine takes the finding's path), and `restore_ignored(IgnoredEntry)` with `{ kind: "category", category }` or
  `{ kind: "path", path }`. Each goes through the same check and save as `update_setting` and publishes
  `SettingChanged`; the UI never edits the lists itself. Ignoring what is already ignored, or restoring what is not,
  changes nothing and publishes nothing.
- **Left out of the results**, at query time: `get_findings`, `get_finding`, the totals and the overview never hold a
  finding of an ignored category or at or below an ignored path. The findings stay in the engine's memory, so
  **Restore** brings them back at once, without a rescan. The overview carries `ignored: { categories, paths }`, the
  sizes of the two lists, for the quiet "2 ignored" note.
- **Never removed**: a cleanup is prepared from the visible findings only, so an ignored one cannot be selected (its
  id is unknown). The cleanup context gives the cleaner `Protection::with_ignored(paths)`, so a finding that holds an
  ignored path (an ancestor) is skipped with "It is on your list of items to ignore", and the cleaner re-checks it
  right before each item.
- **Fail closed** (since 008): when the settings file is not loaded, cleaning is blocked; scanning works on defaults.
- An ignore has no expiry; an ignored path stays ignored while the path is the same.

### 5. The screen

| Group             | What it shows                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------ |
| General           | `core.motion`                                                                                    |
| Scan              | `core.scan.roots`, editable: **Add folder…** (the system's folder picker) and **Remove** per row |
| Modules           | each module under its name: its switch, then its own settings (as since 003)                     |
| Ignored           | every ignored category (its name) and path, each with **Restore**; "Nothing ignored" when empty  |
| Permissions       | the `core.access.*` opt-ins; plan 006/04 adds each permission's state here                       |
| Your Fresher data | the size of the data folder against its budget; **Reset all settings…** (confirmed in a dialog)  |
| About             | the version and the app's licence (MIT or Apache-2.0)                                            |

- Every row is the `SettingsRow` of plan 003, with **Reset** added; a `Folders` row gets add and remove.
- **The folder picker** is a command, `pick_folder`, run by the app with `tauri-plugin-dialog` (MIT OR Apache-2.0),
  so the web view gets no dialog permission. It answers a path or nothing (cancelled); the UI then sends the new list
  through `update_setting`, and the core checks it as any other value.
- **Ignore** buttons: on a category section of the module page, and on a finding row (a path only). The overview
  shows "N ignored" with a link to the Ignored group when the lists are not empty.
- `setting-changed` marks the UI's overview as stale, so the totals and lists are asked for again after an ignore or a
  restore.

### 6. Your Fresher data

`get_own_data` answers `{ total, budget }` from `DataFolder::footprint` (plan 002). "Show in Finder" waits for the
system's interface of plan 006/03 (today `NoSystemUi`), and clearing the cache waits for a way to delete the app's
own files (Decisions, 5).

## Plans

| #   | Plan                                                | What                                                                                |
| --- | --------------------------------------------------- | ----------------------------------------------------------------------------------- |
| 01  | [Core settings](./01-core-settings/readme.md)       | groups and effects, the new kinds and keys, "Rescan to apply", reset all, the guard |
| 02  | [Ignore list](./02-ignore-list/readme.md)           | the commands, results left out, protection, the overview count                      |
| 03  | [Settings screen](./03-settings-screen/readme.md)   | the groups, reset, folders, the folder picker, Ignored, the Ignore buttons, motion  |
| 04  | [Own data and finish](./04-own-data-and-finish/readme.md) | the footprint, reset all, About, docs, version 0.5.5, e2e, conclusions        |

## Ideas this touches

[User settings](../../ideas/user-settings.md) and the [ignore list](../../ideas/ignore-list.md) become this plan;
[stale project cleanup](../../ideas/stale-project-cleanup.md) (its threshold is a module setting of that module); the
[licence notices](../../deferred/licence-notices.md) stay deferred.

## Open questions

None left: every question of the draft was answered by delegation (below).

## Decisions

All delegated to the agent by the user on 2026-11-16, recommended option taken:

1. **Changes apply at once**; results are marked out of date ("Rescan to apply"), never rescanned automatically.
2. **Settings never change danger levels** and never touch the confirmation or the way items are removed.
3. **Export and import are the settings file itself**: `settings.json` in the data folder is plain JSON a user can
   copy; a screen to import with a preview of the changes is [deferred](../../deferred/settings-export-import.md).
4. **An ignore has no expiry**, and an ignored item stays ignored while its identity (its path) is the same.
5. **An unreadable settings file blocks cleaning, not scanning** (already built in 008). Clearing the scan cache and
   the stale temporary files stays out: the app deletes only through `fresher-cleaner`, and no exception for its own
   data folder is agreed ([deferred](../../deferred/own-data-clearing.md)).
6. **Target version 0.5.5** — the user's decision of 2026-11-16 (replacing the delegated 0.6.0): many features are still
   to come. Plan 009 targets 0.5.5 too: whichever branch is merged into `main` first bumps the workspace to 0.5.5, the
   second keeps it.
7. **Ignored things are left out at query time**, so Restore works without a rescan.
8. **No "default removal" setting**: Trash stays the default and a permanent delete is chosen in the confirmation
   each time (rule 2); a setting that pre-selects it is [deferred](../../deferred/default-removal-setting.md) until
   the user asks for it.

## Agent's calls (to check)

1. **No "folders never scanned" setting**: the ignore list covers it for the results (an ignored folder's findings
   are left out and protected); the walk still reads the folder. A walk-level exclusion comes when a measured scan
   needs it.
2. **"Reset all settings" keeps the ignore list**: the ignore list is the user's record of decisions, not a
   preference; it is restored item by item.
3. **The folder picker is a Rust command** with `tauri-plugin-dialog`, not the plugin's JavaScript API: no new web
   view permission, and the IPC stays the one typed module.
4. **Two new setting kinds** (`ids`, `paths`) rather than one free-form list, so each value is checked by its meaning.
5. **"Show in Finder" for the data folder is left out** until plan 006/03 gives the app a real `SystemUi`.
6. **The overview counts the entries of the ignore list**, not the findings they hide (the hidden sizes are not
   computed: a hidden total would be a number the user cannot act on).

Made while building (same status: to check):

7. **No Ignore button on a single finding yet**: the row is a virtual list item with one tab stop; a second action
   needs a row menu. The core, the engine and the Restore side are built ([deferred](../../deferred/ignore-an-item-button.md)).
8. **Any `setting-changed` asks for the overview again** in the UI, without looking at the key: the UI holds no rule
   about which setting changes the results.
9. **Reset all is refused on a read-only file** even when nothing would change, so the screen never says a reset
   happened that could not be saved.
10. **A change of an ignore list kills the pending cleanup plan**, like a change of the roots: the plan may hold what is
    now ignored.
11. **The scan folders may be emptied**: removing the last folder is allowed, and the next scan finds nothing; the
    settings show "No folders". A guard against it waits for the user's word.
12. **The version files are bumped to 0.5.5 on this branch**; if plan 009 is merged first, the bump is the same line.
