# 033 — Logs and crash reports: design

**Status:** ✅ approved (the user's delegation of 2026-11-22) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the draft [`readme.md`](./readme.md) and from a read of the code on `main` (97d9b314),
read-only. Approved by the user's delegation of 2026-11-22: build autonomously, take the recommended decisions and the
agent's calls; the user checks everything later. Where the code forced a different choice than the draft, the design
says so and the choice is an agent's call. Facts marked "to verify" are checked by the user (03/01 lists them in
`docs/deferred/user-review-2026-11-22.md`).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind." Old logs and crash reports are the first thing a disk cleaner
offers. Fresher offers the logs of removed apps (`app-leftovers`) and of older JetBrains IDE versions (`jetbrains`)
only; the rest of `~/Library/Logs` belongs to no module.

## Goals

1. A module `logs` (`modules/logs`, crate `fresher-module-logs`, group **System**) that offers, one finding per file,
   the files under `~/Library/Logs` that nothing has changed for more than the setting's days (7 by default), in two
   categories: **Old logs of your apps** and **Old crash reports** (both Safe).
2. A log an app is writing now is never touched: a file changed within the days is kept (left out `recent`); a file
   held open at the act is skipped by the cleaner's check 10.
3. **Files go, folders stay**: only regular files are claimed, each `Removal::MoveToTrash` of its own path; no folder
   is ever claimed, so an app always finds its log folder.
4. **One claim per path** with the modules that already own logs: nothing under `Logs/JetBrains` (`jetbrains`) and
   nothing under a top-level folder shaped as a bundle id (`app-leftovers`) is claimed (section 3).

## Non-goals

