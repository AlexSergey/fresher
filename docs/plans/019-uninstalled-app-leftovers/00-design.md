# 019 — Uninstalled app leftovers: design

**Status:** ✅ approved (the user's decision of 2026-11-18) · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, scanner, macos, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-18 from the approved one-file draft (the user approved it with its sixteen agent's calls and
answered its two open questions) and from a read of the code on `feat/019-uninstalled-app-leftovers` (from `main`
2aa9f67). Where the draft was wrong about the code the design is corrected and the correction is listed under "Agent's
calls". Facts about macOS marked "to verify" are checked by hand, read-only, in 04/02.

## Intent

The next module in the user's agreed order (developer caches, then uninstalled-app leftovers, downloads and
installers, permissions): dragging an app to the Trash removes only its bundle; its caches, settings, saved windows,
web data, support folder and container stay in `~/Library` for good. The module finds the data of apps that are
provably no longer installed, matched by bundle id only, explains it in plain English and offers it to the Trash. It
generalizes what [plan 016](../016-jetbrains/readme.md) built for JetBrains IDEs (`jetbrains.uninstalled`) to every
app. The idea: [uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md).

## Goals

1. Only an entry of the user's `~/Library` places of section 2, named after a bundle id, is ever considered; a name
   that is not an id is never matched, whatever it looks like (`Application Support/Notes`).
2. An id is offered only when no app with it is installed, found by Spotlight or running, every check could be made,
   and the id is proven to have been an app's.
3. One category per kind of leftover, each with its danger level, so "keep settings" is a choice of categories.
4. What the module recognizes as a leftover and does not offer is reported as left out with its reason, never dropped.
5. The core gains what the code lacks: interests that match only bundle-id names, what the walk saw of each
   interest's place, a check that Spotlight can see every mounted volume, a removed app's name in the storage map, the
   group Apps and a module's selection choices.

## Non-goals (each stays in its idea)

- Removing an installed app with its data: [smart uninstaller](../036-uninstall/readme.md).
- Group containers (`~/Library/Group Containers/<team>.<name>`): shared by a developer's apps and not named after one
  app's id (the user's decision 1); a later addition.
- Launch agents, login items and helper tools (`~/Library/LaunchAgents/<id>*.plist`): removing a loaded agent needs
  `launchctl`, and a plist may start a program outside the app.
- Folders named after the app's name, not its id (`Application Support/<Name>`, `Logs/<Name>`): matching a name is a
  guess (rule 1); missed, never mistaken.
- Anything outside the user's `~/Library` (`/Library/Application Support`, other users' homes).
- The JetBrains IDEs' own folders: plan 016's module keeps them (named after the product, not the id).
- A choice per app (select one app's leftovers across the three categories): the choices of this plan select whole
  categories (agent's call 2, confirmed by the user's decision 4); a per-app choice needs the findings of one app across categories, a later addition.

## What already exists

| Need                                        | What exists                                                                                                     | Plan     |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| The places                                  | `KnownFolder::{AppCaches, AppSupport, Preferences, Logs, Containers, SavedState, Library}`; `Pattern::Under` with `*` | 002, 009 |
| A name named after an id                    | `BundleId::new` (ASCII letters, digits, `.`, `-`, `_`), `BundleId::names`                                       | 009      |
| Installed apps, helpers, a partial list     | `SystemFacts::installed_apps`, `installed_apps_complete`, `InstalledApp::owns`, `InstalledList::unread`         | 009, 016 |
| Apps elsewhere on the Mac                   | `SystemFacts::find_apps` → `MacInstalledApps::find` (`mdfind`, after a Finder probe); `AppSearch::{Spotlight, Listed}` | 016 |
| Running apps                                | `SystemFacts::running_apps`, `RunningSet::apps` with their bundle ids                                            | 009      |
| A folder summed whole and what it holds     | `Claim::Whole`, `Match.sizes`, `Match.inside` (`newest_change`, `special`, `foreign`, `complete`), `Match.facts` | 004, 017 |
| Answers that wait for the whole walk        | `finish`, `Gathered` keyed by the scan's cancel token (`modules/jetbrains/src/uninstalled.rs`)                  | 016      |
| A day-count setting                         | `modules/jetbrains/src/settings.rs` (`uninstalled-after-days`, 7 to 3650, 30)                                   | 016      |
| A data table in a module                    | `products.toml`, `tools.toml`: `include_str!`, parsed and checked when the module is built                      | 016, 018 |
| Saying why something is not offered        | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                        | 011      |
| A finding's app in the storage map          | `NewFinding::for_app(id)` → `AppPlaces::app(id)`: an id no installed app has is shown by the id itself          | 012      |
| Groups, Ignore                              | `ModuleGroup::{System, Development, Debug}`; a module's, a category's and a group's Ignore                       | 015      |
| A whole category selected                   | `app/ui/src/state/selection.ts` (`toggle-category`), the confirmation naming the highest danger                 | 003, 008 |
| The places that prompt                      | `places_that_prompt(home)` (`crates/macos/src/places.rs`): `Library/Containers` is not opened until plan 006    | 006      |
| The sandbox's apps and world                | `[[case.app]]`, `[[case.running]]`, `world.toml`, stated facts; `AppSearch::Listed` in sandbox mode              | 016, 017 |

**What is missing** (built first, plan 01):

1. **A `*` that matches only bundle ids.** `Under(AppCaches, "*")` claimed whole would make `Caches/JetBrains`,
   `Caches/Homebrew` and `Caches/Yarn` one opaque match: nothing inside is matched again, so `jetbrains` and
   `dev-caches` would see nothing there.
2. **What the walk saw of a place.** `cx.is_walk_complete()` is false on any Mac where one folder anywhere could not
   be read (without Full Disk Access, always), which would make the module offer nothing; and it says nothing of a
   place left out on purpose (`Library/Containers`, avoided) or outside the scan roots. The scanner's tree knows each
   folder's own state (`Flag::{Incomplete, Avoided, OtherVolume}`, unreadable), but no module can ask it.
3. **Spotlight's reach.** `MacInstalledApps::find` asks Spotlight and cannot tell a volume left out of the index from
   a volume with no app ([known issue](../../known-issues/app-spotlight-misses-an-excluded-volume.md)).
4. **A removed app's name** (a deferred item, done by plan 019/01/04), **the group Apps** and **a
   module's selection choices**: none exists.

## Design

### 1. Core prerequisites

#### 1.1 Interests that match only bundle-id names

```rust
/// Whether `name` is shaped as a bundle id: what `BundleId::new` accepts, with at least one dot
/// (`com.example.App`, `com.example.App.plist`); `Notes`, `JetBrains` and `.DS_Store` are not.
pub fn is_bundle_id_shaped(name: &[u8]) -> bool;

impl Interest {
    /// This interest, its pattern's last `*` matching only a bundle-id-shaped name. Only for a `Pattern::Under`
    /// whose last segment is `*`.
    pub fn only_bundle_ids(self) -> Result<Self, InterestError>;
    pub fn matches_only_bundle_ids(&self) -> bool;
}
```

- `InterestError::NoStarToShape` for any other pattern or a path whose last segment is not `*`.
- The matcher's trie gains the shaped segment: a deterministic node keeps, beside `any`, an `id` branch (the node of
  `any` and the shaped branches together); `child_state` takes a literal first, then `id` for a shaped name, then
  `any`. The fingerprint covers the shape, so a cache made with other interests is refused once (a full scan).
