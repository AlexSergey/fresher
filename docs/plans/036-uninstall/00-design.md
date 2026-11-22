# 036 — Uninstall an app: design

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, engine, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the draft [`readme.md`](./readme.md) and a read of the code on `main` at 33d51a6c (plans 033
and 034 merged), read-only. Plans 037, 031, 032 and 035 merge into `main` before this plan is built, in that order; the
branch is rebased on them, so the design keeps its changes to shared files small and additive (section 10). Facts were
checked on the developer's Mac (macOS 27.2.1), read-only: no app was moved, opened or changed. Where a fact forced a
different choice than the draft, the design says so and the choice is an agent's call.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind." Removing an app together with what it left is the feature
nearly every tool of the field has. Fresher finds the leftovers only after the app is gone (plan
[019](../019-uninstalled-app-leftovers/readme.md)); the matching exists, the "before" side does not.

## Goals

1. A sidebar entry **Apps**: the user's installed apps, largest first, each with one size (the bundle and the data
   Fresher can tie to it by bundle id), a breakdown by kind, the last time it was used when that is known, and one
   plain sentence ("You have not used Editor since March, and it takes 14 GB.").
2. An app's page lists every file Fresher would remove, item by item, with sizes and the danger of each kind, and
   links to the app's cleanable categories on Scan.
3. **Uninstall…** offers two choices, each with its sentence (rule 7): keep the settings and data (the default), or
   remove everything. Either is one item the user confirms in the normal confirmation; everything goes to the Trash
   together and comes back together.
4. Personal data is kept unless the user chooses otherwise; what another installed app may share is never removed.
5. An app Fresher must not remove (Apple's, one a package manager installed, one that needs an administrator, an open
   one) is shown with the reason, or not listed at all, and never offered.

## Non-goals

- Apple's apps and the system's components (`com.apple.` ids, anything under the system's apps folder): not listed.
- Apps of other users; apps outside the apps folders (`/Applications`, one folder below it, `~/Applications`).
- Login items and launch agents as a managed list ([`../../goal.md`](../../goal.md), non-goals): an app's own launch
  agent files in `~/Library/LaunchAgents` are among its files only.
- System-wide files of an app (`/Library/LaunchDaemons`, `/Library/PrivilegedHelperTools`, `/Library/Application
  Support`): owned by root, so removing them needs an administrator (the no-administrator rule). Deferred
  (`docs/deferred/uninstall-system-files.md`, written in 05/01).
- Running a package manager's own uninstall command (deferred, same file); dragging an app onto the window; noticing an
  app dropped into the Trash (needs background work, goal 8).
- Watch mode's "this app keeps growing" (plan [010](../010-history/readme.md) is a draft): the Apps list gives the
  numbers it will compare; the comparison is plan 010's (agent's call 12).

## Facts checked on this Mac (2026-11-22, read-only)

| Fact                                     | Found                                                                                                                 | What it decides |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | --------------- |
| Who owns the bundles in `/Applications`  | 30 bundles: 20 the user's (`<user>:admin` or `<user>:staff`), 7 `root:wheel` (4 of them with an App Store receipt, `Contents/_MASReceipt`), 1 `root:admin`, 2 links (one Apple's, into `/System/Cryptexes`) | App Store apps are owned by root: moving one needs an administrator, so they are **not offered** (the draft said "removed like any other bundle"; agent's call 5) |
| Spotlight's last-used date               | `kMDItemLastUsedDate` set for 7 of 29 bundles; `(null)` for the rest, including apps in daily use                      | Spotlight alone leaves most apps "not known" (agent's call 3) |
| The executable's access time             | follows use, but Fresher's own code-signature reader (`crates/system/src/signature.rs`) reads the main executable of every installed app at every scan | the access time is Fresher's own read after the first scan: **never used** |
| Spotlight's sizes                        | `kMDItemPhysicalSize` set for every bundle                                                                            | not used: Spotlight may be off; Fresher measures the bundle itself (section 3) |
| Measuring every bundle                   | 330,461 entries in 29 bundles; `find` warm in 1 s, `du` cold in 10 s                                                  | measured once per scan, on demand, cached (section 3) |
| Homebrew casks                           | `/opt/homebrew/Caskroom/<token>/<version>/<Name>.app` is a link to `/Applications/<Name>.app` (7 casks here); `brew` at `/opt/homebrew/bin/brew` | a bundle a cask links to is "installed by Homebrew" (section 4) |
| `/Applications` in the protection list    | `never` (`crates/macos/src/protection.rs`, "Apps are removed only by an uninstaller, which is not planned yet")        | the list changes: section 6 |
| `/Applications` as a scan root            | allowed by the settings check (`crates/core/src/settings/folders.rs`)                                                 | an ordinary cleanup's context refuses the apps folder (section 6) |
| `/Applications` and the home's volume     | the same device (`stat -f %d`), canonical path `/Applications`                                                        | the guard's checks 5 and 6 hold for a bundle |

