# 037 — The Trash holds the freed space: design

**Status:** 🔨 in progress, awaiting the user's walk (the user's delegation of 2026-11-22) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, engine, testing, sandbox, e2e, app, app/tauri, app/ui, docs

Written on 2026-11-22 from the one-file draft (main 97d9b314) and from a fresh, read-only read of the code on that
commit. Approved by the user's delegation of 2026-11-22: build autonomously, take the recommended decisions and the
agent's calls; the user checks everything later. Where the code forced a different choice than the draft, the design
says so and the choice is an agent's call. Facts "to verify" are checked by hand in step 04/01's walk.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Fresher moves items to the Trash and stops, so after a cleanup the disk is as full as before: macOS frees the space
only when the Trash is emptied. The user checks the Storage settings, sees nothing changed, and stops trusting every
other number in the app. A first release cannot ship that.

## Goals

1. The result of a cleanup says what was moved and what was actually freed, and why they differ.
2. The user sees, per volume, how much waits in the Trash, and how much of it this app put there.
3. The user is told where to look and that emptying is theirs to do: one action, **Open Trash**.

## Non-goals

- The Trash as a cleanable category: what the user put there is theirs (agent's call 2).
- Emptying the Trash, in any form: the app never does it and never offers it (decision 1). No code path, port or
  command of this plan can empty or delete anything; the cleaner is not touched.
- The history's comparison of two scans ("shrank by itself"): plan [010](../010-history/readme.md)'s, a draft. This
  plan gives it what it needs (the journal already knows every item moved) and adds one line to 010's draft (04/01).
- A size measured fresh on every look: the Trash's size comes from the last scan (section 2), never from a walk of its
  own.

## What already exists (read on 97d9b314)

- **The result** (`app/ui/src/product/result-summary/result-summary.tsx`, plan 008): a move to the Trash is titled
  "Moved to the Trash", never "Freed"; it shows the moved size, "Moved {count} items to the Trash", "You get the space
  back when you empty the Trash." and "Space freed on the disk now: about {size} (measured)." (`CleanupReport.measured`,
  the available space measured before and after on every affected volume, summed). **Show in the Trash** reveals the
  first moved item in Finder (`reveal_in_trash`). Goal 1 is met but for the invitation sentence (section 5).
- **The confirmation** (`confirm-dialog.tsx`): "You get the space back when you empty the Trash." under a move to the
  Trash; the permanent choice's description "Frees the space now. You cannot put these items back." — exactly the
  draft's sentences. Nothing to change (agent's call 6).
- **The history** (`crates/engine/src/cleanup/history.rs`): `Located` finds every journalled Trash entry again with
  the cleaner's `locate` (`InTrash`, `Gone`, `Replaced`), and `CleanupsPage.waiting` sums the allocated sizes of the
  items still there over one page; the cleanups screen shows "Waiting in the Trash from cleanups: {size}".
- **The storage map** (plan 012): the user's Trash (`KnownFolder::Trash`, `~/.Trash`) is a place of its own, "Trash —
  You get this space back when you empty the Trash.", sized by the scan's walk.
- **The ports**: `SystemUi` has `open_access_settings` and `reveal` (`/usr/bin/open -R`), implemented by
  `MacSystemUi` (`crates/system/src/system_ui.rs`) and `FakeSystemUi` (`crates/testing/src/fake_access.rs`).
  `CleanupPorts` (`crates/engine/src/engine.rs`) holds the cleaner, the journal and the volumes.
- Every Trash entry the cleaner journals (`TrashedItem.landed`) is a direct child of a Trash folder: the system's
  Trash call and the sandbox's `FakeTrash` both put an item at the top of its volume's Trash, and an emptied folder
  journals one top-level entry per item that went. So **the Trash folder of an entry is `landed.parent()`**.

## Facts verified on this Mac (2026-11-22, read-only)

