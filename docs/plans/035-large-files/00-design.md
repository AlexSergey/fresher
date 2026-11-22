# 035 — The largest files: design

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, scanner, engine, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the draft [`readme.md`](./readme.md) and from a read of the code on `main` at 33d51a6c
(plans 033 and 034 merged), read-only. Approved by the user's delegation of 2026-11-22: build autonomously, take the
recommended decisions and the agent's calls; the user checks everything later. Where the code or the facts of this Mac
forced a different choice than the draft, the design says so and the choice is an agent's call.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

A handful of forgotten files often holds most of the space. Since plan [027](../027-scan-screen/readme.md) the app shows
nothing outside its categories: the map's "Your folders" is one number, and the core's place queries wait without a UI
([deferred](../../deferred/place-queries-without-ui.md)).

## Goals

1. One screen, **Largest**, answers "what are my biggest files and folders", from the scan already made: no second
   walk.
2. The user's own files are never called junk, never pre-selected and never shown on Scan; a row the user ticks goes
   through the normal Review and confirmation at **High**, to the Trash by default.
3. What the app could not see is said, not hidden.

## Non-goals

- A treemap or sunburst (later, a second view of the same rows); duplicates and similar images (their own idea).
- Anything inside `~/Library`, inside an app bundle, inside the Trash, or in a place a module already explains.
- "Last opened" and "the app that made it" (agent's call 3 and 4): postponed with what they need in
  [`../../deferred/largest-files-more.md`](../../deferred/largest-files-more.md).
- Places outside the home folder (another scan root, a volume): the screen is the home's.

## Facts checked on this Mac (2026-11-22, macOS 27.2.1, read-only)

| Fact                                       | What was found                                                                                                                                        |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Spotlight's last-used date                 | `mdls -name kMDItemLastUsedDate` is `(null)` for every sampled large archive and disk image in `~/Downloads` and for a 9 GB memory dump at the home's top; `mdfind -onlyin ~ 'kMDItemLastUsedDate >= $time.today(-30)'` finds 87 items. Indexing is on (`mdutil -s /`). It is set only when a file is opened through the system's launch services. |
| The access time                            | On the data volume (APFS, no `noatime` in `mount`), a read moves the access time only when it is not newer than the modification time: tested on a scratch file (access 2025 > modification 2024, `cat`, the access time stayed). So the access time is the first read after the last change, not the last read. |
| A cloud file not on the disk               | the bulk reader sets `cloud_only` from `SF_DATALESS` and gives it 0 allocated bytes (`crates/macos/src/bulk/parse.rs`).                                |
| Synced folders                             | no File Provider domain (`~/Library/CloudStorage` empty); iCloud Drive lives in `~/Library/Mobile Documents`: inside `Library`, out of scope.           |
| A guarded folder                           | listing `~/Movies/TV` from a terminal without the access gives "Operation not permitted": the scan reports such a folder as not read, with why.          |
| What the tree keeps                        | every folder; every file of 1 MiB and up as its own node, with its modification time and sizes; an access time only for matched nodes; a claimed folder is opaque (nothing inside is a node). |

## What already exists

| Need                                        | What exists                                                                                                     | Plan     |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| The scan's tree                             | `DiskState` (`node`, `children`, `path`, `find`, `owned`, `largest_files`), `TreeState` in the scanner, `FixedDiskState` in `fresher-testing` | 004, 012 |
| What modules explain, per node              | `Shares::of(state, ignored)`, `Shares::explained(state, id)`, kept beside the state                             | 012      |
| A folder's children with their explained part | the engine's `storage::place::place_view` (`Descent::Stay`)                                                    | 012      |
| What was not seen, and how to allow it      | `LastScan.coverage` (`Coverage.not_entered`, `unreadable`), `screens/permission/coverage-notices`               | 006      |
| The home and the standard folders           | `Engine::home()` (`KnownFolders`); the cleaner's `Protection` (`whole`: the home, its standard folders, the Library's; `never`) | 008, 009 |
| One confirmed removal                       | `Finding` → `PlanItem` → rehearsal → confirmation → the cleaner, the journal, put back                           | 008      |
| A selection of single findings              | `state/selection.ts` (`toggle-finding` of a `Finding`), Review's "picked" groups by category                     | 008, 027 |
| A core-owned id namespace                   | `ModuleId::system()` (`core`)                                                                                    | 009      |

