# 024 — Unused toolchain versions: design

**Status:** ✅ approved (the user's delegation of 2026-11-20) · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** core, testing, sandbox, modules, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Unused toolchain versions](../../ideas/unused-toolchain-versions.md) and from a
read of the code on plan 023's head (b8180b53), read-only. Approved by the user's delegation of 2026-11-20: after plans
027, 028 and 023, build plan 024 autonomously, take the recommended decision on every question and record each as an
agent's call; the user checks everything together later. Where the code or the rules forced a different choice than
the idea, the design says so and the choice is an agent's call. Facts marked "to verify" are checked by hand in 04/02.

## Intent

Version managers install a new version next to the old one and never remove anything: a Rust toolchain for every
nightly tried and every project that pinned one, a Node version for every project from years ago. Each is hundreds of
megabytes. The idea asks for one finding per version that **nothing on the Mac points at** ("not used", not "old"):
not the manager's default, not an alias or an override, not a project's pin file. The user added on 2026-11-19 that
Node versions installed by nvm must never be cleaned by default: the category starts off.

## Goals

1. A module `unused-toolchains` (`modules/unused-toolchains`, crate `fresher-module-unused-toolchains`, group
   Development) that offers, one finding per version, the **rustup toolchains** (`~/.rustup/toolchains/<name>`) and
   the **nvm Node versions** (`~/.nvm/versions/node/v<x.y.z>`) that nothing it can see points at.
2. "Points at" means: the manager's default and its other pointers (rustup's overrides, nvm's aliases), and every pin
   file in the scanned folders (`rust-toolchain.toml`, `rust-toolchain`, `.nvmrc`, `.node-version`, `.tool-versions`),
   except a pin inside a project's artifact folder (a dependency's own file in `node_modules`).
3. Never offered: a manager's default, rustup's moving channels (`stable`, `beta`, `nightly` without a date), the
   newest installed Node version, a link, a folder off the manager's name shape. When the module cannot know what is
   pointed at (a default or a pin it cannot read, a folder the walk could not read, a guarded folder not entered), it
   offers nothing of that manager and says why.
4. Node versions are found only when the user turns on the module's switch **Node versions (nvm)**, off by default.
5. Removal moves the version's own folder to the Trash, through the cleaner's existing checks; no tool command.

## Non-goals

- **Python (pyenv), Java (SDKMAN, `~/Library/Java/JavaVirtualMachines`, Gradle's JDKs), Android SDK parts and AVDs,
  fnm, volta, asdf's and mise's own installs** (agent's call 1): each needs a fact this plan does not build (a link's
  target, a virtualenv's base interpreter anywhere on the Mac, a build file's toolchain block, a package file read in
  every dependency, administrator rights). Postponed with what each needs:
  [`../../deferred/other-toolchain-versions.md`](../../deferred/other-toolchain-versions.md).
- The managers' commands (`rustup toolchain uninstall`, `nvm uninstall`): the folder goes to the Trash (agent's call
  4). Both commands remove exactly that folder; neither keeps an index that lists it.
