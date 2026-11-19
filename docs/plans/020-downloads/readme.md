# 020 — Downloads and installers

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, macos, system, scanner, engine,
testing, modules, sandbox, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The idea [Downloads and installers](../../ideas/downloads-and-installers.md): the Downloads folder collects what the
user thinks of as temporary — downloads that never finished, installers of apps that are long installed, old macOS
installers, big files downloaded once and forgotten. Nothing removes them, and together they take gigabytes. Plan
[017](../017-temp-files/readme.md) kept them out of the temporary-files module on purpose (the user's decision of
2026-11-17): they live among the user's own files, some can still be resumed, and the folder is guarded by macOS.

## Goals

1. A new module `downloads` (`modules/downloads`, crate `fresher-module-downloads`) with five categories, each with a
   danger level and a plain reason that says why this file was matched and what it costs to lose it.
2. Nothing in Downloads is read before the user opted in to the folder (plan [006](../006-permissions/readme.md)); without
   access the module is `Unavailable` and its notice says why and how to allow it.
3. A file is claimed only with proof. An installer whose app cannot be identified without opening or running anything is
   not claimed; a file that is only big and old is the user's call, never more.
4. The day setting shared by every module that has one, once in the core (deferred item
   [shared-day-setting](../../deferred/shared-day-setting.md), due now).

## Non-goals

- Mounting a disk image, expanding an archive or running an installer to find out what is inside.
- Looking outside `~/Downloads` (the Desktop, a browser's custom folder): later, if users ask.
- `Install macOS ….app` in `/Applications`: the apps folder is the [smart uninstaller](../../ideas/smart-uninstaller.md)'s
  ground.
- Archives (`.zip`, `.tar.gz`) whose content was already expanded next to them: a separate idea, it needs to compare
  trees.
- Duplicates of a download (`file (1).dmg`): the [duplicate finder](../../ideas/duplicate-finder.md).

## Design

### Categories

| Category                         | What                                                                                                                     | Danger | Offered when                                                                                                   |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------ | -------------------------------------------------------------------------------------------------------------- |
| `downloads.unfinished`           | `.crdownload`, `.part` (with its empty placeholder of the final name, together or not at all), `.opdownload`, `.partial` | Low    | the browser that writes the suffix is not running, and unchanged for the module's days                        |
| `downloads.safari-resumable`     | a `.download` bundle (its `Info.plist` and the partial data)                                                             | Medium | Safari is not running, and unchanged for the module's days; the reason says it can still be resumed           |
| `downloads.installers`           | a `.dmg` or `.pkg` of an app that is installed (see "Proof")                                                             | Low    | proof holds, and the installed app is the same or newer than the installer                                    |
| `downloads.macos-installers`     | `Install macOS <name>.app` in Downloads (12–15 GB each)                                                                  | Low    | the running macOS is the same version or newer than the installer's                                           |
| `downloads.large-untouched`      | a file of at least 1 GB not opened for the module's days, not matched by any category above                             | High   | always "your call": the user's own file; the reason gives its size, kind and last opened date                 |

- The Low levels: everything here downloads again; an unfinished download loses only the part already fetched.
- One setting, `downloads.older-than-days` (7–3650, 30 by default), through the shared day setting.
- Every category `Need::required(Folder(Downloads))`. The walk stays inside `~/Downloads`, one level deep for files and
  bundles; folders inside are not entered except to recognise a bundle.

### Proof for an installer

- **`.pkg`**: the system's installer receipts (`/var/db/receipts/*.plist`, readable without rights) record the package
  file name and the install date. A `.pkg` in Downloads whose file name equals a receipt's package file name, installed
  after the file was downloaded, is proven. Read as files; `pkgutil` is not run. (To verify: the key name and that every
  installer writes it.)
- **`.dmg`**: the image is not opened. A `.dmg` is proven when its file name names exactly one installed app (the app's
  name or the last part of its bundle id, then the end, a separator or a version — `Firefox 133.0.dmg`,
  `docker.dmg`), and that app's bundle changed after the image was downloaded (it was installed or updated from it or
  later). Two apps match, or none → not claimed.
