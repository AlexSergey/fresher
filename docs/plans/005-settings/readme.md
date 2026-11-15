# 005 — Settings

**Status:** 📝 draft · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** core, store, engine, app/ui

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "We should probably also draft a plan for settings right away."

Plan [002](../002-base-architecture/readme.md) gives the mechanism (a typed schema, `settings.json`, the
`SettingChanged` event). This plan is the feature on top of it: which settings exist, how they behave, and the screen.

## Goals

1. One settings screen, grouped so that each group answers one question.
2. Every setting has a plain-English label, a one- or two-sentence description of what it changes, a default and a
   "reset" (rule 7 of [`rules.md`](../../rules.md)).
3. A module brings its own settings; the screen renders them with no module-specific UI code.
4. A setting can hide or filter, but never makes deletion less safe.

## Non-goals

- Syncing settings between Macs; accounts.
- Settings for features not built yet: each arrives with its module.

## Design

### Behaviour

- **Applied at once**: no "Save" button. A change is written atomically and announced with `SettingChanged`.
- A change that affects results (scan folders, a module switched off, a threshold) marks the current result as out of
  date with a quiet "Rescan to apply" — the app does not rescan on its own.
- A broken or unknown value falls back to its default and the screen says so.
- **Guard rails, by construction**: no key turns off the confirmation; no key lowers a danger level; permanent delete
  can be made the default choice but is still confirmed every time.

### Groups

| Group         | Settings                                                                                              |
| ------------- | ----------------------------------------------------------------------------------------------------- |
| General       | language; the default removal (Trash, or permanent — still confirmed); motion (system, reduced)       |
| Scan          | folders to scan; folders never scanned; folders that hold projects                                    |
| Modules       | each module on or off, then that module's own settings (for example the age of a stale project)       |
| Ignored       | everything the user ignored — sections and items — each with "Restore"                                |
| Permissions   | what the app may and may not see, with what each permission unlocks (plan [006](../006-permissions/readme.md)) |
| Own data      | the size of Mac Fresher's own data; clear the scan cache; reset all settings; show the folder in Finder |
| About         | version, licence, the licences of dependencies                                                        |

### Ignore list

- A section is ignored by its module or category id; an item by a stable identity its module defines (a path, or an
  image id for Docker).
- Ignored things are skipped entirely — not offered, not tracked — as decided in
  [the idea](../../ideas/ignore-list.md). The overview shows a quiet "2 sections ignored".
- Ignoring a folder protects what is above it from being removed as a whole.
- **Fail closed**: if the settings file cannot be read, nothing can be cleaned until the user resolves it; scanning
  still works with defaults.

### Export and import

`settings.json` is the export: "Export…" copies it, "Import…" validates a file against the schema and shows what will
change before applying it.

### The screen

Built from the `SettingsRow` of plan [003](../003-base-ui/readme.md): label, description, control, reset. The control
follows the setting's type (switch, choice, number with a unit, list of folders). Module settings come from the module's
descriptor.

## Plans (after approval)

| #   | Plan              | What                                                                    |
| --- | ----------------- | ----------------------------------------------------------------------- |
| 01  | Core settings     | the catalogue of the core's settings, validation, the guard rails       |
| 02  | Ignore list       | identities, skipping in the engine, fail closed                         |
| 03  | Settings screen   | the groups, typed controls, reset, "Rescan to apply"                    |
| 04  | Own data and I/O  | the footprint, clear and reset, export and import                       |

Depends on plans 002 and 003.

## Ideas this touches

[User settings](../../ideas/user-settings.md) and the [ignore list](../../ideas/ignore-list.md) (both become this plan),
[stale project cleanup](../../ideas/stale-project-cleanup.md) (its threshold is a module setting), the deferred
[licence notices](../../deferred/licence-notices.md) (the About group is where they are shown).

## Decisions

None yet.

## Agent's calls (to check)

1. **Changes apply at once**, and results are marked out of date rather than rescanned automatically.
2. **Settings never change danger levels** and never touch the confirmation.
3. **Export and import are the settings file itself.**
4. **An ignore has no expiry**, and an ignored item stays ignored while its identity is the same.
5. **An unreadable settings file blocks cleaning, not scanning.**
6. **Target version 0.4.0.**

## Conclusions

Not written yet.