- macOS 27.2.1. Listing `~/.Trash` without Full Disk Access is refused (`Operation not permitted`), so the scan sizes
  the Trash only with Full Disk Access; **`stat` of a path inside it is allowed** (a missing entry answers "No such
  file or directory", not a refusal), so the cleaner's `locate` of journalled entries works without it.
- Finder's preferences (`~/Library/Preferences/com.apple.finder.plist`) are readable without a prompt;
  `defaults read com.apple.finder FXRemoveOldTrashItems` answers "does not exist" on this Mac, where the setting is
  off (its default). **To verify by the user**: turn on Finder → Settings → Advanced → "Remove items from the Trash
  after 30 days" and run the same command: it should print `1`.
- No other volume is mounted here (`/Volumes` holds only the system disk), so a second volume's Trash
  (`<volume>/.Trashes/<uid>`) is proven on fakes and in the core e2e only. **To verify by the user** with a USB disk.
- **To verify by the user**: `open ~/.Trash` (what Open Trash runs) opens Finder's Trash window, with and without
  Fresher holding Full Disk Access.

## Design

### 1. The Trash view (core types, `crates/core/src/cleanup/trash.rs`)

```rust
/// The answer of `get_trash`: each Trash that holds something, the home volume's first.
pub struct TrashView { pub trashes: Vec<TrashLine> }

pub struct TrashLine {
    /// What `open_trash` names it by: its place in this answer.
    pub index: u32,
    /// Its volume's name; `None` when the volume could not be read.
    pub volume: Option<String>,
    /// Everything in this Trash; `None` when no scan measured it.
    pub waiting: Option<Sizes>,
    /// What the app's cleanups moved there and is still there, found again now.
    pub from_cleanups: Sizes,
    /// With Finder's 30-day setting on: when the last of `from_cleanups` leaves the Trash by itself.
    pub gone_by: Option<Timestamp>,
}
```

Serialized in camel case and exported to TypeScript (`ts-rs`). No path crosses the IPC: the UI names a Trash by its
`index`. A new refusal, `CommandError::UnknownTrash` ("This Trash is no longer listed. Look again on the scan
screen."), answers an index the last answer did not have.

### 2. The numbers (engine, `crates/engine/src/cleanup/trash.rs`)

`Engine::trash() -> TrashView`, computed on each call, read-only:

1. **The Trash folders**: the home volume's, `KnownFolder::Trash` from the known folders port (`~/.Trash`; in sandbox
   mode the sandbox's `home/.Trash`), then the folder (`landed.parent()`) of every journalled entry still in the
   Trash. Folders are joined by exact path equality (the composition hands canonical paths; the sandbox autotest's
   invariant catches a split, section 7). Order: the home Trash first, then the others by path.
2. **From cleanups**: every cleanup of the journal (all pages, not one), each item through `Located` (made
   `pub(crate)` in `history.rs`): an item with an entry `InTrash` counts its journalled allocated size once, in the
   folder of its first entry still there. Put back, gone, replaced, deleted, tool and skipped items count nothing.
3. **Waiting**: the last scan's state (`Engine::disk_state`) holds the folder as a node that was scanned
   (`not_scanned` is `None`): its sizes, plus the from-cleanups sizes of the items whose cleanup began at or after the
   state was taken (`DiskState::taken`), which the scan could not see; the result is never below `from_cleanups`. No
   state, no node, or a node not scanned (no Full Disk Access, or no scan root holds that volume's Trash): `None`.
4. **A line is left out** when `waiting` is `None` or zero and `from_cleanups` is zero.
5. **The volume's name**: `Volumes::space(folder).name`; an error gives `None`.
6. **Gone by**: the new port `FinderSettings` (section 3) answers `AutoEmpty::After30Days` → for each line with
   something from cleanups, the latest moment any of its items went (the cleanup's end, else its start) plus 30 days;
   `None` when the setting is off or unknown, or when that moment is not after the clock's now (Finder is late; no
   promise is made).
7. The folders of the answer are kept in the engine's shared state (a `Mutex<Vec<PathBuf>>`), for `open_trash`.

`Engine::open_trash(index)`: the folder at `index` of the last answer is opened in Finder through
`SystemUi::open_folder`; an index not there is `UnknownTrash`, a failure of the system's interface is `Internal`, as
`reveal_finding`'s. Nothing else: no read, no write, no delete.

**Why the last scan and not a fresh walk** (agent's call 4): a Trash that just received 20 GB of caches can hold
hundreds of thousands of small files, and the line is asked for on every look at the scan screen; the scan already
walks the Trash. The cost is a known gap: when the user empties the Trash in Finder, `from_cleanups` falls to zero at
once but `waiting` keeps the scan's measure until the next scan (a known issue written in 04/01).

### 3. The ports

- `SystemUi::open_folder(&self, path: &Path)`: opens a folder in the system's file manager. `MacSystemUi` runs
  `/usr/bin/open <path>` (an absolute path only, one `OsStr` argument, never formatted: `fresher_macos::open_folder_args`
  beside `reveal_args`); `FakeSystemUi` records `SystemUiCall::OpenFolder(path)`.
- `FinderSettings` (core `ports.rs`): `fn trash_auto_empty(&self) -> AutoEmpty`, with
  `enum AutoEmpty { After30Days, Never, Unknown }`. `MacFinderSettings` (`crates/system/src/finder_settings.rs`) runs
  `/usr/bin/defaults read com.apple.finder FXRemoveOldTrashItems` through the `CommandRunner`: status 0 with `1` →
  `After30Days`, with `0` → `Never`; a non-zero status whose error says the key "does not exist" → `Never` (Finder's
  default); anything else → `Unknown`. It only reads; it never writes a preference. `FakeFinderSettings` answers what
  it is given.
- `CleanupPorts` gains `finder: Arc<dyn FinderSettings>`: the real reader in the release and development compositions
  and in sandbox mode (the sandbox's commands are the Mac's, only read), a fake in every test (the sandbox autotest's
  fake command runner answers no `defaults`, so `Unknown`).

### 4. The commands (app/tauri)

`get_trash() -> TrashView` (a read) and `open_trash(index: u32) -> Result<(), Refusal>` (an action, recorded by the
development run record), both `#[tauri::command(async)]`, in `build.rs`'s list, the capability file, the run
record's lists and the typed IPC module (`getTrash`, `openTrash`). No path is taken from the UI.

### 5. The screens (app/ui)

- **The Trash line** (`app/ui/src/product/trash-line/`), on the scan screen under the coverage notices, information
  only, shown whenever the core answers a line and no scan runs. One block per line; the volume's name leads a line
  only when there are two lines or more:
  - `waiting` and a share below it: "Waiting in the Trash: 23 GB, 20 GB of it from cleanups here."
  - `waiting` equal to the share: "Waiting in the Trash: 20 GB, all of it from cleanups here."
  - `waiting` with no share: "Waiting in the Trash: 23 GB."
  - no `waiting`: "Waiting in the Trash from cleanups here: 20 GB. The rest of the Trash was not measured."
  - `goneBy`: "Finder removes them by itself by {date}."
  - Then "Look at what is there and empty it yourself if you no longer need it." and **Open Trash**, whose
    description is that sentence (rule 7).
- The line is asked for (`get_trash`) when the scan screen shows, after `scan-finished`, after `cleanup-finished`, and
  when the window gains focus (the user may have emptied the Trash in Finder meanwhile). A failed answer hides it.
- **The result**: under **Show in the Trash** (which already opens the Trash window with the first moved item
  selected), the same invitation sentence. The result's title, number and sentences stay as they are.
- No control anywhere empties the Trash; a unit test of each component and an e2e scenario assert that no button
  names emptying (decision 1).

### 6. History is not fooled

The journal already knows every item moved; plan 010 (draft) compares two scans. 04/01 adds to 010's draft readme one
sentence: the space of items the journal shows moved between the two scans is reported as cleaned by Fresher, and the
growth of the Trash folders is attributed to no app. Nothing is built here for it.

### 7. Tests

- **Core**: the types' serialization (camel case, no path), the refusal's text and code.
- **macOS adapters**: `open_folder` runs `/usr/bin/open` with the path as one argument, refuses a relative path;
  `MacFinderSettings` maps each `defaults` answer (1, 0, missing key, other failure, runner error) — negatives first.
- **Engine** (`trash_tests.rs`, fakes): negatives first — no scan and an empty journal give no line; a journal item
  gone from the Trash, replaced, put back, a permanent cleanup's, a tool's and a skipped item count nothing; a Trash
  not scanned gives `waiting: None`; an emptied Trash with no item from cleanups and no scan measure gives no line;
  `open_trash` of an unknown index is refused and calls nothing; Finder's setting off or unknown gives no `gone_by`, a
  date already past gives none. Then: the share counts each item once (an emptied folder's several entries); items
  moved after the scan are added to its measure, items before it are not added twice; a second volume's Trash gets
  its own line with its volume's name and no `waiting`; `open_trash(0)` opens the home Trash; `gone_by` is the
  latest end plus 30 days.
- **Sandbox** (`app/tauri/tests/sandboxes/`): for **every** case file, the Trash view before and after the cleanup
  goes into the golden report, and two invariants are checked: nothing from cleanups before the first cleanup; after
  it, the sandbox Trash's share equals the allocated sizes of the cleanup's items found in the Trash, in one line.
  Two case files on the `trial` module (agent's call 7): `trash-empty.toml` (an empty Trash: no line before the
  cleanup; after it, one line, all of it from cleanups here) and `trash.toml` (a file the user put in the Trash is
  never offered and counts in the first number only; a trial cache's cleanup adds to both numbers).
- **Core e2e** (`crates/e2e/tests/trash.rs`): the real cleaner, a fake Trash at the fixture home's `.Trash`, the real
  journal: a cleanup, a rescan (no double count), an entry emptied from the Trash by hand (the share drops), a put
  back, a permanent cleanup, a second Trash outside the scan roots, Open Trash.
- **UI**: the component's unit tests and stories; the screen's refresh triggers; the result's sentence; Playwright
  flows, visual baselines and axe for the line (one and two volumes, not measured, the date).

### 8. The walk (04/01)

`npm run dev:sandbox -- trash` and `-- trash-empty`, each with and without `--real-trash`, walking `EXPECTED.md` and
the Trash line by hand; with `--real-trash` the moved items wait in the Mac's Trash, which the sandbox's scan does not
measure: the line reads "from cleanups here … The rest of the Trash was not measured.", in a line of its own.

## Risks

- **The journal's whole history is read on each look** (every cleanup, a `lstat` per Trash entry): thousands of
  entries cost tens of milliseconds; a long history more. Accepted for 0.1.0; the walk on the real Mac notes the time.
- **Two spellings of one folder** (a link above the sandbox, `/var` against `/private/var`) would split one Trash into
  two lines; the sandbox invariant fails then, and the composition hands canonical folders.
- **A stale `waiting`** after the user empties the Trash in Finder, until the next scan (known issue, 04/01).
- **The history's "Waiting in the Trash from cleanups"** sums one page (100 cleanups) while the line sums all: they
  differ only past 100 cleanups (noted in the review file).
- **Rebase over plans 033 and 034**: their new sandbox goldens lack the `trash` section; regenerate with
  `UPDATE_GOLDEN=1 npm run sandbox:test` after the rebase and review the diff.

## Open questions

None.

## Decisions

1. **The app does not let the user empty the Trash** (the user, 2026-11-22): it only says that something waits there
   and invites the user to look and, if they want, empty it themselves. In line with "Trash only, never empty it"
   ([best practices](../../ideas/best-practices.md), 9).
2. **2026-11-22 — the user delegated**: build autonomously, the recommended decisions and agent's calls taken.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong.

1. **"Moved", not "freed"**, with both numbers on the result (the draft's): already built by plan 008; this plan adds
   only the invitation sentence. Cost if wrong: none.
2. **The Trash is information, never a category** (the draft's). Cost if wrong: none for safety.
3. **Open Trash is the only action** (the draft's), on the Trash line; on the result the existing **Show in the
   Trash** stays (it opens the same window, with a moved item selected) and gets the sentence, rather than a second
   button doing almost the same. Cost if wrong: one button's name.
4. **The Trash's size comes from the last scan, plus what cleanups moved since**, not from a walk of its own on each
   look (section 2). Cost if wrong: a stale first number after the user empties the Trash, until the next scan.
5. **Finder's 30-day setting is read through `defaults`** (cfprefsd's answer, not the plist file, which may lag), and
   the date is given for the app's share only: the latest end of a cleanup whose items are still there, plus 30 days.
   The draft asked for a date for the whole Trash; the app knows when its own items went in, not when the user's did.
   Cost if wrong: a date one day off at worst; nothing is promised for the user's own items.
6. **The confirmation is not changed**: its two sentences are already the draft's. Cost if wrong: none.
7. **The sandbox proves the numbers through the golden report and two invariants for every case file**, not through a
   new `[case.*]` table: the case file format does not change. A journal entry whose item was already emptied cannot
   be laid out by the generator (it writes no journal), so that case lives in the engine tests and the core e2e. Cost
   if wrong: every sandbox golden gains a `trash` section (reviewed once).
8. **Per volume** means one line per Trash folder that holds something: the home's, and any other the journal names
   (`landed.parent()`); a second volume's Trash gets `waiting` only when a scan root holds it. The result keeps one
   measured number summed over volumes. Cost if wrong: a per-volume split of the result is a later change.
9. **The `store` crate is not touched** (the draft listed it): every cleanup is read through the journal's existing
   page and record calls. `fresher-macos` changes by one argument helper (`open_folder_args`), no FFI. The cleaner,
   the scanner, the store, the rules and every module stay unchanged.
10. **Refreshed on window focus** so a Trash emptied in Finder is reflected at once (its share; the first number at
    the next scan). Cost if wrong: one more read per focus.
11. **The history screen's sum stays per page**; the Trash line sums the whole journal. Cost if wrong: two numbers
    that differ past 100 cleanups.

## Conclusions

Built as designed, with these changes (details in the steps): the scanner shares the composition's clock; only known
Trash folders count (the home's, a volume's `.Trashes/<uid>`, the Mac's home Trash with `--real-trash`);
`UnknownTrash` carries its index as a field; one new fake method each in `FakeCommandRunner` and
`MemoryCleanupJournal`. Agent's calls 1 to 11 stand as written. Facts to verify on the Mac are in the review list.