- System-wide logs (`/Library/Logs`, `/private/var/log`, `/Library/Logs/DiagnosticReports`): they need administrator
  rights ([`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md), "No administrator
  rights"); not offered.
- The unified log, and anything a tool must rotate.
- **The logs of an installed app kept under its bundle id** (`~/Library/Logs/com.example.App/`): `app-leftovers`
  claims every bundle-id-shaped folder of `Logs` whole, so the walk never reports a file inside one to any other
  module (section 3). Offering them needs either a second whole interest and an emptied folder, as plan 030 did for
  caches, or a core change. Deferred (agent's call 2):
  `docs/deferred/logs-more-places.md`, written in 03/01.
- Files deeper than six names below `Logs` (section 1); deferred with the above.
- Naming the app behind a log by its loose folder name: the rules tie a path to an app by bundle id only
  ([`../../architecture/modules.md`](../../architecture/modules.md), "The system's facts"). The reason names the folder
  the log lies in, which is what the user recognises; no finding is counted in an app's line of the map (agent's
  call 5).

## What already exists

| Need                                         | What exists                                                                                                   | Plan     |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | -------- |
| The place                                    | `KnownFolder::Logs` (`~/Library/Logs`), resolved by the `KnownFolders` port, in the scan roots (the home)       | 002, 009 |
| A match per entry at a depth, walk going on  | `Pattern::Under(KnownFolder::Logs, "*/*")`, `Claim::Item`: each `*` is one name (`Segment::Any`, `crates/scanner/src/matcher/trie.rs`); used by `local-ai-models` (`.lmstudio/models/*/*`) | 002, 023 |
| Another module's folder kept out of the walk | a `Claim::Whole` match is summed, not indexed further: nothing inside it is reported to any interest (`crates/scanner/src/walk/worker.rs`) | 004      |
| `jetbrains`' logs                            | `Under(Logs, "JetBrains/*")` `Claim::Whole`                                                                     | 016      |
| `app-leftovers`' logs                        | `Under(Logs, "*")` `Claim::Whole` `.only_bundle_ids()` (row `logs`, kind folder)                                | 019      |
| A file's times and facts                     | `Match.modified`, `changed`, `kind`, `facts` (`EntryFacts`), `cx.is_restored()`                                 | 004, 017 |
| A day setting that can only become stricter  | `DaySetting::new(name, min, max, default)`, `.days(cx.settings())`                                               | 017      |
| Bundle-id shape                              | `fresher_core::is_bundle_id_shaped(&[u8])`                                                                       | 019      |
| Left-outs                                    | `out.left_out(LeftOutItem { … })`, `.with_problems(keys)`                                                        | 011      |
| The shape to follow                          | `modules/temp`: one decision per match in `on_match`, nothing in `finish`, a day setting, the restored scan kept `scan-again` | 017      |

**Nothing in the core, the scanner, the cleaner or the engine is missing or changes.**

## Design

### 1. The interests

Six interests, in this order, each `Interest::new(Pattern::Under(KnownFolder::Logs, <path>), Claim::Item, vec![])`:
`*`, `*/*`, `*/*/*`, `*/*/*/*`, `*/*/*/*/*`, `*/*/*/*/*/*` (`MAX_DEPTH = 6`). Every entry at depth 1 to 6 below
`Logs` is reported once, to the interest of its depth; the module claims files only. A `Claim::Item` makes nothing
opaque, so `jetbrains`' and `app-leftovers`' whole claims keep working, and theirs keep this module out of their
folders (section 3).

Measured on the developer's Mac (2026-11-22, macOS 27.2.1, read-only): `~/Library/Logs` holds 318 MB; outside
`JetBrains` its files lie at depths 1 to 5 (6, 87, 47, 27 and 37 files); `JetBrains` holds about 2,100 files at depths
2 to 4. Six covers every file seen; deeper ones are deferred.

### 2. The categories, by place

The first name below `Logs` decides:

| First name                   | Category              | Danger | Identity                                   |
| ---------------------------- | --------------------- | ------ | ------------------------------------------ |
| `DiagnosticReports`          | `logs.crash-reports`  | Safe   | `crash-reports/<path below DiagnosticReports>` |
| any other (section 3 aside)  | `logs.app-logs`       | Safe   | `app-logs/<path below Logs>`               |

Names compared exactly, case ignored (ASCII). Each name of an identity's path is written as `temp`'s identities are:
its UTF-8 as it is, a backslash doubled, any other byte `\xNN`, joined by `/`. On the developer's Mac
`DiagnosticReports` holds 80 `.ips` files (owned by the user, group `_analyticsusers`, mode 600; the folder 770) and an
empty `Retired/`; files in `Retired/` are crash reports too.

### 3. Who claims what under `Logs`

| Path below `Logs`                                     | `jetbrains`        | `app-leftovers`             | `logs` (this plan)       |
| ----------------------------------------------------- | ------------------ | --------------------------- | ------------------------ |
| `JetBrains/<version folder>` and inside               | **decides** (whole) | —                           | not claimed (row 2)      |
| `<folder shaped as a bundle id>/` and inside          | —                  | **decides** a folder (whole) | not claimed (row 2)      |
| `<file shaped as a bundle id>` at the top             | —                  | — (folder rows only)        | **decides** (section 4)  |
| any other file                                        | —                  | —                           | **decides** (section 4)  |

In the full app the whole claims make those folders opaque, so this module never even sees a file inside them; it
refuses them by name anyway (row 2), so its own tests, its case file (which registers this module alone) and any
future change of another module's interest keep the split. A top-level **file** shaped as a bundle id
(`com.example.App.log`, but also `system.log`, `install.log`, `fsck_apfs.log`: the shape needs only one dot) is this
module's: `app-leftovers`' `logs` row is for folders only (`modules/app-leftovers/places.toml`), so nobody else
decides it (agent's call 3). `temp`,
`app-caches`, `dev-caches` and the rest work outside `Logs`.

### 4. The decision (`decide.rs`, a pure function)

Each match is decided on its own in `on_match`, as in `temp`; `finish` does nothing. For one match, the first row that
applies:

| #   | When                                                                                                 | Verdict             |
| --- | ---------------------------------------------------------------------------------------------------- | ------------------- |
| 1   | a folder or a link                                                                                   | not claimed         |
| 2   | the first name below `Logs` is `JetBrains`, or a folder (more names follow) shaped as a bundle id     | not claimed         |
| 3   | the name ends with a state or lock suffix: `.plist`, `.db`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`, `.lock`, `.pid` (ASCII case ignored) | not claimed |
| 4   | the scan is restored from the cache                                                                  | left out `scan-again` |
| 5   | a socket, a pipe or a device                                                                         | left out `special`  |
| 6   | not the user's, the system's immutable flag, or restricted                                           | left out `not-yours` |
| 7   | the user's lock (`user_immutable`)                                                                   | left out `locked`   |
| 8   | the newest of `modified` and `changed` unknown, in the future, or not more than the days ago         | left out `recent`   |
| 9   | otherwise                                                                                            | offered, with its age in whole days |

- Rows 1 to 3 say nothing: a folder, another module's place, an app's state file are not this module's.
- Row 3: a property list or a database in `Logs` is an app's state, not a log (on the developer's Mac:
  `CrashReporter/DiagnosticLogs/Search/spotlight_heartbeat_last.plist`); a lock or pid file marks something in use.
- Row 4: a restored scan's times are as old as the cached scan, and a log may have been written since (`temp`, row 1).
- Row 8: `changed` (the status change) moves when a log is rotated (renamed), so a rotated log waits the days again.
  The access time is not read: reading a log does not keep it.
- The module never asks the installed or the running apps: nothing in a verdict depends on them (agent's call 5).
  A file an app holds open is skipped at the act by the cleaner's check 10 (`InUse`), which the core e2e proves.
- Nothing is judged by another match, so the verdict does not depend on the walk's order.

### 5. The place's contract: an unknown name claimed

`deletion-safety.md`, "Unknown is not junk", allows **one** exception today (`temp`'s places). Every file in
`~/Library/Logs` is claimed whatever its name (the extensions seen on the developer's Mac: `.ips`, `.json`, `.log`,
`.html`, `.txt`, `.csv`, `.asl`, `.aapbz`, numbered and dated rotations), because the folder's contract, not the name,
says what it is: the place macOS gives apps for their logs. This plan adds it as the **second exception**, bounded by
rows 1 to 8: regular files only, no folder, no other module's place, no state or lock name, the user's and unflagged,
unchanged for more than the setting's days, and only to the Trash (agent's call 1). 01/02 writes it into
`deletion-safety.md` in the same commit as the decision.

### 6. Findings

| Part     | Value                                                                                                          |
| -------- | -------------------------------------------------------------------------------------------------------------- |
| category | `logs.app-logs` or `logs.crash-reports` (section 2), Safe                                                      |
| identity | section 2                                                                                                      |
| subject  | `Subject::Path` of the file; removal `Removal::MoveToTrash` of the same path                                   |
| sizes    | the match's                                                                                                    |
| title    | `logs.title` with `name` (the file's name, lossy UTF-8)                                                        |
| reason   | app logs at the top of `Logs`: `logs.reason.top` with `days`; deeper: `logs.reason.folder` with `folder` (the parent's path below `Logs`, names joined by `/`, lossy UTF-8) and `days`; crash reports: `logs.reason.report` with `days` (`TextArg::Count`) |
| app      | none                                                                                                           |

Left-outs: `LeftOutItem { category (the place's), reason: logs.left-out.<reason>, sizes }` for rows 4 to 8.

### 7. The setting

`logs.older-than-days`: `DaySetting::new("older-than-days", 7, 90, 7)`, one for both categories. It can only make the
check stricter: never fewer than 7 days.

### 8. Texts (`locales/en.json`)

Plain English (rule 7); polished when built:

- `logs.app-logs.name` "Old logs of your apps"; `.what` "Records apps write about what they did, kept for finding
  problems. Nothing has written to these files for at least a week."; `.made-by` "The apps themselves and parts of
  macOS; the folder a log is in usually names its app."; `.if-removed` "The files go to the Trash and their folders
  stay, so each app finds its log folder where it left it. A log changed recently is kept, and one in use when you
  confirm is skipped."; `.comes-back` "Apps write new logs as they run. The old records do not come back."
- `logs.crash-reports.name` "Old crash reports"; `.what` "Reports macOS writes when an app crashes, stops responding
  or uses too much of your Mac, to help its developer find the cause."; `.made-by` "macOS, for every app and part of
  the system."; `.if-removed` "The reports go to the Trash and leave Console's list of reports. Nothing else changes.
  If you mean to send one to its developer, do it first."; `.comes-back` "A new report is written the next time
  something crashes. The old ones do not come back."
- `logs.enabled.label` "Logs and crash reports"; `.description` "Finds old logs and crash reports in your Library.
  The files go to the Trash; their folders stay."
- `logs.older-than-days.label` "Offer logs and crash reports unchanged for"; `.description` "Anything changed more
  recently is kept. It cannot be set below 7 days, so the check never becomes less careful."
- `logs.title` "{name}"; `logs.reason.top` "In your Logs folder, unchanged for {days} days."; `logs.reason.folder` "In
  {folder} in your Logs folder, unchanged for {days} days."; `logs.reason.report` "A crash or diagnostic report,
  unchanged for {days} days."
- `logs.left-out.scan-again` "Shown from the last scan. Scan again to check how old it is."; `.special` "A program uses
  it to talk to another program, so it is kept."; `.not-yours` "It belongs to the system or another user; removing it
  needs an administrator."; `.locked` "It is locked. To let Fresher remove it, unlock it in Finder's Get Info.";
  `.recent` "Written or changed recently; an app may still be using it."

### 9. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit**: `names.rs` (the place of a path, the bundle-id and `JetBrains` refusal, the state suffixes, the identity
  and the folder text, bytes that are not UTF-8), `decide.rs` (one test per row, keep before offer; the boundary of
  exactly the days; a property: a verdict never depends on anything but the item and the scan), `tests.rs` (the module
  on hand-built matches: the category by place, the reason by depth, the removal, the setting, the order of matches).
- **Fixtures** (`fixtures/homes/logs-*.toml`, `run_module`, golden `fixtures/golden/logs.json`): `-not-claimed`,
  `-kept`, `-offered` (01/03).
- **Contract**: `assert_module_contract` with `Catalogue::Required`; every text and every declared left-out used by a
  fixture run; every removal a `MoveToTrash` of a matched file.
- **Sandbox**: `fixtures/sandboxes/logs.toml` with its golden; `EXPECTED.md` walked by the user. Its names are apart
  from every other case file's.
- **Core e2e**: `crates/e2e/tests/logs.rs` with `jetbrains` and `app-leftovers` registered beside `logs`: no path
  offered by two modules; a cleanup moves the old files to the sandbox's Trash and keeps their folders and the fresh
  log; put back; a file held open is skipped `InUse`; a restored scan leaves every log out `scan-again`; a longer
  setting keeps a ten-day-old log.
- **UI e2e**: the two cards, Details with their texts and left-out line, a finding's title and reason, the review
  naming Safe, a cleanup and put back, the setting and the switch in Settings.

## Risks

- **An app keeps state that is not a log in `Logs`** under a name row 3 does not know (a `.json` it reads back): it
  must be unchanged for a week, it goes to the Trash, and the app rewrites it or starts fresh. 03/01 asks the user to
  look at the largest offered files on the real Mac (to verify).
- **Many small findings**: one per file (on the developer's Mac about 80 crash reports and 80 files of one vendor's
  folder). The UI lists them largest first inside each category; selecting the category takes them all. A grouping per
  folder would need several-path items capped at 64 parts and whole-or-nothing skips; not worth it now (agent's call 4).
- **A log written by a process that keeps it open but quiet for a week**: moved while open, the writer keeps writing
  into the file in the Trash; the cleaner's check 10 sees the open descriptor of the user's processes and skips it.
  Processes of other users are not seen (a known issue).
- **More matches in the walk**: every entry at depths 1 to 6 under `Logs` becomes a match of this module (a few hundred
  on the developer's Mac; `JetBrains` and id folders stay opaque). The walk itself does not change shape and reads
  nothing more, so no scan bench is due (agent's call 6); the user's walk notes the scan time.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated (translated): "work autonomously with subagents, take the recommended decisions;
  implement all the features; write into `docs/deferred` what I will need to look at." This design is approved by it;
  the draft's agent's calls (only the user's Library; older than 7 days, a setting; files go, folders stay; a module of
  its own) stand.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

1. **The place's contract as the second exception of "Unknown is not junk"** (section 5): every regular file under
   `~/Library/Logs` is claimed whatever its name, bounded by rows 1 to 8, at Safe. Cost if wrong: an app's file kept in
   `Logs` that is not a log goes to the Trash after a week untouched; the fallback is a list of log extensions, which
   would miss most of the real Mac's files (`.json`, `.html`, dated rotations).
2. **The logs of an installed app under its bundle id are not offered** (non-goals), against the draft's "the same
   installed apps decide, as in plan 030": the walk never shows this module a file inside a bundle-id folder, since
   `app-leftovers` claims those folders whole. Deferred. Cost if wrong: such logs wait (on the developer's Mac the only
   id-named folders are Apple's).
3. **A first name shaped as a bundle id is refused only as a folder** (more names follow it), the very folders
   `app-leftovers` claims; a top-level file shaped as one (`system.log`, `install.log`, `com.example.App.log`) is
   decided by this module's rows. Revised in 01/02's fix round: refusing every bundle-id-shaped name kept almost every
   top-level log, since the shape needs only one dot. Cost if wrong: a top-level file an app names by its bundle id
   is offered here once a week old, though `app-leftovers` would have attributed it to its app.
4. **One finding per file**, not per folder. Cost if wrong: long lists in Details.
5. **No app attribution and no running-apps check**: the reason names the folder; the open-file check at the act
   protects a log in use. Cost if wrong: logs are not counted in their app's line of the map.
6. **No scan bench**: the walk's shape does not change. Cost if wrong: an unmeasured cost on a home with a very large
   `Logs` (the user's walk notes it).
7. **State and lock suffixes are not claimed** (row 3): `.plist`, `.db`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`,
   `.lock`, `.pid`. Cost if wrong: a few small files stay.
8. **Six names deep**; deeper files are deferred. Cost if wrong: a deeper log stays.
9. **The setting from 7 to 90 days, 7 by default**, one for both categories. Cost if wrong: a number.
10. **Group System; module `logs`; categories `logs.app-logs` and `logs.crash-reports`, both Safe.** Cost if wrong: a
    name or a group to change.
11. **No new dependency, no new UI component.**

## Conclusions

Built as designed, with one change (call 3, a top-level file shaped as a bundle id is offered) and the calls taken
while building: see [the global readme](./readme.md#conclusions).