- The download date is the file's "date added" (birth time on APFS; the quarantine attribute's date when it is set).
- An installer of an app that is not installed is the user's file and never claimed (not even as large-untouched: it is
  excluded by name, so it is not offered as "big and old" by accident).

### Last opened

- `large-untouched` needs the last time a file was opened. APFS does not keep a reliable access time; Spotlight's "last
  used" date is reliable when Spotlight indexes the folder. The `AppSearch` port of plan 019 grows one question (a
  file's last used date); without an answer (Spotlight off, not indexed) the file is not claimed and the coverage says
  why.

### What this asks of the core and the sandbox

- **The shared day setting** (plan 01): `fresher-core` gets the descriptor and the reader of a days setting with its
  range and default; `jetbrains`, `app-leftovers` and `temp` use it, their copies and tests are removed.
- **Installer receipts** (`fresher-system`, a new port `InstallerReceipts` with its fake in `fresher-testing`).
- **The access part of the sandbox's world** (deferred item [sandbox-world](../../deferred/sandbox-world.md), item 2,
  `access` only): a case states `access = { downloads = "denied" }`; sandbox mode and the autotest take the access
  state from `world.toml` through `FakeAccess`, and the banner says access is simulated. Open files, tools and steps
  stay deferred.
- The sandbox lays out `Downloads/` in its home; its protection list keeps `Downloads` whole (it already is in the real
  list).

## Plans (after approval)

| #   | Plan                   | What                                                                                                                    |
| --- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| 01  | Core prerequisites     | the shared day setting (three modules moved to it); `InstallerReceipts` port, adapter, fake; Spotlight's last used date |
| 02  | Sandbox world: access  | `access` in `world.toml`, `FakeAccess` in sandbox mode and the autotest, the banner                                     |
| 03  | The module             | the five categories, the proof, settings, category files in `docs/categories/`, fixtures and the case files            |
| 04  | End to end             | case files (`downloads.toml`, `downloads-denied.toml`, `downloads-browser-open.toml`), core e2e, UI e2e and baselines   |
| 05  | Finish                 | the run by hand on the user's Mac and in the sandbox, conclusions                                                       |

Depends on plans 006 (access), 008 (removal), 009 (installed and running apps), 015 (groups), 019 (`AppSearch`).

## Open questions

None for the user; the calls below are waiting for a confirm or an overrule.

## Agent's calls (to check)

1. **Five categories, one module**, `downloads.large-untouched` included as High and "your call" rather than left for
   the large-files finder: it answers "what is this old 8 GB file in Downloads" now, and the finder can reuse its last
   used date later.
2. **One day setting for the module** (30 days), not one per category.
3. **Installers are Low**, not Medium as in the idea: the app is installed, and the installer downloads again; the
   reason names the app and its version.
4. **A `.dmg` is proven by its name and the app's change date**, never by mounting it; an ambiguous name is not
   claimed. Some real installers will be missed (`install.dmg`), and that is accepted.
5. **A `.pkg` is proven by the installer receipts**, read as files.
6. **Only `~/Downloads`**, one level deep plus bundles.
7. **`Install macOS ….app` only in Downloads**; the apps folder is left to the uninstaller.
8. **The shared day setting is plan 01** and moves all three modules (`temp`'s range 7–30 included, through the range
   parameter).
9. **Only the access part of `sandbox-world`** is taken; open files, tools and steps stay deferred.
10. **The size bar for large-untouched is 1 GB**, fixed for now (no setting).

## Ideas this touches

[Downloads and installers](../../ideas/downloads-and-installers.md) (becomes this plan),
[large files finder](../../ideas/large-files-finder.md) (shares the last used date),
[duplicate finder](../../ideas/duplicate-finder.md) (`file (1).dmg`, not here),
[smart uninstaller](../../ideas/smart-uninstaller.md) (macOS installers in `/Applications`),
[the Trash holds the freed space](../../ideas/trash-holds-freed-space.md) (installers are large: the result should say
"moved to the Trash").