Not checked (they need an app moved, which this design may not do): whether macOS asks for **App Management** when a
bundle is moved to the Trash, or refuses silently, or shows its own notice; the pane's address. Both are facts for the
user's walk (05/01).

## What already exists

| Need                                           | What exists                                                                                                  | Plan |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ---- |
| Installed apps, helpers, team ids              | `SystemFacts::installed_apps()`, `InstalledApp { bundle_id, team_id, name, version, path, helpers, changed }` | 009  |
| An app's data folders by bundle id             | `storage/attribution.rs`: `AppPlaces::of`, the first-level children of seven known folders named by an id (`named_after`, the longest naming id wins), found once at the commit | 012  |
| A folder named after an id                     | `BundleId::names(name)`: the id, or the id and a dot; case ignored                                           | 009  |
| Running apps                                   | `RunningApps`, `RunningSet::owner_of(path)`, `RunningApp.bundle_ids`                                         | 009  |
| Removability from entry facts                  | `EntryFacts`, `Removability::of`                                                                             | 006  |
| One item of several paths, all or nothing      | `Removal::MoveTogether(Parts)` (2 to 64 paths, the subject first); every part checked before the first moves; put back as a whole, the subject last | 029  |
| The plan, the confirmation, the journal        | `CleanupPlan`, `Pending`, `confirm_cleanup`, `FileCleanupJournal`, the history and put back                  | 008  |
| Show in Finder by id                           | `SystemUi::reveal`                                                                                           | 006  |
| A sandbox world of apps                        | `[[case.app]]` (`running`, `changed`), `[[case.running]]`, stated entry facts (`owner`, `flags`)             | 016, 017 |

**What is missing**: a bundle's size and facts outside the scan roots, the last-used date, the package manager, a
plan that is not a selection of findings, the apps folder as a place an uninstall may act in, the App Management
permission, the screen.

## Design

### 1. Where it lives

