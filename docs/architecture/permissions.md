# Permissions and privileges

What the app may see and do on a Mac, and how it says so, as plan [006](../plans/006-permissions/readme.md) built it;
its design argues every choice ([`00-design.md`](../plans/006-permissions/00-design.md)). The user's view of it is in
[`goal.md`](../goal.md); the rules it keeps are in [`restrictions/privacy.md`](../restrictions/privacy.md), "Least
permissions", and [`restrictions/deletion-safety.md`](../restrictions/deletion-safety.md), "No administrator rights".

## The model, in short

All of it is in `fresher-core` (`access.rs`, `removability.rs`) and knows nothing about macOS.

- A **`Capability`** is something the app may or may not be able to do: Full Disk Access (`AllFiles`), one of the
  guarded folders (`Folder(Desktop | Documents | Downloads)`), data of other apps (`OtherAppsData`), a removable or
  network volume (`Volume(..)`), or a program a module runs (`Tool(ToolId)`, named only by modules).
- An **`Access`** is the state of one capability: `Granted`, `Denied`, `NotAsked`, `Unknown` or `NotApplicable` (this
  Mac has nothing that guards it). `Granted` and `NotApplicable` allow; the rest are missing. `AccessStatus` holds one
  `CapabilityState` per capability.
- A **`Need`** is what a category says about a capability: `Need::required` (without it the category cannot run) or
  `Need::sees_more` (without it the category runs and sees less). `Availability::of(needs, status)` is `Available`,
  `Limited` (a `SeesMore` need is missing) or `Unavailable` (a required one is missing).
- **Coverage** is what a finished scan says it did not see: the guarded places it did not enter, with their state, and
  the folders it could not read, grouped by capability (`scan-coverage`). A result is never shown without it.
- **Removability** is a fact of every finding, worked out by the core from what the walk read about the entry (its
  owner, whether the app may write to its folder, its flags): `Removable`, `NeedsAdministrator`, `Locked` or
  `ProtectedBySystem`. A module cannot set it. An item that is not `Removable` is shown with its size and a reason,
  never selected, and the plan builder leaves it out (`SkipReason::NotRemovable`).

The engine resolves the model: `AccessView` merges the probe's status with the state of each tool, `modules()` carries
each category's availability, a module whose every category is `Unavailable` is not called, and a scan's coverage is
sent as an event. The UI renders what the core says (notices, limited and unavailable sections, the Permissions group
of the settings screen) and holds no rule of its own.

## Where the macOS knowledge lives

- **`fresher-macos::privacy`** (`crates/macos/src/privacy/`): the one table of guarded places under a home folder with
  the guard of each (`places.rs`, constant `PLACES`); the checks (`check.rs`: the Full Disk Access check, `try_place`,
  the candidates in `FDA_CANDIDATES`); the guard of a volume (`volumes.rs`); the System Settings pane of each guard and
  the fixed argument lists of `/usr/bin/open` (`panes.rs`, constant `ADDRESSES`). It only builds paths and arguments
  and opens a file read-only; it runs nothing.
- **`fresher-system`**: the adapters of the core's ports. `MacAccess` implements `AccessProbe` over `privacy` and
  remembers the answer of each of its own tries; other apps' data is `NotApplicable` before macOS 14 (the version is
  read once from the system's version file). `MacSystemUi` implements `SystemUi`: it opens a pane or reveals a path by
  running `/usr/bin/open` through the `CommandRunner` port.
- **`fresher-core`**: `expand_home` and the ports; **`fresher-testing`**: `FakeAccess` and `FakeSystemUi`.
- **The shell** (`app/tauri`): `get_access_status`, `refresh_access`, `open_access_settings`, `relaunch` and
  `reveal_finding`; the re-check when the window gains focus; `Info.plist`. See [`README.md`](./README.md),
  "Permissions in the shell".

Only `fresher-macos` and the adapters on top of it know what macOS guards. A module never names a path or a pane.

## When macOS may prompt

**A check never prompts. A prompt may appear only inside a scan the user started, for a place the user opted into.**

- The Full Disk Access check opens a file that only that permission allows (and that never prompts), read-only and
  without reading a byte. Nothing else is asked before the open: inside a guarded folder even `exists` and `metadata`
  answer `EPERM`.