Nothing in the walk changes: the rows are read from the committed tree.

## Design

### 1. What a row is

The screen has two views of one scan, **Files** and **Folders** (agent's call 1).

- **Files** ranks **items** below the home, the largest allocated size first: every kept file (1 MiB and up) and every
  **package** (a folder that is one document: a virtual machine, a photo library, a project bundle; a table, section
  2), counted as one item and never opened. The walk below the home, per node:
  - **skipped with everything below**: a node a module claimed (`owned`), a name that starts with `.`, `Library` and
    `.Trash` directly under the home, an app bundle (`.app`), a link, a special entry, a hard link counted elsewhere,
    and a folder whose whole allocated size is below the minimum (nothing in it can reach it);
  - **an item**: a kept file, a package, a collapsed subtree;
  - **entered**: any other folder.
  An item below the minimum size (`core.largest.min-size`, 100 MB by default) is not listed. A cloud file not on the
  disk takes 0 allocated bytes and is never listed: removing it frees nothing (agent's call 5). At most 10 000 items
  are kept (`MAX_ITEMS`), the largest; the answer says when more were left (`truncated`).
- **Folders** is the place view brought back: the home's children largest first, each with its explained part and
  whether it can be chosen; a folder row opens its children, with the way back to the home (a breadcrumb). This closes
  [`place-queries-without-ui`](../../deferred/place-queries-without-ui.md): the view returns as this screen's, and the
  Tauri command `get_storage_place`, which nothing calls since plan 027, is removed (the engine keeps
  `storage_place`, which the sandbox's map check uses).

### 2. Kinds (`fresher-core`, `largest::kind`, data)

`ItemKind` names what a row is, from its name's extension (case ignored) and whether it is a folder, by two tables in
the core (rules as data, rule 5); the UI shows `core.largest.kind.<kind>`.

| Kind               | Files                                                    | Packages (folders)                                             |
| ------------------ | -------------------------------------------------------- | -------------------------------------------------------------- |
| `disk-image`       | `dmg`, `iso`, `img`, `sparseimage`, `toast`              | `sparsebundle`                                                 |
| `archive`          | `zip`, `7z`, `rar`, `tar`, `gz`, `tgz`, `bz2`, `xz`, `zst`, `xip` | —                                                     |
| `installer`        | `pkg`, `mpkg`                                            | `pkg`, `mpkg`                                                  |
| `video`            | `mov`, `mp4`, `m4v`, `mkv`, `avi`, `wmv`, `webm`, `mts`  | —                                                              |
| `audio`            | `mp3`, `m4a`, `wav`, `aif`, `aiff`, `flac`, `aac`         | —                                                              |
| `image`            | `jpg`, `jpeg`, `png`, `heic`, `tif`, `tiff`, `psd`, `raw`, `dng`, `cr2`, `nef` | —                                        |
| `virtual-machine`  | `vdi`, `vmdk`, `qcow2`, `vhd`, `vhdx`                    | `vmwarevm`, `pvm`, `utm`                                       |
| `media-library`    | —                                                        | `photoslibrary`, `musiclibrary`, `tvlibrary`, `imovielibrary`, `fcpbundle`, `aplibrary` |
| `project`          | —                                                        | `logicx`, `band`, `xcarchive`                                  |
| `memory-dump`      | `hprof`, `dmp`, `core`                                   | —                                                              |
| `document`         | `pdf`, `key`, `pages`, `numbers`, `docx`, `pptx`, `xlsx` | —                                                              |
| `device-backup`    | `ipsw`                                                   | —                                                              |
| `file` / `folder`  | anything else                                            | — (a folder not in the table is a plain folder)                |

`file` and `folder` are not a catch-all category: they are the row's real kind, shown with its real name (rule 3 is
about categories). `app` is not a kind: an app bundle is never listed (the Apps screen of plan 036 is its place).

### 3. Can a row be chosen (`Offer`, the engine, one function)

The engine judges every row, in Files, in Folders and when a row is picked, by one pure function, the first that
holds:

| #   | When                                                                    | Verdict                       | Opens (Folders) |
| --- | ----------------------------------------------------------------------- | ----------------------------- | --------------- |
| 1   | a module claims it or a folder above it (`owned`)                       | withheld `explained`          | no              |
| 2   | the home's `Library` (`KnownFolder::Library`)                           | withheld `library`            | no              |
| 3   | the home's Trash (`KnownFolder::Trash`)                                 | withheld `trash`              | no              |
| 4   | its name starts with `.`, or it is inside such a folder or a package    | withheld `hidden`             | no              |
| 5   | an app bundle (`.app`)                                                  | withheld `app`                | no              |
| 6   | a link                                                                  | withheld `link`               | no              |
| 7   | not scanned (`not_scanned` is set: guarded, avoided, unreadable, another volume) | withheld `not-seen`  | no              |
| 8   | the cleaner's protection refuses it as a whole folder (`Protected::WholeFolder`: the home, the standard folders) | withheld `standard-folder` | yes |
| 9   | the cleaner's protection refuses it otherwise (`Never`, `Ancestor`)                 | withheld `protected`          | no              |
| 10  | a module claims something below it, its category ignored or not         | withheld `holds-explained`    | yes             |
| 11  | read in part (its sizes are not complete)                               | withheld `read-in-part`       | yes             |
| 12  | at, below or above a path of the ignore list (`core.ignore.paths`)      | withheld `ignored`            | above: yes      |
| 13  | otherwise                                                               | **offered**                   | a folder: yes; a package: no |

Only a folder that is not a package opens, and a row is reached (opened, listed as offered in Files, or picked by its
id) only when every folder between the home and it opens; otherwise it takes the reason of the first folder that does
not. Each withheld reason has its sentence, `core.largest.withheld.<reason>` (section 7). The cleaner remains the last
wall: it refuses the standard folders (`WholeFolder`), protected places, a link's target, an item held open (a package
in use: check 10), a lock or a foreign owner at the rehearsal, each with its reason in the confirmation.

### 4. The queries (the engine, `largest.rs`)

- `Engine::largest(&LargestQuery) -> LargestPage`: refused `NoScan` before a scan and `Busy` while one runs, as the
  place view. The items of a state (section 1) are computed once per state and minimum size and kept beside the state
  (as `Shares`): a list of `Item { id, path, allocated, kind, modified, top }` (`top`: the folder directly under the
  home that holds it). A page then filters that list in memory:
  - `kind: Option<ItemKind>`, `place: Option<NodeId>` (one of the home's folders), `unchanged: Unchanged`
    (`any`, `month`, `half-year`, `year`, `two-years`, counted back from the state's `taken()`, never the clock);
  - drops what the user removed since the scan (section 5) and what the ignore list holds;
  - answers `LargestPage { rows, offset, total, sizes, kinds, places, truncated, min_size }`: `rows` a page (at most
    100) of `LargestRow { node, place, kind, offer, unpacked, picked }`; `sizes` the filtered rows' sum; `kinds` and
    `places` the counts the filters offer (`KindCount`, `PlaceCount` with the folder's `NodeInfo`).
  - `place` is the row's folder written from the home (`~/Movies/Trips`, lossy for display only).
  - `unpacked`: for an `archive`, the name of a folder beside it named as the archive without its extension
    (`Foo.zip`, `Foo.tar.gz` → `Foo`): "a folder named `Foo` beside it may hold what was unpacked from it" (the
    draft's likely leftover, pointed out, never decided for the user).
  - `picked`: the finding id when the row was picked (section 5).
- `Engine::largest_place(Option<NodeId>, offset, limit) -> OwnPlace`: the folder (`None`: the home), its way from the
  home (`path`, home first), and a page of its children largest first (`OwnRow { node, explained, kind, offer, opens,
  picked }`), over `place_view(…, Descent::Stay, …)`. A node below the home only; a node of another state is
  `StaleNode`, a node outside the home `NotOwnItem`.
- The items query is one pass over the tree in the scanner (section 6); its budget, as the map's shares: under 50 ms
  and 5 MB on the `LARGE` tree (rule 4), measured by an ignored e2e test run by hand.

### 5. A row picked, reviewed and removed (the engine, `own.rs`)

A ticked row becomes a **finding of the core's own category** so that everything after it is the existing path: the
selection, Review, the plan, the rehearsal, the confirmation, the cleaner, the journal, history and put back (agent's
call 2).

- `Engine::pick_own_item(NodeId) -> Finding`: refused `NoScan`, `Busy`, `StaleNode`, and `NotOwnItem` unless the row is
  offered (section 3, judged again now). The same node answers the same finding (idempotent).
- The finding: module `core` (`ModuleId::system()`), category `core.own-file` (`OWN_FILE`, High), key
  `own/<path from the home>`, `Subject::Path` and `Removal::MoveToTrash` of the node's path, the node's sizes, title
  `core.own-file.title` (`name`), reason `core.own-file.reason.file` or `.folder` ("This is your own file; Fresher cannot
  tell whether you still need it."), removability `Removable` (the rehearsal reads the disk and refuses what it must;
  agent's call 7). The id is the next free id of the session (`Results::next_id`, bumped).
- Kept in `Results.own` (id order), apart from the scan's findings: never in the overview's totals, a category card,
  the storage map's explained part or a findings page. `Results::finding(id)` and the plan builder's `expand` look in
  both lists; a scan's `begin` and a new state forget `own`.
- After a cleanup, `Results::remove` drops removed own findings too and remembers their paths (`gone`), so Files and
  Folders leave those rows out until the next scan; sizes of the folders above stay the scan's (the screen says the
  scan is out of date, `LastScan.changedSince`, as Scan does).
- The category `core.own-file` has the five texts of every category (rule 3) and a catalogue file,
  [`../../categories/core-own-file.md`](../../categories/core-own-file.md). It is not a module's category: no switch,
  no card, nothing pre-selected; it exists only for the rows the user picks.

### 6. The scanner's items query (`DiskState::items`)

One new trait method, the only change to the scanner and to `fresher-testing`'s `FixedDiskState`:

```rust
/// How the items query treats one node below its root, by its name, kind and depth (a child of the root is 1).
pub trait ItemRule {
    fn visit(&self, name: &str, kind: NodeKind, depth: u32) -> ItemVisit;
}
pub enum ItemVisit { Skip, Item, Enter }

/// The items below `root` (never `root` itself), the largest allocated first, ties in path order, at most `limit`:
/// a node `rule` calls an item, of at least `min` allocated bytes. Never enters an owned node, a link, a special entry
/// or a folder `rule` skips; never answers an owned node or a hard link counted elsewhere; does not enter a folder
/// smaller than `min`. `None` for a node of another state.
fn items(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, limit: u32) -> Option<Vec<NodeId>>;
```

The engine's rule (section 1) is `HomeItems { library, trash }`; the scanner knows nothing of homes, packages or names.
The walk, the cache format and the tree's memory do not change, so no scan bench is due; the query is measured
(section 4).

### 7. Texts

The core's (`crates/core/locales/en.json`), drafts polished when built (rule 7):

- `core.own-file.name` "Your own files"; `.what` "Files and folders you picked yourself on the Largest screen."; `.made-by`
  "You, or the apps you use."; `.if-removed` "They go to the Trash. Fresher cannot tell what they hold or whether you
  still need them."; `.comes-back` "No. Put them back from the Trash or from Cleanups if you need them."
- `core.own-file.title` "{name}"; `core.own-file.reason.file` "This is your own file; Fresher cannot tell whether you
  still need it."; `.reason.folder` "This is your own folder; Fresher cannot tell whether you still need what is in
  it."
- `core.largest.kind.<kind>` ("Disk image", "Archive", "Installer", "Video", "Audio", "Image", "Virtual machine",
  "Media library", "Project", "Memory dump", "Document", "Device software", "File", "Folder").
- `core.largest.withheld.<reason>`: `explained` "Fresher explains this on Scan; choose it there."; `hidden` "Kept by an
  app or a tool for itself. Fresher cannot tell what it holds."; `library` "Your apps keep their own files here. Scan
  shows what of it can go."; `trash` "What is already in the Trash. Empty it from the Finder."; `app` "An app. It is not
  removed from here."; `link` "A link to another place. Only the link would go."; `not-seen` "Fresher could not look
  inside."; `standard-folder` "A standard folder of your Mac. Open it to choose what is inside."; `protected` "macOS
  or Fresher protects this place."; `holds-explained` "Part of it is explained on Scan. Open it to choose the rest.";
  `read-in-part` "Fresher could not read all of it, so it cannot tell what it holds."; `ignored` "You chose to ignore
  this place."
- `core.largest.unpacked` "A folder named {folder} beside it may hold what was unpacked from it."
- The setting `core.largest.min-size` (a choice, group `scan`, effect `now`): options `10-mb`, `50-mb`, `100-mb`
  (default), `500-mb`, `1-gb`, sizes in decimal units (10 MB is 10 000 000 bytes, as the Finder counts); label
  "Smallest size on Largest", description "Files and folders smaller than this are not listed on the Largest screen.
  Nothing is removed by this setting."; one text per option ("10 MB", …).
- `core.largest.unchanged.<age>` for the filter ("Any time", "Not changed for a month", … "for two years").
- A new `CommandError::NotOwnItem` with its refusal `core.error.not-own-item` "This item cannot be chosen here.".

The UI's chrome (`app/ui/src/locales/en.json`, `ui.screen.largest.*`): the nav entry "Largest", the heading "Largest
files and folders", the view tabs "Files" and "Folders", the filters' labels, "Show more", the empty states ("Scan
first to see your largest files.", "Scanning. The list comes back when the scan ends.", "Nothing here is 100 MB or
larger." with the size from the answer), "Changed {when}", the truncated note, the out-of-date note.

### 8. The screen (`app/ui/src/screens/largest/`)

- A sidebar entry **Largest** after Scan (`Screen` `{ kind: 'largest' }`); the view, the filters and the open folder
  are the screen's own state.
- Files: three filters (kind, place, unchanged for) above a list of rows, 100 a page with "Show more"; a row (product
  component `product/own-item-row/`): a checkbox, the name, the place, the kind, the size, "Changed …", the hint
  `unpacked`, or, withheld, the reason's sentence and no checkbox. The total of the filtered rows and their count in the
  heading.
- Folders: the breadcrumb from the home, the rows with their explained part, a folder that opens is a button.
- Ticking a row calls `pickOwnItem(node)` then toggles that finding in the selection; unticking toggles it off. The
  selection bar shows on Largest as on Scan; Review lists "Your own files" in the High zone with its "if removed"
  sentence; nothing is ever pre-selected (no "Select all" on this screen).
- Above the list, the last scan's coverage notices (`CoverageNotices`), so the folders not seen and how to allow them
  are said here too; `truncated` shows "Only the 10,000 largest are listed."
- Before a scan, during a scan and after a cleanup the screen says so (section 7); the list is asked again when a scan
  ends and when findings are removed.

### 9. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit (core)**: the kind tables (case, double extensions, a folder not in the table, `.app` is no kind), the
  query types' serialization, the setting's options, every new key has its text.
- **Unit (scanner)**: `items` on built trees: owned, link, hidden by the rule, a folder below the minimum not entered, a
  hard link counted elsewhere, another state's node; then items ranked, a package counted whole, ties in path order,
  the limit.
- **Unit (engine)**: the rule; every row of section 3; the filters, `unchanged` from `taken()`, the cache built once
  per state and minimum, ignored and gone paths; `unpacked`; the place view and its breadcrumb; the pick (refusals,
  idempotence, the finding's fields), `expand` with own findings, `remove` with own findings, `begin` forgetting them.
- **Core e2e** (`crates/e2e/tests/largest.rs`, a fixture home through the real scanner, the trial module and the real
  cleaner in a sandbox): the list and the folders view; a claimed folder, `Library`, `.Trash`, a hidden folder, an app
  bundle never listed; a picked file to the sandbox's Trash, gone from the list, put back; a picked folder; a restored
  scan lists the same; a budget test on `LARGE`, ignored, run by hand.
- **Sandbox**: `fixtures/sandboxes/large-files.toml` on the `trial` module with a new `[case.largest]` expectation and
  its golden report; files are small, filled with data, under a minimum of 10 MB (agent's call 6).
- **UI**: the row component and its stories, the screen's unit tests, the e2e flows, visual baselines and axe.

## Risks

- **The user removes something they need**: High, never pre-selected, the reason's sentence, Review and the
  confirmation, the Trash, put back. Rows that a module or the system owns are withheld with why.
- **A synced folder outside `Library`** (a legacy sync client's folder in the home): its files are offered like any
  other; removing one may remove it in the cloud too. No marker is read in this plan (agent's call 8); the fact is the
  user's to check.
- **A package in use** (a virtual machine running): it is offered; the cleaner's open-file check at the rehearsal and
  at the act skips it `InUse`.
- **A folder's "Changed" date** is its own modification time, not its newest file's
  ([deferred](../../deferred/folder-age-from-children.md)): a folder of old files that just had one added reads as new.
- **The folders' sizes after a cleanup** stay the scan's until a rescan; the removed rows leave at once.
- **Rebase onto plans 037, 031 and 032**: the shared files are listed in the [readme](./readme.md); each is an append or
  a new arm, so the rebase is mechanical.

## Open questions

None.

## Decisions

- The user's delegation of 2026-11-22 (translated): "work autonomously with subagents, take the recommended decisions;
  implement all the features; write into docs/deferred what I will need to look at." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong.

1. **Two views, Files and Folders**, instead of one ranked list of files and folders together: a folder holds the
   files ranked beside it, so one list would count bytes twice. Cost if wrong: one tab more.
2. **A picked row becomes a finding of the core's category `core.own-file` (High)**, kept apart from the scan's
   findings, so Review, the plan, the cleaner, the journal and put back are reused unchanged. Cost if wrong: a category
   that is not a module's appears in Review and history.
3. **No "last opened" in this plan; "Changed" (the modification time) instead**, against the draft. Spotlight's
   last-used date is missing for nearly every file, and the access time is only the first read after the last change
   (facts, above): neither can say "not opened for a year" truthfully (rule 1). Deferred with what it needs. Cost if
   wrong: the filter is "not changed for", weaker than "not opened for".
4. **No "the app that made it"**: it needs Spotlight's metadata per row, a new port; the kind and the name say most of
   it. Deferred. Cost if wrong: one column fewer.
5. **Ranked by allocated size**: what removing frees. A cloud file not on the disk and a sparse file are not listed.
   Cost if wrong: a large sparse disk image shows its real, small cost.
6. **The sandbox uses small files filled with data and a minimum of 10 MB**, not sparse files (the draft), since the
   list ranks by allocated bytes. Cost if wrong: about 120 MB written per sandbox run.
7. **Removability `Removable` for a picked row**, with the cleaner's rehearsal as the judge: the tree does not keep an
   entry's facts outside matches, and the rehearsal reads them from the disk anyway. Cost if wrong: a locked file shows
   its refusal at the confirmation instead of on the row.
8. **No synced-folder marker** (a folder of a sync client in the home): none exists on this Mac and the system's sync
   folders live in `Library`. Cost if wrong: a file of such a folder is offered at High.
9. **Hidden names are never offered** (`~/.android`, `~/.gradle` and the like): they are tools' data, which modules
   explain; rule 1. Cost if wrong: a large tool folder no module knows stays unlisted.
10. **The minimum is a setting with five choices** (10 MB to 1 GB, 100 MB by default), not a free number: the settings
    screen shows a byte count as bytes. Cost if wrong: none.
11. **`get_storage_place` is removed from the Tauri commands**; the engine's `storage_place` stays for the sandbox's
    map check. Cost if wrong: one command to add back.
12. **A place a module claims is skipped even when the user ignored that module or category**: the screen lists the
    map's "Your folders" as the scan claimed them, and a folder or package holding such a place is withheld
    `holds-explained` (never offered whole). Cost if wrong: an ignored module's folders do not appear here.
13. **At most 10 000 items are kept per state**: a home with more files over the minimum lists the largest. Cost if
    wrong: the smallest of a huge list are not shown, and the screen says so.

## Conclusions

The plan was built as designed; what changed and the numbers are in [the global readme](./readme.md#conclusions).
Calls 2, 3, 5, 7, 8, 9, 12 and 13 stand and are on the user's review list; the judgement changed in two rounds of
review (table rows 1, 4, 10 and 12, call 12), as the design now says.
