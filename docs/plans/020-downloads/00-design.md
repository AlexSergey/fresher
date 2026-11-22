# 020 — Downloads and installers: design

**Status:** ✅ approved (the user's decision of 2026-11-19) · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-19 from the approved one-file draft (the user approved it with its ten agent's calls: "start
implementing, work autonomously, take the recommended decisions") and from a read of the code on
`feat/020-downloads` (from `main` 5c61c57) and of this Mac, read-only. Where the draft was wrong about the code or
about macOS, the design is corrected and the correction is listed under "Agent's calls". Facts marked "to verify" are
checked by hand in 05/02.

## Intent

The idea [Downloads and installers](../../ideas/downloads-and-installers.md): the Downloads folder collects what the
user thinks of as temporary — downloads that never finished, installers of apps that are long installed, old macOS
installers, big files downloaded once and forgotten. Nothing removes them, and together they take gigabytes. Plan
[017](../017-temp-files/readme.md) kept them out of the temporary-files module on purpose (the user's decision of
2026-11-17): they live among the user's own files, some can still be resumed, and the folder is guarded by macOS.

## Goals

1. A module `downloads` (`modules/downloads`, crate `fresher-module-downloads`) with five categories, each with a
   danger level and a plain reason that says why this file was matched and what it costs to lose it.
2. Nothing in Downloads is read unless the user allowed it (plan [006](../006-permissions/readme.md)): Full Disk
   Access counts as allowing Downloads; without it the Downloads switch decides, even when macOS remembers an earlier
   grant (decision 12). Without access the module is `Unavailable` and its notice says why and how to allow it — and
   in the scan that first tries the folder, the module runs.
3. A file is claimed only with proof. An installer whose app cannot be identified without opening or running anything
   is not claimed; a file that is only big and old is the user's call, never more.
4. The day setting shared by every module that has one, once in the core (a deferred item, due now).

## Non-goals

- Mounting a disk image, expanding an archive or running an installer to find out what is inside.
- Looking outside `~/Downloads` (the Desktop, a browser's custom folder), or below its first level.
- `Install macOS ….app` in `/Applications`: the [smart uninstaller](../036-uninstall/readme.md)'s ground.
- Archives (`.zip`, `.tar.gz`) whose content was already expanded next to them: a separate idea.
- Duplicates of a download (`file (1).dmg`): the [duplicate finder](../../ideas/duplicate-finder.md).
- Removing an unfinished download and its browser's record of it together: the browser's own list is not touched.

## What already exists

| Need                                    | What exists                                                                                              | Plan     |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------- |
| The folder                              | `KnownFolder::Downloads`; `MacKnownFolders` and `FakeKnownFolders::home_layout` resolve `~/Downloads`     | 002, 009 |
| Its guard                               | `Capability::Folder(GuardedFolder::Downloads)`, the opt-in switch `core.access.downloads`, `try_place`     | 006      |
| A category's need                       | `CategoryDescriptor::with_needs(vec![Need::required(…)])`; `Availability::of`; a module whose every category is unavailable is not run (`ModuleOutcome::Unavailable`) and the UI shows `unavailable-reason.tsx` | 006 |
| The folder kept whole                   | `HOME_WHOLE` of `crates/macos/src/protection.rs` holds `Downloads`; an item inside may go                 | 008      |
| Installed and running apps              | `SystemFacts::{installed_apps, installed_apps_complete, running_apps}`; `InstalledApp { bundle_id, name, version, path, helpers, … }`; `RunningSet::apps` | 009, 016 |
| A folder summed whole, what it holds    | `Claim::Whole`, `Match.inside` (`newest_change`, `special`, `foreign`, `complete`), `Interest::holding`   | 004, 017, 018 |
| What the walk saw of a place            | `ModuleContext::place_seen(interest)` → `PlaceSeen`                                                       | 019      |
| A file's times                          | `Match.{modified, changed, accessed}`; the status change moves on every write and attribute change        | 017      |
| A day setting                           | copied three times: `modules/{jetbrains,app-leftovers,temp}/src/settings.rs`                              | 016–019  |
| Answers that wait for the whole walk    | `finish`, a `Gathered` keyed by the scan's cancel token (`modules/app-leftovers/src/gathered.rs`)         | 016, 019 |
| Left-outs                               | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                 | 011      |
| A data table in a module                | `places.toml`, `tools.toml`: `include_str!`, parsed and checked when the module is built                 | 018, 019 |
| A finding's app in the storage map      | `NewFinding::for_app(id)`                                                                                  | 012      |
| The sandbox's world                     | `world.toml`: `now`, running apps, Spotlight's answers, stated facts (`FactsLens`); `[[case.app]]` bundles | 016–019  |
| Property lists                          | `fresher-system`'s `plist::read_strings` (XML and binary, string values only)                             | 009      |
| The system's version                    | `fresher_system::macos_version(root) -> Option<MacosVersion>` (used by `MacAccess` only)                  | 006      |

**What is missing** (built first, plans 01 and 02):

1. **A `*` that matches only some names.** `Under(Downloads, "*")` claimed whole would make every folder of the
   user's in Downloads opaque: the storage map could not open it and no other module would see inside it (a project
   with `node_modules` unpacked there). The bundles (`.download`, `Install macOS ….app`) must be summed whole, every
   other entry left open.
2. **An installed app's change time, the installer's receipts and the system's version**: none reaches a module.
3. **A module that needs a guarded folder runs in the scan that first tries it.** Today `decide_access` decides each
   module's availability before `decide_places` tries the opted-in places: in the first scan after the user switches
   Downloads on, the folder is walked but the module is `Unavailable` (the probe said `NotAsked`) and offers nothing
   until a later scan after a focus refresh. No module needed a guarded folder before.
4. **The day setting in the core** (the deferred item) and **a group** for a module about the user's own files.
5. **The sandbox's access**: sandbox mode answers every permission `NotAsked` (`UnknownAccess`) and keeps a standing
   avoid list holding the sandbox's own `Downloads`, so no case can show the module at all.

## Design

### 1. Core prerequisites (plan 01)

#### 1.1 The shared day setting

```rust
/// A module's "unchanged for this many days" setting, `<module>.<name>`: a number of days from `min` to `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaySetting { name: &'static str, min: u32, max: u32, default: u32 }

impl DaySetting {
    pub const fn new(name: &'static str, min: u32, max: u32, default: u32) -> Self;
    /// The descriptor of `<module>.<name>`, `SettingKind::Number { min, max, unit: Some(Unit::Days) }`; refused
    /// (`SettingError`) for a name that does not make a key or a default outside the range.
    pub fn descriptor(&self, module: ModuleId) -> Result<SettingDescriptor, SettingError>;
    /// The days: a value the engine did not resolve counts as the default, one out of range as its nearest limit.
    pub fn days(&self, settings: &ModuleSettings) -> u32;
}
```

In `crates/core/src/settings/days.rs`, re-exported. `jetbrains` (`uninstalled-after-days`, 7–3650, 30),
`app-leftovers` (the same) and `temp` (`older-than-days`, 7–30, 7) each keep one `const` and lose their
`settings.rs`; the keys, ranges and defaults do not change, so no stored setting moves. The deferred item's file and
row are removed.

#### 1.2 Interests that match only names with a suffix

```rust
impl Interest {
    /// This interest, its pattern's last `*` matching only a name that ends with one of `suffixes` (ASCII case
    /// ignored), as `only_bundle_ids` does for bundle ids. Only for a `Pattern::Under` whose last segment is `*`.
    pub fn only_suffixes(self, suffixes: Vec<String>) -> Result<Self, InterestError>;
    pub fn name_suffixes(&self) -> &[String];
}
```

- Refused: another pattern or a last segment that is not `*` (`NoStarToShape`); an empty list, a suffix that is not
  `.` followed by a plain name's characters, or listed twice (`BadSuffix`); an interest already shaped as bundle ids,
  or the other way round (`TwoShapes`).
- The matcher's trie gains a suffix branch beside `bundle_id` (`trie.rs`): a node keeps its suffixed branches, and a
  name that ends with one of a branch's suffixes reaches that branch's node, the node of `any` with it (as a shaped
  `*` does today: a plain `*` takes the name too). The fingerprint covers the suffixes. `NaiveWalk` matches the same;
  the reference property test generates suffixed and plain names.
- Cost: one suffix check per entry of a folder whose trie node has a suffix branch (`~/Downloads` only), for a name no
  literal took. The scan bench is run before and after (rule 4), with a suffixed interest added to it.
- A node that ends a `Whole` interest and an `Item` interest is summed whole and both hits are emitted (the
  aggregator already does so); the module takes a bundle from its suffixed interest only.

#### 1.3 An installed app's change time, the receipts and the system's version

```rust
pub struct InstalledApp {
    // …
    /// The last status change of the bundle folder itself: moved when the app is copied in, replaced or updated,
    /// never set back by a copy; `None` when it could not be read.
    pub changed: Option<Timestamp>,
}

/// A package the system's installer recorded.
pub struct Receipt {
    /// The package's file name as the installer saw it (`Example-1.0.pkg`); for a product archive, often the name of
    /// a component inside it.
    pub package_file: String,
    /// When it was installed: the receipt's own modification time.
    pub installed: Timestamp,
}

/// The version of the running system.
pub struct SystemVersion { pub major: u16, pub minor: u16 }

/// What the system recorded about installs.
pub trait InstallRecords: Send + Sync {
    /// Every receipt the system's installer wrote; a receipt that cannot be read is left out (it only proves less).
    fn receipts(&self) -> Result<Vec<Receipt>, SystemError>;
    fn system_version(&self) -> Result<SystemVersion, SystemError>;
}
```

- `SystemPorts` gains `installs: Arc<dyn InstallRecords>`; `SystemFacts` gains `receipts()` and `system_version()`,
  each asked once per scan and kept, as the other facts.
- `fresher-system`: `MacInstalledApps` reads `changed` from the bundle's own `symlink_metadata` (`MetadataExt::ctime`);
  `MacInstallRecords::new(root)` reads `<root>/var/db/receipts/*.plist` (at most 10 000, each through the existing
  capped reader and `plist::read_strings`, key `PackageFileName`; `installed` the file's modification time) and the
  version through `macos_version(root)`. The app composes it with `/`.
- `fresher-testing`: `FakeInstallRecords` (receipts and a version given, or failing); `installed_app(…)` keeps
  `changed: None`.

#### 1.4 A module that needs a guarded folder runs in the scan that first tries it

- `coverage::Entry` gains `tried: Vec<(Capability, Access)>`, every answer of `try_place` of the scan.
- `AccessView::learn(&self, tried: &[(Capability, Access)]) -> bool` writes those answers into its system states
  (as `MacAccess` already remembers them; a fake probe does not) and answers whether anything changed.
- `session::run`: the guarded places are decided first; when `learn` changed something, `AccessChanged` is published;
  only then is each module's availability decided (`unavailable`, `limited`). A restore tries nothing and changes
  nothing.

#### 1.5 The group "Your files"

`ModuleGroup::Files` ("Your files": "Files you downloaded or keep yourself, such as old installers and big
downloads."), after `Apps` and before `Development` in the screen's order; `downloads` is its first module.

### 2. The sandbox's world (plan 02)

A part of the deferred item [sandbox-world](../../deferred/sandbox-world.md), item 2: the `access` of the world, and
what plan 01 adds to the system's facts.

- **Access.** A case may state `access = { downloads = "granted" }` (also `desktop`, `documents`; values `granted`,
  `denied`, `not-asked`). The world joins them; when two cases state a capability differently the most restrictive
  wins (`denied`, then `not-asked`, then `granted`), as `spotlight = "off"` wins, so `npm run sandbox -- all` stays
  valid. `world.toml` writes `[access]` only when a case states any. `World::access(home) -> Option<FakeAccess>`:
  every capability of `Capability::SYSTEM` (Full Disk Access `Unknown`, a stated folder as stated, the rest
  `NotAsked`), the stated folders guarded at `home/<Folder>`, `try_place` answering `granted` for `granted` and
  `not-asked` (the simulated prompt is accepted) and `denied` for `denied`. Sandbox mode and the autotest take it
  instead of `UnknownAccess`; the standing avoid list keeps the real home's places whole and drops the sandbox home's
  places of the stated folders, which the engine's coverage then decides as on a Mac.
- **Installs.** A case may state `macos = "15.4"` (refused when two cases state different versions) and
  `[[case.receipt]]` (`package`, a plain file name; `installed`, an age). `World::install_records()` →
  `FakeInstallRecords` with them; without a stated version the version is an error ("the sandbox states no macOS
  version"), never the Mac's.
- **An app's change time.** `[[case.app]]` (and the tree's `[[app]]`) may state `changed` (an age): a stated fact of
  the bundle's path. Sandbox mode and the autotest see the sandbox's apps through `StatedInstalledApps` (in
  `fresher-testing`: an `InstalledApps` that replaces `changed` of an app whose bundle path is stated); an app that
  states nothing has the bundle's real status change, the moment it was generated.
- **The banner.** `Build::Sandbox` carries `simulated: Vec<Simulated>` (`Running`, `Spotlight`, `Access`, `Installs`)
  instead of its two booleans; the banner says "Simulated: open apps, Spotlight, access, installs." with only the
  parts the world states.
- `docs/hacks/README.md` ("Sandbox mode avoids the places that prompt"), `docs/architecture/permissions.md`
  ("Developer builds") and the deferred item are updated; open files, tools and the steps between scans stay deferred.

### 3. The module (plan 03)

#### 3.1 Interests

| # | Interest                                                                                       | Claim | What it is for                         |
| - | ---------------------------------------------------------------------------------------------- | ----- | -------------------------------------- |
| 0 | `Under(Downloads, "*")`                                                                        | Item  | every first-level file; folders ignored |
| 1 | `Under(Downloads, "*")` `.only_suffixes([".download", ".app"])` `.holding(["Info.plist", "Contents"])` | Whole | Safari's bundles, macOS installers      |

Every category `with_needs(vec![Need::required(Capability::Folder(GuardedFolder::Downloads))])`. Matches are gathered
for the scan (`gathered.rs`, keyed by the cancel token) and decided in `finish`, the whole folder's listing known (the
placeholder of a `.part`, a name claimed by another category).

#### 3.2 Categories

| Category                     | Danger | What                                                                                                          |
| ---------------------------- | ------ | ------------------------------------------------------------------------------------------------------------- |
| `downloads.unfinished`       | Low    | a file ending `.crdownload`, `.part`, `.opdownload` or `.partial`, and the empty placeholder of a `.part`     |
| `downloads.safari-resumable` | Medium | a folder ending `.download` that holds `Info.plist`                                                            |
| `downloads.installers`       | Low    | a `.dmg` or `.pkg` file proven an installer of an installed app at its version or older (section 3.4)        |
| `downloads.macos-installers` | Low    | a folder named in `macos.toml` (`Install macOS Sonoma.app`) that holds `Contents`, for a macOS the Mac has passed |
| `downloads.large-untouched`  | High   | a file of at least 1 GB not changed or opened for the days, matched by no category above, not a `.dmg`/`.pkg` |

One setting, `downloads.older-than-days` (`DaySetting::new("older-than-days", 7, 3650, 30)`): it applies to the
unfinished, resumable and large categories; an installer is offered as soon as it is proven.

#### 3.3 The tables (rule 5)

`modules/downloads/unfinished.toml` — a row per suffix: which apps write it (checked running), whether it is a bundle,
whether it has a placeholder:

```toml
[[suffix]]
suffix = ".crdownload"
kind = "file"                  # "file" or "bundle"
apps = ["com.google.Chrome", "com.microsoft.edgemac", "com.brave.Browser", "com.vivaldi.Vivaldi", "org.chromium.Chromium"]
# placeholder = true           # the browser keeps an empty file of the final name beside it (`.part`)
```

Rows: `.crdownload` (the Chromium browsers above, to verify), `.part` (`org.mozilla.firefox`,
`org.mozilla.firefoxdeveloperedition`, `org.mozilla.nightly`; `placeholder = true`), `.opdownload`
(`com.operasoftware.Opera`, `com.operasoftware.OperaGX`), `.partial` (`com.microsoft.edgemac`, to verify),
`.download` (`com.apple.Safari`, `com.apple.SafariTechnologyPreview`; `kind = "bundle"`). Refused: not valid TOML, an
unknown field, no row, a suffix not `.` and a plain name's characters or listed twice, a row with no app or an app
that is not a bundle id, `placeholder` on a bundle, no bundle row or more than one.

`modules/downloads/installers.toml` — the words an installer's file name may carry after the app's name:

```toml
words = ["installer", "install", "setup", "mac", "macos", "osx", "darwin", "universal", "arm64", "aarch64",
         "x64", "x86", "amd64", "intel", "apple", "silicon", "latest", "stable", "release"]
```

Refused: empty, a word that is not lowercase ASCII letters and digits, a word listed twice.

`modules/downloads/macos.toml` — the macOS installers by bundle name and the version they install:

```toml
[[installer]]
name = "Install macOS Sonoma.app"
version = "14.0"
```

Rows from `Install macOS High Sierra.app` (10.13) to the newest names (to verify on the developer's Mac and Apple's
support page in 05/02; a missing name only keeps its installer). Refused: empty, a name not `Install macOS ….app` or
`Install OS X ….app`, listed twice (case ignored), a version that is not `<major>.<minor>`.

#### 3.4 The decision

Per first-level entry of the gathered listing (sorted by name), the first rule that holds; a link or an `Other` entry
is never claimed, nor anything below the first level.

**Unfinished** (a file whose name ends with a file row's suffix, case ignored, and is longer than it):

| # | Situation                                                                                        | Result                       |
| - | ------------------------------------------------------------------------------------------------ | ---------------------------- |
| 1 | a restored scan                                                                                  | left out `scan-again`        |
| 2 | the running apps cannot be listed                                                                 | left out `no-system`         |
| 3 | an app of the row is running                                                                      | left out `browser-open`      |
| 4 | its newest of `modified` and `changed` is unknown, in the future or within the days               | left out `recent`            |
| 5 | otherwise                                                                                         | **offered**; with a `.part`, its placeholder too (below) |

The placeholder of `<name>.part` is the file `<name>` beside it: offered as a second finding of the category when it
is a regular file of 0 bytes and the `.part` is offered; a placeholder that is not empty is the user's and never
claimed. When the place was not seen in full (`!place_seen(0).is_complete()`), the `.part` is decided alone and no
placeholder is offered. An entry is decided once: a placeholder offered with its `.part` (`setup.dmg` beside
`setup.dmg.part`) is not decided again as an installer.

**Safari-resumable** (a folder of interest 1 ending `.download` and holding `Info.plist`): rows 1 to 4 with Safari's
ids and the folder's `inside.newest_change`; then `inside` not `complete` → `scan-again`; `inside.special` →
`special`; `inside.foreign` → `not-yours`; otherwise **offered**. A `.download` folder without `Info.plist` is not
claimed.

**macOS installers** (a folder of interest 1 whose name is a row of `macos.toml`, case ignored, holding `Contents`):

| # | Situation                                                                             | Result                       |
| - | ------------------------------------------------------------------------------------- | ---------------------------- |
| 1 | a restored scan, or `inside` not `complete`                                           | left out `scan-again`        |
| 2 | `system_version()` fails                                                              | left out `no-system`         |
| 3 | the running macOS is older than the row's version                                     | left out `newer-macos`       |
| 4 | `inside.special` / `inside.foreign`                                                    | left out `special` / `not-yours` |
| 5 | otherwise                                                                              | **offered**                  |

**Installers** (a file whose name ends `.dmg` or `.pkg`, case ignored): the installer's time `t` is its newest of
`modified` and `changed` (opening a disk image from Finder moves its status change, so `t` is at least the download
and usually the moment it was opened). Proof, then the rows:

- **By receipt** (`.pkg` only): a receipt whose `package_file` equals the file name (case ignored) and whose
  `installed` is at or after `t`.
- **By name** (`.dmg` and `.pkg`): the file's stem (its name without the extension) names exactly one installed app
  that is not Apple's (`com.apple.`), by one of its forms — its name, or the last part of its bundle id, lowercased,
  forms shorter than 3 characters dropped; a space in a form matches one separator (` `, `-`, `_`) or none —
  followed by the end, a separator (` `, `-`, `_`, `.`) or a digit; and every token of the rest (split at separators)
  is a version (digits and dots with at least one dot, an optional leading `v`), a plain number, or a word of
  `installers.toml`. The installer's version is its first version token, if any. Examples: `Firefox 133.0.dmg`,
  `docker.dmg`, `GoogleChrome.dmg` (name `Google Chrome`), `Example-2.1-arm64.pkg` are named; `notes-backup.dmg`
  (`backup` is no word), `Firefox (1).dmg` (a copy mark), `install.dmg` (no app) are not.

| # | Situation                                                                                              | Result                     |
| - | ------------------------------------------------------------------------------------------------------ | -------------------------- |
| 1 | no receipt proves it, and the installed apps cannot be listed                                          | left out `no-system`       |
| 2 | no receipt proves it, and the name names no installed app or two                                       | not claimed (the user's file) |
| 3 | no receipt proves it, and the list of installed apps is partial                                        | left out `apps-unread`     |
| 4 | a restored scan                                                                                        | left out `scan-again`      |
| 5 | by name: the app's `changed` unknown or before `t`; or the installer's version newer than the app's `version`, or a version that cannot be compared with it | left out `not-older` |
| 6 | otherwise                                                                                              | **offered**, `for_app` the app (by name) |

Receipts that cannot be read prove nothing (logged); the name rule still applies. An installer proven by receipt only
has no app (no `for_app`). A `.dmg` or `.pkg` that is not offered is never offered as large-untouched.

**Large-untouched** (a regular file, not a `.dmg`/`.pkg`, claimed by no rule above, `sizes.logical` at least
1 000 000 000 bytes):

| # | Situation                                                                                              | Result                  |
| - | ------------------------------------------------------------------------------------------------------ | ----------------------- |
| 1 | a restored scan                                                                                        | left out `scan-again`   |
| 2 | its newest of `modified` and `changed` is unknown, in the future or within the days                    | left out `recent`       |
| 3 | otherwise                                                                                              | **offered**, High       |

"Not opened" is the status change: opening a file from Finder writes its last-used attribute
(`com.apple.lastuseddate#PS`, what Spotlight shows as its last used date), which moves the status change; so does a
rename, a tag or a move into the folder. A file opened only by an app that bypasses Finder is "not changed", which
the reason says.

- Rows that keep an entry report it left out in its category with its sizes; "not claimed" says nothing (rule 1).
- At the act: an item a process holds open is skipped by the cleaner's in-use check; everything goes to the Trash.

A finding:

| Part     | Value                                                                                                     |
| -------- | --------------------------------------------------------------------------------------------------------- |
| identity | `<category name>/<file name>` (`unfinished/movie.mp4.crdownload`)                                          |
| subject  | `Subject::Path`; `Removal::MoveToTrash` of the same path                                                   |
| sizes    | the match's (a bundle's whole)                                                                             |
| title    | `downloads.title` with `{name}` (the file name)                                                            |
| reason   | per category, below                                                                                        |
| app      | an installer proven by name: `for_app(id)`; the map counts it in the installed app's line                  |

#### 3.5 Texts

Every text in `modules/downloads/locales/en.json` (rule 7):

- **unfinished** — name "Unfinished downloads"; what: "Downloads that stopped before they finished: the part a
  browser fetched before it was closed or the download failed."; made-by: "Your browser (Chrome, Firefox, Opera, Edge
  and others), while it downloads a file."; if-removed: "They go to the Trash. To get the file, download it again."
  comes-back: "Only if you download the file again."
- **safari-resumable** — name "Safari downloads you can resume"; what: "Downloads Safari paused or could not finish.
  Double-clicking one in Finder resumes it."; made-by: "Safari, while it downloads a file."; if-removed: "They go to the
  Trash, and the download cannot be resumed: you start it again from the beginning."; comes-back: "No."
- **installers** — name "Installers of installed apps"; what: "Disk images and packages you used to install apps that
  are on this Mac now, at this version or newer."; made-by: "You, when you downloaded the app."; if-removed: "They go to
  the Trash. The app stays installed; to install it again, download it again."; comes-back: "Only if you download it
  again."
- **macos-installers** — name "Old macOS installers"; what: "Installers of a macOS version this Mac already runs or has
  passed. Each takes 12 to 15 GB."; made-by: "You or the App Store, when you downloaded macOS."; if-removed: "They go to
  the Trash. To make a bootable installer later, download it again."; comes-back: "Only if you download it again."
- **large-untouched** — name "Big downloads you have not opened"; what: "Files of 1 GB or more in Downloads that you
  have not opened or changed for a long time. Only you know whether you still need them."; made-by: "You, when you
  downloaded them."; if-removed: "They go to the Trash. Anything you have only here is lost once you empty the Trash, so
  look at each one first."; comes-back: "No."

Titles and reasons: `downloads.title` "{name}"; `downloads.reason.unfinished` "This download stopped and has not
changed for {days} days."; `.placeholder` "The empty file a browser keeps beside an unfinished download. It holds
nothing."; `.safari-resumable` "Safari has not continued this download for {days} days."; `.installer` "{app} {version}
is installed, so you no longer need its installer." (`{version}` is "" when the app has none: the step writes the text
so both read well — two keys, `.installer` and `.installer-no-version`); `.installer-receipt` "This package was
installed on {date}."; `.macos-installer` "This Mac runs macOS {running}, which is as new as this installer or newer.";
`.large-untouched` "{size}, not opened from Finder or changed since {date}. It is your own file: keep it if you are
not sure."

The switch: `downloads.enabled.label` "Downloads and installers", `.description` "Finds what piles up in your Downloads
folder: unfinished downloads, installers of apps you already have, old macOS installers and big files you have not
opened." The setting `downloads.older-than-days`: label "Offer downloads untouched for", description "Unfinished
downloads and big files are offered only once they have not changed for this many days, at least 7. Installers are
offered as soon as their app is installed."

The left-out reasons, declared in this order:

| Key                                   | Text                                                                                                          |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `downloads.left-out.no-system`        | Fresher could not ask macOS about your apps or its version, so it keeps this file. Scan again.               |
| `downloads.left-out.apps-unread`      | Fresher could not read every app in your apps folders, so it cannot be sure which app this installer is for. |
| `downloads.left-out.browser-open`     | The browser that was downloading this is open, so the download may still finish. Quit it, then scan again.   |
| `downloads.left-out.recent`           | It changed within the days set in Settings. A scan after those days offers it.                               |
| `downloads.left-out.not-older`        | Fresher cannot tell that the app you have is as new as this installer, so it is kept. Install it, or remove it yourself if you do not need it. |
| `downloads.left-out.newer-macos`      | This installer is for a newer macOS than this Mac runs, so it is kept.                                        |
| `downloads.left-out.scan-again`       | Fresher could not look at this in full, so it keeps it. Scan again to check.                                  |
| `downloads.left-out.special`          | It holds a connection a running program may be using, so it is kept.                                          |
| `downloads.left-out.not-yours`        | Some of it belongs to the system or another user, or is locked. Removing it needs an administrator.          |

### 4. Tests

**Unit** (`modules/downloads/src/*`, negatives first): the three tables' refusals, then the shipped tables; `name.rs`
— the name rule's examples above both ways, a 2-character form dropped, two apps named → none, Apple's app never
named, the version token and the comparison (`133.0` ≤ `133.0.1`, `2.10` > `2.9`, `1.0b2` not comparable);
`decide.rs` — each row's keep before its offer, the precedence (restored over everything, a running browser over the
age, receipt proof without an app); properties: adding a running app or a later change never turns a keep into an
offer.

**Fixture** (`modules/downloads/tests/fixtures.rs`, `run_module`, `Catalogue::Required`), every entry stating `age`
and `changed`:

| Home                          | Kind     | What                                                                                          | Expected                    |
| ----------------------------- | -------- | --------------------------------------------------------------------------------------------- | --------------------------- |
| `downloads-not-claimed.toml`  | negative | `report.pdf`, `notes-backup.dmg`, `install.dmg`, `Firefox (1).dmg`, a link `movie.mp4.part`, a folder `photos/` holding `big.mov` (2 GB, 400 days), `Example.app` (an app run from Downloads), `trip.download` without `Info.plist`, a 900 MB file 400 days old | nothing offered, nothing left out |
| `downloads-kept.toml`         | negative | one per reason: `a.crdownload` with Chrome running, `b.crdownload` 3 days old, `Firefox 140.0.dmg` beside Firefox 133.0, `Docker.dmg` older than Docker's `changed`… (each in its own home or with its own name) | left out by reason with each count |
| (the same, ports failing)     | negative | the running apps failing; the installed list partial, then failing; the version failing; a restored run | `no-system`, `apps-unread`, `no-system`, `no-system`, `scan-again` |
| `downloads-offered.toml`      | positive | every category: `.crdownload`, `.part` with its empty placeholder, `.opdownload`, `.partial`, a `.download` bundle, `Firefox 133.0.dmg`, `Example-2.0.pkg` with its receipt, `Install macOS Sonoma.app` on macOS 15.1, `video.mov` 2 GB 400 days | each offered at its danger, title and reason; golden `fixtures/golden/downloads.json` |

Plus: the matches handed in two orders give the same answers; the module never asks the system for anything when
Downloads holds no candidate (the fakes count their calls).

**Sandbox** (rule 8), three case files, each a world of its own:

- `fixtures/sandboxes/downloads.toml` (`access = { downloads = "granted" }`, `macos = "15.4"`), negatives first: a
  document is never claimed; a disk image of no installed app is never claimed; a copy of an installer is never
  claimed; a link named like a download is never claimed; a folder of yours is never claimed (`[case.map]`
  `your-folders`); a big file inside a folder of yours is never claimed; an unfinished download changed lately is
  kept (`recent`); an installer newer than the installed app is kept (`not-older`); an installer for a newer macOS is
  kept (`newer-macos`: `Install macOS Tahoe.app`, a row of the table newer than the stated 15.4); a big file opened
  lately is kept (`recent`). Positives: an unfinished Chrome download goes; an
  unfinished Firefox download goes with its empty placeholder; a Safari download goes, at Medium; an installer of an
  installed app goes (`[[case.app]]` `Example.app` 2.0 with `changed = "10d"`, `Example-2.0.dmg` 20 days); a package
  with its receipt goes; an old macOS installer goes; a big file untouched for a long time goes, at High.
- `fixtures/sandboxes/downloads-denied.toml` (`access = { downloads = "denied" }`): negative "nothing in Downloads is
  read without access": an unfinished download and an installer, untouched, the module unavailable.
- `fixtures/sandboxes/downloads-browser-open.toml` (Chrome `[[case.running]]`, access granted): negative "an
  unfinished download of an open browser is kept" (`browser-open`).

What the sandbox cannot show (an app installed before its installer was downloaded needs the installer to be newer
than the app's stated change, which `changed` gives; a placeholder that is not empty) is covered by the fixtures and
the core e2e.

**Core e2e** (`crates/e2e/tests/downloads.rs`, harness `downloads_home/mod.rs` on the shared `support/home.rs`,
fixture `fixtures/homes/downloads-flow.toml`, golden `fixtures/golden/downloads-flow.json`; `FakeAccess` with the
home's Downloads guarded):

1. **Not opted in**: Downloads `NotAsked` and the switch off: the module `Unavailable` naming Downloads; nothing read
   in Downloads (`FakeAccess::tried` empty).
2. **The first scan after opting in**: the switch on, `try_place` answers granted: the module runs in that scan and
   offers; `AccessChanged` was published.
3. **Denied**: `try_place` answers denied: `Unavailable`, the coverage names Downloads not entered.
4. **Scan → confirm → Trash → put back** of every category; the plan names High; put back restores byte for byte.
5. **The browser quits**: `browser-open`; the running apps change; a rescan (asserted `ScanKind::Rescan`) offers it.
6. **Days pass**: an unfinished download 20 days old kept `recent`; the clock 15 days on; a rescan offers it.
7. **The app is updated**: an installer kept `not-older` (the app older than it); the app's `changed` moves after it;
   a rescan offers it.
8. **The storage map**: an offered installer counts in its app's line; a folder of the user's in Downloads opens in
   the browse view (nothing whole-claimed but bundles).
9. **Ignore**: the large-untouched category ignored: in no total; restored.
10. **Full Disk Access**: granted, the switch off: the module offers, nothing is tried (decision 12).
11. **A remembered grant, the switch off**: opted in, scanned (granted), the switch turned off: the next scan tries
    nothing, the module is `Unavailable`, Downloads is not entered (decision 12).

**UI e2e** (Playwright, mocked core; `app/ui/src/test/downloads-scan.ts`, the module's real texts):

1. **Your files → Downloads and installers** in the sidebar; its page lists the five categories with their dangers.
2. Before access: the module's unavailable notice names Downloads and how to allow it (the plan 006 notice).
3. The left-out lines ("The browser that was downloading this is open…"), not selectable.
4. Select the installers and the large files: Review names High; confirm; result; Cleanups; put back.
5. Settings → Downloads and installers: the switch and the days.

Visual and accessibility: the module's page and its unavailable state, in `visual.spec.ts` and `axe.spec.ts`.

**By hand** (05/02): the three sandboxes with `EXPECTED.md`; on the developer's Mac a scan only (`npm run dev`; its
cleaner reaches only a sandbox), with Downloads switched on: every offered item checked by hand, the facts "to verify"
read.

### 5. Where the code lives

| Crate or folder              | What it gains                                                                                              |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `crates/core`                | `DaySetting`; `Interest::only_suffixes`; `InstalledApp.changed`; `Receipt`, `SystemVersion`, `InstallRecords`, `SystemPorts.installs`, `SystemFacts::{receipts, system_version}`; `ModuleGroup::Files` |
| `crates/scanner`             | the suffix branch of the trie and its fingerprint                                                          |
| `crates/system`              | `MacInstalledApps` reads `changed`; `MacInstallRecords`                                                    |
| `crates/engine`              | `coverage::Entry.tried`, `AccessView::learn`, availability after the places                                |
| `crates/testing`             | suffixes in `NaiveWalk`; `FakeInstallRecords`; `StatedInstalledApps`; `[[app]] changed`; `Ports.installs` |
| `crates/sandbox`             | `access`, `macos`, `[[case.receipt]]`, `changed` on an app, in the cases and `world.toml`                  |
| `modules/downloads/` (new)   | `fresher-module-downloads`                                                                                 |
| `modules/{jetbrains,app-leftovers,temp}` | the shared day setting                                                                        |
| `crates/e2e`                 | the downloads flows                                                                                        |
| `app/tauri`, `app/ui`        | registration, the world's access and installs in sandbox mode, the banner, the group, the UI's test scan and scenarios |

No new third-party dependency (`fresher-macos` is untouched: its protection list already keeps `Downloads` whole and
its privacy table already guards it).

## Interfaces with other plans

- **006** (in progress): the guard, the opt-in, `unavailable-reason.tsx`; 01/04 makes the first scan after an opt-in
  run the module. Its run by hand of the privacy table is still open; this plan's run checks Downloads only.
- **019** (merged): `place_seen`, `Gathered`, the data tables' shape, `AppSearch::Declared` and the world's Spotlight.
- **016–018**: the day setting, the installed and running apps, `Inside`, the stated facts, the e2e helpers.
- **015**: groups and Ignore. **012**: `for_app`, the storage map. **011**: left-outs. **008**: the Trash, put back.

## Deferred items

- A shared day setting in the core: was due here; done in 01/01, its file and row removed.
- [The sandbox world](../../deferred/sandbox-world.md): partly taken in 02/01 and 02/02 (access for the guarded
  folders, installs, an app's change time); open files, tools and the steps between scans stay deferred.
- Not due: [the debug panel](../../deferred/dev-debug-panel.md) (it waits for the whole world);
  [why an item matched](../../deferred/match-explanations.md) (code, not rules).

## Open questions

None.

## Decisions

The user's, of 2026-11-19 (translated): "Start implementing, work autonomously, take the recommended decisions." The
draft was approved with its ten agent's calls, which became these decisions:

1. **Five categories, one module**, `downloads.large-untouched` included as High and "your call" rather than left for
   the large-files finder; the finder can reuse its rule later.
2. **One day setting for the module** (30 days), not one per category.
3. **Installers are Low**, not Medium as in the idea: the app is installed, and the installer downloads again; the
   reason names the app and its version.
4. **A `.dmg` is proven by its name and the app's change time**, never by mounting it; an ambiguous name is not
   claimed. Some real installers will be missed (`install.dmg`), and that is accepted.
5. **A `.pkg` is proven by the installer receipts**, read as files.
6. **Only `~/Downloads`**, one level deep plus bundles.
7. **`Install macOS ….app` only in Downloads**; the apps folder is left to the uninstaller.
8. **The shared day setting is plan 01** and moves all three modules (`temp`'s range 7–30 included, through the range
   parameter).
9. **Only the access part of `sandbox-world`** is taken from item 2; open files, tools and steps stay deferred (plan
   02 also takes what this module's own facts need, agent's call 6).
10. **The size bar for large-untouched is 1 GB**, fixed for now (no setting).
11. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").
12. **Full Disk Access counts as allowing Downloads; without it the switch decides** (2026-11-19, controller's ruling
    during the final review; the user accepted it the same day with the other rulings: "ok, continue"). With Full Disk Access the module runs and the walk enters
    Downloads with the switch off, as the walk already treated it. Without it, a folder whose switch is off is not
    read and its module is unavailable even when macOS remembers an earlier grant (the switch turned off after a grant
    in the same session). One function of the engine decides it for the walk and for the modules
    (`coverage::as_opted`), for every guarded place with a switch.

## Agent's calls (to check)

Made while writing this design from the code and from this Mac (read-only); each with what it costs if wrong.

1. **"Not opened" is the status change, not Spotlight's last used date** (the draft's "Last opened" corrected). On this
   Mac Spotlight's `kMDItemLastUsedDate` equals the file's `com.apple.lastuseddate#PS` attribute, and writing it moved
   the file's status change (5 of 5 files checked); a file never opened has no last used date at all (`(null)` for
   every never-opened download checked), so Spotlight would need the date added as a fallback anyway. The status
   change is at least both, needs no port, no command and no index, and the sandbox already states it. Cost if wrong:
   a file opened only through an app that writes no last-used attribute is offered (High, the user's call, the reason
   says "not opened or changed").
2. **An installer's time is its newest change, an app's install its bundle's status change** (the draft's "date
   added", birth time or quarantine date, corrected): the walk delivers no birth time, and a Finder copy keeps an app's
   birth and modification times from its build (seen on this Mac), while the bundle's status change moves when it is
   copied in or replaced. On this Mac every app's status change (and Spotlight's date added) moved to one day at once,
   a migration or a system update: such a day makes the check more permissive, never stricter; the version in the
   name and "the app is installed" still hold. Cost if wrong: an installer of a version newer than the installed one,
   with no version in its name, offered at Low.
3. **The receipt's `PackageFileName` is mostly a component's name**: on this Mac about 14 of the 15 non-Apple receipts
   name a component package (short names without a version), not a downloaded file; so a `.pkg` is also proven by the
   name rule (a `.dmg`'s), and the receipt proof stays for flat packages. Its install time is the receipt's modification
   time, within 5 s of its `InstallDate` on 8 of 8 receipts checked (the plist reader reads strings only). Cost if
   wrong: a few packages proven by receipt are missed.
4. **The rest of an installer's name must be versions, numbers and installer words** (a table): the draft's "then the
   end, a separator or a version" alone would take `notes-backup.dmg` for Apple Notes' installer; Apple's apps are
   never named at all. Cost if wrong: installers with other words in their names are missed.
5. **A `*` matching only suffixed names, in the core** (not in the draft): claiming `Downloads/*` whole would make the
   user's folders opaque to the map and to other modules; bundles need a whole claim. Cost if wrong: a trie branch and
   its bench run.
6. **Plan 02 also takes the installs and an app's change time into the world** (beyond the draft's "access" only):
   without them a sandbox's installers and macOS installers would depend on the Mac's receipts and version, and no
   case could show them deterministically. Cost if wrong: two more world entries to maintain.
7. **One port for the receipts and the system's version** (`InstallRecords`), not a receipts port alone: the macOS
   installers need the version, and one field of `SystemPorts` is one change at each of its 14 construction sites.
   Cost if wrong: a later split.
8. **The availability is decided after the guarded places are tried** (not in the draft): found in `session.rs`; the
   module would otherwise offer nothing in the first scan after the user allows Downloads. Cost if wrong: a scan order
   change in the engine, covered by its tests.
9. **A new group "Your files"** after Apps: Downloads is neither the system's, an app's leftovers nor development;
   the large-files and duplicate finders belong there later. Cost if wrong: one enum variant and one text.
10. **A `.part`'s placeholder is a second finding of the same category**, not "together or not at all": a finding has
    one path (as plan 019's agent's call 1). Both go when the category is selected; one left alone is harmless (an
    empty file, or a part with no name). Cost if wrong: a finding of two paths through the core and the cleaner.
11. **Installers wait for no days**; the day setting applies to unfinished, resumable and large files: an installer is
    worth removing right after the app is installed, and the proof already needs the app installed after it. Cost if
    wrong: one more row.
12. **1 GB is 1 000 000 000 bytes of the file's logical size**, as Finder counts. Cost if wrong: a constant.
13. **The sandbox's access: the most restrictive statement wins** across cases, and `not-asked` answers `granted` when
    the user opts in (the simulated prompt is accepted), so the opt-in can be walked by hand. Cost if wrong: a
    vocabulary change.
14. **`Build::Sandbox` lists what is simulated** (`simulated: Vec<Simulated>`) instead of growing a boolean per part:
    four booleans would need sixteen banner texts. Cost if wrong: the banner's one text.
15. **The unfinished suffixes' apps and the macOS installers' names are data to verify** (05/02): a wrong app id only
    makes the running check miss a browser (the days still hold); a missing name only keeps an installer.

## Conclusions

Not written yet.