- The guarded folders, other apps' data and the volumes have no call that tells their state safely. They are
  `NotAsked` until the user opts in: a folder or other apps' data by its switch in Settings, a removable or network
  volume by being a scan root (there is no switch for a volume). Then the scan tries the place (`try_place`) and the
  answer is remembered. With Full Disk Access granted they are `Granted` without being touched. Once macOS refused
  one, the notice and the Permissions row offer "Open System Settings" at its pane.
- The walk is given the places it must not enter as its avoid list, built from the capability state: a guarded place
  whose capability is not granted and not opted into is never opened. Sandbox mode is the exception: it keeps a
  standing list of the guarded places of both homes ([`../hacks/README.md`](../hacks/README.md)).
- Focus re-checks only the cheap OS permissions; tools are checked at the start of a scan and by "Check again".
- A grant of Full Disk Access made while the app runs applies after a relaunch, so the card offers "Reopen Fresher"
  (the user's click; the engine is shut down first). The focus re-check still runs in case it applies at once.

## How a module declares a need

A category that cannot see everything without a permission says so in its descriptor; the module does not check
anything itself.

```rust
descriptor.with_needs(vec![Need::sees_more(Capability::OtherAppsData)])
```

No module of the app declares a `SeesMore` need today: `app-leftovers` reads no app container (its
[deferred item](../deferred/app-leftovers-containers.md) brings that back with the need). A tool is a need too
(`Need::required(Capability::Tool(tool))`), and the module declares how to check it with
`ModuleDescriptor::with_tool_check(ToolCommand)`: a program that answers success when the tool is ready.
`with_tool_check` itself refuses a check no category needs (`UnusedToolCheck`) and a second check of the same tool; a
needed tool with no check is refused when the module is registered, by `ModuleDescriptor::check_tools()`
(`DescriptorError::MissingToolCheck`, `RegistryError::Descriptor`). The engine runs the check through `CommandRunner`;
running a program is not an OS permission, so no adapter is involved. See [`modules.md`](./modules.md).

## The places and the probes

The table of guarded places, the Full Disk Access candidates and the pane addresses came from Apple's documentation,
not from measurement: every row of "Probe results" in the design is marked unverified. The user's run of plan 006/01
on a real Mac confirms them or changes them; only the three constants named above change, and the change is written
in the design. **That run is still open**; until it is done, treat the table as a good guess. What a run may change:
which places prompt and which only refuse, whether a Files and Folders grant applies at once, whether Full Disk Access
needs a relaunch (if not, "Reopen Fresher" goes), and the pane addresses.

## Administrator rights

The app never runs as administrator: no privileged helper, no launch daemon, no password prompt, no `sudo`. An item
the app may not remove is shown with its size and "Show in Finder" (`reveal_finding`, by the finding's id, never a
path); the user removes it by hand and macOS asks for the password. By design, `reveal_finding` reveals any finding of
the current scan that names a path, not only the ones the app may not remove. No privileged command is ever shown to
copy. `Info.plist` holds exactly five usage descriptions (the three folders and the two volume kinds) and no other usage
key, and `app/tauri/tests/info_plist.rs` checks it.

## Developer builds

- macOS ties a grant to the app's code signature. An ad-hoc signed build gets a new identity on every rebuild, so its
  grants are lost.
- Under `npm run tauri dev` the binary runs from the terminal, and macOS attributes its access to the terminal app
  (or the editor that started it): grant Full Disk Access to that app to see a real grant.
- A stable local signing identity for development builds is part of plan
  [013](../plans/013-release/readme.md), step 01; until then, expect to grant again after a rebuild.
- Sandbox mode answers access with a stand-in (`UnknownAccess`: `Unknown` for Full Disk Access, `NotAsked` for every
  other capability), because the sandbox's world is not the Mac's; it never raises a prompt.

### How to test each state

1. **The gallery and the fakes first.** The gallery shows the permission card and the rows in every state. Rust tests
   use `FakeAccess` (any state per capability, guarded places, scripted answers of `try_place`) and `FakeSystemUi`
   (records every call); the engine's availability and coverage tests run on them. No test asks macOS for anything.
2. **A real grant by hand**, on a development Mac only: run the app, scan, and grant or deny in System Settings;
   check the notices, then reset with `tccutil reset All <bundle id>` to start again:
   `io.github.alexsergey.fresher.dev` for `npm run dev` (it applies `app/tauri/tauri.dev.conf.json`, which sets that
   identifier) and `io.github.alexsergey.fresher` for a built app (`app/tauri/tauri.conf.json`). The list of checks by
   hand is in the plan's [readme](../plans/006-permissions/readme.md), "By hand".