Not a module. A module's findings are offered on Scan, and an installed app is not junk (rule 1): a zone's "Select
all" must never reach an app. The Apps list is an engine view over the last committed scan, like the storage map
(agent's call 1):

- **core** (`crates/core/src/apps/`, new): the IPC types, the bundle facts, the two uninstall categories and their
  texts, the trait method `InstalledApps::bundle` (with a default, section 3), `Protection`'s app-folder rule.
- **system** (`crates/system/src/bundle_facts/`, new): `MacInstalledApps::bundle` — the measure, Spotlight's date, the
  cask link, the entry facts.
- **engine** (`crates/engine/src/apps/`, new): the footprint, the list, the page, the uninstall plan.
- **app** (`app/tauri/src/apps_commands.rs`, new) and **app/ui** (`screens/apps/`, new).
- **macos**: one row of data (the apps folder's protection) and one pane address.

No module changes; the cleaner does not change (section 7 uses `MoveTogether` and `MoveToTrash` as they are).

### 2. The apps listed

From the installed apps of the last committed scan (the list `AppPlaces` was made from; `AppPlaces` keeps it):

| #   | When                                                                                     | Shown                                     |
| --- | ---------------------------------------------------------------------------------------- | ----------------------------------------- |
| 1   | the installed apps could not be listed                                                   | nothing; the screen says why (`no-apps`)  |
| 2   | the id starts with `com.apple.`, or the bundle is under the system's apps folder         | not listed                                |
| 3   | the bundle is not directly in `Applications` or `UserApplications` or one folder below (an older JetBrains Toolbox's IDE, which its own app manages), or the cleaner's protection refuses its path (`Cleaner::protection()`: Fresher's own bundle) | not listed |
| 4   | the bundle's facts cannot be read (section 3)                                            | listed, not offered: `unmeasured`         |
| 5   | a package manager links to it (section 4)                                                | listed, not offered: `installed-by`       |
| 6   | the bundle is not removable (`Removability::of` of its facts: owner, parent, flags)      | listed, not offered: `needs-administrator`, `locked` or `protected` |
| 7   | the app or one of its helpers runs now (asked at each listing)                           | listed, not offered: `open`               |
| 8   | otherwise                                                                                | listed, offered                           |

Rows 4 to 7 keep the size and the breakdown: the user still learns where the space went.

### 3. The bundle's facts (`InstalledApps::bundle`)

```rust
/// What a bundle takes and what is known of it; reads metadata only, never a file's contents.
pub struct BundleFacts {
    pub sizes: Sizes,                 // logical, allocated (hard links once), reclaimable; completeness
    pub entry: EntryFacts,            // owner, the apps folder writable, the bundle writable, flags
    pub last_used: Option<Timestamp>, // Spotlight's kMDItemLastUsedDate
    pub installed_by: Option<Manager>,
}
pub enum Manager { Homebrew }

trait InstalledApps {
    // … list, find, search_complete as today …
    /// The facts of `app`'s bundle. The default answers an error, so a port that does not know bundles lists no size.
    fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> { Err(…) }
}
```

- **The default method** keeps every existing implementation (six of them, and the fakes in every module's tests)
  untouched: only `MacInstalledApps`, `FakeInstalledApps` and `StatedInstalledApps` answer it (agent's call 2).
- **The measure**: the bundle walked with `lstat`, depth first, never following a link (a link counts as itself),
  never crossing into another device, a hard link counted once by (device, inode); `logical` the sum of the files'
  lengths, `allocated` of their blocks, `reclaimable` the allocated bytes of entries with one link. An entry that cannot
  be read makes the sizes `Partial`. The cancel token is read between folders. At most 2,000,000 entries per bundle
  (`MAX_BUNDLE_ENTRIES`), then `Partial`.
- **Spotlight's date**: `mdls -raw -name kMDItemLastUsedDate <bundle>` through the `CommandRunner` port with its 60 s
  limit, no shell; `(null)` or any other answer is `None`. One run per bundle, only when the bundle is measured.
- **The executable's access time is never read**: Fresher's own signature reader moves it (facts table).
- **The entry facts**: from the bundle's `lstat` (owned by the user; the user immutable, system immutable and
  restricted flags) and its folder's and its own mode bits for the user (the owner's bits when the user owns it, the
  group's when the user is in its group, else the others'), as the scanner's access byte is read; `Removability::of`
  then decides. Nothing is opened for writing to find out.
- **Cost**: measured only when the Apps screen asks (section 5), once per committed scan, in parallel over at most
  four threads, outside every engine lock. On this Mac: about 1 s warm, 10 s cold (facts table). The scan's own time
  does not change: no new walk, no new interest.

### 4. A package manager (`installed_by`)

A table in `crates/system/src/bundle_facts/managers.rs`, data only (rule 5):

| Manager    | Where its records are                                              | Proof                                                        |
| ---------- | ------------------------------------------------------------------ | ------------------------------------------------------------ |
| `Homebrew` | `/opt/homebrew/Caskroom`, `/usr/local/Caskroom`                      | `<token>/<version>/<name>.app` is a link whose target is the bundle's path |

- Read once per measure run: two levels of folders, then `read_link` on each `*.app` entry (never followed). A
  missing or unreadable Caskroom is "no manager", never an error.
- An app installed by a manager is listed with "Homebrew installed this app. Uninstall it with Homebrew so it stays
  in step." and the command the user may copy (`brew uninstall --cask <token>`), never run (agent's call 6).
- MacPorts and other managers: not known in this plan (the deferred file names them).
- In the sandbox the Caskroom is never read: the stated world answers (section 9).

### 5. The footprint

`crates/engine/src/apps/footprint.rs`, pure over the committed `DiskState` and the installed list. For each listed
app, the places a table names, first-level children of a known folder (or one folder below it), named after the app's
bundle id or a helper's (`BundleId::names`, the longest naming id wins as in `named_after`):

| Part (`PartKind`)   | Place                                                                      | Name                          | Keep choice | Everything | Shown danger |
| ------------------- | -------------------------------------------------------------------------- | ----------------------------- | ----------- | ---------- | ------------ |
| `bundle`            | the bundle                                                                 | —                             | yes         | yes        | Medium       |
| `caches`            | `app-caches`                                                               | `<id>`                        | yes         | yes        | Low          |
| `logs`              | `logs`                                                                     | `<id>`                        | yes         | yes        | Safe         |
| `saved-state`       | `saved-state`                                                              | `<id>.savedState`             | yes         | yes        | Low          |
| `launch-agents`     | `library` / `LaunchAgents`                                                 | `<id>.plist`, `<id>.<x>.plist` (files) | yes | yes        | Medium       |
| `web-data`          | `library` / `HTTPStorages`, `WebKit`, `Cookies`                            | `<id>`, `<id>.binarycookies`  | no          | yes        | Medium       |
| `settings`          | `preferences`                                                              | `<id>.plist` (file)           | no          | yes        | Medium       |
| `support`           | `app-support`                                                              | `<id>`                        | no          | yes        | High         |
| `container`         | `containers`                                                               | `<id>`                        | no          | yes        | High         |
| `group-container`   | `group-containers`                                                         | `<team id>.<anything>`        | no          | yes        | High         |

The table is a `const` slice in `apps/places.rs` beside the one `attribution.rs` already has (agent's call 7); a place
is a row. A launch agent goes with the app in both choices: an agent left without its app starts at every login and
fails.

Each part is **kept**, listed with its reason and never in a plan, when:

| Reason (`KeptReason`) | When                                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| `shared`              | another installed bundle has the same id or helper id (a second copy of the app, or one app's helper is another's id) |
| `team-shared`         | a group container whose team id another installed app shares, or the app has no team id                    |
| `link`                | the node is a link                                                                                           |
| `needs-permission`    | a container or group container while other apps' data is not `Granted` or `NotApplicable`                    |
| `not-scanned`         | the node was not read in full (`not_scanned`, or partial sizes)                                              |
| `ignored`             | the user ignored the path, a path inside it or one holding it                                                |

- A look-alike id (`com.example.Editorx`) or a vendor's folder (`com.example`) is never named by `com.example.Editor`
  (`BundleId::names`), so it is never a part.
- A folder named after an app's name rather than its id (`Application Support/Editor`) is not found (the rules tie a
  folder to an app by id only, [`../../architecture/modules.md`](../../architecture/modules.md)). The page says so in one
  line ("Files the app keeps under other names are not listed."), and `app-leftovers` finds them by id once the app is
  gone; a name table is deferred with the system files.
- The breakdown on the list sums the parts by kind, kept parts included; the choices' sizes count only what goes.

**Last used**: the later of Spotlight's date and the newest `modified` of the app's `caches`, `saved-state`,
`settings` and `support` nodes in the state (an app writes them while it runs). Either may be missing; both missing is
"not known". An earlier date can only come from less activity, never more, so the "not used for" sentence never claims
an app is unused while its files changed since (agent's call 3).

### 6. The apps folder in the protection

Today `/Applications` is a `never` place: nothing in it, at any depth, can be removed. It becomes a **never place that
opens its bundles**, in the core's `Protection` (pure) and in the macOS list (data):

- `Protection::with_app_folders(self, folders: Vec<PathBuf>) -> Result<Self, ProtectionError>`: each folder must be one
  of the list's own `never` places (`LooseAppFolder` otherwise). `check(path)` then lets `path` through that never place
  only when it is **exactly** `<folder>/<name>.app` or `<folder>/<sub>/<name>.app`, `<sub>` not itself ending in `.app`;
  every other check still applies (another never place such as Fresher's own bundle, an ancestor of a never place, a
  whole folder, an ignored path). Anything inside a bundle stays `Never`; the folder itself and `<folder>/<sub>` stay
  `Never`.
- `joined` keeps each list's app folders with its own never places, as it keeps carve-outs.
- `crates/macos/src/protection.rs`: `/Applications` moves from `SYSTEM` to a list `APP_FOLDERS`, still pushed to
  `never`, and `protection()` calls `with_app_folders`. The system's apps folder (`/System/Applications`) stays under
  `/System`: never.
- **The second wall, in the engine** (`cleanup/context.rs`): an ordinary cleanup's context refuses every located
  `KnownFolder::Applications` as a whole never place (`Protection::with_refused`, new, same answer as `never`); only an
  uninstall plan's context leaves it out. So a scan root set to `/Applications` and a module that claimed a bundle there
  could still never remove it. Put back refuses nothing extra: it only brings back what the journal holds.
- The roots: the context of every cleanup and put back now holds the located apps folder besides the scan roots and
  the walked places; the cleaner acts only on roots both the plan and the context name (`Roots::new`), and only an
  uninstall plan names the apps folder.

The deletion-safety restriction changes in the same plan (05/01): "nothing in `/Applications` may go but a whole app
bundle the user confirmed for uninstall".

### 7. The uninstall plan

`Engine::prepare_uninstall(app: AppId, choice: UninstallChoice) -> Result<CleanupPlanView, CommandError>`, in
`crates/engine/src/apps/uninstall.rs`:

1. `Busy` while an operation runs; the pending plan dies (as `prepare`).
2. `UnknownApp` for an id of another state or no app; the app's row of section 2 must be "offered", else
   `UninstallRefused { reason }` with the not-offered reason; the running apps are asked again now (`open`).
3. The parts of the choice (section 5), minus the kept ones, in the table's order; with the bundle they are at most 64
   (`MAX_PARTS`), else `UninstallRefused { reason: too-many-files }` (nothing truncated; agent's call 8).
4. **One finding**, never in the results: category `core.uninstall-keep-data` (Medium) or `core.uninstall-everything`
   (High); identity `uninstall/<bundle id>`; subject the bundle; removal `MoveTogether([bundle, parts…])`, or
   `MoveToTrash(bundle)` when there is no part; sizes the bundle's plus the parts', the weakest completeness; title
   `core.uninstall.title` (`{app}`), reason `core.uninstall.reason.<choice>` (`{app}`, `{size}`); `for_app(id)`;
   removability the bundle's (the parts are checked by the guard's check 8 at the rehearsal). Its `FindingId` is taken
   from the results' next id, so it never names a scan's finding.
5. The plan's roots are the last scan's walked roots and the apps folder that holds the bundle (when no root holds it
   already); the rehearsal (`Depth::Quick`) runs with the uninstall context (section 6). A refused part leaves the
   **whole** item out with that part's reason (`MoveTogether`): the confirmation shows it and cannot confirm.
6. The plan is kept pending as `Pending::Uninstall { app }` beside the ordinary one (one pending plan at a time); the
   UI confirms it with the existing `confirm_cleanup`, which builds the uninstall context for it.

After the cleanup (`cleanup/operation.rs`): the run keeps each item's paths, and the results drop every finding at or
below a path that went (`Results::remove` also takes the paths that went, not only the cleared findings' subjects), so `app-caches`' finding
of the removed app's cache leaves Scan at once. The apps cache forgets an app whose bundle went. Put back is plan
029's: the parts first, the bundle last; the app reappears in the list after the next scan.

**Choices** (`UninstallChoice`, with its two texts each, rule 7):

| Choice       | Label                                       | Description                                                                                                     |
| ------------ | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `keep-data`  | "Remove the app, keep its settings and data" | "Moves the app, its caches, logs and saved windows to the Trash. Its settings and files stay, in case you install it again." |
| `everything` | "Remove the app and everything it left"     | "Also moves its settings, sign-ins and the files it keeps for you to the Trash. If you install it again, it starts fresh." |

`keep-data` is pre-chosen (agent's call 4). Nothing is pre-selected on Scan, and the confirmation stays the gate.

### 8. App Management

macOS guards other developers' app bundles with the **App Management** permission (macOS 13 on). No call tells its
state without trying.

- `Capability::AppManagement` joins the core's capabilities, with its five texts (`name`, `unlocks`, `how`, `missing`,
  `denied`) and the pane `Privacy_AppBundles` in `crates/macos/src/privacy/panes.rs` (address to verify, 05/01).
- Its state is the engine's own memory for the session (`apps/management.rs`): `NotAsked` at start; after an uninstall
  item ends `Failed(TrashRefused)` while its bundle was the user's (row 6 passed), `Denied`; after a bundle moved,
  `Granted`. `AccessView` merges it like a tool's state. Never probed, never asked before the first uninstall.
- An app's page carries a notice from the core when the state is not `Granted`: `NotAsked` "macOS may ask you to let
  Fresher manage apps. Fresher needs it only to move apps you uninstall to the Trash."; `Denied` the capability's
  `denied` text with **Open System Settings** (`open_access_settings`).
- The Permissions group of Settings lists it like any capability (it is in the status); no switch (it is no place).

### 9. Sandbox

- `[[case.app]]` gains `installed-by = "homebrew:<token>"`, `last-used = "<age>"` (the case file's age syntax) and
  `owner` (stated for the bundle's own path, as a `[[case.file]]`'s `owner`). The generator writes them into `world.toml`;
  `StatedInstalledApps::bundle` measures the real bundle in the sandbox and overlays the stated date, manager and owner.
- `[case.uninstall]`, the expectation of a case about one app: `app` (the bundle's path), then either `offered = false`
  with `reason = "<not-offered code>"`, or `listed = false`, or `choice = "keep-data" | "everything"` with `trashed`
  (paths that must be in the sandbox's Trash after the cleanup) and `kept` (paths that must be in place). The harness
  (`app/tauri/tests/sandboxes/uninstall.rs`) scans, lists the apps, checks each expectation, prepares and confirms the
  choice, checks the paths, puts the cleanup back and checks every path is back. `EXPECTED.md` gets a line per
  expectation.
- `fixtures/sandboxes/uninstall.toml` (on `module = "trial"`, as `storage-map.toml`), negatives first: an app under
  `System/Applications` and one with a `com.apple.` id (not listed); a Homebrew app (`installed-by`); a bundle owned by
  the system (`needs-administrator`); a look-alike id's cache beside a positive's app (untouched); then `Keeper` with caches,
  logs, saved state, settings and support under `keep-data`; `Purger` with the same and a launch agent under
  `everything`; `Twin`, two copies of one id, the user's copy under `everything` (its data kept `shared`).
- `fixtures/sandboxes/uninstall-open.toml`: an open app (`open`) and a closed one uninstalled beside it.
- Goldens `fixtures/golden/sandbox-uninstall*.json` hold the apps list (name, sizes, breakdown, offer) and both choices'
  plans.

### 10. Files shared with other plans (for the rebase)

| File                                                   | This plan's change                                       | Others before it          |
| ------------------------------------------------------ | -------------------------------------------------------- | ------------------------- |
| `crates/core/src/lib.rs`, `error.rs`, `access.rs`      | `mod apps` and its exports; `UnknownApp`, `UninstallRefused`; `Capability::AppManagement` | 032 (Insight), 035, 037 |
| `crates/core/locales/en.json`                          | `core.uninstall*`, `core.apps.*`, the capability's texts  | all                       |
| `crates/core/src/cleanup/protection.rs`                | `with_app_folders`, `with_refused`                       | none known                |
| `crates/macos/src/protection.rs`, `privacy/panes.rs`   | one list, one pane                                       | 037 may read the Trash    |
| `crates/engine/src/engine.rs`, `lib.rs`                | one `Shared` field, five methods at the end of `impl Engine` | 032, 035, 037          |
| `crates/engine/src/results.rs`, `cleanup/{context,operation,pending}.rs`, `storage/attribution.rs` | the paths that went, the uninstall context, the pending kind, the kept installed list | 037 (the Trash numbers) |
| `app/tauri/src/lib.rs` (invoke handler), new `apps_commands.rs` | five commands                                    | 032, 035, 037             |
| `app/ui/src/screens/frame-sidebar.tsx`, `state/navigation.ts`, `screens/screen-stack.tsx`, `ipc/index.ts`, `ipc/mock.ts`, `design/icons/icons.ts`, `locales/en.json`, `app/ui/e2e/core-mock.ts` | the Apps entry and screens | 035 (Largest), 032, 037 |
| `crates/sandbox/src/{case,expected,world,generate}.rs`, `app/tauri/tests/sandboxes.rs` | `[case.uninstall]`, two `[[case.app]]` keys, two `case_files!` lines | 031, 032 (`[[world.tool]]`), 035, 037 |

Each change is an addition at the end of a list, an enum or a table; nothing is renamed or moved. `Cargo.lock`,
`package-lock.json` and `bindings.generated.ts` are regenerated after the rebase, never merged by hand. The sidebar
order after the rebase: Scan, Largest, Apps, then Review and Cleanups.

### 11. Texts (core, `crates/core/locales/en.json`)

Drafts, polished when built (rule 7):

- Categories: `core.uninstall-keep-data.{name,what,made-by,if-removed,comes-back}` "Apps you uninstall, keeping their
  data" … ; `core.uninstall-everything.*` "Apps you uninstall with everything they left" … (05/01 writes their catalogue
  files).
- `core.uninstall.title` "{app}"; `core.uninstall.reason.keep-data` "{app} and the caches, logs and saved windows it
  made: {size}. Its settings and files stay."; `core.uninstall.reason.everything` "{app} and every file of it Fresher
  found: {size}."
- `core.uninstall.choice.<choice>.{label,description}` (section 7).
- `core.apps.part.<kind>` ("The app", "Caches", "Logs", "Saved windows", "Starts at login", "Website data and
  sign-ins", "Settings", "Files it keeps for you", "Its sandbox folder", "Shared with its maker's other apps").
- `core.apps.kept.<reason>`, `core.apps.not-offered.<reason>` (each one sentence with what to do), `core.apps.summary.
  {unused,used,unknown}` ("You have not used {app} since {date}, and it takes {size}." / "{app} takes {size}." /
  "Fresher cannot tell when you last used {app}. It takes {size}."), `core.apps.no-apps`, `core.apps.other-names`,
  `core.apps.management.not-asked`.
- `core.capability.app-management.*`; `core.refusal.unknown-app`, `core.refusal.uninstall-refused`.

The UI's own dictionary holds only its chrome: "Apps", the filter's labels, "Uninstall…", the column names.

### 12. IPC

| Command                           | Answer                                                                                     |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| `get_apps(query: AppsQuery)`      | `AppsView { apps: AppSummary[], notice?: Text }`; `query.unused: none | three-months | six-months | a-year`, filtered by the engine's clock; largest first |
| `get_app(app: AppId)`             | `AppDetail { summary, parts: FootprintItem[], choices: ChoiceView[], categories: CategoryId[], notice?: Text }` |
| `prepare_uninstall(app, choice)`  | `CleanupPlanView`, confirmed by `confirm_cleanup`                                          |
| `reveal_app(app: AppId)`          | shows the bundle in Finder (`SystemUi::reveal`), for a bundle that needs an administrator   |

A package manager's command is a text of the page (`AppDetail.command`), shown as selectable text; no command runs it.

`AppSummary { app: AppId, name, version?, sizes, parts: { kind, sizes }[], lastUsed?: Timestamp, summary: Text, offer:
{ kind: offered } | { kind: not-offered, reason: NotOffered, text: Text } }`; `FootprintItem { kind, path, sizes,
danger, goes: { keepData: bool, everything: bool }, kept?: { reason, text } }`. `AppId { state, index }`, as `NodeId`.

### 13. Tests

Negative cases first everywhere ([`../../testing.md`](../../testing.md)).

- **core**: the protection rule (a path inside a bundle, the folder itself, a sub-folder, two `.app` levels, Fresher's
  own bundle, a carve-out of another list, a loose app folder; then a bundle at depth one and two), `with_refused`;
  the types' serialization; the categories' texts present.
- **system**: the measure on fixture trees (a link not followed, another device not crossed — skipped when no second
  device, a hard link counted once, an unreadable folder partial, the entry cap, cancel), the cask table (no Caskroom,
  a link elsewhere, a broken link; then a cask link), Spotlight's answer parsed (`(null)`, garbage, a date) on a fake
  runner, the entry facts (root-owned through a stated lens is not possible on disk: the facts function tested on plain
  values).
- **engine**: the footprint (every kept reason, a look-alike, a vendor folder, the longest naming id; then each part),
  the list's rows 1 to 8, the filter, the page, `prepare_uninstall` (every refusal; then both choices' parts, the
  single-path case, the roots, the context), the second wall (an ordinary plan naming a bundle is skipped
  `Protected`), the paths that went clearing findings, App Management's states.
- **core e2e** (`crates/e2e/tests/uninstall.rs`, real cleaner on a fixture tree, `Reach::Sandbox`): both choices move
  exactly their paths; put back brings the whole app back, bundle last; a part replaced after the plan, a part held
  open, a part grown: nothing moves; the app opened after the plan: nothing moves (its executable is mapped); an
  ordinary cleanup with a bundle finding moves nothing; a second copy keeps the data.
- **sandbox**: the two case files and goldens; a deliberate break of an expectation fails (by hand once, 04/01).
- **UI**: the list (empty, not known, filtered, not offered with each reason), the page (kept parts, the notice, the
  choices with their sentences, keep-data pre-chosen), the flow (prepare, confirm, result, put back from Cleanups), the
  sidebar; Playwright scenarios, axe, visual baselines in both browsers.

## Risks

- **App Management unknown**: the first move of another developer's bundle may be refused or may raise the system's
  own notice. The item ends `Failed`, nothing of it moved (the bundle is the first part), the page says what to do.
  The user's walk on the real Mac settles it (05/01).
- **Removing a bundle while the app starts**: the guard's check 10 sees its mapped executable and skips the whole item.
  A helper that keeps running from another place (a login item outside the bundle) is caught only by its open files.
- **A data folder of an app that shares it under the same id** (a suite's two apps): the second copy rule catches the
  same id; a different id under the same vendor is never named. A wrong part is High at most and in the Trash.
- **Many parts** (more than 63): refused, not truncated; `app-leftovers` offers the rest once the app is gone.
- **Measuring time** on a Mac with very large apps (Xcode): about a second warm; the screen shows its progress line
  while it waits. Measured on the real Mac in 05/01.
- **The rebase** on 037, 031, 032 and 035: the shared files of section 10; regenerated lock and bindings files.

## Open questions

None. Each question the draft left is settled as an agent's call below.

## Decisions

- 2026-11-22 — the user delegated (translated): "Restore the last session's context and start implementing; work
  autonomously with subagents, take the recommended decisions; implement all the features; write into docs/deferred
  what I will need to look at — I will check everything tomorrow; bring everything up to the security audit, merge when
  ready." Build autonomously; the recommended decisions and the agent's calls are taken.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

1. **An engine view, not a module.** A module's findings land on Scan and in "Select all"; an app is not junk. Cost if
   wrong: a module would need a way to keep its findings off Scan.
2. **`InstalledApps::bundle` with a default answer**, so the six implementations and every module's fakes stay as they
   are (rebase-friendly). Cost: a port that forgets to implement it lists apps without sizes (tested for the three that
   must).
3. **Last used = the later of Spotlight's date and the newest change of the app's own data folders**; never the
   executable's access time (Fresher's signature read moves it). Spotlight alone knew 7 of 29 apps here. Cost if
   wrong: a background helper writing caches makes an app look used.
4. **Keep-data is the default choice** (the draft's call 1).
5. **App Store apps are not offered** (fact: owned by root): "needs an administrator", with Show in Finder, against the
   draft's "removed like any other bundle". Cost: the user drags them to the Trash in Finder.
6. **Package-managed apps are shown, not removed**, with the manager's command to copy (the draft's call 2); Homebrew
   casks only.
7. **The footprint's places are a `const` table in the engine**, beside `attribution.rs`'s, not a TOML file: the engine
   has no rule data of its own today. Cost: a place is a code change, reviewed as such.
8. **More than 64 paths: refused**, never truncated. Cost: a rare app cannot be uninstalled here.
9. **One item of several paths** (`MoveTogether`): the app and its files go together or not at all and come back
   together; the confirmation lists the files on demand. Cost: the confirmation shows one line with "and N more files",
   the page above it lists them item by item.
10. **The apps folder opens only whole bundles, and only for an uninstall plan** (the protection rule plus the
    engine's second wall). Cost: none for safety; a deletion-safety rule changes, reviewed in 05/01.
11. **App Management is learned, never probed**: `NotAsked` until the first uninstall, then `Denied` or `Granted` for
    the session. Cost: the user may meet the system's notice once.
12. **No watch mode**: plan 010 compares the list's numbers when it is built (the draft's line about it is deferred).
13. **No drag and drop, no Trash watcher, no resource profile** (the draft's calls 3 and 4).
14. **A finished uninstall keeps the Scan selection** (an ordinary cleanup empties it): the uninstall did not come from
    it. Cost: none.
15. **Containers need other apps' data**: without it they are kept (`needs-permission`), never a reason to prompt.

## Conclusions

The plan's conclusions, what changed from this design and why: [`readme.md`](./readme.md), "Conclusions".
