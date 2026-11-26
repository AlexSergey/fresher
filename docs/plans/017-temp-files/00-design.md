# 017 — Temporary files: design

**Status:** ✅ approved (the user's decisions and delegation, 2026-11-17) · **Version:** 0.1.0 · **Date:** 2026-11-17 ·
**Modules:** core, macos, system, scanner, engine, cleaner, testing, sandbox, modules, app, app/ui, docs

Written on 2026-11-17 from the agent's research (kept outside the repository) and the user's four decisions of the same
day. Every other open question is answered with the research's recommendation, taken by delegation and recorded under
"Decisions"; the agent's calls wait for the user's check. Facts about macOS marked "to verify" are checked by hand,
read-only, in 04/02.

## Intent

The user's request of 2026-11-17, translated: a small, careful module for temporary files — one category, "Old
temporary files", of the user's own temporary folder and the user's own items in `/private/tmp`, Safe, with a clear
explanation of what macOS already cleans on its own. Unfinished downloads are not part of it (a future module of their
own). Seven days by default, a number the user may only raise. What belongs to the system is shown and never offered,
with the reason.

## What the research found

- **macOS already cleans both temporary folders.** The user's temporary folder (`$TMPDIR`, canonically
  `/private/var/folders/<xx>/<hash>/T`) is swept by a system job daily at 03:35 of whatever is older than 3 days.
  `/private/tmp` is emptied at every start of the Mac and, since macOS 14, swept daily at midnight of the regular files
  whose access, modification and status-change times are all older than 3 days, and of empty folders older than 3 days,
  never touching the X11 and quota lock names.
- **So the module finds little on a healthy Mac.** On the developer's Mac: `T` held 790 MB in 4,720 entries, one older
  than 3 days — a root-owned, system-locked updater package from 2024 that the system's own sweep cannot remove either.
  `/private/tmp` held 899 MB, all from that day.
- **What survives the sweep** is what the module mostly sees: items of another owner (root), items with protective
  flags, items read in the last days, items of a Mac asleep at every sweep. And what is young is young because a program
  is using it.
- **The value is honest explanation** as much as space: "your Mac clears this on its own; this one item it cannot, and
  here is why".
