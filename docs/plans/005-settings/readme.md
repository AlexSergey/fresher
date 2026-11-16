# 005 — Settings

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine, app, app/ui

Approved on 2026-11-16 by delegation: the user delegated every open question to the agent, and the recommended option
was taken on each ([`00-design.md`](./00-design.md), "Decisions"); the agent's calls wait for the user's look. The
target version is 0.5.5, the user's decision of 2026-11-16.

**Built and verified on 2026-11-16; the user looked at the result and asked for the squash-merge the same day.** Every
plan and step below is ✔️; the conclusions are at the end. The agent's calls stay open for the user's check.

## Goal

One settings screen in groups, each answering one question; every setting with a plain-English label, a description,
a default and a Reset; changes applied at once, with a quiet "Rescan to apply" when they change what a scan finds; the
scan folders editable with the system's folder picker; an ignore list for categories and items, left out of the
results and never removed, restored from the screen; the app's own data and its version. No setting makes deletion
less safe. The design: [`00-design.md`](./00-design.md).

Branch: `feat/005-settings`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [002](../002-base-architecture/readme.md), [003](../003-base-ui/readme.md) and
[008](../008-safe-removal/readme.md) (done); plan [006](../006-permissions/readme.md)/04 fills the Permissions group
later.

## Plans

| #   | Plan                                                       | Status      | Depends on | What                                                                                |
| --- | ---------------------------------------------------------- | ----------- | ---------- | ----------------------------------------------------------------------------------- |
| 01  | [Core settings](./01-core-settings/readme.md)              | ✔️ done     | —          | groups and effects, the new kinds and keys, "Rescan to apply", reset all, the guard |
| 02  | [Ignore list](./02-ignore-list/readme.md)                  | ✔️ done     | 01/02      | the commands, results left out, protection, the overview count                      |
| 03  | [Settings screen](./03-settings-screen/readme.md)          | ✔️ done     | 01, 02     | the groups, reset, folders and the picker, Ignored, the Ignore buttons, motion      |
| 04  | [Own data and finish](./04-own-data-and-finish/readme.md)  | ✔️ done     | 01–03      | the footprint, reset all, About, docs, version 0.5.5, e2e, conclusions              |

### Order

```
01 ── 02 ── 03 ── 04
```

In order, in one branch: each step changes the IPC types the next one renders, and the bindings file is regenerated
by each, never merged by hand.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 %.
- No core setting names the confirmation, a danger level or the way items are removed (a test).
- An ignored category or path is in no answer of the engine, and a cleanup never removes it or what holds it (engine
  tests and the core e2e).
- The UI e2e: change and reset a setting, add and remove a scan folder, ignore and restore, "Rescan to apply".

## Ideas this touches

[User settings](../../ideas/user-settings.md) and the [ignore list](../../ideas/ignore-list.md) become this plan;
[stale project cleanup](../../ideas/stale-project-cleanup.md) brings its threshold as a module setting later.

## Conclusions

Built on 2026-11-16 in one session; version 0.5.5. Each plan's readme has its own conclusions; this is the whole.

### What was built

- **The core's catalogue** (01): every setting with its group and effect, `core.motion`, the ignore lists as two new
  kinds, the guard test that no core key names the confirmation, a danger level or the removal.
- **The engine** (01, 02): "Rescan to apply" (`LastScan.settingsChanged`), `reset_settings` (the ignore lists kept),
  `ignore` and `restore_ignored`, ignored findings left out of every answer and plan, the cleaner protected from
  ignored paths and what holds them, a pending plan killed by a change of what is scanned or ignored.
- **The app** (03, 04): `pick_folder` with `tauri-plugin-dialog` 2.8.4 called from Rust, `get_own_data`.
- **The screen** (03, 04): the groups in order, Reset per row, editable scan folders, Ignored with Restore, Reset all
  in a dialog, Fresher's own data, About; Ignore on a category; the overview's "N ignored" and "Rescan to apply";
  the motion setting on the page.

### Measured

`npm run pre-push` passes on the branch's last code commit: lint, crate deps, knip, machete, deny, the scripts' tests;
1418 Rust tests passed (3 skipped), Rust coverage 97.27 % of lines; 844 UI tests passed, UI coverage 98.36 % of lines.
The UI e2e (gallery, WebKit, Chromium): 213 passed after the screenshots of the settings, the module page and the
folders story were updated on purpose and looked at; the flows gained a folder added with the picker and reset, and
Reset all asking first.

### Deviations and what is left

- The Ignore button on a single finding is not built (agent's call 7); the core supports it
  ([deferred](../../deferred/ignore-an-item-button.md)).
- "Show in Finder" for the data folder and clearing the scan cache are left out (design, 6; decision 5); export and
  import with a preview, and a default-removal setting, are deferred (decisions 3 and 8). The licences of the
  dependencies stay [deferred](../../deferred/licence-notices.md).
- Known issue: [a setting changed during a scan](../../known-issues/core-settings-changed-during-a-scan.md) does not
  ask for a rescan after that scan.
- The Permissions group shows the four opt-in switches; each permission's state comes with plan 006/04.
- No hack was added (`docs/hacks/` unchanged).
