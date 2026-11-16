# 005/03 — Settings screen

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** app/ui, app, core (texts)

The screen in groups built from the core's group of each setting, Reset on every row, "Rescan to apply", the editable
scan folders with the system's folder picker, the Ignored group and the Ignore buttons, and the motion setting
applied. Every text from the dictionaries; no business logic in the UI. The design:
[`../00-design.md`](../00-design.md), section 5.

Depends on 01 and 02 (the types and commands).

## Steps

| #   | Step                                                       | Status      | Shared files                                                              |
| --- | ---------------------------------------------------------- | ----------- | ------------------------------------------------------------------------- |
| 01  | [Groups, reset and rescan](./01-groups-reset-rescan.md)    | ✔️ done    | `app/ui/src/screens/settings/`, the overview, `app/ui/src/state/reduce.ts` |
| 02  | [Folders and the picker](./02-folders-and-picker.md)       | ✔️ done    | `app/tauri/`, `docs/dependencies.md`, the row, `Cargo.lock`, `package-lock.json` |
| 03  | [Ignored and Ignore](./03-ignored-and-ignore.md)           | ✔️ done    | the settings screen, the module page, the finding row, the overview      |

In order.

## Conclusions

Built, with one deviation (agent's call 7 of the design):

- The groups come from the core's `group`: General, Scan, each module, Ignored, Permissions, then Fresher's own data
  and About. A changed row shows **Reset** (the default sent through `update_setting`); a read-only file hides it.
- The scan folders: **Remove** per folder and **Add folder…**, which calls `pick_folder` (`tauri-plugin-dialog` 2.8.4,
  called from Rust; no web view permission); a folder in the home folder comes back as `~/…`; a cancelled pick or a
  folder already listed sends nothing.
- `setting-changed` marks the overview stale, so the totals and the lists are asked for again after an ignore or a
  restore. `core.motion` is applied by `useMotionSetting` in the frame, which asks for the settings once, quietly.
- **Ignore** is on each category of a module's page; the overview says how many ignored things are left out, with a
  link to the settings, and shows "Rescan to apply" too.
- **Deviation**: the Ignore button on a single finding is not built: the finding row is a virtual list item with one
  tab stop, and a second action needs a row menu first. The core supports it fully
  ([deferred](../../../deferred/ignore-an-item-button.md)).
- The settings fixtures of the UI used a made-up `core.removal.method` (Trash or permanent); it is `core.motion` now,
  so no fixture shows a setting that the guard rails forbid. The screenshots of the settings, the module page and the
  folders story were updated and looked at.
