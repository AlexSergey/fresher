# 044 — Big folders left alone, on the Largest screen: design

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, scanner,
engine, modules, testing, sandbox, e2e, app/tauri, app/ui, docs

The design the plans argue from. Written from the draft of 2026-11-26, the user's answers of the same day and a read
of the code on `feat/044-largest-folders` (f12bdcf4). Each step reads its code again before it changes it.

## Intent

The first tester asked where the rest of the disk went
([`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)); the user extended it
(translated, 2026-11-26): "A section like Large Files that shows files and folders, does not offer to remove, only
opens the Finder." Largest gets two sections: **Files** as today (removable at High after the confirmation) and
**Folders**: big folders nobody changed for a long time, with Show in Finder only. It takes the idea
[Big projects left alone](../../ideas/big-stale-projects.md). Later the same day the user took every recommended
decision ("further by the recommended ones") and, asked again once it was clear that nothing else in the app browses
folders, kept today's folder tree as a third tab, **Browse**, read-only: "the same tree with no ticks, only going
into folders and Show in Finder; only files stay removable on Largest".

## Goals

1. Largest has three tabs: **Files** (unchanged: the largest files, packages and folded subtrees, ticked one by one,
   removed at High after the confirmation), **Folders**: the folders of the home over a size whose newest change is
   older than the stale projects' days, and **Browse**: today's folder tree, read-only (go into a folder, Show in
   Finder; no tick on any row).
2. A folder row shows its name, the folder it is in (from `~`), its size, how long nothing in it changed, and
   "Project" when it holds a `.git` folder. Its only action is **Show in Finder**: no tick, no pick, nothing sent to
   the cleaner.
3. The rule: the highest folder of each branch that qualifies is listed, never one inside a listed one; a folder that
   holds a recent change is not listed, its stale children may be.
4. Never listed: `~/Library`, the Trash, the home itself and its standard folders (their children may be), app
   bundles, packages, hidden folders, links, protected places, ignored places, and a folder a Scan category explains
   whole.
5. The newest change is folded into the scan tree during the walk (small files included), kept in the cache and kept
   true by a rescan: the list costs no second walk and reads nothing from the disk.
6. The size is a new setting (default 1 GB); the age is the stale projects' days (default 365).
7. The screen balances data and beauty: a bar of the home with the listed folders, a separator, tiles, then the rows;
   its heading says what to do. A mockup is approved before the screen is built.

## Non-goals

- Removing a folder from Largest, in any way: the picks of plain folders go, and Browse ticks nothing.
- Running `git` (uncommitted or unpushed work): a later plan may add it as a line.
- Places outside the home, other scan roots, other volumes.
- A module's folder age ([`../../deferred/folder-age-from-children.md`](../../deferred/folder-age-from-children.md)):
  it needs the age in a module's `Match` during the walk, which the tree's `newest` does not feed. It stays deferred.

## What already exists (read on f12bdcf4)

| What | Where |
| --- | --- |
| The scan tree's node | `crates/scanner/src/tree/node.rs`: `Node` (72 bytes, `a_node_takes_72_bytes`), `modified: i64` (`NO_TIME = i64::MIN`) is the entry's own time only, set by `Tree::apply` (`tree/apply.rs`), the rescan's `copy`/`name_hits` (`rescan/patch.rs`) and the codec (`codec/decode.rs`). Sums hold a node's own bytes until `freeze` and its subtree's after (`arena.rs::sum_bottom_up`) |
| The fold | `tree/fold.rs::fold`: files under 1 MiB (`KEEP_THRESHOLD`) fold into `FolderBatch.folded` without a time; folders, large and matched files are `Kept` with their `RawEntry` (and its `modified`) |
| The entry's times | `RawEntry` (`crates/core/src/folder.rs`) already carries `modified`, `changed` (the status change, ctime: "moved by every write, never set back by a tool") and `accessed` for every entry of a listing, folded ones too: the walk needs no new field |
| The cache | `codec/header.rs::TREE_FORMAT = 6`; a record writes `modified` as a delta from its parent's (`encode.rs`, `decode.rs`); a cache of another format is refused (`OtherFormat`) and the scan reads everything once. Budgets: heap ≤ 96 B a node (`heap_bytes_per_node_stays_under_96`), cache ≤ 32 B a node ([`../../testing.md`](../../testing.md)) |
| The rescan | `rescan/patch.rs`: a folder read again gets its new listing; a copied node gets its own sums from `Frozen::own` (subtree minus children), which a maximum cannot do |
| The state | `DiskState` (`crates/core/src/disk_state.rs`) with `items(root, rule: &dyn ItemRule, min, limit)`; `ItemVisit::{Skip, Item, Enter}`. Implemented by `TreeState` (`crates/scanner/src/state/{mod,items}.rs`), `FixedDiskState` (`crates/testing/src/fixed_disk_state.rs`) and two engine test doubles (`SlowState` in `crates/engine/src/disk_tests.rs`, `Counting` in `largest/largest_tests.rs`) |
| Largest's Files | `Engine::largest` (`crates/engine/src/largest/mod.rs`) over `Items::cached` with `HomeItems` (`largest/rule.rs`): top-level `Library` and Trash, hidden names, `.app` and links skipped; **packages and collapsed subtrees are items too** (not only files); folders are entered |
| Largest's Folders | `Engine::largest_place` → `own_place` (`largest/place.rs`, with `Ground::gone_children` and `way`), `OwnRow`/`OwnPlace` (`crates/core/src/largest/page.rs`); IPC `get_largest_place`; UI `screens/largest/folders-view.tsx` and the `OwnRow` branch of `product/own-item-row/` |
| The judgement | `largest/judge.rs::judge` → `(Offer, opens)` in a fixed order (explained, library, trash, hidden, app, link, not seen, standard folder, protected, holds explained, read in part, ignored); `Ground::judge`/`judge_reached` (`largest/mod.rs`); `Withheld` has 12 reasons |
| The pick | `Engine::pick_own_item` (`crates/engine/src/own.rs`) → `pickable` (`largest/mod.rs`); re-judged at prepare and confirm (`check_picks`, `still_offered`) |
| Reveal | `Engine::reveal_finding` (`crates/engine/src/reveal.rs`) → `ports.system_ui.reveal(path)`; `FakeSystemUi` records `SystemUiCall::Reveal` |
| Projects | `Project` (`crates/core/src/system/projects.rs`) is built during the scan into `SystemFacts` (`crates/engine/src/session.rs:130`), not kept in `Committed`, and a state read from the cache has none |
| Stale days | `UNUSED_FOR = DaySetting::new("unused-for-days", 30, 3650, 365)` (`modules/stale-projects/src/lib.rs:38`), private to the module; the engine resolves module settings by `Settings::module(id)` |
| The size setting | `core.largest.min-size` (`crates/core/src/settings/core_schema.rs`): a Choice, group Scan, effect Now, read by `Settings::largest_min_size` (`crates/engine/src/settings.rs`); texts `core.largest.min-size.{label,description,options.*}` (`crates/core/locales/en.json`) |
| IPC | `app/tauri/src/largest_commands.rs` (`get_largest`, `get_largest_place`, `pick_own_item`), registered in `app/tauri/src/lib.rs` and allowlisted in `app/tauri/build.rs` and `app/tauri/src/dev_runs/commands.rs`; `apps_commands.rs`' test uses `get_largest_place`; UI wrappers in `app/ui/src/ipc/index.ts` |
| UI pieces | `design/data/usage-bar`, `design/layout/{tile,section-separator,surface,tabs}`, `design/data/list-row`; texts `ui.screen.largest.*` (`app/ui/src/locales/en.json`, plurals as `.one`/`.other`) |
| Tests | Rust: `largest/{largest_tests,judge_tests}.rs`, `own_tests.rs`, `crates/e2e/tests/{largest.rs,largest_home/,largest_budget.rs}`, `fixtures/golden/largest-flow.json`; sandbox `crates/sandbox/src/largest.rs` (`Shown::{Offered,Absent,Withheld,NotSeen}`), harness `app/tauri/tests/sandboxes/largest.rs`, `fixtures/sandboxes/large-files.toml` (one `withheld` case: `home/Public`). UI: `largest-screen.test.tsx`, `folders-view.test.tsx`, e2e `flows.spec.ts` (Folders at :394, :1023, :3262), `visual.spec.ts` (`largest-folders.png`), `axe.spec.ts` (:505), `keyboard.spec.ts`, `core-mock.spec.ts` (:492), `e2e/support/largest-mock.ts` |
| Mockups | `docs/mockups/<version>/*.dc.html`, changed only with the user's approval ([`../../mockups/README.md`](../../mockups/README.md)) |

Three facts of the recon were wrong and are corrected here:

- **The storage map does not browse.** The engine still has `Engine::storage_place`, but no command calls it since
  plan 035 and the UI dropped "by place" in plan 027; Largest's Folders tree is the only way to look inside a folder in
  the app. Told so, the user kept the tree as the read-only tab Browse (Decisions).
- **Files lists more than files**: packages and folded subtrees are Files items and stay pickable. Only a plain folder
  (`NodeKind::Folder`, not a package) is refused by `pick_own_item`.
- **Projects are not reachable from the committed state** (they live in the scan's `SystemFacts`), so the Project
  label reads the tree: a folder with a `.git` child folder (agent's call 8).

## Design

### 1. The newest change in the scan tree (plan 01)

- `Node` gains `newest: i64` (`NO_TIME` for none), with the sums' rule: the node's own value until `freeze`, its
  subtree's after. `Node` grows to 80 bytes (`a_node_takes_80_bytes`).
- **A change.** An entry's change is the later of its `modified` and its `changed` (ctime): a file copied, unzipped
  or restored keeps an old `modified` but gets a new `changed`, so a folder made of such files is not called old
  (agent's call 18). A `changed` moved by a metadata change only (a rename, a permission, an extended attribute) makes
  a folder look newer: it hides a folder from the list, never lists a wrong one.
- **Own value.** `FolderBatch` gains `newest: i64`: the newest change of every entry of the listing that was read
  and is a file, a link or a special entry, large or small, except one named `.DS_Store`. A folder's own time is left
  out, its own and its subfolders' (agent's call 1: the Finder writes `.DS_Store` and so changes the folder's time when
  the user only looks). `fold` fills it; `Tree::apply` sets the folder's `newest` to it (`Tree::join_newest(index,
  bits)`, the larger of the two).
- **Subtree value.** `sum_bottom_up` also takes `parent.newest = max(parent.newest, child.newest)`. `.git` is walked
  like any folder, so a commit (its objects and refs) counts. Links are never followed.
- **Unknown.** A node marked `Partial` (anything below it not read in full) has an unknown newest: the query never
  lists it (section 2). A folder with no file below it has `NO_TIME` and is never listed either. A folder holding
  another volume's mount does not count that volume's changes, as its sums do not.
- **Cost.** Two `max` per entry in the worker's fold (no allocation, no extra read: `RawEntry.modified` and
  `RawEntry.changed` are already read) and one per node at `freeze`. Both times fold into the one field, so `Node`
  stays at 80 bytes and the cache record holds one value; the tree format changes once (to 7). The `LARGE` tree's scan time is measured before and after; the bar is plan
  004's ([`../../testing.md`](../../testing.md), "Timing bars").
- Reader: `Tree::newest(index) -> Option<Timestamp>` (`time_of`).

### 2. The cache and the rescan (plan 01)

- **Codec.** `TREE_FORMAT` 6 → 7. Each record writes its frozen `newest` (the subtree's) right after `modified`, as
  `write_i64(newest.wrapping_sub(parent_newest))`, the parent's newest carried in the decoder's `Frame` like
  `modified`; a value below zero other than `NO_TIME` is refused (`OutOfRange`). A collapsed record keeps its
  subtree's value. Decoding sets `newest` and `freeze` maxes it again, which changes nothing (a subtree's maximum
  already holds its children's). A cache of format 6 is refused and the next scan reads everything once, as at every
  format change. The cache stays under 32 bytes a node (the child's delta is mostly 0 or small).
- **Rescan.** A folder read again takes its new batch's `newest` (as a full walk); a folder walked again is a full
  walk; a copied node takes its cached (subtree) `newest`. `freeze` then maxes every patched folder's value up through
  its ancestors. So every folder read again, and everything below it, is exact; an ancestor of one is never older
  than the truth (it keeps the larger of its cached value and what is below it): a file that went may leave it newer
  than it is, until the next full scan (agent's call 4). A newer folder is never listed by mistake; only one that
  should be may wait.

### 3. The query (plan 02, step 01)

A new method of `DiskState`, beside `items` and reusing its rule:

```rust
/// One folder found left alone: no file or link below it changed since `newest`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeftAlone {
    pub folder: NodeId,
    pub newest: Timestamp,
}

/// The folders below `root` (never `root` itself) left alone before `before`: of at least `min` allocated bytes, read
/// in full, whose newest change is known and older than `before`. Only a folder `rule` enters is looked at (its
/// `Item` and `Skip` are never answered nor entered); a folder looked at that is not left alone is entered. The highest
/// of each branch: never one inside another answered. Never a node a category claimed, a link, a special entry or a
/// hard link counted elsewhere; never enters a folder smaller than `min`. The largest allocated size first, ties in
/// path order. `None` for a node of another state.
fn left_alone(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, before: Timestamp) -> Option<Vec<LeftAlone>>;
```

- `TreeState` answers it in `crates/scanner/src/state/left_alone.rs` (depth first with an explicit stack, as
  `state/items.rs`, sharing its `offered` test and `Largest` ranking without a cap: the answers are disjoint folders of
  at least `min`, so at most the home's size over `min` of them).
- `FixedDiskState` gains `with_newest(path, Timestamp)` and answers it from the recorded values (sizes and newest as
  given, a `Partial` completeness unknown). The two engine doubles delegate.
- The rule is `HomeItems`, unchanged: packages and folded subtrees are its items, so never folders here; `Library`
  and the Trash at the top, hidden names, `.app` and links are skipped (agent's call 5).

### 4. The rule on top: what the engine lists (plan 02, step 03)

`crates/engine/src/largest/folders.rs` (new), `Engine::largest_folders(&FoldersQuery)`:

1. Refused as `largest` is: `NoScan`, `Busy`; a state without the home answers an empty page.
2. `min` = `Settings::largest_folder_min_size()`, `days` = `Settings::stale_days()`, `before` = the state's `taken`
   minus `days` (never the clock).
3. A worklist from the home: each `state.left_alone(root, &rule, min, before)` answer is skipped when a cleanup removed
   it (`Ground::is_gone`), else mapped by `verdict(ignored, judged)`, the ignore list first (ignored wins), then
   `Ground::judge`. Files' `judge` and its order do not change; `Ground::ignored(path) -> Ignored` is taken out of
   `Ground::judge` so both read it the same way:

   | Answer | Verdict | Why |
   | --- | --- | --- |
   | `Ignored::AtOrAbove` (any judgement) | `Drop` | on the ignore list or under an ignored folder: never listed, even when it holds explained items |
   | `Ignored::Inside` (any judgement) | `Enter` | it holds an ignored path: not listed, its other children may be |
   | `(Offered, _)` | `List` | a plain folder of the user's |
   | `(Withheld(HoldsExplained), _)` | `List` | it holds what Scan explains; shown with its whole size (agent's call 7) |
   | `(Withheld(_), true)` | `Enter` | a standard folder or an ancestor of one: not listed, its children may be |
   | `(Withheld(_), false)` | `Drop` | explained, library, trash, hidden, app, link, not seen, protected: neither it nor anything inside |

   `Enter` pushes the folder as a new root (the query below it answers its stale children). The home itself is never
   answered (it is the first root).
4. The rows: largest allocated first, ties in path order; paged by `offset`/`limit` (capped at `MAX_PAGE`).
5. Nothing is kept beside the state (agent's call 10): the query enters only folders of at least `min` (500 MB at the
   least), and the ignore list and the removed paths are read as they are now.

### 5. The core's types (plan 02, step 03)

`crates/core/src/largest/folders.rs` (new), exported from `fresher_core`:

```rust
/// One page of the Folders list. Only where to start: the rule takes its size and age from the settings.
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FoldersQuery { pub offset: u32, pub limit: u32 }

/// How long nothing in a folder changed, as the row says it: months under two years, then years.
#[serde(tag = "unit", content = "count", rename_all = "kebab-case")]
pub enum LeftFor { Months(u32), Years(u32) }

/// One folder left alone.
#[serde(rename_all = "camelCase")]
pub struct FolderRow {
    pub node: NodeInfo,
    /// The folder it is in, as a path from the home, for display (`~`, `~/Code`).
    pub place: String,
    /// The newest change of anything inside it.
    pub newest: Timestamp,
    pub left_for: LeftFor,
    /// It holds a `.git` folder.
    pub project: bool,
}

/// The answer to a `FoldersQuery`.
#[serde(rename_all = "camelCase")]
pub struct FoldersPage {
    pub rows: Vec<FolderRow>,
    pub offset: u32,
    pub total: u32,
    /// The sizes of every folder listed.
    pub sizes: Sizes,
    /// How many of them are projects.
    pub projects: u32,
    /// The home's sizes, for the bar.
    pub home: Sizes,
    /// The smallest size listed (the setting's value).
    pub min_size: ByteSize,
    /// The days nothing in a listed folder changed, at least (the stale projects' setting).
    pub days: u32,
}
```

- `LeftFor::between(newest, taken)`: whole days `d` from `newest` to `taken` (0 for a time after it); `Months(d / 30)`
  while `d < 730`, else `Years(d / 365)` (agent's call 9). The derives are `LargestPage`'s (`Clone, Debug, PartialEq,
  Eq, Serialize, TS`; `FoldersQuery` also `Copy, Hash, Deserialize`; `LeftFor` `Copy`).
- `project` is `state.find(path.join(".git"))` naming a `Folder` node (agent's call 8).
- `place` reuses `page.rs::from_home` (moved to `largest/mod.rs` as `pub(crate)` for both pages).

### 6. The setting (plan 02, step 02)

- `core.largest.folder-min-size` (`LARGEST_FOLDER_MIN_SIZE` in `core_schema.rs`): a Choice of `500-mb`, `1-gb`,
  `5-gb`, `10-gb` (decimal bytes, as `MIN_SIZES`), default `1-gb`, group Scan, effect Now, listed right after
  `core.largest.min-size`. `folder_min_size_of(choice) -> ByteSize` (an unknown choice is the default's). Engine:
  `Settings::largest_folder_min_size()`.
- Texts (`crates/core/locales/en.json`): `core.largest.folder-min-size.label` "Smallest folder on Largest";
  `.description` "Folders smaller than this are not listed under Folders on the Largest screen. Nothing is removed by
  this setting."; `.options.500-mb` "500 MB", `.options.1-gb` "1 GB", `.options.5-gb` "5 GB", `.options.10-gb` "10 GB".
- The age: `fresher_core::STALE_PROJECTS_UNUSED_FOR` (`DaySetting::new("unused-for-days", 30, 3650, 365)`, in
  `settings/days.rs`) and `STALE_PROJECTS_MODULE` (`"stale-projects"`). The module declares and reads its setting
  through them (behaviour unchanged); the engine reads `Settings::stale_days()` =
  `STALE_PROJECTS_UNUSED_FOR.days(&self.module(id))`, 365 when the id does not parse (agent's call 11).

### 7. The commands, and Browse made read-only (plan 02 step 04, plan 03 step 05)

- `get_largest_folders(query: FoldersQuery) -> Result<FoldersPage, Refusal>` → `Engine::largest_folders`.
- `reveal_node(node: NodeId) -> Result<(), Refusal>` → `Engine::reveal_node(NodeId)` (`crates/engine/src/reveal.rs`):
  `NoScan`/`Busy` as Largest; `StaleNode` for a node of another state or no node; `NotOwnItem` for the home, a path
  outside it or one a cleanup removed; then `ports.system_ui.reveal(path)`, a failure an internal error. The UI names
  the id only; the engine holds the path (agent's call 12).
- Both registered in `lib.rs`, allowlisted in `build.rs` and `dev_runs/commands.rs`, in the bindings.
- `pickable` refuses a plain folder (`NodeKind::Folder` and not `is_package`): `pick_own_item` answers `NotOwnItem`
  for it, and a plan re-judges the same way (agent's call 13).
- **Browse stays**: `Engine::largest_place`, `largest/place.rs`, `OwnPlace`, the breadcrumb and the command
  `get_largest_place` are kept. Plan 03, step 05 makes it read-only: `OwnRow.picked` goes (`own_place` no longer asks
  `Ground::picked`), `OwnRow.offer` and `opens` stay. In the UI a Browse row has no checkbox and no empty checkbox
  place, keeps its Open button on a folder that opens, and gains Show in Finder (`reveal_node`); the reason of a row
  that does not open is still its note. `Withheld` keeps its 12 reasons and texts: `judge` still makes them for Files,
  Browse's notes, `verdict` and `blocked` (agent's call 14).

### 8. The UI (plan 03, steps 01, 03 and 04)

- **Mockup first** (03/01): `docs/mockups/0.2.0/largest.dc.html`, the three tabs, on the design canvas, approved by
  the user on 2026-11-26. The layout below is what the mockup draws. Its dark frames bind; its light frames are a
  sketch until the app has a light theme.
- **Folders tab** (`screens/largest/folders-list.tsx`, new; data from
  `screens/largest/use-largest-folders.ts`, as `use-largest.ts`: asked when a scan ends, after a cleanup removed
  something and when an ignore list changed; nothing asked before a scan or while one runs):
  1. The heading (`h2`): "Open these in the Finder and decide what to keep".
  2. The rule line: "Folders of {size} or more where nothing changed for {days} days. Fresher never removes them."
  3. In a card (the surface of Scan's disk card), the bar (`UsageBar`): the home's size, the listed folders as
     segments (the five largest by colour, the rest as one), the rest of the home neutral; its caption "{size} of your
     home's {home}" (the listed size bold); its legend: the five folders' names and sizes, then "{count} more" with
     the rest's size.
  4. `SectionSeparator` labelled "In this list", then three `Tile`s, each with an icon (a folder, a disk, code) tinted
     with one of the first three segments' colours: "Folders" {count}, "Space" {size}, "Projects" {count}.
  5. The rows in a `Surface`, a list named "Folders left alone, largest first"; "Show more" while rows remain.
- **The row** (`product/folder-row/folder-row.tsx`, presentational): the folder's name, then `{place}` (`~` for a
  folder directly in the home) and, for a project, a "Project" badge; the line "Not changed for {count} months. Open
  it in the Finder to decide." (or years); the size (`SizeText`, `honestSize`); a secondary button "Show in Finder" whose accessible name names the
  folder. No checkbox and no empty checkbox place.
  A long place is cut in the middle (`product/middle-path/`): the last two segments stay whole, the first folder names
  shrink to an ellipsis; the Files rows read the same.
- **States** (each under the tabs, in place of the whole list): loading "Reading the list." and unavailable "The list
  could not be read." (the Files keys); empty "No folder of {size} or more has gone unchanged for {days} days."; a
  refused reveal goes to the store (`command-failed`), as `useRowMenu` does.
- **Browse tab** (`screens/largest/browse-view.tsx`, today's `folders-view.tsx` renamed, `BrowseView`): the
  breadcrumb and the rows of `get_largest_place` as today, read-only from 03/05 (section 7); its list is named
  "Folders and files, largest first" and its heading, above the breadcrumb, says "Go into a folder, or show it in the
  Finder".
- **The screen**: the tabs "Files", "Folders" and "Browse"; the page subtitle follows the tab (Files' summary, or
  "{count} folders, {size}" on Folders, none on Browse); the selection bar stays (Files' picks); the opened folder is
  Browse's state, kept while the user looks at another tab; "Scan first to see your largest files." becomes "Scan first
  to see your largest files and folders.".
- New texts (`app/ui/src/locales/en.json`): `ui.screen.largest.folders.heading`, `.folders.rule`, `.folders.bar`,
  `.folders.tile.count`, `.folders.tile.space`, `.folders.tile.projects`, `.folders.empty`,
  `.folders.separator` "In this list", `.folders.legend-more` "{count} more",
  `.folders.summary.{one,other}` ("{count} folder, {size}" / "{count} folders, {size}"), `ui.folder-row.months.{one,
  other}` ("Not changed for {count} month. Open it in the Finder to decide." / "... {count} months. ..."),
  `ui.folder-row.years.{one,other}`, `ui.folder-row.project` "Project", `ui.folder-row.reveal` "Show in Finder",
  `ui.folder-row.reveal-label` "Show in Finder: {name}"; `ui.screen.largest.folders` becomes "Folders left alone,
  largest first"; `ui.screen.largest.view.browse` "Browse", `ui.screen.largest.browse` "Folders and files, largest
  first", `ui.screen.largest.browse.heading` "Go into a folder, or show it in the Finder"; Browse's Show in Finder
  reuses `ui.folder-row.reveal` and `.reveal-label`.

### 9. The e2e mock and the gallery (plan 03, steps 02 to 04)

- `e2e/support/largest-mock.ts` answers `get_largest_folders` from the script's folders (largest first, paged) and
  records `reveal_node`; `get_largest_place` stays, its rows without `picked` from 03/05. `app/ui/src/test/largest-scan.ts` gains `FOLDERS`, a
  page of five folders (two projects, one of years). The gallery shows `FolderRow` (plain, project, years) and the
  Folders tab's summary.
- `flows.spec.ts`: the Folders tab lists the mocked folders and Show in Finder sends `reveal_node` with the row's id;
  the old Folders flows move to the Browse tab (opening a folder, the breadcrumb), and ticking in it goes (03/05): a
  Browse row has no checkbox and its Show in Finder sends `reveal_node`. `visual.spec.ts`: `largest-folders.png`
  becomes the Browse tab's baseline (re-photographed, chromium and webkit); the Folders list gets its own,
  `largest-left-alone.png`; `axe.spec.ts` and `keyboard.spec.ts` cover both lists.

### 10. Sandbox cases (plan 02, step 05)

- `Shown` keeps `Withheld`, now read as "the Browse row at `at` carries this reason" (the note a read-only row shows,
  as the tree still judges its rows; agent's call 15), and gains `Listed` ("a Folders row at `at`", with an optional
  `project = true`) and `Unlisted` ("no Folders row at `at`"). The harness (`app/tauri/tests/sandboxes/largest.rs`)
  checks `Withheld` through `largest_place` as today and the two new ones through `largest_folders`.
  `large-files.toml`'s Public case stays (`withheld`, `standard-folder`).
- A new case file, `fixtures/sandboxes/largest-folders.toml` (feature `largest-folders`, module `trial`, setting
  `core.largest.folder-min-size = "500-mb"`, the stale days left at 365, ages 400 days): every folder and file of a
  case aged, as `stale-projects.toml` does. About 2.6 GB of filled data (five 520 MB files), said in its header
  (agent's call 15).
- **The ctime in a sandbox.** The generator can set `modified` and `accessed` but not `changed`: macOS moves a file's
  ctime to the moment of any write and offers no way to set it back, so every generated file is "changed today" and
  nothing would be listed. Sandbox mode and the harness therefore read through `StatedTimes`
  (`crates/sandbox/src/stated_times.rs`, a `FolderReader` around the real one, as `StatedSpace` wraps the volumes):
  for an entry the world aged, `changed` is the case's `changed` (a new optional key of `[[case.file]]` and
  `[[case.dir]]`, as an age: `"1d"`) or else its `age`'s time; every other entry is passed through. In unit tests the
  seam is `RawEntry.changed` itself (the fold's tests build entries; `ScriptedWalk` and the codec specs take a time).
  The cases:
  - negative: "a folder with a recent file is not listed" (`home/Code`, holding `new-app/main.rs` of today);
    "an ignored folder is never listed, even holding explained items" (`home/Old-tools`, on `core.ignore.paths`,
    holding a `Sample app cache` folder the `trial` module explains; 520 MB);
    "a folder inside a listed one is not listed" (`home/Archive/2019`); "a standard folder is never listed"
    (`home/Public`); "a small stale folder is not listed" (`home/Small`, 12 MiB).
  - negative, too: "a folder of old files copied today is not listed" (`home/Restored`, a 520 MB file with
    `age = "400d"` and `changed = "1d"`).
  - positive: "a stale project is listed as a project" (`home/Code/old-app`, with `.git/`); "the stale folder around a
    stale one is listed" (`home/Archive`); "a stale folder in a standard folder is listed" (`home/Public/old-export`).
  - Its `EXPECTED.md` is generated; `npm run dev:sandbox -- largest-folders` is walked by hand in 04/03.

### 11. Docs (plan 04)

- `docs/architecture/README.md`: "The scan tree" names `newest` (the later of `modified` and `changed`) and the
  format 7; "The largest files" describes Files, Folders (the query, the verdict table, the commands) and Browse (the
  read-only tree, Show in Finder); the storage map's paragraph says the by-place view lives on as Largest's Browse.
- `docs/categories/README.md`: Largest's Folders is an information view, not a category: nothing removed, nothing
  counted. `core-own-file.md`: a plain folder is never picked.
- `docs/deferred/largest-files-more.md` loses "A folder's Changed from its newest file" (the tree has it; the Files
  rows of packages may show it later); `folder-age-from-children.md` stays, with a line that `newest` exists after
  the walk and does not serve a module during it.
- `docs/ideas/big-stale-projects.md` taken by 044; its row in `docs/ideas/README.md`.
- `docs/deferred/tester-feedback-0.1.0.md`: "where did the rest go" partly answered by Largest's Folders.
- `docs/mockups/README.md`: the 0.2.0 section with `largest.dc.html`.
- `docs/test-cases/largest-folders.md` (new, LF-01 to LF-05) and HL-03 of `high-and-largest.md` (no tick on Folders or Browse).

## Tests

- Scanner: the fold's and `freeze`'s newest (a copied file: old `modified`, new `changed`, through `RawEntry`); the codec's round trip and refusals; the rescan's newest against a full
  scan; the query's every refusal, then its answers.
- Core: `LeftFor::between` and the serialized shapes; the setting's choices and default.
- Engine: `largest_folders` refusals and every row of the verdict table (ignored before explained), then the listed
  rows, the page and the settings; `reveal_node`'s refusals then the reveal; `pick_own_item` refuses a plain folder
  and still picks a package; `largest_place` answers rows without `picked`.
- Shell: the commands' refusals and answers; the bindings.
- Sandbox: `largest-folders.toml` through the harness; `large-files.toml` unchanged.
- UI: `FolderRow`, `FoldersList`, `useLargestFolders`, `BrowseView` read-only, `LargestScreen`'s three tabs; e2e
  flows, visual, axe, keyboard, core mock.
- Budget: `crates/e2e/tests/largest_budget.rs` gains `folders_of_the_large_tree` (ignored in the hooks, run by hand
  in release): the list at `min` 0 and `days` 0 (every folder entered) under 50 ms and 5 MB.

## Risks

- **Scan time.** Two `max` per entry in the fold and per node at freeze; measured on `LARGE` in 01/01 against plan
  004's bars. If it moves a bar, the step stops and reports.
- **Memory.** `Node` 72 → 80 bytes; `heap_bytes_per_node_stays_under_96` must still pass. If it does not, the step
  stops (the bar is not raised without the user); the fallback is a side table of folders' newest.
- **The cache.** Format 7 discards every user's cache once (one full scan, as every past format change); the cache's
  32 bytes a node is held by its test.
- **The rescan patch.** An ancestor of a changed folder may stay newer than the truth until a full scan (section 2):
  a folder may be missing from the list, never listed wrongly. Tested against a full scan.
- **The query on a big home.** It enters only folders of at least `min`; the worklist enters each subtree once. The
  budget test runs the worst case.
- **The rule's edge.** A folder that holds explained items is listed with bytes Scan also lists (agent's call 7); the
  row is information only, so nothing is counted twice in a total.

## Open questions

None. The one question of the first version (no folder browsing would be left) is answered under Decisions.

## Decisions

- 2026-11-26: the design approved by the user, with every agent's call taken.
- 2026-11-26 (the draft's answers, the user): Largest has two sections; Folders shows big folders not changed for a
  long time, Show in Finder only, never removed; files stay removable at High after the confirmation. Takes the idea
  `big-stale-projects.md`.
- 2026-11-26 (the user): removing a whole folder from Largest goes away; every recommended decision is taken: the newest change folded during the walk into the tree,
  the codec and the rescan, no `git` run; the highest qualifying folder listed; the exclusions of goal 4; the size a
  Choice setting `core.largest.folder-min-size` (500 MB, 1 GB, 5 GB, 10 GB; default 1 GB); the age the stale
  projects' days read through the settings; the Project label; `reveal_node` by id; the row's texts; paging like
  Files; a mockup first; the docs of section 11; uncommitted or unpushed work not read.
- 2026-11-26 (the user, told that the storage map does not browse): the old folder tree stays, read-only, as a third
  tab "Browse": the same tree with no ticks, only going into folders and Show in Finder. Only files stay removable on
  Largest.
- 2026-11-26 (the user: "the mockups look good"): the mockup
  [`docs/mockups/0.2.0/largest.dc.html`](../../mockups/0.2.0/largest.dc.html) approved; what it changed is written into
  section 8.
- 2026-11-26 (the user): a long path is cut in the middle, keeping the first folder names and the last ones, so it
  shows which folder it is and where it is; the Folders and Files rows alike.

## Agent's calls (to check)

1. **Folders' own times and `.DS_Store` are not changes**: only files, links and special entries count, so the
   Finder looking into a folder does not make it recent.
2. **One field, `Node.newest`**, own until `freeze` and the subtree's after, as the sums: 8 bytes a node, not a side
   table.
3. **The cache stores the subtree's newest** per record, as a delta from the parent's; format 7.
4. **The rescan is never older than the truth**: copied nodes keep their cached newest, maxed with what is below;
   exact for every folder read again and below it; a full scan makes the rest exact.
5. **The query reuses `ItemRule` and `HomeItems`**: `Enter` means looked at; packages and folded subtrees (its items)
   are never folders of this list.
6. **Standard folders and folders holding an ignored path are entered, not listed** (the first by the judgement's
   `opens`, the second by the ignore list read first); a protected folder (never or ancestor) is dropped with
   everything inside, as Browse does not open it.
7. **A folder holding explained items is listed**, its size whole.
8. **"Project" means a `.git` child folder** in the tree; the stale projects' markers are not used (the projects are
   not in the committed state, nor in a cached one).
9. **The age is the core's**: `LeftFor` in months (whole days / 30) under two years, in years (days / 365) after.
10. **No cache of the folder list**: computed per page, as cheap as the query is.
11. **The stale days' setting moves to the core** (`STALE_PROJECTS_UNUSED_FOR`), one definition for the module and
    the engine.
12. **`reveal_node` reveals any node strictly below the home** that a cleanup did not remove, not only a listed
    row: revealing removes nothing.
13. **Only a plain folder is refused as a pick**: packages and folded subtrees, which Files lists, stay pickable.
14. **`Withheld` keeps its reasons and texts**, as the judgement still makes them; a Browse row that does not open
    shows its reason as a note.
15. **A new sandbox feature `largest-folders`** (about 2.6 GB of filled data) instead of growing `large-files`;
    `Shown` gains `listed` and `unlisted` and keeps `withheld` for Browse's rows (their note); the sandbox states each
    file's ctime from its case (`changed`, default its `age`), as the disk cannot be told one.
16. **A row's `place` is the folder it is in**, from `~`, as a Files row's.
17. **The page carries `home` and `projects`** for the bar and the tiles, so the UI computes no total.
18. **A change is the later of `modified` and `changed` (ctime)** (the controller's call on the builder's check): a
    folder copied, unzipped or restored recently is not called old; a metadata-only ctime only hides a folder. Read
    from `RawEntry.changed`, which the walk already fills; settled in 01/01 so the tree format changes once.
19. **Ignored wins** (the controller's call on the builder's check): a folder on the ignore list or under an ignored
    folder is never listed, even holding explained items; `verdict` reads the ignore list before the judgement, and
    Files' `judge` order is unchanged.
20. **Browse keeps `OwnRow.offer`** (the note of a row that does not open) and loses `picked`; it ticks nothing.
21. **The old Folders baseline becomes Browse's** (`largest-folders.png`); the new list's is `largest-left-alone.png`.
22. **The bar's rest is the page's size minus the drawn rows**: a chart remainder of sizes the core gave, not a new
    total.

## Conclusions

See [`readme.md`](./readme.md), "Conclusions".
