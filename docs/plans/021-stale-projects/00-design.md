# 021 — Stale projects: design

**Status:** ✅ approved (the user's decision of 2026-11-19) · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-19 from the approved one-file draft (the user approved it with its ten agent's calls: "the rest
seems fine; work autonomously, take the recommended decisions") and from a read of the code on
`feat/021-stale-projects` (from `main` f19e872) and of this Mac, read-only. Where the draft was wrong about the code or
about macOS, the design is corrected and the correction is listed under "Agent's calls". Facts marked "to verify" are
checked by hand in 04/02.

## Intent

The idea [Stale project cleanup](../../ideas/stale-project-cleanup.md): old projects keep gigabytes of what their tools
can make again — `node_modules`, Cargo's `target/`, a Python `.venv`, Gradle's `build/` — and nobody remembers they
are there. Plan [009](../009-system-knowledge/readme.md) already finds projects in the shared walk by their marker
files and lists their artifact folders; what is missing is a module that says which of them can go and why, and a
precise "last activity" (the deferred item *A project's last activity, precisely*, due now).

A second need came from building Fresher itself: an **active** Rust project's `target/` grew to 20–50 GB in a few
days. Part of it is build cache that Cargo makes again on the next build; that part is offered even for an active project.

## Goals

1. A module `stale-projects` (`modules/stale-projects`, crate `fresher-module-stale-projects`, group Development) that
   offers the artifact folders of projects untouched for a set number of days, one finding per artifact folder, named
   by its project, with the project's last activity and the command that makes it again.
2. A project's last activity is the newest change of its own folders, outside its artifact folders and outside `.git`,
   or its last commit, whichever is newer — never the project folder's own time alone.
3. For any Rust project, active or not, the parts of `target/` that are pure cache are offered on their own after 7 days
   without a build: the incremental compilation folders and the coverage build.
4. The source code, the project folder, `.git` and anything not listed as an artifact or a cache of the project's kind
   are never offered; nothing becomes opaque to the storage map or to another module because this module exists.

## Non-goals

- Running a tool (`cargo clean`, `npm`, `git`): the module reads what the walk saw; the cleaner moves folders to the
  Trash.
- Deleting single stale files inside `target/deps` by age: a later idea, it needs Cargo's fingerprints to be safe.
- `dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` and any other framework output: never (decision 1).
- Projects inside `~/Library`, the apps folders, an app bundle or a hidden folder of the home (best practices #22).
- Unused toolchain versions and `nvm`: plan 024, which reads this plan's project list (decision 2).
- A project's uncommitted changes and a running build: not read (decisions 6 and 10).

## What already exists

| Need                                    | What exists                                                                                                    | Plan     |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------- |
| Projects                                | `ProjectKind` (markers, `artifacts()`), `Project { root, kinds, artifacts: Vec<PathBuf>, last_activity }`      | 009      |
| Finding them                            | `crates/engine/src/projects.rs`: `interests()` (a `MarkerFile` per marker, a `FolderName` per artifact name, all `Claim::Item`, owned by `ModuleId::system()`) and `ProjectCollector` (`add`, `finish`); `session::run` calls `system.set_projects(projects.finish())` after the walk, before every `finish` | 009 |
| Reading them                            | `cx.system().projects()`: `None` during the walk, the list in `finish`                                         | 009      |
| Last activity                           | the project folder's own modification time (the marker match's `modified`) — the deferred item                | 009      |
| The walked tree                         | the scanner keeps every folder as a node with its own `modified` (`tree/node.rs`), frozen with its subtree's sums; files under 1 MiB are folded into their folder (`tree/fold.rs`, `KEEP_THRESHOLD`); `Scanner::last` holds the tree from the walk to `commit`, also after a restore | 004 |
| A query of the tree before the commit   | only `places_seen` (what the walk saw of an anchored place), computed inside `walk`                             | 019      |
| A folder's size                         | an `Item` match carries the folder entry's own size; only a `Claim::Whole` folder carries everything inside it (`Match.sizes`, `Match.inside`) | 004, 017 |
| A finding's removability                | the engine keeps the `EntryFacts` of each match of the module (`Output::facts`); a removal no match of the scan names is `NeedsAdministrator` | 011 |
| The day setting                         | `DaySetting` (`crates/core/src/settings/days.rs`)                                                              | 020      |
| Answers that wait for the whole walk    | `finish`, a `Gathered` keyed by the scan's cancel token (`modules/downloads/src/gathered.rs`)                  | 019, 020 |
| Left-outs                               | `out.left_out(LeftOutItem { category, reason, sizes })`, `with_problems(keys)`                                 | 011      |
| A data table in a module                | `include_str!`, parsed and checked when the module is built (`modules/downloads/unfinished.toml`)              | 018–020  |
| Folder ages in a tree                   | `[[dir]]` with `age` in a fixture home and `[[case.dir]]` in a case file, set deepest first after every file   | 016, 017 |
| The module harness                      | `run_module` (`crates/testing/src/run_module.rs`) walks with `NaiveWalk`; it computes no projects               | 002, 009 |

**What is missing** (built first, plan 01):

1. **A question to the walked tree before the commit.** The draft's "newest time per project from the walked tree" has
   no API: the tree is reachable only after `commit`, which waits for every module's `finish`.
2. **An artifact folder's size.** The engine's artifact interests are `Item` claims: the match has the folder entry's
   own size only. Claiming `target`, `build` or `venv` whole instead would make every folder of that name in the home
   opaque to the storage map and to every other module, the module on or off (the walk carries every registered
   module's interests): a user's own `build` folder could not be opened, and no project inside one would be found.
3. **The last commit, through the walk.** No pattern matches a file by name and keeps its time (`FolderName`,
   `Under`, `MarkerFile`, `Extension`), and `.git/logs/HEAD` is a small file folded into its folder.
4. **The projects in the module harness.** Modules may dev-depend only on `fresher-testing`
   (`scripts/check-crate-deps.ts`), and the collector lives in the engine: `run_module` cannot give a module the
   projects the engine would.

## Design

### 1. What the walk saw below a folder (plan 01/01)

```rust
// crates/core/src/walk.rs

/// A folder of the last walk to describe, and the folders below it whose times are left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtreeQuery {
    pub folder: PathBuf,
    /// Folders below `folder` left out of `newest` and `complete`, with everything inside them; their sizes count.
    pub skip: Vec<PathBuf>,
}

/// What the last walk saw below one folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subtree {
    /// The folder and everything inside it, as the walk summed it.
    pub sizes: Sizes,
    /// The newest modification time of the folder and of every folder below it, the skipped ones and everything
    /// inside them left out; `None` when none has one.
    pub newest: Option<Timestamp>,
    /// Every folder counted in `newest` was read in full: none unreadable, avoided, on another volume or collapsed.
    pub complete: bool,
}

pub trait DiskWalk: Send + Sync {
    // walk, commit: unchanged
    /// What the last walk saw below each of `queries`, in their order, from its tree before `commit`: no disk is
    /// read (a restore answers from the cached tree). `None` for a path that is not a folder of that tree (never
    /// walked, inside a folder claimed whole, a link, a file) and for every query before any walk.
    fn subtrees(&self, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>>;
}
```

- **Folders only.** A folder's modification time moves when an entry is added, removed or renamed in it: every
  atomic save (write a temporary file, rename it over the old one), every new file, every `git checkout` of a file.
  A file edited in place moves only its own time, which the walk does not keep for a file under 1 MiB. Kept files'
  times are not used either, so the answer never depends on the fold threshold. On this Mac, over 20 git projects, the
  newest folder time was never older than the newest file time outside the artifact folders and `.git` (agent's call
  2); in place editors are "to verify" in 04/02.
- **The scanner** (`crates/scanner/src/subtrees.rs`, new): under the lock of `Scanner::last`, each query's folder by
  `Tree::lookup`, its skips by `lookup` too (a skip that is not a node is ignored); a depth-first pass over the
  children (`Tree::children`) that does not enter a skipped node; `newest` the max of `Node::modified` of every folder
  node visited; `complete` false when a visited node `is_incomplete()` or has `OtherVolume` or `Collapsed`; a node
  claimed whole (`Opaque`) counts with its own time and adds nothing inside it; one
  below the queried folder makes the answer incomplete (what changed inside it is not known). `sizes` from the frozen
  node as `TreeState::info` builds them (the subtree's sums, `Partial` when the node is). A link, a file or a missing node:
  `None`.
- **The reference walk** (`NaiveWalk`) records each walked folder's own modification time and whether it was listed,
  and answers from them; the reference test compares the scanner's answers with it on generated trees.
- **The other walks** (`ScriptedWalk` in the engine's tests, `NoWalk` and `HeldWalk` in the app's and the e2e demo's)
  answer `None`, the scripted one what its test gives it.
- **Cost.** One pass over each queried subtree after the walk, only over the nodes the skips leave (the artifact
  folders, the biggest parts, are skipped for the projects' own times). The scan bench gains a `subtrees` group on the
  large tree's projects; the scan of the developer's home is timed before and after in 04/02 (rule 4).

### 2. Projects with what the walk saw of them (plans 01/02 and 01/03)

#### 2.1 The finder moves to the core

`crates/engine/src/projects.rs` moves to `crates/core/src/system/finder.rs` as `ProjectFinder`, so that the engine and
`run_module` find the same projects from the same walk:

```rust
/// Finds the projects of one walk from the matches of its own interests, then asks the walk what it saw of them.
#[derive(Debug, Default)]
pub struct ProjectFinder { /* the looks, the marker folders by root, the artifact and cache folders matched */ }

impl ProjectFinder {
    /// The interests the walk carries for the projects, in a fixed order, owned by `ModuleId::system()`: a
    /// `MarkerFile` per marker, a `FolderName` per artifact name and per cache name, all `Claim::Item`.
    pub fn interests() -> Vec<Interest>;
    /// Takes one match; `local` indexes `interests()`. Only folders count.
    pub fn add(&mut self, local: usize, matched: &Match);
    /// The projects, by root (as today: the markers of one folder merged, a project inside another's artifact folder
    /// or inside `node_modules` dropped), each with its artifacts, its caches and its last activity, asked of `walk`
    /// in one call; `None` (a walk that did not end) gives every size `Sizes::ZERO` partial and every time `Unknown`.
    pub fn finish(self, walk: Option<&dyn DiskWalk>) -> Vec<Project>;
}
```

The engine's `projects.rs` and its tests move with it (the engine keeps nothing of its own); `session::run` takes
`ProjectFinder::interests()` in `all_interests` and calls `system.set_projects(finder.finish(walk))` with
`Some(&*shared.ports.walk)` when the walk (or the restore) returned a summary, `None` otherwise. `run_module` does the
same with its `NaiveWalk`: the system's matches go to the finder, never to the module and never into the run's
`matched` paths.

#### 2.2 Artifacts and caches with their sizes

```rust
/// When something last changed, as far as the walk could tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastChange {
    /// Every folder counted was read: this is the newest change.
    Known(Timestamp),
    /// A folder could not be read: the newest change is this one or later.
    AtLeast(Timestamp),
    /// The walk saw nothing to date it.
    Unknown,
}

/// A folder of a project that its tools make again, with what the walk saw of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub path: PathBuf,
    /// Everything inside it, as the walk summed it.
    pub sizes: Sizes,
    /// The newest change of it and of every folder inside it.
    pub last_change: LastChange,
}

pub struct Project {
    pub root: PathBuf,
    pub kinds: Vec<ProjectKind>,
    /// The kinds' artifact folders that exist in it, by path.
    pub artifacts: Vec<Artifact>,
    /// The kinds' cache folders that exist inside its artifact folders, by path; none inside another.
    pub caches: Vec<Artifact>,
    /// The newest change of its own folders and of its git branches (section 2.3).
    pub last_activity: LastChange,
}

impl ProjectKind {
    /// Folders inside the kind's artifact folders that only make the next build faster, relative to the project root,
    /// `*` one name: Rust's `target/*/incremental`, `target/*/*/incremental` (a target triple's) and
    /// `target/llvm-cov-target`; none for the other kinds.
    pub const fn caches(self) -> &'static [&'static str];
}
```

- An artifact or a cache is a folder the walk matched (a link is never one: the finder takes folders only, as today)
  and the tree has a folder node of; one it has none of is dropped.
- A cache is a matched folder named like a cache's last segment whose path relative to a project of the kind matches
  the pattern segment by segment, and that is not inside another cache of the project
  (`target/llvm-cov-target/debug/incremental` is the coverage build's, not a second cache).
- The cache names join the finder's interests (`incremental`, `llvm-cov-target`), `Claim::Item`: a folder named so
  anywhere stays open. The matcher's fingerprint changes once, so the first scan after the update reads every folder.

#### 2.3 Last activity

For each project, two queries in the finder's one call:

- **Its own folders**: `SubtreeQuery { folder: root, skip }`, `skip` its artifact folders, `root/.git`, and for every
  project whose root lies inside this one, that project's artifact folders and `.git`.
- **Its branches**: `root/.git/refs/heads` with no skip. Git writes a branch's new commit to the branch's file by
  writing a lock file and renaming it, so the folder's time moves on every commit, merge, pull or reset of a local
  branch and on a new branch (`refs/heads/feature/` gets its own folder); a fetch writes `refs/remotes`, not here.
  `git` is never run and no file is read: the times are the walk's (agent's call 1).
- `last_activity` = the newer of the two `newest`; `Known` when both answers are complete (a missing `refs/heads`, a
  project without git or a worktree whose `.git` is a file, counts as complete), `AtLeast` when one is not, `Unknown`
  when neither has a time.
- Status changes are never used: a migration to a new Mac or a copy sets every status change to the day it ran, which
  would make every project active for a year (agent's call 3).
- Removing an artifact folder moves the project folder's time (its listing changed): a project the user cleaned, with
  this module or another tool, looks active from that day. Its other artifacts wait for the days again (a risk, written
  in the category docs).

The deferred item *A project's last activity, precisely* is closed by 01/03 (its file and row
removed); `docs/architecture/modules.md` describes the projects' facts.

### 3. The module (plan 02)

#### 3.1 Interests and what it gathers

One `FolderName` interest per name of every kind's `artifacts()` and of the last segment of every kind's `caches()`
(`node_modules`, `target`, `build`, `.gradle`, `.venv`, `venv`, `.build`, `.dart_tool`, `incremental`,
`llvm-cov-target`), each `Claim::Item`: nothing becomes opaque. The module gathers the folder paths it matched for the
scan (`gathered.rs`, keyed by the cancel token, as `downloads`); in `finish` it offers only a path it matched itself, so
every removal is its own match (the contract) and the engine has the folder's facts for its removability.

The categories need no capability; a project in a guarded folder is seen only when the walk entered it.

#### 3.2 Which projects

Each project of `cx.system().projects()`, by root; **not claimed** (nothing reported) when:

| # | Its root is…                                                                                               |
| - | ---------------------------------------------------------------------------------------------------------- |
| 1 | the home folder or a folder holding it (`~/package.json` and `~/node_modules` from an `npm install` run in the home), or the home cannot be located |
| 2 | inside `KnownFolder::Library`, `Applications`, `UserApplications` or `SystemApplications` (each as located) |
| 3 | inside a folder whose name ends `.app`, ASCII case ignored (an app's own `node_modules`)                     |
| 4 | inside a folder of the home whose name starts with `.` (`~/.cargo`, `~/.npm`, `~/.config`: tools' own)       |

The projects left are grouped by their **outermost** project: one that lies inside no other project left. A monorepo
reads as one group: the group's last activity, the newest of its members' own (agent's call 15), decides for every
project inside it, and every artifact is named by the outermost project.

#### 3.3 Categories and the table

| Category                             | What                                                                                     | Danger | Offered when                                    |
| ------------------------------------ | ---------------------------------------------------------------------------------------- | ------ | ----------------------------------------------- |
| `stale-projects.dependencies`        | `node_modules` of a Node project                                                         | Low    | the group is stale                              |
| `stale-projects.build-output`        | `target` (Rust, Maven), `build` and `.gradle` (Gradle), `.build` (Swift), `.dart_tool` and `build` (Dart) | Low | the group is stale            |
| `stale-projects.python-environments` | `.venv`, `venv` of a Python project                                                      | Medium | the group is stale                              |
| `stale-projects.rust-build-cache`    | the caches of `ProjectKind::Rust`                                                        | Low    | unchanged for 7 days, whatever the group's age  |

`modules/stale-projects/artifacts.toml` (rule 5) maps each artifact of each kind to its category and the command that
makes it again:

```toml
[[artifact]]
kind = "node"                   # node, rust, gradle, maven, python, swift-package, go, dart
folder = "node_modules"         # one of the kind's artifacts in the core (ProjectKind::artifacts)
category = "dependencies"       # dependencies, build-output, python-environments
command = "npm install"         # what the reason names
```

Rows: node `node_modules` dependencies `npm install`; rust `target` build-output `cargo build`; maven `target`
build-output `mvn package`; gradle `build` and `.gradle` build-output `gradle build`; swift-package `.build`
build-output `swift build`; dart `.dart_tool` build-output `dart pub get`; dart `build` build-output `flutter build`;
python `.venv` and `venv` python-environments `python3 -m venv .venv` and `python3 -m venv venv`. Refused: not valid
TOML, an unknown field, no row, an unknown kind or category, a folder that is not an artifact of its kind in the core,
a pair listed twice, an artifact of the core without a row, a command that is empty or not one line of printable
ASCII. A cache's command is its artifact's row's (`cargo build`).

One setting, `stale-projects.unused-for-days` (`DaySetting::new("unused-for-days", 30, 3650, 365)`); the caches'
7 days are a constant (`CACHE_UNUSED_DAYS`, agent's call 6).

#### 3.4 The decision

Plain functions on plain values (`decide.rs`), the module only gathers and reports. "Within the days" is a time after
`now − days`, or after `now`. A group is decided by its last activity: the newest time of its members'
`last_activity`, `Known` only when every member's is `Known`, `Unknown` when none has a time (agent's call 15):

| # | Situation                                                         | The group is |
| - | ----------------------------------------------------------------- | ------------ |
| 1 | a restored scan                                                   | restored     |
| 2 | `Known(t)` or `AtLeast(t)`, `t` within the days or in the future   | active       |
| 3 | `AtLeast(t)` with `t` before the days, or `Unknown`                | unread       |
| 4 | `Known(t)`, `t` before the days                                    | stale since `t` |

**Artifacts** of every project of the group, in path order, each once:

| # | Situation                                                                                  | Result                              |
| - | ------------------------------------------------------------------------------------------ | ----------------------------------- |
| 1 | not matched by the module, or a found project's root, or holding one                       | not claimed                         |
| 2 | the group restored                                                                         | left out `scan-again`               |
| 3 | the group active                                                                           | left out `recent`                   |
| 4 | the group unread                                                                           | left out `unread`                   |
| 5 | the group stale, the artifact's own `last_change` within the days or in the future (`Known` or `AtLeast`) | left out `recent` |
| 6 | the group stale, the artifact's own `last_change` not `Known`, or its sizes `Partial`      | left out `unread`                   |
| 7 | the group stale                                                                            | **offered**, in its row's category  |

An artifact no row of its project's kinds names is not claimed (agent's call 16). A project folder is never offered as
another's artifact, and an artifact changed lately is kept even in a stale project (agent's calls 21 and 22).

**Caches** of every Rust project of the group whose `target` is not offered (an offered `target` holds them):

| # | Situation                                                                        | Result                                   |
| - | -------------------------------------------------------------------------------- | ---------------------------------------- |
| 1 | not matched by the module, or a found project's root, or holding one             | not claimed                              |
| 2 | a restored scan                                                                  | left out `scan-again`                    |
| 3 | its `last_change` within 7 days or in the future (`Known` or `AtLeast`)          | left out `cache-recent`                  |
| 4 | `Unknown`, `AtLeast(t)` before the 7 days, or its sizes `Partial`                | left out `unread`                        |
| 5 | otherwise                                                                        | **offered**, `rust-build-cache`          |

- A `target` left out while caches inside it are offered is reported with its sizes less theirs (field by field,
  never below zero), so a kept line never counts what is offered.
- Properties: a later activity, a later artifact or cache change or a restored scan never turns a keep into an offer.
- At the act: an item a process holds open is skipped by the cleaner's in-use check; everything goes to the Trash.

A finding:

| Part     | Value                                                                                                         |
| -------- | ------------------------------------------------------------------------------------------------------------- |
| identity | `<category name>/<path relative to the home>` (`dependencies/Code/shop/node_modules`)                          |
| subject  | `Subject::Path`; `Removal::MoveToTrash` of the same path                                                       |
| sizes    | the artifact's or the cache's, as the walk summed them                                                         |
| title    | `stale-projects.title` with `{project}` (the outermost root's name) and `{folder}` (the path relative to it): "shop · packages/web/node_modules" |
| reason   | per category, below: `{date}` the group's last activity, `{command}` the row's, `{days}` 7 for a cache         |

#### 3.5 Texts

Every text in `modules/stale-projects/locales/en.json` (rule 7):

- **dependencies** — name "Packages of old projects"; what: "The packages a Node.js project installed into its
  node_modules folder, in projects you have not changed for a long time."; made-by: "Your package manager (npm, Yarn and
  others), when the project's packages were installed."; if-removed: "They go to the Trash. Your code stays; installing
  the project's packages again brings them back."; comes-back: "Only when you install the project's packages again."
- **build-output** — name "Build output of old projects"; what: "What a project's build tool made and can make again
  (Cargo's and Maven's target, Gradle's build and .gradle, Swift's .build, Dart's .dart_tool and build), in projects
  you have not changed for a long time."; made-by: "The project's build tool, when you built it."; if-removed: "They go
  to the Trash. Your code stays; the next build makes them again, which takes a while."; comes-back: "Yes, with the
  next build."
- **python-environments** — name "Python environments of old projects"; what: "A Python project's virtual environment
  (.venv or venv): its own Python and the packages installed into it, in projects you have not changed for a long
  time."; made-by: "You or your Python tools, when you set the project up."; if-removed: "They go to the Trash. Your
  code stays. To work on the project again, make a new environment and install its packages; a package you installed
  by hand and listed nowhere has to be found again."; comes-back: "Only when you make the environment again."
- **rust-build-cache** — name "Rust build cache"; what: "Files Cargo keeps only to build a Rust project faster: the
  incremental compilation folders and the separate build made to measure test coverage. They often take gigabytes,
  even in a project you work on every day."; made-by: "Cargo and the Rust compiler, on every build; cargo llvm-cov,
  when it measured coverage."; if-removed: "They go to the Trash. Your code and your built programs stay; the next
  build takes longer once, while it makes them again."; comes-back: "Yes, with the next build."

`stale-projects.title` "{project} · {folder}". Reasons: `stale-projects.reason.dependencies` "Nothing in this project
has changed since {date}. Running {command} in it downloads these packages again."; `.build-output` "Nothing in this
project has changed since {date}. Running {command} in it builds this again, which takes a while.";
`.python-environments` "Nothing in this project has changed since {date}. {command} and an install of the project's
packages make it again; packages you added by hand and listed nowhere are lost."; `.rust-build-cache` "Only Cargo
reads this, to build faster, and no build has used it for {days} days. The next {command} makes what it needs again."

The switch: `stale-projects.enabled.label` "Old projects", `.description` "Finds the installed packages and build
output of projects you have not changed for a long time, and Rust's build cache. Your code is never touched." The
setting `stale-projects.unused-for-days`: label "Offer projects unchanged for", description "A project's packages and
build output are offered only once nothing in it has changed for this many days, at least 30. Rust's build cache is
offered after 7 days without a build, whatever this says."

The left-out reasons, declared in this order:

| Key                                     | Text                                                                                                    |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `stale-projects.left-out.scan-again`    | These come from the last scan, and the projects may have changed since. Scan again to check.            |
| `stale-projects.left-out.unread`        | Fresher could not read every folder of this project, so it cannot tell when it last changed. It is kept. |
| `stale-projects.left-out.recent`        | It or its project changed within the days set in Settings, so it is kept. A scan after those days offers it. |
| `stale-projects.left-out.cache-recent`  | A build used it within the last 7 days, so it is kept.                                                  |

### 4. Tests

**Unit**, negatives first. Core: `subtrees` of the scanner (`subtrees_tests.rs`: a missing path, a link, a file and a
folder inside a whole claim are `None`; a skipped subtree's newer time does not count; an unreadable, avoided,
other-volume or collapsed folder makes it incomplete unless skipped; a whole-claimed folder counts with its own time;
the sizes are the frozen sums); the finder (`finder.rs`: the moved tests; a cache outside its kind's artifact, of
another kind, deeper than its pattern or inside another cache is none; a nested project's artifacts and `.git` are
skipped for the outer one; `refs/heads` newer than the folders wins; an incomplete answer is `AtLeast`; no walk gives
`Unknown`); `ProjectKind::caches` (every pattern starts with one of the kind's artifacts). Module: the table's
refusals, then the shipped table; `place.rs` (each row of 3.2, the outermost of a monorepo); `decide.rs` (each row of
3.4, keep before offer, the precedence; properties: a later activity, a later cache change, a restored scan never turn
a keep into an offer).

**Reference**: the scanner's `subtrees` equal `NaiveWalk`'s on generated trees with random skips
(`crates/scanner/tests/reference.rs`).

**Fixture** (`modules/stale-projects/tests/fixtures.rs`, `run_module`, `Catalogue::Required`); every folder of a
project that must look stale is a `[[dir]]` with an `age` (a folder made at the test's time is "changed lately"),
parents of files included:

| Home                                 | Kind     | What                                                                                                   | Expected                          |
| ------------------------------------ | -------- | ------------------------------------------------------------------------------------------------------ | --------------------------------- |
| `stale-projects-not-claimed.toml`    | negative | a stale Node project with `dist/`, `.next/`, `.turbo/`, `src/`, `.git/` and no `node_modules`; `notes/node_modules` without a marker; a link `linked/node_modules` in a stale Node project; `~/package.json` with `~/node_modules`; a project in `.config/tool/`, in `Library/Application Support/X/`, in `Applications/Tool.app/Contents/Resources/app/`; a Go project with a `target/`; a Node project's `build/`; a stale Gradle project whose `build/` holds a `package.json` | nothing offered, nothing left out |
| `stale-projects-kept.toml`           | negative | an active Node project; old folders with `.git/refs/heads` 3 days old; a monorepo whose nested package changed 3 days ago; an active Rust project with `target/debug/incremental` 3 days old; a project with an avoided folder (`Ports.avoid`); a 400-day Python project whose `.venv` changed 3 days ago | `recent`, `recent`, `recent` (each of the monorepo's), `recent` and `cache-recent`, `unread`, `recent` |
| (the same, restored)                 | negative | `Ports.restored`                                                                                        | everything `scan-again`           |
| `stale-projects-offered.toml`        | positive | stale Node, Rust (`target` with `debug/incremental` inside: one finding), Maven, Gradle, Swift, Dart, Python `.venv` and `venv`; a stale monorepo with two packages' `node_modules`; an active Rust project with `target/debug/incremental` and `target/aarch64-apple-darwin/debug/incremental` and `target/llvm-cov-target` 10 days old | each at its danger, title and reason; the active `target` left out `recent` with its sizes less its caches'; golden `fixtures/golden/stale-projects.json` |

Plus: the setting moves the days; the matches in two orders give the same answers; a cancelled scan stops between
groups.

**Sandbox** (rule 8), `fixtures/sandboxes/stale-projects.toml`, every project folder dated, negatives first: the
source, `.git` and the project folder are never claimed (`[case.map]` `your-folders` on `src`); `dist` and `.next` are
never claimed; a `node_modules` without a project is never claimed; a link named `node_modules` is never claimed; a
`node_modules` in the home itself is never claimed; a project in a hidden folder of the home is never claimed; a
project inside an app bundle is never claimed; an active project's packages are kept (`recent`); a project with a
recent commit is kept (`recent`); a Rust build cache used this week is kept (`cache-recent`). Positives: an old Node
project's packages go; an old Rust project's `target` goes; an old Gradle project's `build` goes; an old Python
project's `.venv` goes, at Medium; a monorepo's package's `node_modules` goes under the monorepo's name; an active Rust
project's incremental cache goes; an active Rust project's coverage build goes.

**Core e2e** (`crates/e2e/tests/stale_projects.rs`, harness `stale_projects_home/mod.rs` on the shared
`support/home.rs`, fixture `fixtures/homes/stale-projects-flow.toml`, golden `fixtures/golden/stale-projects-flow.json`;
the real scanner, engine, cleaner and journal):

1. **Active projects**: nothing of them offered; their artifacts `recent`, the source and `.git` untouched.
2. **A recent commit**: a project whose folders are 400 days old and whose `refs/heads` is 3 days old is `recent`.
3. **The first scan**: every category offered, a monorepo under its name, a stale `target` once (its cache not again),
   an active Rust project's old `incremental` offered, its fresh `llvm-cov-target` `cache-recent` and its `target`
   `recent` with its sizes less the offered cache's; the golden.
4. **Scan → confirm → Trash → put back**: the plan names Medium; every item in the Trash and the journal; put back
   restores them; `src`, `.git`, `dist` and the project files untouched.
5. **A change in a stale project**: offered; a file added to its `src` (the journal scripted); a rescan (asserted
   `ScanKind::Rescan`) leaves its artifacts `recent`.
6. **The setting**: a project 300 days old kept `recent` at 365 days; `unused-for-days` set to 200; the next scan
   offers it.
7. **A restore at launch**: every item `scan-again`, nothing offered.
8. **The storage map**: an offered `node_modules` in the explained part; the project's `src` in the user's folders; the
   browse view opens the project and its `target` (nothing claimed whole).
9. **Ignore**: the build-output category ignored: in no total; restored.
10. **Cleaned once, active after**: one artifact of a stale monorepo removed alone; the next scan keeps the other
    `recent` (the removal moved the project folder's time: the risk, asserted so it stays known).

**UI e2e** (Playwright, mocked core; `app/ui/src/test/stale-projects-scan.ts`, the module's real texts):

1. **Development → Old projects** in the sidebar; its page lists the four categories with their dangers.
2. A finding's title reads "shop · packages/web/node_modules" and its reason names the date and the command.
3. The left-out lines (`recent`, `cache-recent`), not selectable.
4. Select the Python environments and the build output: Review names Medium; confirm; result; Cleanups; put back.
5. Settings → Old projects: the switch and the days.

Visual and accessibility: the module's page, in `visual.spec.ts` and `axe.spec.ts`.

**By hand** (04/02): the sandbox with `EXPECTED.md`; on the developer's Mac a scan only (`npm run dev`), every offered
item checked by hand, this repository's own `target/` among them, the facts "to verify" read.

### 5. Where the code lives

| Crate or folder                    | What it gains                                                                                                   |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `crates/core`                      | `SubtreeQuery`, `Subtree`, `DiskWalk::subtrees`; `ProjectFinder` (moved from the engine); `LastChange`, `Artifact`, `Project.{artifacts, caches, last_activity}`; `ProjectKind::caches` |
| `crates/scanner`                   | `subtrees.rs`: the answers from the walked tree; the `subtrees` bench group                                     |
| `crates/engine`                    | `projects.rs` removed; `session.rs` uses `ProjectFinder` and asks the walk; `ScriptedWalk` answers subtrees     |
| `crates/testing`                   | `NaiveWalk::subtrees`; `run_module` finds the projects                                                          |
| `modules/stale-projects/` (new)    | `fresher-module-stale-projects`                                                                                 |
| `crates/e2e`                       | `projects.rs` updated; the stale-projects flows                                                                 |
| `app/tauri`, `app/ui`              | registration; the UI's test scan and scenarios                                                                  |
| `crates/sandbox`                   | nothing: `[[case.dir]]` ages already make a stale project                                                       |

No new third-party dependency.

## Interfaces with other plans

- **009** (merged): the projects; this plan changes `Project` (artifacts with sizes, caches, `LastChange`) and moves
  the collector to the core.
- **024** (draft): unused toolchain versions read `cx.system().projects()`; it gets `LastChange` and the artifacts'
  sizes for free.
- **020**: `DaySetting`, the `Gathered` and table patterns. **018**: tools' own stores (`~/.cargo`, `~/.npm`) stay
  there. **015**: the group Development and Ignore. **012**: the storage map. **011**: left-outs. **008**: the Trash.

## Deferred items

- *A project's last activity, precisely*: was due here; closed in 01/03, its
  file and row removed.
- [Why an item matched](../../deferred/match-explanations.md): due at "the first rule-based module that needs more than
  its reason"; this module is code, and the date and the command fit in the reason: stays deferred.
- Not due: [the sandbox world](../../deferred/sandbox-world.md) (no open files, tools or steps between scans needed).

## Open questions

None.

## Decisions

The user's, of 2026-11-19 (translated):

1. "`dist` and the rest must definitely not be touched": `dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` and other
   framework output are never offered by this module, not even later.
2. "Maybe think about cleaning nvm too, but it must definitely not be on by default": recorded in
   [unused toolchain versions](../../ideas/unused-toolchain-versions.md) for plan 024; not part of this plan.
3. "The rest seems fine; work autonomously, take the recommended decisions": the draft's ten agent's calls became
   these decisions:
   1. **One module, four categories by what the folder is** (dependencies, build output, environments, Rust cache),
      not one category per language.
   2. **365 days by default**, 30–3650, one setting for the module.
   3. **Python environments are Medium**, every other artifact Low.
   4. **The Rust build cache is offered for active projects too**, after 7 days unchanged; the rest of an active
      project's `target/` is not offered.
   5. **The last commit is read as a file fact; `git` is never run** (the draft named `.git/logs/HEAD`; how it is read
      is agent's call 1).
   6. **Uncommitted changes do not matter**: only regenerable folders are offered, never source; no git status is read.
   7. **Projects under `~/Library`, the apps folders, app bundles and hidden home folders are not projects here.**
   8. **A nested project follows its outermost project's staleness.**
   9. **`dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` are never offered** (decision 1).
   10. **No running-process check**: a stale project has no build running; the cache's 7 days and the cleaner's in-use
       check cover an active one.
4. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code and from this Mac (read-only); each with what it costs if wrong.

1. **The last commit is the newest folder time of `.git/refs/heads`, not `.git/logs/HEAD`'s time** (the draft's
   mechanism corrected; decision 5's "a file fact, git never run" kept). The walk folds `logs/HEAD` into its folder
   and appending to it does not move the folder's time; no pattern matches a file by name; reading it after the walk
   would be a new port and a read a restore cannot make. A commit renames a lock file over the branch's file, which
   moves `refs/heads`. On this Mac, over 15 repositories: the same day as `logs/HEAD` in 13, newer in 1 (a branch
   made after the last commit), and dated in the one that had no `logs/HEAD` at all. A checkout of an existing branch
   moves `logs/HEAD` but not `refs/heads`; it usually rewrites files, which moves their folders. Cost if wrong: a
   project whose only activity is checkouts looks older than it is (its artifacts are Low or Medium and come back).
2. **Last activity counts folders only, never files' times** (the draft's "folders and files"): the walk keeps no time
   for a file under 1 MiB, and counting the big files would tie the answer to the fold threshold. On this Mac the
   newest folder time was never older than the newest file time over 19 git projects (outside artifacts and `.git`).
   Cost if wrong: a project edited only in place, with no new file, no rename and no commit for the whole period,
   looks stale; the editors are checked in 04/02 (to verify).
3. **`.git` is skipped, and status changes are never used**: on this Mac `.git`'s own time and its `index` moved on
   one day in 4 of 15 repositories whose branches were months older (a tool refreshing the index), and a migration or a
   copy moves every status change at once. Cost if wrong: none for safety (both only ever make a project look newer).
4. **The artifacts' sizes and the caches come from the walked tree** (`DiskWalk::subtrees`), not from whole claims
   (the draft's "the Rust cache paths are this module's own interests"): claiming `build`, `target`, `venv` or
   `incremental` whole would make every folder of that name opaque to the storage map and hide the projects inside
   one, the module on or off. Cost if wrong: a new port method and its six implementations.
5. **The finder moves to the core** (not in the draft): `run_module` must give a module the projects the engine would,
   and a module may dev-depend only on `fresher-testing`. Cost if wrong: a moved file.
6. **The caches are core data (`ProjectKind::caches`), 7 days a module constant**: the same shape as the artifacts;
   the 7 days are the draft's and decision 4's, not a setting. Cost if wrong: one more setting later.
7. **A link named like an artifact is never claimed and not reported** (the draft's `artifact-is-a-link` left-out
   dropped): the finder takes folders only, so a link is no artifact; the walk never follows it. Cost if wrong: a link
   that holds gigabytes elsewhere is not explained (it is not this project's to remove).
8. **The home folder and its ancestors are never projects** (not in the draft): an `npm install` run in the home makes
   `~/package.json` and `~/node_modules`; as a project the home would be the outermost project of every project and
   always active. Cost if wrong: `~/node_modules` is never offered.
9. **An unread project keeps its artifacts (`unread`)**: an `AtLeast` time before the days cannot prove the project
   stale, and an artifact read in part has a partial size. Cost if wrong: fewer offers on a Mac without Full Disk
   Access when a project holds a folder the walk cannot read.
10. **A restored scan offers nothing** (`scan-again`), as the modules that judge by age do (`is_restored`'s rule): the
    cached tree can be days old. Cost if wrong: the overview after launch shows the module's kept lines until a scan.
11. **A kept `target` is reported with its sizes less its offered caches'**: a kept line never counts what is offered.
    Cost if wrong: a small helper.
12. **The commands are a table row each** (`artifacts.toml`, checked against the core's artifacts when the module is
    built), `flutter build` for Dart's `build` and `python3 -m venv <folder>` for the environments: rule 5, and a new
    artifact in the core cannot ship without its row. Cost if wrong: a text.
13. **A worktree or a submodule (a `.git` file) is judged by its own folders**: following the file needs a read; its
    commits rename files in the main repository's `refs/heads` and usually change files in the worktree. Cost if
    wrong: a worktree used only for commits looks older (worktrees are short-lived and active).
14. **The cache patterns include a target triple's profile** (`target/*/*/incremental`): a cross build keeps its own
    incremental folders. Cost if wrong: one pattern.
15. **A group's last activity is the newest of its members' own** (the lead's ruling R1 on finding C12 of the
    preflight; the design said "its outermost project's"). The outermost project's own folders skip a nested
    project's `.git`, so a commit in a nested repository moved nothing the outermost counts; the newest member's time
    keeps the whole group active. `Known` only when every member's is `Known`, so a member that is `AtLeast` or
    `Unknown` keeps the group from being stale (an old `AtLeast` makes it `unread`, a recent one `active`). Monotone:
    a later change of any member never turns a keep into an offer (a property test). Cost if wrong: a monorepo with
    one undatable package is never offered.
16. **An artifact with no table row is not claimed** (ruling on finding C5): the core lists only artifacts the table
    has rows for, so this never happens with the shipped table; a row of another kind, an artifact not right inside
    its project's root, or a name that is not text is not claimed either. A cache is decided only inside its own
    project's `target`, and only when the table has the Rust `target` row (its command). Cost if wrong: none for
    safety.
17. **A restored scan offers nothing** (ruling R2, as call 10 says): every artifact and every cache is left out
    `scan-again`, whatever the times.
18. **`AtLeast(t)` never makes a group stale** (ruling R3): the group table's row 3 already says so (an old `AtLeast`
    is `unread`), so the design is followed as written. Sizes that are `Partial` **or `Unknown`** count as read in part
    (`unread`): only `Complete` sizes are offered.
19. **A folder claimed whole below a project makes its activity `AtLeast`** (a ruling of the review of 01/01): the walk
    cannot tell what changed inside it, so it counts with its own time and the answer is incomplete, never `Known`.
    Cost if wrong: a project with a whole-claimed folder is never offered.
20. **Two known risks are lived with, not fixed** (known issues): only folder times count, so a file edited in place
    moves none (the commits, the 365-day threshold, the regenerable-only artifacts and the user's confirmation limit
    the harm: [the issue](../../known-issues/core-project-activity-misses-edits-in-place.md)); and removing an artifact
    moves the project folder's own time, so a project cleaned once looks active for the set days (core e2e flow 10 pins
    it: [the issue](../../known-issues/core-cleaned-stale-project-looks-active.md)).
21. **A project folder is never offered as another's artifact** (the final review's Important 1): the finder keeps a
    project whose root is another's artifact folder (plan 009's rule, a Gradle `build` holding a `package.json`), so
    the module does not claim an artifact or a cache that is a found project's root or holds one. Cost if wrong: such
    an artifact is never offered (rare; it is a project).
22. **An artifact changed within the days is kept, even in a stale project** (the lead's ruling on the final review's
    Minor 6): a `pip install` into an old project's `.venv` moves only the `.venv`, so the group stays stale, but a
    change made last week reads as "in use". The artifact's own `last_change` (the finder's, of it and every folder
    inside it) within the days or in the future keeps it `recent`; not `Known` keeps it `unread`. The artifacts still
    do not count in the group's activity: one recent artifact keeps only itself. Cost if wrong: fewer offers.

## Conclusions

Not written yet.