- `RUSTUP_HOME`, `NVM_DIR`, `RUSTUP_TOOLCHAIN`, a shell profile's `nvm use`, an editor's settings: only the default
  places and the files named above, as plan 018 decided for every tool and plan 023 kept (agent's call 5). A pointer
  the module cannot see is the reason the levels are never Safe and the texts say how the version comes back.
- An age threshold: a version is offered because nothing points at it, whatever its age (agent's call 9).
- Rust components and targets inside a toolchain, and global npm packages inside a Node version, as findings of their
  own.

## What already exists

| Need                                   | What exists                                                                                                    | Plan     |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------- |
| A place under the home                 | `Pattern::Under(KnownFolder::Home, ".rustup/toolchains/*")`, `*` one name                                       | 002, 016 |
| A folder summed whole                  | `Claim::Whole`: `Match.sizes`, `Match.inside` (`newest_change`, `newest_file_access`, `special`, `foreign`, `complete`) | 004, 017 |
| A named file in a folder               | `Interest::holding(names)` → `Match.holds` (any kind; only for `Pattern::Under`)                               | 018      |
| A folder holding a named file anywhere | `Pattern::MarkerFile(name)` with `Claim::Item`: the folder is reported, the walk goes on                       | 002, 009 |
| Projects and their artifact folders    | `cx.system().projects()` in `finish`: `Project { root, kinds, artifacts: Vec<Artifact { path, sizes, last_change }>, caches, last_activity }` | 009, 021 |
| A walk that read every folder          | `cx.is_walk_complete()`: `false` when a folder could not be read; a place not entered on purpose does not count | 011, 016 |
| A category that needs a guarded folder | `CategoryDescriptor::with_needs(vec![Need::required(Capability::Folder(..))])`: the module is not run without it, and the UI says what it lacks | 006 |
| A module's own switch                  | a `SettingDescriptor` of kind `Switch` with its default; `ModuleSettings::switch(name)`                        | 005      |
| Left-outs                              | `out.left_out(LeftOutItem { category, reason, sizes })`, `with_problems(keys)`                                 | 011      |
| Removal                                | `Removal::MoveToTrash(path)`; the cleaner's guard (open files inside skip the folder)                          | 008, 009 |
| Reading the first bytes of a file      | `FolderReader::head(folder, name, limit)`: a regular file only, never follows a link; the walk uses it for `CACHEDIR.TAG` only | 004, 018 |
| A data table in a module               | `modules/local-ai-models/stores.toml`, parsed and checked when the module is built                              | 018, 023 |
| A sandbox with denied access           | `access = { documents = "denied" }` in a case (`downloads-denied.toml`)                                        | 020      |

**What is missing** (built first, plan 01):

1. **A module cannot read a file.** rustup's default lives inside `~/.rustup/settings.toml`, nvm's inside
   `~/.nvm/alias/default`, a project's pin inside its `.nvmrc`. `Match` carries names and sizes, never contents, and
   `ModuleContext` has no file access. Every toolchain in the idea needs this; none can be done without it.
2. **A fixture tree cannot hold a file's text.** `[[file]]` has a size and a fill (`sparse`, `data`, `cachedir-tag`),
   no content: no fixture home and no case file can state a pin.
3. **A case file cannot set a module's setting.** The Node switch is off by default; a case file whose Node cases are
   offered needs it on.

## Design

### 1. The small-file reader (plan 01/01)

```rust
// crates/core/src/system/small_file.rs

/// The most bytes a module may read of one file.
pub const SMALL_FILE_LIMIT: usize = 4096;

/// What reading one small text file found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SmallFile {
    /// The whole file, as text.
    Text(String),
    /// Nothing of that name is there any more.
    Missing,
    /// Something of that name is there but is not a regular file (a link, a folder, a socket): never followed.
    NotAFile,
    /// Longer than `SMALL_FILE_LIMIT`.
    TooLong,
    /// Not UTF-8.
    NotText,
    /// It could not be read (permissions, privacy protection, a broken disk).
    Unreadable,
}

/// Reads the first bytes of one file: the port behind `SystemFacts::small_file`.
pub trait SmallFiles: Send + Sync {
    /// The first `limit` bytes of the regular file `name` in `folder`; never follows a link.
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError>;
}

impl SystemFacts {
    /// The file `name` in `folder`, read whole when it is a regular text file of at most `SMALL_FILE_LIMIT` bytes.
    /// Read anew at every call; nothing is kept, logged or written. A module reads only a file its own matches showed:
    /// a name its anchored interest holds, the marker of its marker match, or a file it matched.
    pub fn small_file(&self, folder: &Path, name: &str) -> SmallFile;
}
```

- `SystemPorts` gains `files: Arc<dyn SmallFiles>`. The app gives it a `ReaderFiles(Arc<dyn FolderReader>)` adapter
  over the reader the scanner already uses (`MacFolderReader`, or the sandbox's stated lens over it), so the read goes
  through the same no-follow, regular-file-only call the walk uses for `CACHEDIR.TAG`. `small_file` asks for
  `SMALL_FILE_LIMIT + 1` bytes: more than the limit is `TooLong`. `ReadError::Vanished` is `Missing`, `Replaced` (a
  link or another kind where a file was asked) is `NotAFile`, the rest `Unreadable`.
- `fresher-testing` gives `DiskFiles` (the standard library: `symlink_metadata` first, a regular file only, then a
  bounded read) for `fake_system` and `run_module`, and `NoFiles` (every read `Unreadable`) for the hand-built ports of
  the other tests. Every `SystemPorts { .. }` literal gains one line (about twenty sites, mechanical).
- **Why a port and not the walk** (agent's call 2): the walk would have to keep the bytes in the tree and the scan
  cache (the codec, the rescan's patch, the reference walk), and a rescan copies an unchanged folder's values from the
  cache, so a pin edited in place (`echo 20 > .nvmrc` moves no folder time) would be read stale. A read in `finish`
  is always the disk's, costs a handful of small reads per scan, and changes no hot path: the walk, the scanner and
  the scan cache are untouched, so no scan bench is due ([`../../testing.md`](../../testing.md)).
- **Rule of use** (agent's call 3): a module reads only a file its own matches showed. It is a rule of review and of
  [`../../architecture/modules.md`](../../architecture/modules.md), not a check of the engine: `SystemFacts` does not
  know a module's matches, and building that bookkeeping for one module is more surface than the risk (a module is
  our own code; the read is bounded, read-only, never follows a link, and never logged).

### 2. File text and settings in the test worlds (plan 01/02)

- A fixture tree's `[[file]]` (and so a case file's `[[case.file]]`, which is read as a `TreeDescription`) gains
  `text = "..."`: the file's bytes as UTF-8, at most 4 KiB, its size taken from them; refused with `size`, `fill` or
  `kind`. The reference walk and the sandbox writer write it as is.
- A case file gains a file-level `[settings]` table (`"unused-toolchains.node-versions" = true`): the values the
  sandbox writes into its data folder's settings file before the app starts, and the sandbox autotest resolves the
  engine's settings from. A key the registered modules' schema does not know, or a value of the wrong kind, fails the
  autotest. In `npm run sandbox -- all`, two files that set one key to different values are refused (the same rule as
  a path written twice). Like `[[case.running]]`, the settings are shared by every case of a file.

### 3. The managers (`toolchains.toml`)

One row per manager, plain names relative to the home (rule 5); the formats of the files are read by code per manager
(section 5), the places and names by the table.

| Row      | Versions (under `~`, claimed whole) | Shape             | Pointers of the manager (under `~`)                              | Pin files (anywhere scanned)                               | Category                  | Switch          |
| -------- | ----------------------------------- | ----------------- | ---------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------- | --------------- |
| `rustup` | `.rustup/toolchains/*`              | `rustup-toolchain`| `.rustup` holding `settings.toml`                                | `rust-toolchain.toml`, `rust-toolchain`, `.tool-versions` (`rust`) | `unused-toolchains.rust` | —               |
| `nvm`    | `.nvm/versions/node/*`              | `node-version`    | every file of `.nvm/alias/*` and `.nvm/alias/lts/*`              | `.nvmrc`, `.node-version`, `.tool-versions` (`nodejs`, `node`) | `unused-toolchains.node` | `node-versions` |

The table is read with `deny_unknown_fields` and refused, one error each, for: no row; an id twice or not kebab case;
a category that is not the module's or a category no row uses; a place that is not a plain relative path or does not
end in `*`; a pointer place that is not plain; a pin name that is not one name; a `.tool-versions` key list empty or
with a key twice; a switch that is not one of the module's settings.

### 4. Interests

In the table's order, per row: the versions `Under(Home, <versions>)` `Claim::Whole`; the pointers (`Under(Home,
".rustup")` `Claim::Item` `.holding(["settings.toml"])`; `Under(Home, ".nvm/alias/*")` and `Under(Home,
".nvm/alias/lts/*")` `Claim::Item`); then one `MarkerFile(<name>)` `Claim::Item` per pin name of every row, each name
once (`.tool-versions` is shared). An `Item` claim makes nothing opaque: the walk goes on in a pin's folder and the
storage map still shows it. The versions' whole claim makes a version folder a summed entry in the storage map,
switch on or off (agent's call 10).

### 5. What points at a version (`pointers/`, pure parsers)

Every pointer is read as a **reference**, and every reference resolves to one of: the set of version folders it keeps,
`Nothing` (it points at no folder of the manager: `system`, a rustup `path` toolchain) or `Unknown` (the module cannot
tell). A reference never removes anything from the kept set; `Unknown` blocks offers (section 6, rows 10 and 11).

**rustup.**

- A **toolchain folder name** is `<channel>[-<YYYY-MM-DD>]-<triple>`: channel `stable`, `beta`, `nightly`, `X.Y` or
  `X.Y.Z` (digits); the date four, two and two digits; the triple three or four non-empty dash-separated parts of
  `[a-z0-9_.]`. Any other name (a custom name, `1.83.0-beta.3`, `.DS_Store`) is off shape. A **moving** toolchain is
  a named channel without a date (`stable-aarch64-apple-darwin`): never offered (agent's call 7).
- A **reference** is the same grammar with the triple optional. It keeps a folder when the channels are equal (or the
  reference is `X.Y` and the folder `X.Y.Z`: a conservative reading of rustup's own resolution), the dates are equal
  (both absent or the same) and the reference's triple is absent or equal. A reference equal to a folder's whole name
  keeps it whatever its shape; any other reference that does not parse is `Unknown`.
- **`settings.toml`** (TOML, unknown keys ignored): `default_toolchain` is the default reference; each value of
  `[overrides]` is a reference (the override's folder is not checked: an override for a deleted folder still keeps its
  toolchain). Missing, unreadable, not TOML, or without `default_toolchain`, or a default that keeps no installed
  folder: the default is `Unknown`.
- **`rust-toolchain.toml`**: `[toolchain] channel` is a reference; `[toolchain] path` alone is `Nothing`; neither is
  `Unknown`. **`rust-toolchain`**: read as `rust-toolchain.toml` when it is TOML with a `toolchain` table, else its
  one trimmed line without spaces is a reference, else `Unknown`.

**nvm.**

- A **version folder name** is `v<major>.<minor>.<patch>` (digits); any other is off shape. The **newest** installed
  version is never offered (agent's call 7).
- A **reference** (an alias file's or a pin's trimmed first line) resolves by, in order: `system` → `Nothing`; `node`
  or `stable` → the newest installed; `v?X`, `v?X.Y`, `v?X.Y.Z` → every installed version with that prefix (nvm takes
  the newest of them; keeping all is conservative); `lts/<name>` or `lts/*` → the alias file `alias/lts/<name>`'s
  reference; any other name with an alias file `alias/<name>` → its reference; otherwise `Unknown`. A chain longer
  than eight or a loop is `Unknown`.
- The **default** is `alias/default`'s reference; no `alias/default` is "no default" (`Nothing`): new shells then use
  no nvm version, and the newest is kept anyway. **Every other alias file** is a reference too (a user's alias names a
  version on purpose; nvm's own `lts/*` aliases name the newest of each line it knows).
- An nvm default or prefix that matches no installed version keeps nothing; it is not `Unknown`, unlike the rustup
  default rule. The newest installed Node version stays protected by call 7.

**`.tool-versions`** (both managers): each line without its `#` comment is `<tool> <version>…`; for a key of the row
(`rust`; `nodejs`, `node`) each version is a reference of that manager; `system`, `ref:…` and `path:…` are `Nothing`.

**Which pins count.** Every pin file a marker match showed, read with `small_file`, except one whose folder is inside
an artifact folder of a project of `cx.system().projects()` (`node_modules/<dep>/.nvmrc` is a dependency's file; nvm
reads the pin of the folder the user works in and its parents, never one inside `node_modules`). A pin read as
`Missing` (gone since the walk) is not counted; `NotAFile` (a link named `.nvmrc`), `TooLong`, `NotText` and
`Unreadable` are `Unknown` (agent's call 8).

### 6. The decision (`decide.rs`, a pure function)

For one version folder of one row, with the row's resolved pointers, the first row that applies:

| #   | When                                                                                                  | Verdict                    |
| --- | ----------------------------------------------------------------------------------------------------- | -------------------------- |
| 1   | not a folder (a link: `rustup toolchain link`, a file)                                                | not claimed                |
| 2   | the name is off the row's shape                                                                       | not claimed                |
| 3   | the row has a switch and it is off                                                                    | not claimed                |
| 4   | a moving rustup channel, or the newest installed Node version                                         | not claimed                |
| 5   | not read in full (`inside` missing or not `complete`), or no time at all                              | not claimed                |
| 6   | a known reference keeps it (the default, an override, an alias, a pin)                                | not claimed                |
| 7   | a socket, a pipe or a device inside                                                                   | left out `special`         |
| 8   | not the user's, a system flag, restricted, the user's lock, or such an entry inside                   | left out `not-yours`       |
| 9   | the walk did not read every folder (`!cx.is_walk_complete()`)                                         | left out `partial-scan`    |
| 10  | the default is `Unknown`                                                                              | left out `default-unknown` |
| 11  | another pointer or a pin is `Unknown`                                                                 | left out `pin-unknown`     |
| 12  | otherwise                                                                                             | offered                    |

- Before any row: `cx.system().projects()` is `None` (never in `finish`) → nothing is claimed.
- As built, `decide(candidate, row, version, newest, pointers, scan)` takes the version as `Version { index, name }`
  (its index among the row's installed versions, as `Pointers` names them, and its parsed name; `None` for a name off
  the shape). A row whose switch is off (`settings::is_on(settings, switch)`) is decided with empty pointers: rule 3
  returns not claimed and no file of that row is read. A version folder that is a link counts as installed when the
  newest Node version is found (nvm counts it so), though rule 1 never claims it.
- A version that something keeps (row 6) is not reported at all: it is in use, not junk; only what would be offered
  but for a blocking condition is left out (rows 7 to 11), with its size.
- The verdict does not depend on time or on the order of matches. A restored scan decides from the cached matches and
  the files as they are now (agent's call 11).
- The guarded folders: both categories declare `Need::required` on `Folder(Desktop)`, `Folder(Documents)` and
  `Folder(Downloads)` (agent's call 6). Without Full Disk Access or the three switches, the engine does not run the
  module and the UI says what it lacks; a pin in `~/Documents/dev` is never missed for want of access.

### 7. Findings

- Identity `<row id>/<folder name>`; subject and removal `Subject::Path` and `Removal::MoveToTrash` of the version
  folder; sizes the match's (`logical`, `allocated`, `reclaimable`, with their completeness).
- Title `unused-toolchains.title.<row>`: rust "Rust {name}" (`name` the folder's name), node "Node {name}" (`v18.17.0`).
- Reason `unused-toolchains.reason.<row>` with `date` (the newer of `inside.newest_change` and
  `inside.newest_file_access`), `size` (logical) and `command`: `rustup toolchain install <folder name>` or
  `nvm install <x.y.z>` (`TextArg::Text`).
- Categories:

| Category                 | Danger | Why                                                                                                              |
| ------------------------ | ------ | ---------------------------------------------------------------------------------------------------------------- |
| `unused-toolchains.rust` | Low    | one command and a download bring it back; nothing of the user's lives in a toolchain                             |
| `unused-toolchains.node` | Medium | global packages installed with the version go with it, and an editor set to its exact path stops finding Node (agent's call 12) |

### 8. Texts (`locales/en.json`)

Plain English (rule 7); the drafts, polished when built:

- `unused-toolchains.rust.name` "Unused Rust toolchains"; `.what` "Versions of the Rust compiler that rustup installed
  and that nothing on this Mac points at: not rustup's default, not an override, not a project's toolchain file.";
  `.made-by` "rustup, the installer of Rust."; `.if-removed` "The toolchain's folder goes to the Trash. A project, a
  script or an editor that asks for this exact version gets an error until it is installed again."; `.comes-back`
  "Yes, with one command and a download of a few hundred megabytes. Components and targets you added to it must be
  added again."
- `unused-toolchains.node.name` "Unused Node versions (nvm)"; `.what` "Versions of Node.js that nvm installed and that
  nothing on this Mac points at: not nvm's default or another alias, not a project's .nvmrc, .node-version or
  .tool-versions."; `.made-by` "nvm, the Node version manager."; `.if-removed` "The version's folder goes to the Trash
  with every global package installed with it. An editor set to this exact Node stops finding it."; `.comes-back`
  "With one command and a download. Global packages you installed with it must be installed again."
- `unused-toolchains.title.rust` "Rust {name}"; `unused-toolchains.title.node` "Node {name}".
- `unused-toolchains.reason.rust` "Nothing on this Mac points at it. Last opened or changed on {date}. To get it back:
  {command}." `unused-toolchains.reason.node` the same sentences.
- Left-outs: `unused-toolchains.left-out.special`, `.not-yours` (the `dev-caches` sentences adapted);
  `.partial-scan` "Some folders could not be read, so a project there might use this version.";
  `.default-unknown` "Fresher could not read which version is the default, so it keeps every version.";
  `.pin-unknown` "A version file in your folders could not be read, so Fresher cannot tell which versions are used."
- The switches: `unused-toolchains.enabled.label` "Unused toolchain versions", `.description` "Finds Rust toolchains
  that nothing on this Mac points at. Each goes to the Trash and comes back with one command.";
  `unused-toolchains.node-versions.label` "Node versions (nvm)", `.description` "Also finds Node versions nvm
  installed that nothing points at. Off by default: global tools installed with a version go with it."

### 9. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Core** (01/01): `small_file` for each answer, negatives first (missing, a link, a folder, too long, not UTF-8,
  unreadable), then text at and under the limit; `DiskFiles` the same on a temporary folder; the app's adapter maps
  every `ReadError`.
- **Test worlds** (01/02): `text` refused with `size`, `fill`, `kind`, over 4 KiB; written byte for byte. `[settings]`
  refused for an unknown key, a wrong kind, a conflict in `all`; applied by the autotest.
- **Unit** (02): `toolchains.rs` (every refusal, then the shipped table pinned); `pointers/` (every shape refused,
  then every resolution); `decide.rs` (one test per row, keep before offer; properties: the verdict never depends on
  the order of matches, a known reference always keeps its folder whatever else is `Unknown`, the switch off never
  claims a Node version, the default and the newest are never offered); `tests.rs` (the module on hand-built matches
  with `FakeSmallFiles`, a walk not complete).
- **Fixtures** (`fixtures/homes/unused-toolchains-*.toml`, `run_module`, golden
  `fixtures/golden/unused-toolchains.json`): `-not-claimed`, `-kept`, `-kept-unknown`, `-offered` (no `-off` home: the offered home is run with the
  switch off). `partial-scan` is proven only in `src/tests.rs` (a walk that is not complete).
- **Contract**: `assert_module_contract` with `Catalogue::Required`: two category files with the same dangers, every
  text and every declared left-out used by a fixture run.
- **Sandbox**: four case files (`unused-toolchains`, `-unknown`, `-off`, `-documents-denied`) with their goldens;
  the main file grants Desktop, Documents and Downloads access in its first case; `EXPECTED.md` walked by hand.
  `npm run sandbox -- all` orders the case files by file name without `.toml` (a base file before its variants) and
  leaves out whole a file that writes a path an earlier file wrote, naming it on stderr and in `EXPECTED.md`: the four
  files share fixed paths (`.rustup/settings.toml`, `.nvm/alias/default`), so only the first stays in `all`.
- **Core e2e** (the fixture homes store no settings, so the flow turns the Node switch on through `update_setting`):
  the scan offers; a cleanup moves a toolchain to the sandbox's Trash and put back restores it; a pin written between
  two scans keeps its version on the rescan; a version with a file open is skipped `InUse`; a restored scan offers
  the same.
- **UI e2e** (plan 027 removed module pages and module sidebar entries): the module's two categories on Scan's
  danger-zone cards and on the category Details screen, the Node switch off by default in Settings, a finding's title and
  reason, the review naming Medium for a Node version, the unavailable notice without access to Documents.

## Risks

- **A pointer the module cannot see**: an editor's Node path or Rust toolchain, a script's `cargo +1.75.0`, a CI file,
  `RUSTUP_TOOLCHAIN`, a shell profile's `nvm use 16`, a project on an external drive or outside the scan's roots. The
  version is offered; the Trash keeps it and the text names the command that installs it again. Rust is Low, Node is
  Medium and off by default; neither is Safe.
- **A scan root narrower than the home** (the user's setting): pins outside the roots are not seen. The default root
  is the home; the module cannot read the core's roots. Written in both category docs; to revisit if the roots setting
  becomes common (agent's call 6).
- **A walk that is never complete on a real Mac** (one unreadable folder anywhere makes `is_walk_complete()` false):
  then every version is left out `partial-scan`. To verify in 04/02 on the developer's Mac; if so, a follow-up asks
  the walk whether the home outside `~/Library` was read whole (plan 021's `subtrees` already answers that question
  for a folder).
- **rustup's `update-hashes/<name>`** stays after the folder goes: to verify in 04/02 that `rustup toolchain install`
  of a removed toolchain installs it again (expected: rustup checks the folder, not the hash).
- **A pin linked from elsewhere** (`.nvmrc` as a link to the repository root's): `NotAFile` → `Unknown`, so nvm offers
  nothing while such a link exists. How common it is: to verify in 04/02.

## Open questions

None.

## Decisions

- The user, 2026-11-19 (translated, recorded in the idea): "cleaning Node versions installed by nvm and the like is
  worth considering, but it must never be on by default." Here: the switch `unused-toolchains.node-versions`, off.
- The user's delegation of 2026-11-20 (translated): "build plan 027, then the ignore of one item, then 023, then 024,
  autonomously; take the recommended decisions; I check everything together later." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code and the rules; each with what it costs if wrong.

1. **Only rustup and nvm in this plan; the rest is deferred** ([`other-toolchain-versions`](../../deferred/other-toolchain-versions.md)),
   against the idea's five managers. pyenv needs every virtualenv's `pyvenv.cfg` on the Mac (pipx's, poetry's and
   editors' venvs live outside any project; a missed one breaks with a confusing error); SDKMAN's and fnm's defaults
   are links whose targets the walk does not keep; volta's pins live in every `package.json` (thousands per
   `node_modules`); Gradle's toolchain specs are code in build files; AVDs hold the user's emulator data and system
   JDKs need administrator rights. rustup and nvm keep every pointer in small files in known places. Cost if wrong:
   the other managers' versions stay in the storage map until the deferred item is built.
2. **A module reads a small file through a new system port in `finish`, not through the walk** (the idea proposed "a
   read-only reader with a size cap"; the Ollama note named both ways). No hot path changes and a pin edited in place
   is never read stale; the price is one port with about twenty mechanical construction sites. Cost if wrong: Ollama's
   manifests, if they need the walk's bytes in the cache, would still need the walk route later.
3. **The rule "read only what your matches showed" is review, not an engine check.** Cost if wrong: a later module
   could read a file it did not match; the read stays bounded, read-only, no-follow and unlogged.
4. **The version folder goes to the Trash; no tool command** (the idea preferred the managers' commands). Both
   commands delete for good, need a shell with the manager loaded (nvm is a shell function), and could not run in a
   sandbox; both remove only that folder. Cost if wrong: rustup's `update-hashes/<name>` file stays (to verify).
5. **Only the default places and the named files; no environment variable, shell profile or editor setting is read**
   (plan 018's rule, kept by 023). Cost if wrong: a moved `RUSTUP_HOME` or `NVM_DIR` is not found (its space stays in
   the storage map); a pointer in an editor is not seen (the risk above).
6. **Both categories require Desktop, Documents and Downloads** (`Need::required`): projects live there, and a guarded
   folder not entered does not make the walk incomplete. A scan root narrower than the home is not detected. Cost if
   wrong: without Full Disk Access or the three switches the module does not run (the UI says why); a user with
   narrowed roots may be offered a version a project outside them pins (Low or Medium, in the Trash).
7. **Never offered: rustup's moving channels and the newest Node version**, beside the default. `cargo +nightly` and
   editors name a channel without any file; `nvm use node` names the newest. Cost if wrong: an unused `beta` or
   undated `nightly` stays; the newest Node stays even when nothing uses it.
8. **Conservative pins.** A pin anywhere in the scanned folders counts (not only in a project), except inside a
   project's artifact folder; `X.Y` keeps every `X.Y.Z`; an nvm prefix keeps every match; any pin or alias the module
   cannot read blocks that manager's offers (`pin-unknown`). Cost if wrong: fewer offers (a stray `.nvmrc` in a
   download keeps its version; one unreadable pin keeps them all), never a wrong one.
9. **No age threshold and no day setting**: "not used", not "old"; the date in the reason is the newest change or file
   access of the folder. Cost if wrong: a version installed yesterday and not yet pinned is offered, with its date.
10. **The version folders are claimed whole even with the Node switch off**: interests are fixed when the module is
    built, and the folders are the manager's, not the user's. Cost if wrong: with the switch off, a Node version's
    folder cannot be opened in the storage map.
11. **A restored scan decides from the cached matches and the files read now.** A pin created since the last walk is
    not among the cached matches until the next walk, as any change since then. Cost if wrong: a version pinned after
    the last walk is offered by the restored scan; the next scan keeps it.
12. **Node versions are Medium** (the idea: Low, Medium with global packages). The version is claimed whole, so the
    module cannot see whether its `lib/node_modules` holds global packages; rule 1 takes the higher level. Rust stays
    Low. Cost if wrong: one level too high on a Node version without global packages.
13. **Versions something keeps are not reported** (no left-out "in use"). Cost if wrong: the category's details do not
    say how many versions were kept as used.
14. **One module `unused-toolchains`, group Development, two categories, one switch for Node**; Rust on by default.
    Cost if wrong: a group or a default to change.
15. **The test worlds learn file text and case-file settings** (plan 01/02) rather than proving the Node category only
    in fixtures. Cost if wrong: two small additions to `fresher-testing` and `fresher-sandbox` that other plans may not
    use.
16. **`LastChange` of the projects is not used**: a project's last activity does not tell which version it ran; only
    the artifact folders' paths are. Cost if wrong: none.
17. **No new dependency** (`serde`, `thiserror`, `toml` as in `local-ai-models`; `proptest`, `serde_json` for tests),
    **no new UI component** (the module's switch is a setting the settings screen already renders). Cost if wrong:
    none.

## Conclusions

Built, **awaiting the user's walk**; the numbers, the changes from this design and the calls taken while building are in
the [global readme](./readme.md#conclusions). Of the calls above, 7 (the newest Node version: a linked version folder counts as installed, a plain file does not),
10 (with the switch off the row is decided with empty pointers and no nvm file is read) and 15 (the sandbox now
depends on the store to write the settings file) were refined while building; the others stand as written. Open
questions: none; the real-Mac facts "to verify" are pending the user's dry run.