- `NaiveWalk` matches the same; the reference property test generates shaped and plain names.
- Cost: one shape check per entry of a folder whose trie node has an `id` branch (the module's nine places), only
  for a name no literal took. The scan bench is run before and after (rule 4), with a shaped interest added to it.

#### 1.2 What the walk saw of each interest's place

```rust
/// What the walk saw of the folder an anchored interest matches in (its "place": the pattern's folder without its
/// last segment, `~/Library/Caches` for `Under(AppCaches, "*")`).
pub enum PlaceSeen {
    /// Listed in full: every entry of its own listing was read (what is inside a matched folder is the match's).
    Read,
    /// Not there: its parent was read and holds no such folder.
    Absent,
    /// It, or a folder on the way to it, could not be read, or an entry of its listing could not be read.
    Unread,
    /// Not entered on purpose (avoided, another volume, another user's) or outside every scan root.
    NotScanned,
    /// Not known: during the walk, a restored scan, or an interest without one place.
    Unknown,
}

impl PlaceSeen {
    pub const fn is_complete(self) -> bool; // Read or Absent
}

pub struct WalkSummary {
    // …
    /// Per anchored interest with one place (index in the interest set), what the walk saw of it.
    pub places: BTreeMap<usize, PlaceSeen>,
}

impl ModuleContext<'_> {
    /// What the walk saw of the place of the module's interest `interest`; `Unknown` during the walk and in a restore.
    pub fn place_seen(&self, interest: usize) -> PlaceSeen;
}
```

- An interest has one place when its pattern is `Under` and no segment but the last is `*`.
- The scanner works it out **after** the walk, from its tree (full scan, rescan and restore alike): the place's node
  and its ancestors' own flags, never `Partial` (which an unreadable folder anywhere below sets). Nothing is added to
  the walk's hot path or to the cache format: the flags are the tree's own (to check in 01/02: the codec keeps
  `Incomplete`, `Avoided`, `OtherVolume` and the unreadable reason, so a rescan's copied folder and a restore answer
  as the walk did).
- An avoided child of a place (`Application Support/AddressBook`) does not make the place `Unread`: it was not opened
  on purpose and its entry was read.
- The engine hands each module the states of its own interests in `finish` (`ModuleContext::with_places`);
  `run_module` does too, from `NaiveWalk`, and its `Ports` gain `avoid` so a fixture can make a place `NotScanned`.
- `cx.is_walk_complete()` stays as it is; `jetbrains` keeps it.

#### 1.3 Spotlight's reach

```rust
pub trait InstalledApps: Send + Sync {
    fn list(&self) -> Result<InstalledList, SystemError>;
    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError>;
    /// Whether `find` can see every app on the Mac: Spotlight's index is on and every mounted volume is indexed. An
    /// error names what it cannot see.
    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError>;
}
```

- `SystemFacts::find_apps` asks `search_complete` once per scan (kept, as the other facts) and answers its error
  before any `find`; so every module that asks Spotlight (`jetbrains`, `app-leftovers`) gains the check, and the known
  issue is closed.
- `MacInstalledApps`, Spotlight search (`crates/system/src/spotlight.rs`): the Finder probe moves from `find` to
  `search_complete`; then every mounted volume (`fresher_macos::mounted_volumes()`, from `getfsstat` with
  `MNT_NOWAIT`, as `ffi::volume::mounts` reads them): a volume the system does not show (`MNT_DONTBROWSE`: the
  system's own volumes) is skipped; a volume that is not local (a network share) counts as not indexed without being
  asked (it may hang the command); a local one is asked `mdutil -s <mount point>` through its own runner, with a time
  limit of 10 s (`INDEX_CHECK_TIMEOUT`); it is indexed only when the command ends with status 0 and its output says
  `Indexing enabled.` (to verify). Anything else, a timeout included, is an error naming the volume's mount point
  (logged, never shown with a path).
- A mounted disk image that is not indexed (an installer) keeps everything while it is mounted: safe, and its left-out
  text says what to do.
- The sandbox's search (`AppSearch::Declared`, replacing `Listed`): the sandbox's listed apps, the apps its case files
  declare elsewhere, and whether its Spotlight is on (section 6, "Sandbox"); never the real Mac's.

#### 1.4 A removed app's name, the group Apps, a module's choices

- `NewFinding::for_app_named(app, name)`: the app and the name it showed, for an app that may be no longer installed;
  `AppPlaces::app(id, name)` uses the name when no installed app has the id (the installed app's own name wins).
  `jetbrains` passes its product's name for `jetbrains.uninstalled`; `app-leftovers` has no name and keeps
  `for_app(id)` (a name guessed from the id could mislead). The deferred item is removed.
- `ModuleGroup::Apps` ("Apps": "What apps leave behind, such as the files of apps you removed."), after `System` and
  before `Development` in the screen's order.
- **Choices**: a module may declare selection shortcuts, each a set of its own categories with two texts:

  ```rust
  pub struct ModuleChoice { name: String /* kebab case */, categories: Vec<CategoryId> }
  impl ModuleDescriptor {
      /// Refused for a name that is not kebab case or is listed twice, an empty choice, or a category that is not
      /// the module's (`DescriptorError::{ChoiceName, EmptyChoice, ChoiceCategory}`).
      pub fn with_choices(self, choices: Vec<ModuleChoice>) -> Result<Self, DescriptorError>;
      pub fn choices(&self) -> &[ModuleChoice];
  }
  ```

  Texts `<module>.choice.<name>.label` and `.description`, required by the contract test (`MissingText`).
  `ModuleInfo` carries them (`choices: Vec<ChoiceInfo { name, categories }>`); the module page shows them above its
  categories; one click makes the module's selection exactly those categories, whole; the confirmation is unchanged.

### 2. The places

Every place is data, `modules/app-leftovers/places.toml` (rule 5), embedded and checked when the module is built. A
row is one form of leftover; rows of the same `(folder, path)` share one interest:

| Row id         | Category                 | Known folder   | `path`           | Kind   | Name     | Proof | Optional |
| -------------- | ------------------------ | -------------- | ---------------- | ------ | -------- | ----- | -------- |
| `caches`       | `app-leftovers.caches`   | `app-caches`   | `*`              | folder | `<id>`                | —   | —   |
| `logs`         | `app-leftovers.caches`   | `logs`         | `*`              | folder | `<id>`                | —   | —   |
| `saved-state`  | `app-leftovers.caches`   | `saved-state`  | `*`              | folder | `<id>.savedState`     | yes | —   |
| `preferences`  | `app-leftovers.settings` | `preferences`  | `*`              | file   | `<id>.plist`          | —   | —   |
| `http-storage` | `app-leftovers.settings` | `library`      | `HTTPStorages/*` | folder | `<id>`                | —   | —   |
| `http-cookies` | `app-leftovers.settings` | `library`      | `HTTPStorages/*` | file   | `<id>.binarycookies`  | —   | —   |
| `web-data`     | `app-leftovers.settings` | `library`      | `WebKit/*`       | folder | `<id>`                | —   | —   |
| `cookies`      | `app-leftovers.settings` | `library`      | `Cookies/*`      | file   | `<id>.binarycookies`  | —   | —   |
| `support`      | `app-leftovers.data`     | `app-support`  | `*`              | folder | `<id>`                | —   | —   |
| `container`    | `app-leftovers.data`     | `containers`   | `*`              | folder | `<id>`                | yes | yes |

```toml
[[place]]
id = "saved-state"              # its title (`app-leftovers.title.saved-state`) and its findings' identity
category = "app-leftovers.caches"
folder = "saved-state"          # a known folder, as rules write it
path = "*"                      # plain names, then `*`: the `*` matches only a bundle-id-shaped name
kind = "folder"                 # "folder" or "file": a leftover of another kind here is not claimed
suffix = ".savedState"          # what follows the id in the name; none when the name is the id
proof = true                    # a leftover here proves the id was an app's
# optional = true               # the place may be left unread or not scanned without keeping every id (section 3)
```

- **Proof**: the system makes a container only for a sandboxed app or extension, and a saved state only for an app
  with windows; a command-line tool, a plugin or a framework also writes `Caches/<id>` and `Preferences/<id>.plist`
  and has no bundle to find, so an id without a proof is never offered.
- **`container` is optional**: the app does not open `Library/Containers` until plan 006 settles its permission
  (`places_that_prompt`), so the place is `NotScanned` on every Mac today. A sandboxed app keeps everything inside its
  container (its caches, settings and saved state included), so outside it a sandboxed app leaves almost nothing; an
  unseen container hides no recent leftover of another place. While Containers is not scanned, the proof is a saved
  state only, and the data category holds support folders only.
- The table is refused when: it is not valid TOML or has an unknown field; it has no row; a row id is not kebab case
  or is listed twice; a category is not one of the module's, or a module category has no row; the known folder is a
  walked place (`user-temp`, `shared-temp`) or `group-containers` (the user's decision 1); a `path` is not plain names
  followed by `*`; two rows of one `(folder, path)` have the same kind and suffix; a suffix does not start with `.` or
  is not a plain name's end; two rows of one `(folder, path)` disagree on `optional`; no row is a proof.
- Interests, in the table's order: one `Under(folder, path)` `Claim::Whole` `.only_bundle_ids()` per distinct
  `(folder, path)` (nine); a map from an interest index to its rows.
- **The id of a leftover**: its name without the row's suffix (compared ignoring ASCII case, as the file system does),
  then `BundleId::new` with at least one dot (`src/name.rs`); a name that gives no id (`Cookies.binarycookies` gives
  `Cookies`) is not claimed.

The exclusion table, `modules/app-leftovers/exclude.toml`: ids or id prefixes never claimed, each with why:

```toml
[[exclude]]
prefix = "com.apple."           # Apple's own apps and the system's: never offered
why = "Apple's own apps and the system's parts"
```

Refused when not valid TOML, empty, a `prefix` that is not an id shape ending with `.` or an `id` that is not a
bundle id, an entry listed twice, or without `com.apple.` (so it cannot be dropped by mistake). Other entries (the ids
shared frameworks write for every app) are added in 04/02 from what the run on the developer's Mac shows (to verify);
an entry only ever keeps.

**Categories and dangers are code** (`src/categories.rs`), as in `dev-caches`; the table names a category by id.

### 3. The decision

The leftovers of an id lie in several places, and matches come in no order: every match is gathered for the scan
(keyed by its cancel token, as `jetbrains`' `Gathered`) and decided in `finish`, grouped by id, ids sorted.

A match becomes a leftover when its kind is the row's (a link, an `Other` entry or the wrong kind is not claimed) and
its name gives an id. Then, per id, the first row that holds:

| #   | Situation                                                                                                     | Result                                           |
| --- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| 1   | the id is excluded (`exclude.toml`)                                                                           | not claimed                                      |
| 2   | no leftover of the id is a folder of a proof row                                                              | not claimed                                      |
| 3   | the installed apps cannot be listed at all                                                                    | every leftover left out `no-system`              |
| 4   | an installed app (of the list, partial or not) owns the id (`InstalledApp::owns`), or its id or a helper's is the id followed by a dot and more, or the other way round | not claimed |
| 5   | the list of installed apps is partial (`installed_apps_complete` fails)                                       | left out `apps-unread`                           |
| 6   | the running apps cannot be listed                                                                             | left out `no-system`                             |
| 7   | a running app has the id, or one related by a dot as in row 4                                                | left out `running`                               |
| 8   | a restored scan; a place of the module's rows that is not optional not seen in full (`!place_seen.is_complete()`); a leftover folder read in part (`inside` `None` or not `complete`) | left out `scan-again` |
| 9   | a leftover changed within the setting's days: a folder's `inside.newest_change`, a file's newest of `modified` and `changed`; unknown or in the future counts as recent | left out `used-recently` |
| 10  | Spotlight cannot see every app (`find_apps` fails: the index is off, a volume is not indexed)                | left out `spotlight-unread`                      |
| 11  | Spotlight finds a bundle with the id                                                                          | left out `installed-elsewhere`                   |
| 12  | otherwise: each leftover on its own, rows 13 to 15                                                            |                                                  |
| 13  | a socket, a pipe or a device inside it (`inside.special`)                                                     | that leftover left out `special`                 |
| 14  | not the user's, a system flag, restricted or the user's lock on it, or such an entry inside (`inside.foreign`) | that leftover left out `not-yours`              |
| 15  | otherwise                                                                                                     | **offered** in its row's category                |

- Rows 1, 2 and 4 are not leftovers (rule 1): nothing is said. A left-out item is reported in its row's category with
  its sizes; rows 3 to 11 report every leftover of the id.
- Spotlight is asked last, once per id that reached row 10, with `cx.cancel()`; `finish` checks the token between ids
  and returns `ModuleError::Cancelled`.
- What stays uncertain, and the answer to each: an app on a drive that is not connected cannot be found by any means
  (its leftovers waited the setting's days unchanged, the finding says "If this app is on a drive that is not
  connected now, keep these", the user confirms each, and they go to the Trash); an app whose id changed between
  versions leaves the old id's data, a true leftover; an iOS app on Apple silicon or a web app wrapper that the listing
  cannot read makes the list partial (row 5).
- At the act: an item a process holds open is skipped by the cleaner's in-use check; a container the system refuses
  to move is skipped with its reason ([`deletion-safety.md`](../../restrictions/deletion-safety.md)).

A finding:

| Part     | Value                                                                                                   |
| -------- | ------------------------------------------------------------------------------------------------------- |
| category | the row's, at its danger                                                                                |
| identity | `<row id>/<name>` (`caches/com.example.App`, `preferences/com.example.App.plist`)                       |
| subject  | `Subject::Path`; removal `Removal::MoveToTrash` of the same path                                        |
| sizes    | the match's                                                                                             |
| title    | `app-leftovers.title.<row id>` with `{id}`                                                              |
| reason   | `app-leftovers.finding.reason` with `{days}`                                                            |
| app      | `for_app(id)`: the map counts it in a line named by the id, "no longer installed"                       |

### 4. Categories, dangers and texts

| Category                 | Danger | Name                         |
| ------------------------ | ------ | ---------------------------- |
| `app-leftovers.caches`   | Safe   | Caches of removed apps       |
| `app-leftovers.settings` | Medium | Settings of removed apps     |
| `app-leftovers.data`     | High   | Data of removed apps         |

High for support folders and containers (the user's decision 2): a sandboxed app keeps its documents and databases
there, and nothing tells a cache from a library of notes (rule 1: when in doubt, higher).

The five texts of each category (rule 7), in `modules/app-leftovers/locales/en.json`:

- **caches** — what: "Caches, logs and saved window positions of apps that are no longer on this Mac." made-by: "The
  apps themselves, while they were installed. Each item names its app by its id, such as com.example.App."
  if-removed: "They go to the Trash. Nothing you made is in them." comes-back: "Only if you install the app again: it
  makes them anew."
- **settings** — what: "The settings, cookies and website data of apps that are no longer on this Mac." made-by: as
  caches'. if-removed: "They go to the Trash. If you install the app again, it starts with its default settings and
  you may need to sign in again." comes-back: "No. A reinstalled app starts with its default settings."
- **data** — what: "The support folders and containers of apps that are no longer on this Mac. An app may have kept
  documents, databases or downloads here." made-by: as caches'. if-removed: "They go to the Trash. Anything the app
  kept only here is lost once you empty the Trash, so look inside first if you are not sure." comes-back: "No."

The finding texts: `app-leftovers.title.caches` "{id} caches", `.logs` "{id} logs", `.saved-state` "{id} saved
windows", `.preferences` "{id} settings", `.http-storage` "{id} web data", `.http-cookies` "{id} cookies",
`.web-data` "{id} website data", `.cookies` "{id} cookies", `.support` "{id} support files", `.container` "{id}
container"; `app-leftovers.finding.reason` "No app with this id is installed on this Mac, and none of its files has
changed in the last {days} days. If this app is on a drive that is not connected now, keep these."

The switch: `app-leftovers.enabled.label` "Removed apps' leftovers", `.description` "Finds the files that apps you
removed left behind: caches, settings and data, matched by the app's id." The setting
`app-leftovers.uninstalled-after-days` (7 to 3650, 30): label "Offer a removed app's files after", description "The
files of an app that is no longer installed are offered only once none of them has changed for this many days, at
least 7."

The choices (rule 7; the idea's texts, made plural since a choice covers every removed app of the page):

| Choice          | Categories                     | Label             | Description                                                                                     |
| --------------- | ------------------------------ | ----------------- | ----------------------------------------------------------------------------------------------- |
| `everything`    | caches, settings, data         | Remove everything | Deletes all files the removed apps left behind. If you install one of them again, it starts fresh. |
| `keep-settings` | caches                         | Keep settings     | Deletes only caches and logs. Your settings and data stay, in case you install an app again.    |

The left-out reasons, declared with `with_problems` in this order:

| Key                                       | Text                                                                                                                  |
| ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `app-leftovers.left-out.no-system`        | Fresher could not list your apps, so it cannot tell whether this app is gone. It keeps these files. Scan again.      |
| `app-leftovers.left-out.apps-unread`      | Fresher could not read every app in your apps folders, so it cannot be sure this app is gone. It keeps these files. |
| `app-leftovers.left-out.running`          | This app is open, though it is not in your apps folders. Its files are kept.                                         |
| `app-leftovers.left-out.scan-again`       | Fresher could not look at every place this app may keep files in, so it keeps these. Scan again to check.           |
| `app-leftovers.left-out.used-recently`    | These files changed within the days set in Settings, so the app may have been removed only lately. A scan after those days offers them. |
| `app-leftovers.left-out.spotlight-unread` | Fresher could not ask Spotlight about every drive, so this app may be installed elsewhere. If Spotlight is off, or a drive is not indexed, turn it on and scan again. |
| `app-leftovers.left-out.installed-elsewhere` | An app with this id is on this Mac outside your apps folders, so its files are kept.                              |
| `app-leftovers.left-out.special`          | It holds a connection a running program may be using, so it is kept.                                                 |
| `app-leftovers.left-out.not-yours`        | Some of it belongs to the system or another user, or is locked. Removing it needs an administrator.                 |

### 5. The module

`modules/app-leftovers` (`fresher-module-app-leftovers`), id `app-leftovers`, group **Apps**, registered in every build
after `dev-caches`. Code with data tables, not rules: rules cannot report left-outs
([deferred](../../deferred/match-explanations.md)) and the answer depends on every leftover of the id.

| File               | What                                                                                                  |
| ------------------ | ----------------------------------------------------------------------------------------------------- |
| `places.toml`      | the places of section 2                                                                               |
| `exclude.toml`     | the exclusions of section 2                                                                           |
| `src/lib.rs`       | `AppLeftoversModule`, `AppLeftoversError`, the descriptor, `interests`, `on_match`, `finish`          |
| `src/categories.rs`| the three categories and their dangers                                                                |
| `src/places.rs`    | `Places::shipped()`, `Places::parse(text, categories)`, `PlacesError`, the interests and the row map   |
| `src/exclude.rs`   | `Exclude::shipped()`, `Exclude::parse(text)`, `ExcludeError`, `Exclude::covers(&BundleId)`            |
| `src/name.rs`      | `app_id(name, suffix) -> Option<BundleId>`, `related(a, b) -> bool` (the same, or one is the other followed by a dot) |
| `src/settings.rs`  | `uninstalled-after-days`, as `jetbrains`'                                                             |
| `src/gathered.rs`  | the leftovers of one scan, keyed by its cancel token                                                  |
| `src/decide.rs`    | `decide_id(…) -> IdVerdict` (rows 1 to 11) and `decide_item(…) -> ItemVerdict` (rows 13 to 15), on plain values |
| `locales/en.json`  | every text of section 4                                                                               |

Dependencies: `fresher-core`, `serde`, `thiserror`, `toml`; dev-dependencies added by the step that first uses each.

### 6. Tests

**Unit** (`modules/app-leftovers/src/*`, negatives first): `places.rs` — every refusal by a table made for it, then
the shipped table (ten rows, three categories, nine interests, two proof rows, one optional); `exclude.rs` — every
refusal, then `com.apple.Safari` and `com.apple.` covered, `com.applex.App` and `com.example.App` not; `name.rs` —
`Notes`, `.plist`, `Cookies.binarycookies`, `com.example.App.plistx`, a name that is not ASCII refused, then
`com.example.App`, `com.example.App.plist`, `COM.EXAMPLE.APP.PLIST`; `related` — `com.example.Appx` and
`com.example.App` not related, `com.example.App.Helper` and `com.example.App` related both ways; `decide.rs` — each
row's keep before its offer, the precedence (excluded over everything, no proof over the apps, installed over a
partial list, partial list over running, the scan over the age, the age over Spotlight), an optional place not
scanned that keeps nothing, a time in the future; properties: adding an installed or running related app never turns
a keep into an offer; adding a leftover changed within the days never does either.

**Fixture** (`modules/app-leftovers/tests/fixtures.rs`, `run_module`, `Catalogue::Required`); every root injected,
never the real home; **no two entries at one path in one home**: a negative that needs another entry at a place has
its own id or its own home. **Every entry of a leftover states its age and its status change** (`age` and `changed`,
its folder a `[[dir]]`, its files directly inside): the status change cannot be set back on disk, and a whole
folder's age is the newest over everything inside it, so one entry written at the test's real time keeps the id as
changed lately.

| Home                                | Kind     | What                                                                                       | Expected                          |
| ----------------------------------- | -------- | ------------------------------------------------------------------------------------------ | --------------------------------- |
| `app-leftovers-not-ids.toml`        | negative | `Application Support/Notes/`, `Caches/SomeTool/`, `Preferences/settings.plist`, `Cookies/Cookies.binarycookies`, `Logs/DiagnosticReports/`, `Group Containers/ABCDE12345.com.example.Shared/`, the same id's folders elsewhere in `~/Library` (`Library/Application Scripts/com.example.Gone/`) | nothing offered, nothing left out |
| `app-leftovers-apple.toml`          | negative | `com.apple.Safari` and `com.apple.Terminal` in every place, with saved states              | nothing offered, nothing left out |
| `app-leftovers-unproven.toml`       | negative | `org.example.tool`: caches, logs, preferences, support; no container, no saved state; `com.example.FileState`: a saved state that is a file | nothing offered, nothing left out |
| `app-leftovers-installed.toml`      | negative | installed `com.example.Editor` with helper `com.example.Editor.Helper`; leftovers of `com.example.Editor`, `com.example.Editor.Helper`, `com.example.Editor.Updater` and `com.example` (each with a saved state) | nothing offered, nothing left out |
| `app-leftovers-kept.toml`           | negative | one id per reason, each with a saved state: `com.example.Running` (running), `com.example.Recent` (changed 5 days ago), `com.example.Elsewhere` (Spotlight finds it), `com.example.Socket` (a socket inside its support folder), `com.example.Foreign` (an `owner = "system"` file inside its caches), `com.example.Linked` (its caches a link, its saved state 5 days old) | left out by reason with each item's count; the link not claimed (Linked's saved state alone in `used-recently`) |
| (the same, with each port failing)  | negative | the running apps failing; the installed list partial, then failing; Spotlight failing; a restored run; the logs place avoided (`Ports::avoid`) | `no-system`, `apps-unread`, `no-system`, `spotlight-unread`, `scan-again` ×2 for every leftover |
| `app-leftovers-leftovers.toml`      | positive | `com.example.Gone` in all ten rows' forms; `com.example.Appx` (caches and a saved state) beside installed `com.example.App`; `com.example.Boxed` with only a container and its caches; every file filled with data, dated 40 days back | every leftover offered in its category at its danger, its title and reason, `for_app` its id; golden `fixtures/golden/app-leftovers.json` |
| (the same, Containers avoided)      | positive | `Ports::avoid` holds `Library/Containers`                                                  | `com.example.Boxed` not claimed; the rest the same |

Plus: the matches handed in two orders give the same answers.

**Sandbox** (rule 8). `Library/Containers` is not opened in sandbox mode either (`places_that_prompt` of the
sandbox's home), so no case can offer a container; a case shows it is not scanned. Two case files:

`fixtures/sandboxes/app-leftovers.toml` (`feature = "app-leftovers"`, golden `sandbox-app-leftovers.json`), each case
with its own ids and paths, negatives first:

| Case (negative)                                        | Files and stated facts                                                               | Expected                                 |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------ | ---------------------------------------- |
| a name that is not an app's id is never claimed        | `home/Library/Application Support/Notes/notes.db`, `…/Caches/SomeTool/x`, `…/Preferences/settings.plist` | nothing offered; `[case.map]` `your-folders` at `…/Application Support/Notes` |
| Apple's own files are never claimed                    | `…/Caches/com.apple.Safari/x`, `…/Saved Application State/com.apple.Terminal.savedState/data.data` (40 days old) | nothing offered |
| a tool's files with no app behind them are not claimed | `…/Caches/org.example.tool/x`, `…/Preferences/org.example.tool.plist` (40 days old)  | nothing offered                          |
| an installed app's files are not claimed               | `[[case.app]]` `Applications/Editor.app` `com.example.Editor`, helper `com.example.Editor.Helper`; its caches, saved state, the helper's caches, `com.example.Editor.Updater`'s caches | nothing offered |
| an open app outside the apps folders keeps its files   | `[[case.running]]` `com.example.Portable`; its saved state and caches (40 days old)   | left out `running` 2 in caches           |
| files changed lately are kept                          | `com.example.Recent`: saved state and caches, 5 days old                              | left out `used-recently` 2 in caches     |
| an app Spotlight knows elsewhere keeps its files       | `[[case.elsewhere]]` `com.example.Elsewhere`; its saved state and preferences (40 days old) | left out `installed-elsewhere`: 1 caches, 1 settings |
| a link at a place is never claimed                     | `…/Caches/com.example.Linked` → `home/elsewhere/Linked/` (holding `x`); a saved state of the id, 5 days old | left out `used-recently` 1 in caches (the saved state alone: the link is no leftover); the link and its target unchanged after the cleanup |
| a leftover holding a socket is kept                    | `…/Saved Application State/com.example.Socket.savedState/{data.data, x.sock (kind = "socket")}` | left out `special` 1 in caches |
| a leftover holding the system's file is kept           | `…/Saved Application State/com.example.Root.savedState/{data.data, x (owner = "system")}` | left out `not-yours` 1 in caches |
| a group container is never claimed                     | `home/Library/Group Containers/ABCDE12345.com.example.Shared/x` and a saved state of `com.example.Shared`, 5 days old | left out `used-recently` 1 in caches (the saved state alone); the group container untouched |
| a container is not opened                              | `home/Library/Containers/com.example.Boxed/Data/x` (40 days old)                     | nothing offered; the container untouched |

Then the positives, every file `fill = "data"`, each with its own id; every entry of a leftover, positive or
negative, states `age` and `changed` (40 days, or 5 for a recent one), as the fixtures do:

| Case (positive)                                             | Files                                                                                     | Expect                                       |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------------- |
| a removed app's caches, logs and saved windows go           | `com.example.Gone`: `Caches/…`, `Logs/…`, `Saved Application State/….savedState/…`         | `app-leftovers.caches`, safe, `in-trash`; map `explained` |
| a removed app's settings go, at Medium                      | `com.example.Gone2`: `Preferences/com.example.Gone2.plist`, its saved state               | `app-leftovers.settings`, medium, `in-trash` |
| a removed app's web data and cookies go with its settings   | `com.example.Gone3`: `HTTPStorages/…/`, `HTTPStorages/….binarycookies`, `WebKit/…/`, `Cookies/….binarycookies`, its saved state | `app-leftovers.settings`, medium, `in-trash` |
| a removed app's support folder goes, at High                | `com.example.Gone4`: `Application Support/com.example.Gone4/…`, its saved state           | `app-leftovers.data`, high, `in-trash`; map in the id's line |

`fixtures/sandboxes/app-leftovers-spotlight-off.toml` (`feature = "app-leftovers-spotlight-off"`): Spotlight off
(`spotlight = "off"`, a world of its own): negative "nothing is offered while Spotlight cannot be asked" (a saved
state and caches of `com.example.Unsearched`, left out `spotlight-unread` 2 in caches). The sandbox world's part of
[the deferred item](../../deferred/sandbox-world.md) taken here: `[[case.elsewhere]]` (a name and a bundle id that
Spotlight answers, at `Volumes/Elsewhere/<name>.app`, which is never written or read) and `spotlight = "off"`, both in
`world.toml`.

**Core e2e** (`crates/e2e/tests/app_leftovers.rs`, harness `app_leftovers_home/mod.rs`, fixture
`fixtures/homes/app-leftovers-flow.toml`, golden `fixtures/golden/app-leftovers-flow.json`): first the deferred
shared home harness (`crates/e2e/tests/support/home.rs`, used by `temp_home`, `dev_caches_home` and this one); then
the flows on the real scanner, the facts lens, the real settings, cache and journal files and the real `DiskCleaner`,
with `FakeClock`, `FakeTrash`, `FakeRunningApps`, `FakeInstalledApps`, `FakeOpenFiles`, `FakeChangeJournal`:

1. **Scan → confirm → Trash → put back**: Apps → Removed apps' leftovers offers `com.example.Gone`'s ten leftovers;
   the three categories selected (as "Remove everything" does), the plan names High; in the Trash and the journal;
   put back restores them byte for byte; the installed app's and `com.apple.`'s files untouched.
2. **Keep settings**: only the caches category selected: the plan names Safe; a rescan (the journal scripted, the scan
   asserted `ScanKind::Rescan`) still offers the settings and the data.
3. **Installed again**: after a scan that offers, the app is installed (`FakeInstalledApps` changed); a rescan (asserted)
   offers nothing of it.
4. **Open from a disk image**: the app running: left out `running`; it quits; a rescan (asserted) offers it.
5. **Changed lately**: the leftovers 20 days old: left out `used-recently`; the clock 15 days on; a rescan (asserted,
   nothing changed on disk) offers them.
6. **Spotlight**: an app found elsewhere: `installed-elsewhere`; the search incomplete: `spotlight-unread`.
7. **A place not read**: the logs place made unreadable (mode 000, restored on drop, even on failure): every id left
   out `scan-again`; the other modules' answers unchanged.
8. **Containers avoided** (the app's `places_that_prompt`): a container-only id not claimed; a saved-state id offered.
9. **Restore at launch**: every id left out `scan-again`; a scan then offers them.
10. **The storage map**: the offered leftovers in a line named by the id, not installed.
11. **Ignore**: the data category ignored: its findings in no total, the others stay; restore brings it back.

**UI e2e** (Playwright, mocked core; `app/ui/src/test/app-leftovers-scan.ts`, the module's real texts; the mock shows
only states the core can produce together: an id is offered or left out, never both, and every left-out reason with a
leftover of its own id):

1. A scan shows **Apps → Removed apps' leftovers** in the sidebar with its size; its page lists the three categories
   with their dangers; a category shows its five texts.
2. The left-out lines ("This app is open…", "These files changed within the days…"), not selectable.
3. **Remove everything** selects the three categories; Review names High; **Keep settings** selects only the caches;
   Review names Safe; confirm to the Trash; result; Cleanups; put back.
4. Settings → Removed apps' leftovers: the switch and the days.
5. The overview's by-meaning view: a removed app's line named by its id, "no longer installed".

Visual and accessibility: the module's page with its choices and left-out lines, in `visual.spec.ts` and `axe.spec.ts`.

**By hand** (04/02): both sandboxes with `EXPECTED.md`; on the developer's Mac a scan only (a development build's
cleaner reaches only a sandbox): every offered id checked with `mdfind` and by hand to be no installed app; nothing
under `com.apple.` or `Group Containers` offered; how many ids the proof leaves out counted; the scan time within 5 %
of `main`'s; the scan bench retaken on a quiet Mac; then one real cleanup, by the developer with Finder, of a removed
test app's leftovers and its put-back.

### 7. Where the code lives

| Crate or folder                 | What it gains                                                                                        |
| ------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `crates/core`                   | `is_bundle_id_shaped`, `Interest::only_bundle_ids`, `PlaceSeen`, `WalkSummary.places`, `ModuleContext::place_seen`, `InstalledApps::search_complete`, `for_app_named`, `ModuleGroup::Apps`, `ModuleChoice` |
| `crates/scanner`                | the shaped trie segment; the places seen, from the tree after every walk                             |
| `crates/macos`                  | `mounted_volumes()`                                                                                  |
| `crates/system`                 | Spotlight's reach (`spotlight.rs`), `AppSearch::Declared`                                            |
| `crates/engine`                 | the places handed to `finish`; the removed app's name in the map                                     |
| `crates/testing`                | the shape and the places in `NaiveWalk` and `run_module`; `FakeInstalledApps::search_complete`        |
| `crates/sandbox`                | `[[case.elsewhere]]`, `spotlight = "off"` in the case files and `world.toml`                         |
| `modules/app-leftovers/` (new)  | `fresher-module-app-leftovers`                                                                       |
| `modules/jetbrains`             | `for_app_named`; its Spotlight text names a drive not indexed                                       |
| `crates/e2e`                    | the shared home harness; the app-leftovers flows                                                     |
| `app/tauri`, `app/ui`           | registration and the Spotlight runners; the case files' tests; the group Apps, the choices, the UI's test scan and scenarios |

No new third-party dependency.

## Interfaces with other plans

- **018** (merged): `Interest::holding` (not used here), the e2e helpers in `crates/e2e/tests/support/`, the shape of
  a module with data tables.
- **016** (merged): `Gathered`, the day setting, `find_apps`, the installed and running apps in the sandbox and the
  fakes; `jetbrains` gains the Spotlight check and the product's name in the map.
- **017** (merged): `Inside`, the stated facts and `FactsLens`, the `temp_home` harness that the shared harness serves.
- **015**: groups, Ignore. **012**: `for_app`, the storage map. **011**: left-outs. **009**: the system's facts.
  **008**: the guard, its in-use check, the Trash and put back.
- **006** (in progress): when it opens `Library/Containers`, containers become offerable with no change here.

## Deferred items

- A removed app's name in the storage map (a deferred item): due here; done in 01/04, its file removed.
- The module harnesses of the core e2e share their code (a deferred item): due at this harness; done in 03/02, its
  file removed.
- [The sandbox world](../../deferred/sandbox-world.md): partly taken in 03/01 (Spotlight's answers); open files,
  tools' answers, access and the steps between scans stay deferred.
- The known issue [Spotlight misses an excluded volume](../../known-issues/app-spotlight-misses-an-excluded-volume.md):
  closed in 01/03.
- Not due: [why an item matched](../../deferred/match-explanations.md) (code, not rules);
  [a module names its app's data folders](../../deferred/module-names-app-data.md) (these folders are named after the
  id already; the map's app lines change only by a removed app's name).

## Open questions

None.

## Decisions

The user's, of 2026-11-18 (translated): the draft approved with its sixteen agent's calls, and these answers:

1. **Group containers: excluded from this plan** (the recommended option); they may be added later.
2. **An app's data and container (`Application Support/<id>`, `Containers/<id>`): offered in this plan, as High, in
   their own category** (the recommended option).
3. "Approve, write the design and start the implementation."
4. **The "Remove everything / Keep settings" choice is per module for now** (the user's decision of 2026-11-18, the
   recommended option, confirming agent's call 2): it selects whole categories of the module; a choice per app stays an
   idea ([uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md)).

The draft's agent's calls became these decisions:

1. **Bundle id only, never a name**: a folder not named after an id is not matched (rule 1).
2. **An id must be proven an app's** (a container or a saved state): tools and frameworks write id-named caches too.
3. **Three categories by kind: Safe caches, Medium settings, High data**: "keep settings" is a choice of categories.
4. **"Not installed" needs every check to answer**: a full app list, no owner by id or helper, not running, not in
   Spotlight, every volume indexed; any unknown keeps everything.
5. **Unchanged for 30 days, a setting**, as `jetbrains`: a just-removed app may be installed again.
6. **`com.apple.*` and an exclusion table never offered.**
7. **Group containers and launch agents left out**: shared or loaded; each a later addition.
8. **Per-place completeness in the core**: the global check would block the module on every Mac without Full Disk
   Access.
9. **Id-shaped interests in the core**: a `*` claim must not swallow other modules' places.
10. **The volume index check closes the known issue for both modules.**
11. **`for_app_named` done here, for `jetbrains`**: this module shows the id, no guessed name.
12. **"Keep settings" as a selection shortcut**: the confirmation is unchanged (made module-wide: agent's call 2, the user's decision 4).
13. **One finding per app and category** (corrected to one per leftover, agent's call 1).
14. **The Trash only**: no app's or the system's own removal is run.
15. **Module id `app-leftovers`, a new group Apps.**
16. **The e2e home harness shared in plan 03**: its deferred item is due at this harness.
17. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code; each with what it costs if wrong.

1. **One finding per leftover, not per app and category** (draft call 13 corrected): a finding has one path
   (`Subject::Path`, `MoveToTrash`), as the cleaner, the guard and the journal take it; each leftover is titled with its
   id and kind. Cost if wrong: a finding of several paths through the core, the cleaner and the journal.
2. **The choices select whole categories of the module, not one app's leftovers** (the draft said "per app";
   confirmed by the user on 2026-11-18, decision 4): the
   selection holds whole categories or single findings, and an app's findings across categories would need a query by
   app and an app list on the page. The texts are made plural. Cost if wrong: a findings query by app and a per-app
   control.
3. **An installed app's files and an unproven id's are not claimed, silently** (the draft listed every row as a
   left-out): they are not leftovers, and reporting them would list every installed app's data as "left out". Cost if
   wrong: two more left-out reasons and much noise.
4. **The container place is optional**: the app does not open `Library/Containers` until plan 006, so a required
   place would keep everything on every Mac; a sandboxed app leaves almost nothing outside its container. Until plan
   006 the proof is a saved state and the data category holds support folders only; the sandbox shows a container is
   not opened. Cost if wrong: the module offers nothing until plan 006.
5. **What the walk saw of a place is read from the tree after the walk**, not counted in it: no hot-path cost, no cache
   format change, the same answer after a rescan and a restore; `Partial` is not used (it climbs from any unreadable
   folder below). Cost if wrong: a count in the worker and the codec.
6. **Claimed whole, each id-named folder of these places is one entry in the map's browse view**, for installed apps
   too (its size, its app in the by-meaning view stay), as every whole claim is today. A later module whose place lies
   inside an app's id-named folder (the Docker disk image in its container) must make this claim stop there. Cost if
   wrong: a claim that keeps the nodes inside, in the scanner.
7. **The index check sits behind `find_apps`, once per scan** (`InstalledApps::search_complete`), with its own 10 s
   runner; a network volume counts as not indexed without asking; a mounted disk image that is not indexed keeps
   everything while mounted. The answer `Indexing enabled.` is to verify. Cost if wrong: one text to match.
8. **The shape is checked on the whole name** (`com.example.App.plist` is shaped), and the module strips the suffix and
   needs a dot in the id. Cost if wrong: none for safety; a few names matched whole and not claimed.
9. **`com.apple.` is a row of `exclude.toml`, required by its check** (the draft had it in code): rule 5, and it cannot
   be dropped by mistake. Cost if wrong: none.
10. **A related id** (one is the other followed by a dot and more) **keeps both ways**, for installed and running apps:
    an app's extensions and updaters (`com.example.App.Updater`). Cost if wrong: a few leftovers of a removed
    extension kept while its app is installed.
11. **A left-out per item for a socket or a foreign entry, per id for the rest**: an item that cannot be removed does
    not keep the id's other leftovers. Cost if wrong: an id kept whole.
12. **The Apps group after System, before Development**, as the screen's order. Cost if wrong: one line.
13. **`jetbrains`' Spotlight left-out text names a drive not indexed**: its reason now covers the volume check. Cost
    if wrong: a text.
14. **Two sandbox case files**: Spotlight off is a world of its own; containers cannot be offered in a sandbox (the
    place is avoided), so their positive is the fixtures' and the core e2e's. Cost if wrong: none.

## Conclusions

Not written yet.