- **Nearby places are not temporary**: `C/` (system and app caches), `0/` (system services' state: the app database,
  notifications), `X/` (code-signing clones of running apps), `T/TemporaryItems` (safe-save staging, the only copy of a
  save that crashed), `Autosave Information` (unsaved documents), the Versions database, `/private/var/tmp` (persistent
  by contract, mostly root's), crash reports (logs, the system-junk idea's), editor swap and lock files next to
  documents, `.DS_Store` (the folder's view settings).
- **Times lie in three ways**: archive tools keep old modification times (a folder unpacked a minute ago looks years
  old); a folder's own times do not move when a file deep inside is written; access times on APFS are updated lazily
  and can be wrong in both directions. The status-change time cannot be set by an app and moves on every write, rename
  or permission change.
- **Sockets and named pipes are common there**, and the core reads a named pipe as a regular file today
  ([known issue](../../known-issues/fixed/core-fifo-read-as-file.md)).

## Goals

1. One category, `temp.old-files`, **Safe**: the top-level items of the user's temporary folder and the user's own
   top-level items of `/private/tmp` that nothing changed for more than the setting's days (7 by default), with texts
   that say what macOS does on its own.
2. **Never offer** what is young, in use by an open app, the system's or another user's, locked, a socket or a pipe, or
   a lock; report each as left out with its reason (plan [011](../011-diagnostics/readme.md)).
3. The **core prerequisites** the research named: the two places as known folders with their protection, walked even
   though they lie outside the home; a fourth entry kind; the status-change time and the newest change of a whole
   subtree; a sandbox that can state the facts it cannot make on disk.
4. Tests at every level, as plan [016](../016-jetbrains/readme.md) did: unit, fixture, a sandbox case file with its
   left-out expectations and its clock, core e2e, UI e2e, a run by hand.

## Non-goals

- **Unfinished downloads and old installers** (decision 2): a future "downloads and installers" module, written down as
  the idea [Downloads and installers](../../ideas/downloads-and-installers.md).
- **Sandboxed apps' `Containers/<id>/Data/tmp`** (decision 8): they need Full Disk Access and held nothing worth it on
  the developer's Mac; added later if a real Mac shows sizes.
- **Crash and diagnostic reports** (decision 10): logs, not temporary; the [system-junk](../../ideas/system-junk-cleaner.md)
  idea's.
- **Admin places** (`/private/var/tmp`, root's items): not walked or never offered (decision 4, 11); a separate admin
  step belongs to the system-junk idea.
- **`~/Library/Caches`**: another module, with other ages and texts.
- **Quitting an app** to free its items (decision 13).
- **A summed finding of many small items** (agent's call 3): every offered item is its own finding.

## What already exists

| Need                                   | What exists                                                                                                     | Plan     |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| A module crate and its checks          | `modules/<id>/`, `ModuleDescriptor`, `assert_module_contract`, `run_module`, goldens ([`modules.md`](../../architecture/modules.md)) | 002 |
| A place under a known folder           | `Pattern::Under(KnownFolder, "*")`, `Claim::Whole` sums a folder                                                | 002, 004 |
| The owner and the flags of the entry   | `Match.facts: EntryFacts` (`owned_by_user`, `user_immutable` with the user's append flag, `system_immutable` with the system's append and no-unlink flags, `restricted`), read by the bulk reader; the guard's check 8 refuses them | 006, 008 |
| A pipe never blocks the guard          | the guard opens every item `O_NOFOLLOW_ANY | O_NONBLOCK` (`crates/cleaner/src/paths.rs`); no test proves it on a pipe yet | 008 |
| Items in use                           | the guard's check 10 (`OpenFiles`, the user's processes)                                                        | 008      |
| Running and installed apps             | `cx.system().running_apps().owner_of(path)`, `installed_apps()`, `BundleId::names`                              | 009      |
| Saying why something is not offered    | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                        | 011      |
| A finding's app in the storage map     | `NewFinding::for_app(bundle_id)`                                                                                | 012      |
| Groups and ignoring them               | `ModuleGroup`, `IgnoreTarget::{Module, Group}`                                                                  | 015      |
| A restore never judges by age          | `ModuleContext::is_restored()`                                                                                  | 016      |
| The sandbox's world and clock          | `world.toml` (`now`, running apps), `[[case.left-out]]`, `[[dir]] age`, the sandbox clock at `now`              | 016      |
| A number setting with a floor          | `jetbrains.uninstalled-after-days` (7 to 3650)                                                                  | 016      |

## Design

### 1. The places

Two new known folders (`crates/core/src/known_folder.rs`), each with its macOS answer (`crates/system/src/known_folders.rs`):

| Known folder | Rules write | macOS answer                                                                                  |
| ------------ | ----------- | --------------------------------------------------------------------------------------------- |
| `UserTemp`   | `user-temp` | the user's temporary folder from the system's configuration (`confstr(_CS_DARWIN_USER_TEMP_DIR)`), made canonical (`/private/var/folders/<xx>/<hash>/T`); never the `TMPDIR` variable, which a shell or a launch agent may change |
| `SharedTemp` | `shared-temp` | `/private/tmp` (never the link `/tmp`)                                                     |

- `confstr` is one call in `fresher-macos`'s `ffi` (one `unsafe` block, its `// SAFETY:` comment); the composition
  root reads it once and hands the canonical path to `MacKnownFolders`, which stays pure. A failure gives no
  `UserTemp`: the folder then matches nothing.
- The canonical form matters: `/var` and `/tmp` are links, and the walk and the cleaner follow no link
  (`O_NOFOLLOW_ANY`).

### 2. Protection

`crates/macos/src/protection.rs` gains, both forms of each path as today:

| List     | Places                                                                                                            |
| -------- | ----------------------------------------------------------------------------------------------------------------- |
| `never`  | `/private/var/folders` (every user's `C/`, `0/`, `X/` and other users' triplets stay there), `/private/var/tmp`, `/private/var/vm`, `/.DocumentRevisions-V100`, the user's `T/TemporaryItems`, `~/Library/Autosave Information` |
| `except` | the user's own `T` (carved out of `/private/var/folders`)                                                        |
| `whole`  | the user's own `T`, `/private/tmp`                                                                                |

So only items **inside** `T` (but `TemporaryItems`) and inside `/private/tmp` can ever be reached; neither folder can be
removed as a whole. A folder setting naming any new place is refused by the core's settings check, as for the others.
The containers' `Data/Library/Autosave Information` cannot be listed without a wildcard; it rests on the module's
narrow places (agent's call 9).

### 3. Walking places outside the scan roots

The scan roots are the home by default; `T` and `/private/tmp` lie outside it. A **place** is a known folder that a
registered module's interest names and no scan root holds: the engine passes the resolved places to the walk
(`WalkRequest.places`) beside the roots (agent's call 8).

- A place is walked like a root: no link followed, never another volume; its folders are not entered when they belong
  to another user (counted `NotScanned::OthersFolder`, never unreadable, so `/private/tmp` with root's folders does not
  make every scan incomplete); `T/TemporaryItems` is never opened (`NotScanned::SystemItems`, see below), since
  listing it is refused by the system's privacy protection.
- Only the module's own interests match inside a place; a place is only walked while one module names it, enabled or
  not (the walk does not change with a module's switch).
- The cleaner counts the places as roots (its check 2: "strictly inside a root"), from the same list the scan used.
- The storage map shows each place as a by-place root of its own after the scan roots, named by its known folder
  (`core.place.user-temp.name` "Your temporary folder", `core.place.shared-temp.name` "Shared temporary folder"),
  on the home's volume; the parts still add up.
- The places are not in the scan cache's key: a cache whose places changed is found and refused
  (`FullScanReason::OtherRoots`, a full scan; while they were in the key it was another, empty slot, `NoCache`). A
  rescan walks every place again in full whatever the journal says, and a restore compares the scan roots only and
  emits nothing inside a place (decision 21): after a restore the map shows the places' cached sizes, and nothing
  there is offered.
- Only `UserTemp` and `SharedTemp` are ever places (`KnownFolder::is_walked_place`), and only outside the home: a
  module's home-anchored interests apply only where the user scans, and no other folder (`/Applications`, say) becomes
  a place read in full on every scan (01/04).
- A place that does not exist or cannot be read is left out of the walk, counted as not scanned with why
  (`NotScanned::Unreadable`), never an error of the walk and never making it incomplete. A place that holds a scan
  root, or lies inside one, is not walked: the scan root is scanned as the user asked (the settings still allow a
  folder inside a temporary folder as a root, as the test fixtures' temporary folders are; 01/01's ruling).
- `T/TemporaryItems` is reported for its own reason (`NotScanned::SystemItems`), never as needing Full Disk Access.

### 4. A fourth entry kind

`EntryKind::Other`: a socket, a named pipe, a device — read from the bulk read's object type (and `NaiveWalk`'s file
type). The matcher never counts it as a marker; an `Under` interest matches it as itself, so a module can see and refuse
it. The guard refuses an item whose kind is now `Other` (check 7: not the confirmed kind). A test proves the guard opens
a pipe without blocking. The [known issue](../../known-issues/fixed/core-fifo-read-as-file.md) is marked fixed.

### 5. Times and the subtree

- `RawEntry` and `Match` gain `changed` (the status-change time, `ATTR_CMN_CHGTIME`) and `accessed`
  (`ATTR_CMN_ACCTIME`), beside `modified`. The creation time is not read: it is never later than the status-change time
  (agent's call 1).
- A `Claim::Whole` match gains `inside: Option<Inside>`, gathered by the walk while it sums the folder (no second pass):

  ```rust
  pub struct Inside {
      /// The newest of `modified` and `changed` over the folder and everything inside it.
      pub newest_change: Option<Timestamp>,
      /// The newest `accessed` of the files inside (folders' access times move when the scan lists them).
      pub newest_file_access: Option<Timestamp>,
      /// A socket, a pipe or a device somewhere inside.
      pub special: bool,
      /// An entry inside that is not the user's, or carries a lock or a system flag.
      pub foreign: bool,
      /// Every folder inside was read.
      pub complete: bool,
  }
  ```

  For a file or a link `inside` is `None` and the entry's own times and facts stand. The cache stores the new fields
  (its format version goes up), so a restore hands them back; the reference walk computes the same (a property test).

### 6. The sandbox's facts

`ctime` cannot be set by any tool, and owners and system flags need root; a socket's path is limited to 104 bytes, more
than a sandbox in a repository allows. So the sandbox **states** these facts instead of making them (decision 18):

- The tree vocabulary of fixture homes and case files (`crates/testing`) gains, on `[[file]]` and `[[dir]]`:
  `changed = "<n>d|<n>h"`, `accessed = "<n>d|<n>h"`, `owner = "system" | "other-user"`,
  `flags = ["user-lock", "system-lock", "no-unlink", "restricted"]`, and on `[[file]]` `kind = "socket" | "pipe"`.
  The file is written as a plain file or folder; the facts go to a list.
- `FactsLens` (`crates/testing`): a `FolderReader` that wraps the real one and, for exactly the listed paths, reports
  the stated facts instead of the disk's. `run_module`, the core e2e and the sandbox composition use it; nothing else
  does, and a release build never contains it.
- The generator writes the list into `world.toml` (`[[entry]]`: path and facts); sandbox mode and the autotest wrap the
  sandbox's reader with it. `EXPECTED.md` names a stated fact ("stated: owned by the system").
- The sandbox's layout gains `private/var/folders/xx/sandbox/T/` and `private/tmp/` beside `home/`, its known folders
  `UserTemp` and `SharedTemp` point there, and its protection list mirrors section 2 on them.
- Real sockets and pipes are tested where paths are short: the scanner's and the cleaner's own tests (a pipe made with
  `mkfifo`, a socket bound under a short temporary folder).

### 7. The module

`modules/temp` (`fresher-module-temp`), group **System** (a new `ModuleGroup::System`, decision 15), registered in
every build. Code, not rules: the answer depends on times, kinds, owners and the running apps.

Interests, `Claim::Whole` each: `Under(UserTemp, "*")`, `Under(SharedTemp, "*")`. One match is one top-level item.

The decision, per match, the first row that applies:

| #  | Situation                                                                                                         | Result                                         |
| -- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| 1  | the scan is restored from the cache (`cx.is_restored()`)                                                          | left out `temp.left-out.scan-again`            |
| 2  | the name is a lock or a system marker: `.X*-lock`, `.X11-unix`, `.ICE-unix`, `.font-unix`, `.XIM-unix`, `quota.user`, `quota.group`, `.vfs_rsrc_streams_*`, `.AddressBookLocks`, `.LINKS`, `*.lock`, `*.pid`, `LCK..*`, `.#*` (bytes, ASCII case ignored); or a folder with `inside.lock_named` (an entry named `*.lock`, `*.pid`, `LCK..*` or `.#*` anywhere inside; ruling of 02/02) | left out `temp.left-out.lock` |
| 3  | the entry is `Other`, or a folder with `inside.special`                                                           | left out `temp.left-out.special`               |
| 4  | not the user's, `system_immutable`, `restricted`, a folder with `inside.foreign`, or a name starting `com.apple.` | left out `temp.left-out.system`                |
| 5  | `user_immutable` (locked by a user, in Finder's Get Info)                                                         | left out `temp.left-out.locked`                |
| 6  | the running apps cannot be listed                                                                                 | left out `temp.left-out.no-system`             |
| 7  | named after a running app (`running_apps().owner_of(path)`; as built, the name's bytes, also with one leading dot stripped: ruling of 02/02) | left out `temp.left-out.app-open` |
| 8  | a folder whose inside was not read in full (`!inside.complete`)                                                   | not claimed (the walk reports what it could not read) |
| 9  | the newest change (of the subtree, or the entry's own for a file or a link) is unknown, in the future, or not more than the setting's days ago; or a file inside was read within those days | left out `temp.left-out.recent` |
| 10 | otherwise                                                                                                         | **offered** in `temp.old-files`                |

- "More than N days" is `now − newest > N × 86,400 s`: exactly 7 days is kept, 7 days and one second offered.
- The clock is `cx.clock()`; a time after it counts as now.
- A **link** at the top level is offered as itself when old; the cleaner moves the link, never its target.
- The module does not check open files at the scan: the guard's check 10 does at the act (skipped `InUse`, named).

A finding:

| Part     | Value                                                                                                                  |
| -------- | ---------------------------------------------------------------------------------------------------------------------- |
| category | `temp.old-files` (Safe)                                                                                                |
| identity | `<place>/<name>` (`user-temp/tmp.IKKNWYIRnq`, `shared-temp/build-cache`)                                               |
| subject  | `Subject::Path` of the item; removal `Removal::MoveToTrash` of the same path                                           |
| sizes    | the match's (the whole folder)                                                                                         |
| title    | `temp.finding.title`: "{name}"                                                                                         |
| reason   | `temp.finding.reason-user`: "In your temporary folder, unchanged for {days} days." / `temp.finding.reason-shared`: "In the shared temporary folder, unchanged for {days} days. A restart clears it too." |
| app      | `for_app(id)` when the name is named after an installed app's bundle id                                                |

### 8. Texts

The category (rule 7: plain, short, what you keep or lose):

- **name:** Old temporary files
- **what:** Working files that apps and tools put in your Mac's temporary folders and did not clean up. Nothing has
  changed them for at least a week.
- **made-by:** Many apps and command-line tools; an item's name often says which.
- **if-removed:** They go to the Trash; the space comes back when you empty it. Apps keep their real data elsewhere.
  An item named after an app that is open is kept, and an item in use when you confirm is skipped.
- **comes-back:** Apps make new temporary files as they work. Your Mac also clears these folders on its own: every day
  it removes what is older than three days, and a restart empties the shared one.

The module's switch: `temp.enabled.label` "Temporary files", `.description` "Finds old working files in your Mac's
temporary folders that apps did not clean up." The setting `temp.older-than-days` (number, days, 7 to 30, 7 by
default; decision 3, 9): `.label` "Offer temporary files unchanged for", `.description` "Anything changed more
recently is kept. It cannot be set below 7 days, so the check never becomes less careful."

The left-out reasons, declared with `with_problems`:

| Key                         | Text                                                                                         |
| --------------------------- | -------------------------------------------------------------------------------------------- |
| `temp.left-out.system`      | It belongs to the system or another user; removing it needs an administrator.               |
| `temp.left-out.locked`      | It is locked. To let Fresher remove it, unlock it in Finder's Get Info.                       |
| `temp.left-out.app-open`    | It belongs to an app that is open. Quit the app and scan again.                              |
| `temp.left-out.special`     | A program uses it to talk to another program, so it is kept.                                 |
| `temp.left-out.lock`        | A program uses it to mark something as in use, so it is kept.                                |
| `temp.left-out.recent`      | Changed or read recently; an app may still be using it. Your Mac clears these on its own when it can. |
| `temp.left-out.scan-again`  | Shown from the last scan. Scan again to check how old it is.                                 |
| `temp.left-out.no-system`   | Fresher could not list your open apps, so it keeps everything. Scan again.                   |

The group: `core.group.system.name` "System", `.description` "What macOS and your apps keep for themselves, such as
temporary files."

### 9. Tests

**Unit** (`modules/temp/src/{names,decide}.rs`, negatives first): the lock and system names (similar names:
`com.apple` without the dot, `lockfile`, `x.locked`); the decision table row by row on plain values, with the
precedence (restored over everything, special over system, system over age); the age boundary (exactly N days, N days
and one second), a future time, an unknown time, the access-time veto, the setting's floor (a value below 7 read as 7).

**Fixture** (`modules/temp/tests/fixtures.rs`, `run_module` with the facts lens): `fixtures/homes/temp-*.toml`,
negatives first — `temp-recent.toml` (a fresh file, an unpacked folder with 2019 modification times and a fresh
status change, an old folder with one file changed yesterday), `temp-system.toml` (stated root owner, no-unlink,
`com.apple.x`, a folder with a root-owned file inside), `temp-special.toml` (a socket, a pipe, a folder holding one,
lock names), `temp-running.toml`; then `temp-old.toml` (an old file, an old folder, an old item of `/private/tmp`, an
old link, an item named after an installed app). Golden `fixtures/golden/temp.json`. The contract test with
`Catalogue::Required`.

**Sandbox** (`fixtures/sandboxes/temp.toml`, `feature = "temp"`, `module = "temp"`, golden
`fixtures/golden/sandbox-temp.json`; the clock at the world's `now`). Negatives first:

| Case (negative)                                                         | Files and stated facts                                                                    | Expected                                   |
| ----------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------ |
| a fresh temporary file is kept                                          | `T/fresh.tmp` (real times: generated now)                                                 | left out `recent`                          |
| an unpacked folder with old modification times is kept                  | `T/unpacked/` with `age = "2000d"` everywhere, no stated `changed`                        | left out `recent`                          |
| an old folder with one file changed yesterday is kept                   | `T/build/` `changed = "30d"`, `build/out.o` `changed = "1d"`                              | left out `recent`                          |
| an old file read yesterday is kept                                      | `T/read.dat` `changed = "30d"`, `accessed = "1d"`                                         | left out `recent`                          |
| six days old is kept                                                    | `T/six-days` `changed = "6d"`, `age = "6d"`                                               | left out `recent`                          |
| an item of a future time is kept                                        | `T/future` `age = "-2d"` (a modification time after now)                                  | left out `recent`                          |
| an open app's temporary folder is kept                                  | `com.example.Editor` running; `T/com.example.Editor.tmp/` old                             | left out `app-open`                        |
| a system service's folder is kept                                       | `T/com.apple.someagent/` old; `T/daemon/` old, `flags = ["no-unlink"]`                    | left out `system` ×2                       |
| an item of the system or another user is kept                           | `T/updater.pkg` old, `owner = "system"`, `flags = ["user-lock"]`; `private/tmp/theirs` old, `owner = "other-user"` | left out `system` ×2 |
| a folder holding the system's file is kept                              | `T/mixed/` old, `mixed/root.dat` `owner = "system"`                                       | left out `system`                          |
| an item you locked is kept                                              | `T/kept.txt` old, `flags = ["user-lock"]`                                                 | left out `locked`                          |
| a socket, a pipe and a folder holding one are kept                      | `private/tmp/dev.sock` `kind = "socket"`, `T/pipe` `kind = "pipe"`, `T/srv/s.sock` socket, all old | left out `special` ×3             |
| locks are kept                                                          | `private/tmp/.X0-lock`, `T/app.lock`, `T/server.pid`, all old                             | left out `lock` ×3                         |
| the temporary folder's neighbours are never claimed                     | old files in `C/`, `0/`, `X/` and `T/TemporaryItems/`                                     | nothing offered, no left-out               |
| unsaved documents are never claimed                                     | `home/Library/Autosave Information/Untitled.rtf` old                                      | nothing offered                            |
| another user's temporary folder is never claimed                        | `private/var/folders/yy/other/T/old.tmp` old                                              | nothing offered                            |
| editor and Finder files in the home are never claimed                   | `home/notes/.notes.txt.swp`, `#notes.txt#`, `report.docx~`, `.DS_Store`, `__MACOSX/` old  | nothing offered                            |
| the target of a link in the temporary folder is never touched           | `home/Documents/linked/keep.txt` (the target of the positive's link)                     | nothing offered; unchanged after the cleanup |

Then the positives:

| Case (positive)                                            | Files                                                              | Expect                                         |
| ---------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------- |
| an old temporary file goes to the Trash                    | `T/tmp.IKKNWYIRnq` `changed = "30d"`, `age = "30d"`, `fill = "data"` | `temp.old-files`, safe, `in-trash`; map explained |
| an old folder goes to the Trash as a whole                 | `T/node-compile-cache/` and its files, all `changed = "20d"`      | `temp.old-files`, safe, `in-trash`             |
| your old item in the shared temporary folder goes          | `private/tmp/old-build.log` `changed = "10d"`                      | `temp.old-files`, safe, `in-trash`             |
| an installed app's old temporary folder counts for the app | `com.example.Viewer` installed, not running; `T/com.example.Viewer/` `changed = "15d"` | `temp.old-files`, safe, `in-trash`; in the app's line |
| seven days and one second is old                           | `T/boundary` `changed` = 7 days + 1 s                              | `temp.old-files`, safe, `in-trash`             |
| an old link goes to the Trash as itself                    | `T/docs-link` → `home/Documents/linked`, the link `changed = "30d"` | `temp.old-files`, safe, `in-trash`; its target stays |

A `[case.map]` on the first positive (`explained`, at the item) and on the fresh file (`your-folders`, at
`T/fresh.tmp`: shown in its place's root, never explained). The future-time case needs a negative `age` (01/05).

**Core e2e** (`crates/e2e/tests/temp.rs`, the engine with the real scanner, the facts lens, the real settings file and
the real `DiskCleaner` in a fixture sandbox whose places lie outside its home; fixture `fixtures/homes/temp-flow.toml`,
golden `fixtures/golden/temp-flow.json`):

1. **Scan → confirm → Trash → put back**: the old items are offered under System → Temporary files; the plan names
   Safe; confirm; they are in the Trash, the journal has them; put back restores them byte for byte.
2. **Real times**: a folder unpacked with 2019 modification times (set with `utimes`, so its status-change time is now)
   is kept; nothing stated.
3. **Real sockets and pipes**: a pipe and a socket in the shared place are left out `special`; the scan never blocks.
4. **Changed between scan and act**: an offered file rewritten after the plan, by a new file renamed over it, is
   skipped `Replaced` (the guard has no `Changed` reason; check 7 compares device, file id and kind); the rest is
   done.
5. **Restore at launch**: nothing offered and nothing left out, since a restore emits nothing inside a place
   (decision 21; the module's `scan-again` row is a second guard); the map still lists both places; a rescan offers
   again.
6. **The setting**: 14 days keeps a 10-day item (left out `recent`); 3 is refused, the value stays 7.
7. **Ignore**: ignoring the module, then the group System, hides every answer; restore brings it back.
8. **The storage map**: the two places are by-place roots; the offered items are explained.
9. **The cleaner's roots**: an item of a place is refused once the place is no longer walked (no module names it).

**UI e2e** (Playwright, mocked core; `app/ui/src/test/temp-scan.ts` with the module's real texts):

1. A scan shows **System → Temporary files** in the sidebar with its size; its page shows the category, Safe, and its
   five texts, with the sentence on what macOS clears on its own.
2. The left-out lines: "4,712 items left out (790 MB): Changed or read recently; …" and the system line.
3. **Select → Review → confirm to the Trash → result → Cleanups → put back** on the temporary findings; the
   confirmation names Safe.
4. Settings → Temporary files: the number field starts at 7 and refuses a value below 7 with its message.
5. **Ignore the module** from its page, then **the group System** from Settings; restore.
6. The overview's by-place view lists "Your temporary folder" and "Shared temporary folder".

Visual and accessibility: the module's page with its left-out lines, and the setting, in `visual.spec.ts` and
`axe.spec.ts`.

**By hand** (04/02): the sandbox walked with `EXPECTED.md`; on the developer's Mac a scan only (a development build's
cleaner reaches only a sandbox), compared with the research's read-only commands.

### 10. Where the code lives

| Crate or folder              | What it gains                                                                                                    |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `crates/core`                | `KnownFolder::{UserTemp, SharedTemp}`, `EntryKind::Other`, `RawEntry`/`Match` `changed`, `accessed`, `inside`, `WalkRequest.places`, `NotScanned::OthersFolder`, `ModuleGroup::System` and its texts |
| `crates/macos`               | `ffi` `confstr`; the bulk reader's object types and two times; protection entries                                |
| `crates/system`              | `MacKnownFolders` with the user's temporary folder                                                               |
| `crates/scanner`             | the kind, the times and `Inside` in the walk, the cache and the reference walk; places as roots                  |
| `crates/engine`              | the places from the interests; the cleaner's roots; the map's place roots                                        |
| `crates/cleaner`             | check 7 refuses `Other`; the pipe test                                                                          |
| `crates/testing`, `crates/sandbox` | the stated facts and `FactsLens`; the sandbox's temporary places                                           |
| `modules/temp/` (new)        | `fresher-module-temp`                                                                                            |
| `app/tauri`, `app/ui`        | registration, composition (`confstr`, the lens in sandbox mode), the group's and places' texts, the tests       |

No new third-party dependency (`libc` is already `fresher-macos`'s).

## Interfaces with other plans

- **016** (merged before this is built): `is_restored`, `world.toml` with `now`, `[[case.left-out]]`, the sandbox clock,
  the number setting's pattern, the generic module page. Its walk-complete flag is not needed: `Inside.complete` is
  per item.
- **015**: `ModuleGroup` and ignoring a group (System added here).
- **012**: the storage map's by-place roots (places added).
- **011**: left-outs and declared problems.
- **009**: running and installed apps.
- **008**: the guard, its checks 2, 7, 8 and 10; the Trash.

## Open questions

None left: the user decided four, the rest were delegated and decided below.

## Decisions

The user's, of 2026-11-17 (translated):

1. **A small, careful module**: one category, "Old temporary files" — the user's own temporary folder
   (`$TMPDIR`, `/private/var/folders/<xx>/<hash>/T`) and the user's own items in `/private/tmp` — danger **Safe**, with
   a clear explanation of what macOS cleans on its own (a daily sweep of what is older than 3 days; `/tmp` at every
   start).
2. **Unfinished downloads are not in this module**: a separate future module, "downloads and installers" (partial
   downloads, old installers of apps already installed, Safari's resumable `.download` at Medium; Downloads is
   protected by the system's privacy prompt). Written down as an idea.
3. **Seven days by default**; a module setting that can only be raised (7 at least).
4. **Items owned by root or the system, or with protective flags, are shown and never offered**: reported as left out
   with a reason ("belongs to the system; removing it needs an administrator"), through plan 011's left-outs.

Taken by delegation on 2026-11-17 (the user's instruction: accept the recommended decisions):

5. **The plan is approved** as designed here, with the agent's calls below still to check.
6. **Opaque names are claimed by the place's contract** (the research's Q1): the system itself removes anything in
   these folders after 3 days, so a top-level item of the user's own `T` or `/private/tmp` that passes every row of
   section 7 is offered whatever its name. This exception is written into
   [`deletion-safety.md`](../../restrictions/deletion-safety.md) ("Unknown is not junk") and the category's "How the
   app decides" (04/01), so it is a decision, not a loophole. Nowhere else does a name the module cannot read count.
7. **Danger Safe** (Q2), with the age, running-app, owner, flag and kind rows; never Medium.
8. **Sandboxed apps' `Data/tmp` not now** (Q4).
9. **The setting runs from 7 to 30 days** (Q7), 7 by default; a setting may only make the check stricter.
10. **Crash reports are not here** (Q8): the system-junk idea's.
11. **Admin places are never offered** (Q9): `/private/var/tmp` is not walked at all; root's items in the two places
    are left out `system`.
12. **The core prerequisites come first** (Q10), as plan 01.
13. **No app is asked to quit** (Q11): an open app's items are left out with "Quit the app and scan again".
14. **Built small** (Q12): one category and the explanations; nothing for containers or admin places until real data
    shows sizes.
15. **A new group, System** (`ModuleGroup::System`), not Development: temporary files are every user's, not a
    developer's.
16. **Age is the newest change of the whole item** (section 2.7 of the research): the newest of the modification and
    the status-change times over the item and everything inside it; the access time only keeps (a file read within the
    days keeps its item), never offers; a time in the future counts as now.
17. **A restore offers nothing**: its times are as old as the cached scan; every candidate is left out
    `scan-again`. Since decision 21 a restore emits nothing inside a place, so this row is the module's second guard
    (ruling R8 in "Conclusions").
18. **The sandbox states the facts it cannot make** (ctime, owners, system flags, sockets and pipes) through the facts
    lens; real ones are tested where they can be made (section 6).
19. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").
20. **Built after plan 016 is merged into `main`**, on this branch rebased onto `main` then.

Ruled by the controller on 2026-11-17, from the review of 01/03:

21. **The walked places are always read from the disk.** A rescan could hand back cached access and change times older
    than the disk's (the change journal reports no read, and a status change through a hard link elsewhere moves no
    event in the folder): the age-based module would then be less careful than after a full scan. So `UserTemp` and
    `SharedTemp` are walked again in full on every scan, whatever the journal says (they are small), and a restore
    emits nothing inside them (the module also offers nothing on `is_restored`). Built in 01/04.

## Agent's calls (to check)

1. **No creation time**: it is never later than the status-change time, so it adds nothing to "the newest change".
   Costs, if wrong: one more attribute in the bulk read.
2. **The access-time veto counts files only**: listing a folder may move its own access time, so the scan would keep
   every folder. Costs, if wrong: a folder read but not written is offered after the days.
3. **One finding per top-level item, no size floor, no summed "small items" finding** (the research's Q6 recommended
   listing items of 1 MiB or more and summing the rest): after the 7-day row few items remain on a real Mac, the lists
   are paged, and a summed finding would need a finding of many paths the core does not have. Costs, if wrong: a
   multi-item finding designed when a real Mac shows many old small items.
4. **System names**: the `com.apple.` prefix, and the system's no-unlink flag on a folder, say "a part of macOS"; the
   lock names of section 7, row 2, are a fixed list in `modules/temp/src/names.rs`. Costs, if wrong: a name added.
5. **The left-out texts**: the user's "belongs to the system; removing it needs an administrator" is extended with "or
   another user", since another user's items in `/private/tmp` need an administrator too; the user's lock gets its own
   line, since it needs none.
6. **The facts lens lives in `crates/testing`**, a `FolderReader` wrapper used by fixtures, the core e2e and the
   sandbox composition only. Costs, if wrong: the stated facts moved into the sandbox crate.
7. **Sockets and pipes in the sandbox are stated, not made** (a socket's path is limited to 104 bytes). Costs, if
   wrong: the generator makes real ones in a short folder outside the sandbox, which breaks "nothing beside the
   sandbox".
8. **Places are walked only while a module's interest names them**, never offered to the user as scan roots, and
   another user's folders inside them are not entered. Costs, if wrong: places as settings rows.
9. **Protection entries** (section 2); the containers' autosave folders rest on the module's narrow places. Costs, if
   wrong: a pattern in the protection list.
10. **A link at the top level is offered as itself** when old. Costs, if wrong: links left out as `special`.
11. **The group's order** is System, Development, Debug, and its texts as in section 8.
12. **The Trash of a `/private/var/folders` item** is the user's `~/.Trash` (one volume, Data) and Finder's Put Back
    restores it: to verify by hand in 04/02 with a scratch file and Finder, never with the app on the real Mac.
13. **The system sweep's age basis** (access or modification time) and whether it runs at start are not verified, so
    the texts say "older than three days" and "a restart empties the shared one" only.
14. **No open-files check at the scan**: the guard's check 10 at the act names the holder; showing "in use by X" at
    the scan is left for later.

## Conclusions

Written when the build ended, before the run by hand ([04/02](./04-finish/02-run-and-conclusions.md)); the conclusions
of the plan are in [`readme.md`](./readme.md). The agent's calls above are 1 to 14; decision 21 was ruled while
building. The other rulings taken while building, R1 to R9, each with what it costs if wrong:

- R1. **A folder inside the user's own `T` or `/private/tmp` may still be a scan root** (01/01): every test whose root
  is a temporary folder lives under the real `T`; `T` itself, its neighbours, `TemporaryItems` and `/private/tmp` itself
  are refused. Cost if wrong: a user who picks such a folder gets it scanned as an ordinary root.
- R2. **The user's temporary folder is exactly `/private/var/folders/<xx>/<hash>/T`** (01/01): one shared check
  (`is_user_temp`) for the composition and the protection list; any other answer carves nothing and gives no `UserTemp`.
  Cost if wrong: on such a Mac the module offers nothing in `T`.
- R3. **A place lies outside the home, and a place that holds a scan root or lies inside one is not walked** (01/04),
  instead of refusing such roots. Cost if wrong: temporary items under such a root are not offered.
- R4. **A place that is missing or cannot be read is not scanned, with why, and never stops a scan or a restore**
  (01/04). Cost if wrong: none for safety; such a place's items are not offered.
- R5. **A folder's own access time is never reported** (01/03, `Match.accessed` is `None`): listing it moves it. Cost if
  wrong: the same as agent's call 2.
- R6. **The app match strips one leading dot and compares bytes; a folder holding a lock-named entry anywhere inside is
  a lock** (02/02 review, `Inside.lock_named`, tree format 5; `.#*` added to row 2). Cost if wrong: a few more items
  kept.
- R7. **The helpers the fixtures and the case file needed beyond the briefs' files are accepted** (02/03, 03/01):
  `Ports::restored`, `MatchOrder`, a link's own age and stated change, a stated entry that is a link itself.
- R8. **Flow 5 follows decision 21** (03/02): a restore emits nothing inside a place, so decision 17's "every candidate
  left out `scan-again`" holds only as the module's second guard (row 1); section 9 was corrected.
- R9. **The core e2e's cleaner is `Reach::SandboxFolder`** (03/02), as sandbox mode composes it: with `Reach::Sandbox`
  the marker would lie inside a place and be matched.

Parked for the final review: small text, comment and test-name points listed in the build's ledger (the category's "at
least 7 days", comment widths, a duplicated test helper in the UI's visual and accessibility specs).
