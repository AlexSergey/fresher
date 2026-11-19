# 006 — Permissions and privileges: design

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, macos, engine, testing, app,
app/ui

## Intent

The user's task (translated): "…and for privileges and permissions, since not every action is possible without
permissions."

## Goals

1. The app always knows what it may see and do, and says so honestly: reduced coverage is shown, never hidden.
2. A permission is offered only when the user starts something that needs it, with a plain-English reason and what it
   unlocks ([`restrictions/privacy.md`](../../restrictions/privacy.md), "Least permissions").
3. Everything works without any permission, with less coverage.
4. Modules declare what they need; none of them talks to the operating system about permissions. The core, the engine
   and the modules know nothing about macOS: its permissions live in adapters behind ports.
5. Every state of every permission is a test case on a fake; no test asks macOS for anything and no test touches the
   real home folder.

## Non-goals

- Running anything as administrator: no privileged helper, no password prompt, no `sudo` (see "Privileges").
- Signing and notarization: plan [013](../013-release/readme.md). This plan only records what they must provide.
- The first-run screen: plan [014](../014-onboarding/readme.md) builds it from the pieces of this plan.
- The cleaner's per-item checks: plan [008](../008-safe-removal/readme.md) builds them on the interfaces of this plan.
- App Management (changing or removing other apps' bundles): it matters only to an uninstaller, which is not planned;
  that plan adds the capability (agent's call 14).
- Automation (Apple Events): the app never sends them (agent's call 13).

## Design

### 1. What macOS guards

The app is not sandboxed: what it may read is decided by POSIX permissions and by macOS's privacy protection. The
second is the subject of this plan.

| Capability (core)          | macOS name                      | What it unlocks                                                    | Prompt?                              | How the state is learnt                                     |
| -------------------------- | ------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ----------------------------------------------------------- |
| —                          | —                               | most of the home folder, `~/Library/Caches`, developer folders     | —                                    | —                                                           |
| `Folder(Desktop)`, `Folder(Documents)`, `Folder(Downloads)` | Files and Folders | the folder                                                      | yes, on first access, one per folder | only by entering it, so only after the user opted in        |
| `OtherAppsData`            | data from other apps (macOS 14+) | other apps' containers in `~/Library/Containers` and group containers | yes, on first access              | only by entering one, so only after the user opted in       |
| `Volume(Removable)`, `Volume(Network)` | Files and Folders   | a removable or network volume                                     | yes, on first access                 | by entering it; the user opted in by adding it as a scan folder |
| `AllFiles`                 | Full Disk Access                | all of the above, plus Mail, Messages, Safari and similar data     | never: only the user, in System Settings | by opening a file only it allows (no prompt)            |
| `Tool(id)`                 | — (not macOS)                   | an external program the module needs, such as the Docker CLI       | —                                    | by its check command through the `CommandRunner` port       |
| administrator              | —                               | items owned by the system or another user                          | —                                    | never taken by the app (section 9)                          |

The exact list of guarded places, which of them prompt and which only refuse, and the behaviour on macOS 13 to the
current version are measured in plan 01 and recorded in this file ("Probe results") before plan 03 encodes them.

### 2. The capability model (core)

All of it is OS-neutral: the names say what is unlocked, not which macOS switch unlocks it. Only the macOS adapter and
the English texts name macOS's settings.

```rust
pub enum Capability {                 // crates/core/src/access.rs
    AllFiles,                         // macOS: Full Disk Access
    Folder(GuardedFolder),            // Desktop | Documents | Downloads
    OtherAppsData,
    Volume(VolumeKind),               // Removable | Network
    Tool(ToolId),                     // an external program, ready to answer
}

pub enum Access { Granted, Denied, NotAsked, Unknown, NotApplicable }

pub struct CapabilityState { pub capability: Capability, pub access: Access, pub note: Option<Text> }
pub struct AccessStatus { pub capabilities: Vec<CapabilityState> }   // AccessStatus::access(&Capability) -> Access

pub struct GuardedPlace { pub path: PathBuf, pub capability: Capability }

pub enum NeedKind { Required, SeesMore }
pub struct Need { pub capability: Capability, pub kind: NeedKind }

pub enum Availability { Ready, Limited { missing: Vec<Capability> }, Unavailable { missing: Vec<Capability> } }
```

- `Access::NotAsked`: the app has not looked, because looking could make macOS ask the user. `Unknown`: the check
  could not tell. `NotApplicable`: this Mac has no such permission (data from other apps on macOS 13).
- `note` carries a plain-English detail when the state needs one ("Docker is installed but not running").
- The existing `AccessStatus { full_disk_access }` is replaced by the list above, and `Unreadable::NeedsFullDiskAccess`
  becomes `Unreadable::Guarded` (refused by the system's privacy protection, `EPERM`), told apart from
  `Unreadable::NoPermission` (POSIX permissions, `EACCES`). Both are small renames in `crates/core`.
- Every type is exported to TypeScript (`ts-rs`), kebab-case tags like the rest of the IPC.

### 3. Ports

The read side stays the existing port `AccessProbe` (extended, not renamed); acting on the system's interface is a new,
separate port, so a probe can never open anything.

```rust
pub trait AccessProbe: Send + Sync {
    /// The state of every permission this system has. Never causes a system prompt.
    fn status(&self) -> AccessStatus;
    /// The places the system guards (under the home folder the adapter was built with, and the mounted volumes of
    /// the scan roots), each with the capability that opens it.
    fn guarded_places(&self, roots: &[PathBuf]) -> Vec<GuardedPlace>;
    /// Tries one guarded place. MAY cause a system prompt: called only inside a scan the user started, for a place
    /// the user opted into.
    fn try_place(&self, place: &GuardedPlace) -> Access;
}

pub trait SystemUi: Send + Sync {
    /// Opens the system's settings at the pane that grants `capability`.
    fn open_access_settings(&self, capability: &Capability) -> Result<(), SystemUiError>;
    /// Shows a file or folder in the system's file manager, selected.
    fn reveal(&self, path: &Path) -> Result<(), SystemUiError>;
}
```

| Port          | Implemented by                                                      | Fake for tests (`fresher-testing`)                |
| ------------- | ------------------------------------------------------------------- | ------------------------------------------------- |
| `AccessProbe` | `MacAccess` in `fresher-system`, on `fresher_macos::privacy`        | `FakeAccess`: any state of any capability, scripted `try_place` answers, guarded places on a fixture tree |
| `SystemUi`    | `MacSystemUi` in `fresher-system`: `/usr/bin/open` with a fixed argument list through the `CommandRunner` | `FakeSystemUi`: records every call |

Where the macOS code lives:

- **`fresher-macos`** (created by plan [004](../004-fast-scan/readme.md)/02) gains a `privacy` module: the table of
  guarded places for a given home folder, the Full Disk Access check, the try of one place, the classification of an
  `io::Error` (`EPERM` → guarded, `EACCES` → no permission), and the System Settings address of each pane. Its own
  small types (`Guard`, `Probe`, `Pane`). Since plan 004 the crate depends on `fresher-core` (its `ReadError` and
  `errors.rs`); `privacy` reuses that errno table rather than writing a second one (2026-11-19).
- **`fresher-system`** (plan [009](../009-system-knowledge/readme.md), which exists now) maps them onto the core's
  ports, the way it already adapts the clock and the process runner: `MacAccess` and `MacSystemUi` live there and the
  app only composes them (agent's call 2, applied 2026-11-19).
- **No new `unsafe` is expected**: the checks are `std::fs` calls and `libc` error constants, the settings are opened
  with `/usr/bin/open`. The one possible exception is reading the macOS version (to pick a pane's address), if plan
  004/02's system facts do not already give it. Any FFI call goes into `fresher-macos`'s one FFI module with a
  `// SAFETY:` comment stating its invariant, behind a safe function.

### 4. Checks that never prompt

- **Full Disk Access**: open, read-only and without reading a byte, a file that only Full Disk Access allows and that
  never prompts (candidates measured in plan 01: the system's privacy database, Safari's data folder), then close it.
  Opened → `Granted`; refused with `EPERM` → `Denied`; no candidate exists, or any other error → `Unknown`. The
  candidate paths are built from the home folder the adapter is given, so tests point them at a temporary tree.
- **Guarded folders and other apps' data**: no public call tells their state without the risk of a prompt. They are
  `NotAsked` until the user opts in (section 6), then learnt by `try_place` inside a scan. With `AllFiles` granted they
  are `Granted` without being touched (Full Disk Access covers them).
- **Volumes**: a scan folder on a removable or network volume (its kind from plan 004/02's volume facts) is the user's
  own opt-in; it is tried like a guarded folder.
- **Tools**: a module declares its tool's check command (`ModuleDescriptor::with_tool_check(ToolCommand)`, for example
  `docker info --format {{.ServerVersion}}`). The engine runs it through `CommandRunner` at the start of a scan and on
  an explicit refresh: success → `Granted`; `NotInstalled` → `Denied` with "Docker is not installed"; a non-zero status
  → `Denied` with "Docker is installed but not running". The engine does this itself — running a program is not an OS
  permission — so tools need no adapter.
- **Re-check on focus**: when the window gains focus the shell calls `Engine::refresh_access()`; the OS permissions are
  probed again (one `open` call, well under a millisecond), and `AccessChanged` is sent only when something changed.
  Tools are not re-run on focus (a program run is too slow for it).
- **A grant that needs a relaunch**: if plan 01 confirms that a running process sees Full Disk Access only after it is
  reopened, the card says so and offers "Reopen Fresher" (the user's click; the engine is shut down first, as on quit).

### 5. Modules declare, the engine resolves

- A category declares what it needs: `CategoryDescriptor::new(id, danger).with_needs(vec![Need::sees_more(AllFiles)])`.
  `Required`: without it the category cannot be found at all (Docker without the CLI). `SeesMore`: it is found, but
  part of it may be hidden.
- A removal's need is derived, never declared: a `ToolCommand` removal needs its `Tool`, a path removal needs the
  capability of the guarded place it lies in.
- The engine combines the declarations with the current `AccessStatus` into an `Availability` per category, shown in
  `ModuleInfo`. A module whose every category is `Unavailable` is not run; it ends
  `ModuleFinished { Unavailable { missing } }` (a new `ModuleOutcome` variant), so the UI shows why it is empty.
- The totals of a category that is `Limited` are `Partial` (`42 GB+`): the hidden part may hold more.
- `CommandError::MissingCapability { capability }` is the refusal an action gets when its need is not granted (used by
  plan 008).

### 6. Coverage of a scan

- **No wall of prompts.** Before the walk the engine asks `AccessProbe::guarded_places(roots)`. A place whose
  capability is `Granted` is walked. A place the user opted into is tried with `try_place` (this is the only moment a
  prompt can appear, inside a scan the user started): `Granted` → walked, anything else → skipped. Every other guarded
  place is skipped and never touched.
- **The opt-in** is a core setting per place, off by default, rendered by plan [005](../005-settings/readme.md)'s screen
  with no special code: `core.access.desktop`, `core.access.documents`, `core.access.downloads`,
  `core.access.other-apps-data`. The notice's button ("Let Fresher look there") turns it on; the next scan asks macOS
  once. A volume is opted into by adding it as a scan folder.
- **The walk's input**: the skipped places go to the walk as folders it must not enter, marked as skipped for access, so
  their parents' sizes are `Partial` (needed from plan 004).
- **The result**: `Coverage { not_entered: Vec<NotEntered { capability, access }>, unreadable: Vec<UnreadableCount {
  reason, folders, unlocked_by: Option<Capability> }> }`, with `is_complete()`. Folders refused with `EPERM` that are not
  in a guarded place are counted as unlocked by `AllFiles`. It is sent as `Event::ScanCoverage { scan, coverage }` right
  before `ScanFinished` and kept in `LastScan.coverage`. An incomplete coverage makes the outcome `Partial`, as an
  unreadable folder already does.
- No path is in `Coverage`: counts and capabilities only.

### 7. Opening System Settings, and the reason strings

- `SystemUi::open_access_settings` runs `/usr/bin/open` with one fixed argument, the `x-apple.systempreferences:` address
  of the pane (Full Disk Access, Files and Folders, …): one address per pane for every supported version ("Probe
  results", agent's call 25). The command takes a
  `Capability`, never an address or a path. It works with the network off; nothing is fetched.
- `Info.plist` (`app/tauri/Info.plist`, merged by Tauri's bundler) holds the reason macOS shows in each prompt, in plain
  English:

  | Key                                  | Text                                                                                                                       |
  | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
  | `NSDesktopFolderUsageDescription`    | Fresher measures what takes space on your Desktop. It reads names and sizes only, and removes nothing unless you confirm it. |
  | `NSDocumentsFolderUsageDescription`  | Fresher measures what takes space in Documents, such as old project folders. It reads names and sizes only, and removes nothing unless you confirm it. |
  | `NSDownloadsFolderUsageDescription`  | Fresher measures what takes space in Downloads, such as old installers. It reads names and sizes only, and removes nothing unless you confirm it. |
  | `NSRemovableVolumesUsageDescription` | You added this drive to the folders Fresher scans. It reads names and sizes only, and removes nothing unless you confirm it. |
  | `NSNetworkVolumesUsageDescription`   | You added this network folder to the folders Fresher scans. It reads names and sizes only, and removes nothing unless you confirm it. |

  Whether data from other apps takes a reason string of its own is checked in plan 01.
- The app declares no Apple Events, camera, microphone, contacts or any other usage key: a test reads `Info.plist` and
  allows only the five above.

### 8. What the user sees

Built from plan [003](../003-base-ui/readme.md)'s components (`Notice`, `PermissionCard`, `SizeText`, `CategoryCard`,
`ModuleSection`, `SettingsRow`); all texts come from the core's dictionary.

- **Next to the result it limits, and nowhere else** (no nagging, no badge on the app):
  "312 folders could not be read. Full Disk Access lets Fresher see them." — `[Open System Settings]` and a short "How"
  (three numbered steps). For a skipped folder: "Desktop, Documents and Downloads were not looked at." —
  `[Let Fresher look there]` with its description: "macOS will ask you once for each folder. Fresher reads names and
  sizes only." 
- **A section or action without its capability** is visible but disabled, with one sentence on what is missing
  ("Docker is installed but not running. Start Docker and scan again."). Nothing is silently absent.
- **Partial sizes** read `42 GB+`, with a tooltip saying what was not seen.
- **The Permissions group** of the settings screen (plan 005) lists every capability with its state, what it unlocks,
  and its button; it is the only place that lists them all.
- **The first run** (plan 014) offers Full Disk Access with "Continue without it" of equal weight; it uses the same card.
- When a grant changes while the app is open, the notices update at once (`AccessChanged`) and offer "Scan again" — the
  app does not rescan on its own.

### 9. Privileges

- **The app never runs as administrator**: no privileged helper, no launch daemon, no password prompt, no `sudo`, no
  authorization call. A helper would be a background component, against [`goal.md`](../../goal.md), section 8, and a
  deleter with root rights is the largest risk the product could carry.
- **Removability is a fact of each finding**, worked out by a pure function of the core from what the walk reports about
  the entry (its owner, whether the app may write to its parent folder, its file flags; needed from plan 004):

  ```rust
  pub enum Removability { Removable, NeedsAdministrator, Locked, ProtectedBySystem }
  ```

  - `NeedsAdministrator`: owned by another user or the system, or its folder is not writable by the app. Shown with its
    size and the sentence "Only an administrator can remove this. Show it in Finder and drag it to the Trash; macOS
    will ask for an administrator password." with `[Show in Finder]` (`SystemUi::reveal`, by finding id).
  - `Locked`: the user locked it in Finder (the user-immutable flag). "This item is locked. Unlock it in Finder's Get
    Info window if you want to remove it."
  - `ProtectedBySystem`: macOS's system integrity protection guards it (the restricted flag). Shown as information
    only; the app never suggests turning that protection off.
  - None of them can be selected for cleaning: plan 008 refuses to plan an item that is not `Removable`.
- No `sudo` command is ever shown for the user to copy (agent's call 9): a copied privileged removal with a wrong path
  is the worst outcome the product could cause.
- **Not sandboxed, outside the Mac App Store**: a sandboxed app cannot scan the disk. The app is distributed signed and
  notarized (plan 013), with the hardened runtime and no extra entitlements.

### 10. Before a deletion (offered to plan 008)

- `AccessStatus::check(&[Need])` → `Ok` or the missing capabilities, and `Removability` re-read for the item right
  before acting, next to the other re-checks of
  [`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md).
- An item the app may not remove is skipped and reported with the reason; it never retries with more rights. A removal
  that fails with `EPERM` anyway is reported as "macOS refused" with the capability that would allow it.

### 11. Development builds and signing

- macOS ties a grant to the app's code signature. Ad-hoc signed builds get a new identity on every rebuild, so their
  grants are lost; under `npm run tauri dev` the binary runs from the terminal, and macOS attributes its access to the
  terminal app. Both are written in the developer docs with how to test each state (the fake states in the gallery
  first; a real grant by hand).
- A stable signing identity for releases and a local one for development builds are plan
  [013](../013-release/readme.md)/01; this plan only states the need.

### 12. Testing

- Every state of every capability is a test case on `FakeAccess`: negative cases first (denied, not asked, unknown,
  not applicable), then granted.
- `fresher-macos::privacy` is tested on temporary trees: the guarded-place table for a fake home, the classification
  of `io::Error::from_raw_os_error(EPERM | EACCES | ENOENT)`, the probe's answer when the candidate is readable, missing
  or unreadable. `EPERM` from the real privacy protection cannot be produced in a test; the classification covers it.
- The adapters in the app are tested with `FakeCommandRunner` (the exact `/usr/bin/open` arguments) and temporary trees.
- The UI states — every capability in every state, limited and unavailable sections, the notices — are in the gallery
  and its screenshots (plan 003).
- No test asks macOS for a permission, opens System Settings or reads the real home folder. The real behaviour is
  checked by hand and written in the conclusions.

### 13. Interfaces with other plans

| Plan | 006 offers it                                                                                                  | 006 needs from it                                                                                      |
| ---- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| 003  | `AccessStatus`, `Availability`, `Coverage`, `Removability` in the bindings; the dictionary texts; the states to render | `Notice`, `PermissionCard`, `SizeText` (partial), `CategoryCard` / `ModuleSection` with a disabled state, `SettingsRow`, the gallery |
| 004  | the guarded places and which of them are skipped; the `Unreadable::Guarded` rename                             | `fresher-macos` (004/02) with volume facts; the walk takes folders it must not enter, marked as skipped for access (parents partial); `EPERM` vs `EACCES` counted apart; each `Match` carries its owner, whether its parent is writable, and its file flags (`ATTR_CMN_USERACCESS`, `ATTR_CMN_FLAGS`) |
| 005  | the `core.access.*` settings; the content of the Permissions group                                            | the settings screen (005/03) to host the group                                                          |
| 008  | `AccessStatus::check`, `Removability`, `CommandError::MissingCapability`, the derived need of a removal       | —                                                                                                      |
| 013  | the reason strings in `Info.plist`; the need for a stable signing identity                                     | the local development identity (013/01), for grants that survive a rebuild                             |
| 014  | the Full Disk Access card and its texts                                                                        | —                                                                                                      |

### 14. Dependencies

No new crate and no new npm package. `libc` (already a workspace dependency) gives the error constants; opening System
Settings uses `/usr/bin/open` through the existing process runner instead of a Tauri plugin or an Objective-C binding.

## Probe results

Filled in on 2026-11-19 from Apple's documentation and well-known macOS behaviour, **not measured**: the probes need a
person at a real Mac watching for prompts, so plan 01 is now the user's run by hand (agent's call 24). Every row below
is **unverified (documentation) — the user verifies in plan 01**. Plan 03 encodes these answers as they stand; where
the run disagrees, the constants of 03/01 and 03/02 change (the table of places, the candidates, the addresses) and
the change is written here. The development Mac runs macOS 27.2.1; the app's minimum is macOS 13.0
(`app/tauri/tauri.conf.json`), so every supported version has System Settings, not System Preferences.

### Places (plan 01/01)

Paths relative to the home folder. "Prompts" means macOS shows its question on the first open and remembers the
answer; "refuses" means a silent `EPERM` without Full Disk Access, no prompt. The last column is the guard 03/01
gives the place.

| Place                                                    | Without Full Disk Access                                                        | Guard in 03/01        | Status                                                         |
| -------------------------------------------------------- | ------------------------------------------------------------------------------- | --------------------- | -------------------------------------------------------------- |
| `Desktop`, `Documents`, `Downloads`                      | prompts, one question per folder (Files and Folders)                            | `Desktop`, `Documents`, `Downloads` | unverified (documentation) — the user verifies in plan 01 |
| `Library/Containers/<another app>` (macOS 14+)           | prompts ("data from other apps"); listing `Library/Containers` itself does not, entering one container does | `OtherAppsData` on `Library/Containers` | unverified (documentation) — the user verifies in plan 01 |
| `Library/Group Containers/<another group>` (macOS 15+)   | prompts like the containers; on macOS 14 not guarded                            | `OtherAppsData`       | unverified (documentation) — the user verifies in plan 01      |
| `Library/Mobile Documents` (iCloud Drive), `Library/CloudStorage` | may prompt (a file provider's own question); no capability of the core opens it | `FullDiskAccess` (never tried, opened only with it) | unverified (documentation) — the user verifies in plan 01 |
| `Pictures`, `Movies`, `Music`                            | the folders open; the Photos and Music libraries inside may prompt or refuse    | `FullDiskAccess`, as the standing avoid list does today (conservative) | unverified (documentation) — the user verifies in plan 01 |
| `Library/Calendars`, `Library/Application Support/AddressBook` | refuses (guarded data of Calendar and Contacts); may prompt for the data class | `FullDiskAccess`      | unverified (documentation) — the user verifies in plan 01      |
| `Library/Mail`, `Library/Messages`, `Library/Safari`, `.Trash` | refuses with `EPERM`, never prompts                                         | none: walked, counted as unreadable `Guarded`, unlocked by `AllFiles` | unverified (documentation) — the user verifies in plan 01 |
| A removable volume as a scan root                        | prompts once (Files and Folders → Removable Volumes); `statfs` of it does not   | `RemovableVolume`     | unverified (documentation) — the user verifies in plan 01      |
| A network volume as a scan root                          | prompts once (Files and Folders → Network Volumes); `statfs` of it does not     | `NetworkVolume`       | unverified (documentation) — the user verifies in plan 01      |
| A scan root inside a guarded place (`~/Documents/Projects`) | prompts like its place                                                       | the place's guard, reported for the root itself | unverified (documentation) — the user verifies in plan 01 |

The table keeps every place of today's standing avoid list (`crates/macos/src/places.rs`, `docs/hacks/README.md`), so
no place that is avoided now is opened by plan 03 without its capability.

### Checks (plan 01/01)

| Question                                                        | Answer                                                                                                                                                                                                                                  | Status                                                    |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Which file tells Full Disk Access without a prompt              | in order: `Library/Application Support/com.apple.TCC/TCC.db` (the user's privacy database, on every account), then `Library/Safari`; both under the home, opened read-only with `O_NOFOLLOW`, closed at once, not a byte read. The first candidate whose open does not answer `ENOENT` decides (an existence check cannot be made first: `lstat` inside a guarded folder answers `EPERM` too). | unverified (documentation) — the user verifies in plan 01 |
| After a denied prompt                                           | the answer is remembered; every later open is a silent `EPERM`; only System Settings (or `tccutil reset`) changes it                                                                                                                      | unverified (documentation) — the user verifies in plan 01 |
| Full Disk Access turned on while the app runs                   | after relaunch: System Settings offers "Quit & Reopen" for a running app. The app offers "Reopen Fresher" (design, section 4), and the focus re-check still runs in case the grant applies at once                                       | unverified (documentation) — the user verifies in plan 01 |
| A Files and Folders grant from a prompt                         | applies at once to the running process: the open that raised the prompt waits for the answer, then succeeds or fails                                                                                                                     | unverified (documentation) — the user verifies in plan 01 |
| `EPERM` (privacy) apart from `EACCES` (POSIX)                   | yes for reads of folders and files; the exception is a path guarded by system integrity protection, which also answers `EPERM` (counted as `Guarded`, unlocked by `AllFiles`: an over-promise the user's run checks)                     | unverified (documentation) — the user verifies in plan 01 |
| Any check that needs an FFI call                                | none: `open` and `statfs` (already in `fresher-macos::ffi::volume`) are enough; the macOS version, for data from other apps, is read from `/System/Library/CoreServices/SystemVersion.plist` with `fresher-system`'s plist reader       | unverified (documentation) — the user verifies in plan 01 |

### Panes, reason strings, development builds (plan 01/02)

| Question                                                        | Answer                                                                                                                                                                                                                  | Status                                                    |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Full Disk Access pane                                           | `x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`, the same on macOS 13 to the current version (System Settings maps the old pane id)                                                           | unverified (documentation) — the user verifies in plan 01 |
| Files and Folders pane (folders and volumes)                    | `x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders`, the same on every supported version                                                                                                   | unverified (documentation) — the user verifies in plan 01 |
| Data from other apps                                            | no pane lists it; the fallback, Privacy & Security, `x-apple.systempreferences:com.apple.preference.security?Privacy`                                                                                                     | unverified (documentation) — the user verifies in plan 01 |
| `/usr/bin/open` with the network off                            | works: the address is handled by System Settings on the Mac; nothing is fetched                                                                                                                                         | unverified (documentation) — the user verifies in plan 01 |
| Tauri 2 and `app/tauri/Info.plist`                              | the bundler merges an `Info.plist` next to `tauri.conf.json` into the bundle's `Contents/Info.plist`; a development run embeds it in the binary, but macOS attributes its access to the terminal app (below)                | unverified (documentation) — the user verifies in plan 01 |
| Reason string of data from other apps                           | none: macOS shows a fixed text; no usage key exists for it                                                                                                                                                              | unverified (documentation) — the user verifies in plan 01 |
| The five keys of the design                                     | `NSDesktopFolderUsageDescription`, `NSDocumentsFolderUsageDescription`, `NSDownloadsFolderUsageDescription`, `NSRemovableVolumesUsageDescription`, `NSNetworkVolumesUsageDescription`: confirmed as the keys macOS reads | unverified (documentation) — the user verifies in plan 01 |
| Under `npm run tauri dev`                                       | macOS attributes access to the app that started the process (the terminal or the editor), so its grants apply                                                                                                           | unverified (documentation) — the user verifies in plan 01 |
| An ad-hoc signed rebuild                                        | loses the grants: an ad-hoc signature's identity is the code's hash, which changes with every build                                                                                                                     | unverified (documentation) — the user verifies in plan 01 |

## Open questions

1. **A container that proves an id while it is not offered** (2026-11-19, from
   [`deferred/app-leftovers-containers.md`](../../deferred/app-leftovers-containers.md)): step 03/05 takes the agreed
   remedy — the `container` row of `app-leftovers` stays not offered — before 03/03 lets the walk enter
   `Library/Containers`. A container still proves its id. An extension of an installed app whose id is unrelated to
   the app's and lies deeper than `InstalledApp::helpers` reads would then be proven, and its caches offered while the
   app is installed. Options: (a) keep the proof (recommended: the deferred item's own wording; Safe and Medium levels,
   confirmation and the Trash limit the harm); (b) neither offer nor prove a container until the helper list covers
   every nested bundle (what the app does today). The step builds (a) unless the user picks (b). **Answered (b) at the
   final review — agent's call 33, to check.**

The signing identity that keeps grants across releases depends on the Apple Developer account, which is plan
[013](../013-release/readme.md)'s open question 1.

## Decisions

- 2026-11-15 — **macOS first, Windows not ruled out** (the user): permission checks and prompts are adapters behind
  ports, built on the macOS crate; the core, the engine and the modules know nothing about the operating system;
  modules declare what they need.
- 2026-11-15 — **The plan is approved, with every agent's call below confirmed** (the user).

## Agent's calls (to check)

The user asked the agent to take the recommended decisions; each waits for a confirm or an overrule.

1. **No administrator rights, ever**: such items are shown with their size, a Finder instruction and "Show in Finder";
   they are never cleaned by the app.
2. **Where the code lives**: the macOS knowledge in `fresher-macos::privacy` (no core types, the crate keeps depending on
   nothing), the port implementations as thin adapters in the app; they move to `fresher-system` if plan 009 builds it
   first.
3. **Checks never prompt**; the one call that may (`try_place`) runs only inside a scan the user started, for a place
   the user opted into.
4. **Guarded folders are skipped by default** and opted into one by one with core settings (`core.access.*`, off by
   default); Full Disk Access makes the opt-ins unnecessary.
5. **Full Disk Access is detected by opening a file only it allows**, read-only, without reading a byte; candidates
   chosen in plan 01.
6. **`AccessProbe` is extended, not renamed**, and acting on the system is a separate `SystemUi` port.
7. **OS-neutral capability names in the core** (`AllFiles`, `Folder`, `OtherAppsData`, `Volume`, `Tool`);
   `Unreadable::NeedsFullDiskAccess` becomes `Unreadable::Guarded`.
8. **Tools are capabilities checked by the engine** through `CommandRunner` with a check command the module declares;
   at scan start and on refresh, not on focus.
9. **No `sudo` command is shown**; Finder is the way for items that need an administrator.
10. **The window's focus triggers a re-check** of the OS permissions; `AccessChanged` only on a change; no automatic
    rescan.
11. **A coverage event of its own** (`ScanCoverage`) rather than a new field on `ScanFinished`, so plans 003 and 004 can
    proceed in parallel; no path in it.
12. **System Settings opened with `/usr/bin/open`** through the process runner; no new dependency, no `unsafe`.
13. **No Apple Events**: the app declares no Automation usage; moving to the Trash goes through the file manager call
    of plan 008, not Finder scripting.
14. **App Management is left out** until an uninstaller is planned.
15. **Not sandboxed; outside the Mac App Store.**
16. **Category-level needs**, `Required` or `SeesMore`; a module whose every category is unavailable is not run and
    ends `Unavailable`.
17. **The developer docs** get a new page, `docs/architecture/permissions.md` (written in 05/03), linked from
    the architecture README.
18. **Target version 0.4.0**, shared with plan 005; whichever of the two finishes last moves the version.

Made while writing the steps:

19. **Removability is set by the engine, never by a module**, from the facts of the match whose path the removal names;
    a path removal with no such match is not removable (fail closed).
20. **Two new refusals**: `NoSettingsPane` (a capability with no pane: a tool, or one this Mac lacks) and
    `NothingToReveal` (a finding with no path, such as a Docker image).
21. **A disabled section stays focusable** (`aria-disabled` with its reason), so keyboard and VoiceOver users hear why
    it is disabled.
22. **The `core.access.*` switches appear only in the Permissions group**, not again in the generic settings list; they
    are added to plan 005's core settings file in a commit of their own.
23. **The probes are a binary run from Finder**, not from a terminal, so macOS attributes access to the probe itself
    and the measured behaviour is the app's.

Made while refreshing the steps against the code (2026-11-19):

24. **Probe results from documentation; plan 01 is the user's run by hand.** The probes need a person watching for
    prompts on a real Mac, so "Probe results" is filled from Apple's documentation and well-known behaviour, every row
    marked unverified. Plan 03 encodes them as they stand; if the user's run disagrees, 03/01's table of places and
    candidates and 03/02's addresses change, and the change is written in "Probe results".
25. **One pane address for every supported version**: the app's minimum is macOS 13, and the documented addresses are
    the same from 13 on, so `pane_address` takes no macOS version and no FFI call reads one. A version split comes
    back only if the user's run finds a version where an address fails.
26. **The places of today's standing avoid list keep their guard**: Desktop, Documents, Downloads by their folder;
    `Library/Containers` and `Library/Group Containers` by `OtherAppsData`; Pictures, Movies, Music, iCloud Drive,
    the cloud storage folder, Calendars and the address book by `AllFiles` (never tried, entered only with Full Disk
    Access). No place avoided now is opened without its capability; the user's run may free some of them.
27. **A scan root inside a guarded place is a guarded place itself**: `guarded_places` reports the root with its
    place's capability, so a root such as `~/Documents/Projects` is skipped or tried like Documents, never opened
    unasked.
28. **`MacAccess` remembers its own tries**: the answer of `try_place` per capability is kept for the session and
    reported by `status()` (no new open), so the Permissions group shows `Granted` or `Denied` after a scan tried a
    folder rather than `NotAsked` forever.
29. **Data from other apps is `NotApplicable` before macOS 14**, the version read once from
    `/System/Library/CoreServices/SystemVersion.plist` with `fresher-system`'s plist reader; unread → applicable
    (fail closed: guarded).
30. **Sandbox mode keeps its stand-ins**: its access answers `Unknown` (the sandbox's world is not the Mac's), and its
    standing avoid list, the guarded places of both the sandbox's home and the real home, stays, so a sandbox scan
    never raises a prompt on the real Mac. The normal app loses its standing list: the capability state feeds the
    walk request's `avoid`.
31. **The deferred containers item is due in 03/05**, before 03/03 lets the walk enter `Library/Containers`: the
    `container` row is not offered (open question 1), and `app-leftovers` declares `SeesMore(OtherAppsData)` on its
    three categories, since an unseen container hides proof. Superseded by call 33: the module reads no container and
    declares no need.
32. **The version stays 0.1.0**: the user's decision of 2026-11-16 ([`deferred/release-versioning.md`](../../deferred/release-versioning.md))
    supersedes agent's call 18; plans no longer move it.

Made at the final review (2026-11-19):

33. **Open question 1 takes option (b)** (to check): a container neither proves an id nor is offered until the helper
    list covers every nested bundle. `places.toml` drops the `container` row rather than keep a row with neither use
    (which `PlacesError::RowWithoutUse` refuses, and should); the `offered` row key and that refusal stay for a future
    row read only as a proof. The module no longer reads `Library/Containers`, so its categories declare no need
    (agent's call 31's `SeesMore(OtherAppsData)` goes). The cost, as before plan 006: an app proven only by its
    container is not found, and once a cleanup takes an app's saved state (its only other proof) its remaining
    leftovers are no longer claimed.

## Conclusions

See the plan's [readme](./readme.md), "Conclusions" and "By hand". The "Probe results" above are still
unverified: the user's run of plan 01 is open.
