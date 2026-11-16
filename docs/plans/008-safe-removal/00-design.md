# 008 — Safe removal and the cleanup journal: design

**Status:** ✅ approved · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core, cleaner, macos, store, engine,
testing, app, app/ui

## Intent

The user's task, as the draft of 2026-11-15 recorded it: the one path by which anything is ever removed — from the
user's selection to the Trash — with every rule of
[`restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md) enforced in code, and a journal of what was
done. Every feature that cleans goes through it. The draft was turned into this design on 2026-11-16, at the user's
request, with the recommended decisions taken as agent's calls.

## Goals

1. **Nothing is removed that the user did not confirm**, in this session, as that exact list, after seeing its count,
   its total size, its highest danger level and how each item goes (rule 2 of [`rules.md`](../../rules.md)).
2. **Every safety rule is a check in the cleaner, below every module**: a wrong rule, a wrong selection, a bug in the
   UI or a compromised webview cannot get past it. The checks that can be types are types; the rest run right before
   each item, on what is on the disk at that moment, and fail closed.
3. **A mistake is recoverable**: the Trash by default, a journal that records where each item landed, and "put back"
   for every item still in the Trash.
4. **The result is honest**: what was moved, deleted or skipped and why; "moved to the Trash", never "freed", for what
   waits in the Trash; the space measured on the volume next to the estimate that was promised.
5. **Safe to develop**: a debug build or a test cannot remove anything outside a sandbox folder, by construction.

## Non-goals

- Running as administrator; items that need one are never removed (plan [006](../006-permissions/readme.md), section
  9). Emptying the Trash: the app never does.
- The look of the confirmation and the result: plan [003](../003-base-ui/readme.md) built `ConfirmDialog`,
  `ResultSummary` and `SelectionBar`; this plan feeds them from the core and adds only what the core's answers need.
- The ignore list itself (plan [005](../005-settings/readme.md)): this plan only takes it as an input of the guard and
  fails closed when it cannot be known.
- Who owns a folder by app (plan [009](../009-system-knowledge/readme.md)'s `RunningProcesses`): this plan checks open
  files; 009 adds "a running app owns this folder" as one more input of the same check.
- Removal from the storage map (plan [012](../012-storage-map/readme.md)): only findings can be removed now; a later
  plan can add storage nodes to the selection, behind the same guard.
- Scenarios, the debug panel and the developer CLI of plan [007](../007-safe-development/readme.md) (section 14 says
  what this plan takes from 007).
- A quarantine folder of our own besides the Trash ([cleanup journal](../../ideas/cleanup-journal.md), its open
  question): the Trash is the quarantine, and only the user empties it.

## Design

### 1. The flow

```
selection (ids) ──prepare_cleanup──▶ plan builder (engine) ──rehearse──▶ cleaner: every check, nothing done
                                          │
                                          ▼
                        CleanupPlan (frozen: items, identities, left out, summary) ──▶ UI: ConfirmDialog
                                                                                         │ the user confirms
selection cleared ◀── report, journal ◀── cleaner.execute(ConfirmedPlan) ◀── confirm(echo) ◀┘
```

1. **Select.** The UI holds findings and whole categories minus exclusions (plan 003's `state/selection.ts`). It sends
   only ids: `FindingId`s the core produced in this session and category ids (plan 002, "IPC safety"). A path never
   crosses the IPC towards the core.
2. **Prepare.** The engine expands the selection to findings of the current scan, turns each into a plan item (its
   identity, kind, sizes, danger level, title, reason and action), leaves out what cannot be removed — with the reason
   — and asks the cleaner to **rehearse** the rest: every check of section 5, nothing done. The answer is a frozen
   `CleanupPlan` with an id, its items, the left-out items and the summary: count, sizes, highest danger level.
3. **Show.** `ConfirmDialog` shows the plan exactly: the action and the count, the highest danger level, the total, the
   items grouped by category with what each group loses, how each group goes (Trash, emptied, a tool's command that
   cannot be put back), what will not be removed and why, and the permanent choice, off.
4. **Confirm.** The UI sends back the plan id **and the summary the user saw** (count, allocated bytes, highest danger
   level, method). The core compares it with the plan; any difference refuses. Confirming turns the plan into a
   `ConfirmedPlan`, once: the plan is consumed.
5. **Execute.** The cleaner's only removing entry point takes a `ConfirmedPlan` by value. It works item by item, in the
   order shown, runs the checks again right before each item, removes it, and reports each outcome. It stops between
   items when cancelled. The journal records the plan before the first item and each outcome as it happens.
6. **Report.** The result names what was moved, deleted or skipped and why, the estimate and the measured space. The
   selection is cleared; the removed findings leave the results; the last scan is marked out of date.

### 2. Types that hold the rules

All in `fresher-core` (`crates/core/src/cleanup/`), exported to TypeScript where the IPC carries them.

```rust
pub struct PlanId(u32);                       // session-scoped, like FindingId; one pending plan at a time
pub struct CleanupId(u32);                    // stable across sessions: the journal's sequence number

#[derive(Deserialize)]                        // from the UI: ids only
pub struct CleanupSelection { findings: Vec<FindingId>, categories: Vec<CategoryPick> }
pub struct CategoryPick { category: String, except: Vec<FindingId> }

pub enum Method { Trash, Permanent }          // chosen in the dialog, one for the whole plan

pub struct ItemIdentity { device: u64, file: u64, kind: EntryKind }   // read by the rehearsal, checked before acting

pub enum Action {                             // from the finding's `Removal`; the method decides Trash or delete
    Remove(PathBuf),                          // Removal::MoveToTrash
    Empty(PathBuf),                           // Removal::EmptyFolder: the contents go, the folder stays
    Tool(ToolCommand),                        // Removal::ToolCommand: never put back
}

pub struct PlanItem { finding: FindingId, key: FindingKey, category: CategoryId, danger: DangerLevel, sizes: Sizes,
                      title: Text, reason: Text, action: Action, identity: Option<ItemIdentity> }
pub struct LeftOut { finding: FindingId, title: Text, reason: SkipReason }
pub struct PlanSummary { count: u32, sizes: Sizes, highest: DangerLevel, irreversible: u32 }

pub struct CleanupPlan { id: PlanId, made: Timestamp, scan: ScanId, roots: Vec<PathBuf>,
                         items: Vec<PlanItem>, left_out: Vec<LeftOut>, summary: PlanSummary }   // never Deserialize

#[derive(Deserialize)]                        // no constructor, no Default, no struct literal outside the core:
pub struct Confirmation {                     // it exists only as the UI's JSON
    plan: PlanId, count: u32, allocated: ByteSize, highest: DangerLevel, method: Method,
}

impl CleanupPlan {
    /// The only way to a ConfirmedPlan: the echo must match the plan field by field, the plan must be fresh.
    pub fn confirm(self, echo: &Confirmation, now: Timestamp) -> Result<ConfirmedPlan, ConfirmError>;
}

pub struct ConfirmedPlan { plan: CleanupPlan, method: ConfirmedMethod }   // not Clone, not (De)Serialize
pub enum ConfirmedMethod { Trash, Permanent(PermanentConsent) }
pub struct PermanentConsent(());              // private field: built only inside `confirm`, for `Method::Permanent`

impl ConfirmedPlan {
    /// The licence to act on this plan's items, borrowed from it: every removal function of the cleaner takes one.
    pub fn act(&self) -> Act<'_>;
}
pub struct Act<'a> { plan: &'a ConfirmedPlan }   // private field: no other way to make one
```

- **Only a confirmation makes a `ConfirmedPlan`**: its fields are private, `confirm` is its only constructor, and
  `confirm` needs a `Confirmation`, which has no constructor at all — it can only be deserialized. Code in the engine
  cannot build one by accident from the plan it holds.
- **No act without a confirmed plan**: each removal function of the cleaner takes an `Act`, which only a
  `ConfirmedPlan` lends. The rehearsal (section 7) holds an unconfirmed plan's items, so it cannot reach any of them.
- **Once**: `confirm` consumes the plan; `ConfirmedPlan` is not `Clone`; `Cleaner::execute` takes it by value.
- **Permanent needs its own consent**: a module describes a removal (`Removal`) but never a method; only the
  confirmation's `Method::Permanent` produces a `PermanentConsent`, and the cleaner's permanent removal takes it.
- **The cleaner removes a `Verified`, not a path** (in `fresher-cleaner`, section 5): `verify(item) -> Result<Verified,
  SkipReason>`; `Verified` is not `Clone` and each removal function consumes it, so one check cannot serve a later
  removal. The removal functions take nothing else that names an item.
- **Compile-fail tests** (`trybuild`) hold all of this: a `ConfirmedPlan`, a `PermanentConsent` or a `Confirmation`
  cannot be written as a literal outside the core; a `ConfirmedPlan` cannot be cloned or executed twice; an unconfirmed
  `CleanupPlan` does not fit `execute`; an `Act` cannot be made without a `ConfirmedPlan`; a `Verified` cannot be
  cloned or used twice.

What the types cannot do — the time between the check and the act — is in the threat model (section 5).

### 3. The plan builder (engine)

Pure logic in `crates/engine/src/cleanup/plan.rs`, over the current results; the rehearsal is the only IO, through the
`Cleaner` port.

1. **Refuse early**: while a scan or a cleanup runs (`Busy`); with no scan (`NoScan`); an unknown finding id
   (`UnknownFinding`) or category (`UnknownCategory`, new); an empty selection (`NothingToClean`); more than
   **10 000 items** after expansion (`TooManyItems { limit }`): an exact list longer than that cannot be reviewed.
2. **Expand**: a picked category stands for all its findings of the current scan minus its exceptions.
3. **Make items**, leaving out (with a `SkipReason`, never silently):
   - a finding that is not `Removable` (plan 006's `Removability`, set by the engine from the walk's facts);
   - a path removal whose path is not the finding's subject path (`Subject::Path`): fail closed, the module kit's
     contract test flags it first;
   - a tool removal whose subject is not that tool's object, or whose arguments hold `-f`, `--force` or `--force=…`: a
     tool must refuse what is in use, not be told to force it;
   - an item inside another item of the plan (by path components): it goes with its parent and is not counted twice
     (`Covered`);
   - an item under a path the user ignored (when the ignore list exists, plan 005).
4. **Rehearse** the items (`Depth::Quick`: every check of section 5 except the folder walk); what fails is left out
   with its reason; what passes gets its `ItemIdentity`.
5. **Freeze**: the plan gets the next `PlanId`, the time from the `Clock`, the scan id and the scan's roots. It
   replaces any earlier pending plan.

**A pending plan dies** — and a later confirmation of it is refused with `PlanGone` — when:

- a scan or a restore starts (its findings are replaced), or the app quits (plans live in memory only);
- a newer plan is prepared, or the UI discards it (the dialog's Cancel);
- a setting that changes what is scanned or ignored changes (the roots, a module switched off, the ignore list);
- it is **10 minutes old** unconfirmed, on the `Clock` port (open question 2).

### 4. The confirmation

- `ConfirmDialog` renders the plan the core sent (generated types), never the UI's preview sums: `SelectionBar` and the
  review screen keep their preview, labelled as one.
- The echo (`Confirmation`) carries what the dialog showed: the plan id, the count, the allocated bytes of the total,
  the highest danger level and the method. The core refuses with `PlanMismatch` on any difference — and the plan
  dies, since the UI showed something else — and with `PlanGone` for a plan that is unknown, replaced, discarded or
  expired. "Confirmed 3 items" can never remove 300.
- The permanent choice is the dialog's checkbox, off at every opening, which turns the button red and its label to
  "Delete permanently". Its description: "Frees the space now. You cannot put these items back." The method applies to
  the plan's path items; a tool's items are irreversible either way and the dialog says so in their group.
- The dialog says, for the Trash: "You get the space back when you empty the Trash."
  ([the Trash holds the freed space](../../ideas/trash-holds-freed-space.md), its open question 2: yes, before).
- Items left out are listed under "These will not be removed", each with its reason, and are not in the count.

### 5. The guard: checks right before each item

`fresher-cleaner`'s `guard` runs the same checks in the rehearsal and right before each removal, in this order; the
first that fails skips the item with its `SkipReason`. It reads the disk only through descriptors and paths that
cannot follow a link.

| #   | Check                     | Skip reason                        |
| --- | ------------------------- | ---------------------------------- |
| 1   | Reach                     | `NotInSandbox`                     |
| 2   | Inside the roots          | `OutsideRoots`                     |
| 3   | Protection                | `Protected`, `WholeFolder`         |
| 4   | Not ignored               | `Ignored`                          |
| 5   | Physically there, no link | `ThroughLink`, `Vanished`          |
| 6   | Same volume               | `OtherVolume`                      |
| 7   | Still what was confirmed  | `Replaced`, `KindChanged`          |
| 8   | Removable now             | `NotRemovable { removability }`    |
| 9   | Capability                | `MissingCapability { capability }` |
| 10  | Not in use                | `InUse { app }`                    |
| 11  | Not grown                 | `Grown`, `OtherVolume`             |

1. **Reach**: in a debug build or a test, the item is under a sandbox folder (below).
2. **Inside the roots, lexically**: absolute, no `.` or `..` component, under one of the plan's roots that is still a
   scan root in the settings, and never a root itself.
3. **Protection**: not a protected path, not inside one, not an ancestor of one; not a hard-blocked folder as a whole.
4. **Not ignored**: not inside an ignored path nor an ancestor of one (plan 005); the ignore list must be known.
5. **Physically there, through no link**: the root's canonical path plus the item's relative components is opened with
   `O_NOFOLLOW_ANY`, so no component may be a link; the handle stays open until the act.
6. **Same volume** as its root (`st_dev`): never a mount point, never another volume.
7. **Still what was confirmed**: the handle's device, file id and kind equal the plan's `ItemIdentity`.
8. **Removable now**: plan 006's `Removability::of(EntryFacts)`, the facts read again from the handle and its parent.
9. **Capability**: the guarded place it lies in, or its tool, is allowed (`AccessStatus::check`).
10. **Not in use**: no open file of the user's running processes is the item or inside it.
11. **Not grown** (`Depth::Full` only, folders): a walk of the folder with `std` (no link followed, a hard link counted
    once, never crossing a device) finds no other volume inside and an allocated size at most 25 % and 64 MiB over
    the confirmed one. It only measures: the act's safety does not rest on it, except that it refuses a folder with
    another volume inside.

A tool item runs checks 1 and 9, then relies on the tool's own refusal; it has no path.

**Protection** is a value of the core, `Protection { never: Vec<PathBuf>, whole: Vec<PathBuf> }`, with pure,
component-wise checks; the macOS adapter builds it for the app from the home folder:

- **never** (nor anything inside, nor any ancestor): `/System`, `/usr` except `/usr/local`, `/bin`, `/sbin`,
  `/private/var/db`, `/private/etc`, `/cores`, other users' homes (everything under `/Users` outside the user's own),
  `/Applications` and every app bundle (until an uninstaller is planned), the app's own bundle and data folder, the
  Trash folders (`~/.Trash`, `/.Trashes`, a volume's `.Trashes`: what the user put there is theirs), synced places whose
  removal also removes the copy in the cloud (`~/Library/Mobile Documents`, `~/Library/CloudStorage`). macOS's system
  integrity protection is also caught by check 8 (`ProtectedBySystem`).
- **whole** (never as a whole; items inside may go one by one): the home folder; Desktop, Documents, Downloads, Movies,
  Music, Pictures, Public, Applications and Library in it; the standard folders of the Library (Caches, Application
  Support, Preferences, Containers, Group Containers, Logs, Saved Application State, …); `/Volumes` and every volume's
  root.

**Fail closed**: nothing is cleaned — `CleaningBlocked { reason }`, the whole command refused, before any item — when
the settings file is not `Loaded` (the ignore list cannot be known: plan 005's rule, and plan 002's `SettingsFile`
states), when the journal cannot be written (section 8), or when the snapshot of open files cannot be taken. Within a
cleanup, a check that cannot tell (an IO error other than "not found") skips the item with `CheckFailed`. The cleaner
never forces, never retries with more rights, never quits an app, never falls back from the Trash to a permanent
delete.

**Reach and the sandbox** (taken from plan 007, section 14):

```rust
pub enum Reach { Sandbox, RealDisk(RealDiskToken) }
pub struct RealDiskToken(());   // `RealDiskToken::new()` exists only under `cfg(not(debug_assertions))`
```

- In `Reach::Sandbox` (every debug build, so every ordinary test) check 1 requires a **sandbox marker** —
  `.fresher-sandbox`, a regular file owned by the user, whose content starts with a fixed signature line — in a folder
  that is the item's root or between the root and the item, and that is not a hard-blocked folder (never the home).
  `FixtureTree` writes it at its root; a developer who wants to clean in `npm run tauri dev` points a scan root at a
  sandbox folder. In a debug build against the real disk every item is left out with "Development build: removal works
  only in a sandbox folder."
- `Reach::RealDisk` needs the token, which a debug build cannot construct. A test of `fresher-cleaner` reads the
  workspace's sources and fails if `RealDiskToken::new` appears anywhere but `app/tauri/src/compose.rs`.

**In use**: the `OpenFiles` port takes one snapshot of the files and folders the user's own processes hold open (their
descriptors and working folders), under the plan's roots, with each process's name; it is taken before the first item
and again when two seconds have passed on the `Clock` (one snapshot takes 7–19 ms, "Measured (01)"). The macOS
adapter also walks the processes' mapped regions, because a file mapped without a descriptor (a database) is not among
the descriptors; that walk takes up to a second, so it is reused for 30 s while the descriptors are read at every
snapshot (call 38). The name is the outermost app bundle's in the process's executable path, else the process's short
name: a helper's short name is not the app's (call 35). A process of another user cannot be seen by an app that is not
administrator (it answers `EPERM`); that limit is written in the threat model, and moving to the Trash is a rename,
which a process holding the file open survives. Plan 009 adds "this folder belongs to a running app" as a second input
of check 10.

**The threat model** (written out in `docs/architecture/removal.md` by step 04/07):

Each threat, its defence, and what is left:

- **A link (a symbolic link, a Finder alias) points elsewhere.** `O_NOFOLLOW_ANY` on the whole path; a link that is
  itself the item is removed as the link; an alias is an ordinary file. Left: firmlinks are transparent by design and
  stay on the same volume.
- **A path outside the roots** (a bug, a bad rule). The lexical check, then the open beneath the canonical root, then
  the same device. Left: nothing.
- **A protected path, a parent of one, a user's own folder.** `Protection`, by components, below every module; check 8
  for what macOS protects. Left: a place the list misses — the list has its own tests and review.
- **The item changed between the prepare and the act.** Its identity, read at prepare, compared through the open
  handle; the folder walk right before the act. Left: a same-size rewrite inside a folder that kept its identity.
- **The item changed between the check and the act** (TOCTOU). The handle stays open until the act; after a move to
  the Trash, the landed item must be the verified one; a permanent delete goes through `std::fs::remove_dir_all`, which
  works beneath the folder's descriptor and follows no link. Left: the system Trash call takes a path, so a component
  swapped for a link in that instant moves the wrong item (measured: it does, and then sometimes gives no landed path)
  — to the Trash, where the post-check, which asks the kept handle where the verified item is now (call 37), detects
  it, reports it and offers to put back what it can name.
- **A file in use.** The open-files snapshot (descriptors, working folders and mapped files), refreshed every 2 s,
  the mapped files every 30 s. Left: processes of other users; a few descriptors of the user's own processes whose
  path macOS withholds (`EPERM`); a file opened after the snapshot, or mapped after the last walk of the regions.
- **A wrong selection** (a UI bug, a compromised webview). Ids only; the echo checked field by field; one pending plan;
  expiry; the 10 000-item cap. Left: nothing the user did not see.
- **A wrong rule or module.** The guard sits below the rules; a path removal must name its subject; a tool's arguments
  are fixed and never force. Left: a rule that claims a wrong but harmless-looking item — the Trash makes it
  recoverable, and every rule has its negative tests.
- **A development accident.** `Reach::Sandbox` in every debug build and test; the token only in the release
  composition. Left: a release build run against the real disk by a developer, which is the product working.
- **A crash mid-cleanup.** The write-ahead journal. Left: the item in flight is "unknown", and is shown so.
- **Two operations at once.** One operation at a time (plan 002).
- **A synced folder.** iCloud Drive and the cloud storage folders are never offered. Left: a sync client that keeps
  its data elsewhere — its own rules decide.
- **A tool command misused.** Arguments fixed in the module's code, never from the UI; no shell; NUL refused; no force
  flags. Left: the tool's own behaviour, held by its module's tests.

### 6. Kinds of removal

- **`Remove(path)`**: with the Trash, the `Trash` port — the system call that returns where the item landed — and the
  landed item's identity must be the verified one; permanently, `std::fs::remove_file` or `remove_dir_all`.
- **`Empty(folder)`**: the folder is verified, then each of its entries, read from the open folder, is verified
  (checks 5–7) and moved to the Trash (or deleted) on its own; the folder stays.
- **`Tool(command)`**: the tool runs with its fixed arguments through `CommandRunner`, whatever the method; exit 0 is
  done, anything else fails with the first line of its error. A tool's removal is never put back.
- **The Trash** is `NSFileManager`'s `trashItemAtURL:resultingItemURL:error:` in `fresher-macos` (`MacTrash`): no
  permission, the volume's own Trash, and the landed path for the journal. A volume without a Trash (a read-only volume,
  some external and network drives) refuses with `NSCocoaErrorDomain` 3328: the item is skipped with `NoTrash` —
  never deleted instead. A missing item (code 4) is `NotFound`; anything else (513: no permission, a locked item, a
  volume where no `.Trashes` can be made) is `Refused`, the item in place.
- **The single deletion point**: the permanent removal is one function in `fresher-cleaner`
  (`remove::permanent::delete(verified: Verified, act: &Act<'_>, consent: &PermanentConsent)`), the only place under
  `#[expect(clippy::disallowed_methods)]` in the workspace. A test checks that no other `expect` or `allow` of that lint
  exists.
- **Tools**: the program is the one plan 006's tool check found ready; a tool whose capability is not granted is
  skipped. Its standard error stays in memory for the report; the journal keeps only the exit status.

### 7. The pipeline and the dry run

- `rehearse(items, cx, depth)` runs every check of every item and does nothing; it returns, per item, its identity or
  its skip reason. `Depth::Quick` (the plan builder) leaves out the folder walk; `Depth::Full` is the **dry run** — the
  simulator, the tests, and the developer CLI of plan 007 use it. It holds only the items of an unconfirmed plan, so it
  has no `Act` and cannot call a removal function: the types say so, not a flag.
- `execute(confirmed, cx, cancel, out)` works **item by item, in plan order, never in parallel**: one item's checks
  and act finish before the next starts, so the open-files snapshot and the journal stay true. The cancel token is read
  between items (and between the entries of an emptied folder); an item in flight finishes.
- Each item ends as one `ItemOutcome`: `Moved`, `Emptied { moved, skipped }`, `Deleted`, `ToolDone`,
  `Skipped { reason }`, `Failed { reason }` (the act itself failed: "macOS refused", the tool failed), or `NotReached`
  (cancelled before it).
- The engine batches the outcomes into events like findings (256 or 50 ms on the `Clock`), and records each in the
  journal as it comes.

### 8. The journal

**One append-only file per generation** in the data folder, `cleanups.jsonl` (current) and `cleanups.previous.jsonl`,
written only by `fresher-store`'s `FileCleanupJournal` behind the core's `CleanupJournal` port.

```
{"format":"cleanup-journal","version":1}
{"type":"begin","cleanup":7,"at":1763290000,"method":"trash","roots":[…],"space":[…],"items":[{"index":0,
 "path":…,"identity":{…},"action":"remove","category":"…","key":"…","reason":"…","danger":"low","sizes":{…}}, …]}
{"type":"item","cleanup":7,"index":0,"outcome":{"kind":"moved","landed":…}}
{"type":"end","cleanup":7,"at":1763290004,"ended":"completed","space":[…]}
{"type":"put-back","cleanup":7,"index":0,"at":1763300000,"outcome":"put-back"}
```

- **Write-ahead**: the `begin` record (every item, with its path, identity, the rule's reason key, category, finding
  key, danger level and sizes) is written and synced before the first item; each outcome is appended as it happens and
  synced every 50 items, every second and at the end. A failed append stops the cleanup after the current item: no
  removal without its record.
- **Paths** are stored as they are: UTF-8 as a string, anything else as `{"bytes": "<hex>"}`. The file is the user's,
  private (the data folder is `0700`), never logged, never sent ([`privacy.md`](../../restrictions/privacy.md)).
- **Versions** (plan 002's policy): the journal holds history, so it is migrated forward one version at a time. A file
  from a newer version, or one that cannot be read, is never written: the history shows what can be read and a new
  cleanup is refused (`CleaningBlocked { reason: journal }`). A file whose header is not a journal is moved aside to
  `cleanups.broken.jsonl` on the next write. A torn last line (a crash) is skipped on reading; the next append starts
  on a new line.
- **The budget**: when the current file passes 4 MiB at the start of a cleanup, it is renamed over the previous one
  and a new file begins. The journal takes at most 8 MiB of the 50 MB footprint (the scan cache takes 32 MiB), tens of
  thousands of items; the oldest generation drops off. The store still deletes nothing: rotation is a rename, like the
  cache's.
- **A crash** leaves a `begin` with no `end`: the history shows that cleanup as "Interrupted" and each item without an
  outcome as "Unknown — check the Trash". Nothing is written at launch.
- **Disk full**: the `begin` record is small, but if even it cannot be written, the cleanup is refused with a text that
  says to empty the Trash or free a little space first (agent's call 22).

**Put back** (an item of a Trash cleanup, or all of one cleanup's items still in the Trash):

- The cleaner's `locate` says, for each journal item with a landed path, whether it is still in the Trash and the same
  item (`lstat` of the landed path, no link followed, its device and file id equal to the recorded identity):
  `InTrash`, `Gone` (the Trash was emptied or the item moved), `Replaced`.
- `put_back` moves the landed item to its original path through the `Trash` port, which never replaces anything: the
  macOS adapter renames with `renamex_np(…, RENAME_EXCL | RENAME_NOFOLLOW_ANY)`, so no component of either path may
  be a link. It is refused, with its reason, when the original path is taken (the app made the folder again: the user
  decides, the item stays in the Trash and "Show in the Trash" is offered), when the original folder is gone, when the
  item is no longer the same, or when the original path fails checks 2–5 today. Each put back is a journal record.
- Put back is an action the user starts and that removes nothing; it runs as a short operation (refused while a scan or
  a cleanup runs) without a confirmation dialog.

**Measured space**: the engine reads each affected volume's available space (`Volumes::available`) when the cleanup
starts and when it ends, and records both. The report shows the estimate (the sum of the items' reclaimable sizes),
what went to the Trash and what was deleted, and the space measured on the volume — "about 0" for a Trash cleanup,
which the result explains. Plan 01 measured no delay: space freed by a permanent delete shows when the delete returns.

### 9. After a cleanup

- The removed findings leave the engine's results (`FindingsRemoved`); the totals and the overview are recomputed; the
  UI drops them from its pages and empties the selection.
- The last scan is marked out of date (`LastScan.changedSince`): the storage screens still show the tree as scanned,
  with "Scan again to see the space now." The app does not rescan on its own ([`goal.md`](../../goal.md), section 8);
  the next scan reads only the changed folders (plan 004).
- **Quitting during a cleanup**: closing the window asks first ("A cleanup is running. Stop after the current item and
  quit?"), as plan 002 promised. The shell holds the close while the engine is cleaning and publishes
  `quit-requested`; the UI asks; on yes (`quit_after_cleanup`) the cleanup is cancelled, the item in flight finishes,
  the journal is closed, then the app quits. On no, nothing changes.

### 10. Commands and events

| Command              | Takes                                                 | Returns                                                  |
| -------------------- | ----------------------------------------------------- | -------------------------------------------------------- |
| `prepare_cleanup`    | `CleanupSelection`                                    | `CleanupPlanView`: summary, groups with items, left out  |
| `discard_cleanup`    | `PlanId`                                              | —                                                        |
| `confirm_cleanup`    | `Confirmation`                                        | `CleanupId`; the cleanup runs on its own thread          |
| `cancel_cleanup`     | —                                                     | — (stops between items)                                  |
| `get_cleanups`       | `{ offset, limit }` (limit ≤ 100)                     | `CleanupsPage`: summaries, newest first                  |
| `get_cleanup`        | `CleanupId`                                           | `CleanupDetail`: each item, its outcome, where it is now |
| `put_back`           | `{ cleanup, items }` (`null`: all still in the Trash) | `PutBackReport`                                          |
| `reveal_in_trash`    | `{ cleanup, item }`                                   | — (plan 006's `SystemUi::reveal` on the landed path)     |
| `quit_after_cleanup` | —                                                     | — (cancels, waits for the item in flight, quits)         |

New events: `cleanup-started { cleanup, method, total }`, `cleanup-items { cleanup, done, items }` (batched),
`cleanup-finished { cleanup, report }`, `findings-removed { scan, findings }`, `quit-requested`.
`EngineStatus::Cleaning` carries its `cleanup`. New refusals (`CommandError`): `UnknownCategory`, `NothingToClean`,
`TooManyItems { limit }`, `PlanGone`, `PlanMismatch`, `CleaningBlocked { reason }`, `UnknownCleanup`,
`UnknownCleanupItem`, each with its text `core.error.<code>`. Every outcome and skip reason has its plain-English text
in the core's dictionary (`core.cleanup.skip.<reason>`, `core.cleanup.outcome.<kind>`), with a "what you can do"
sentence where there is one ("Quit {app} and try again.").

### 11. The interface

Built on plan 003's components; every text from the core's dictionary or the UI's chrome dictionary.

- **Review screen**: "Move to Trash…" is enabled; it calls `prepare_cleanup` and opens `ConfirmDialog` with the core's
  plan. The hack "Review cannot remove yet" goes.
- **`ConfirmDialog`**: its `ConfirmContent` is filled from `CleanupPlanView` (or replaced by it); new parts: a group's
  "how" line (to the Trash, emptied and kept, by Docker — cannot be put back), the "These will not be removed" list,
  the Trash sentence. Static, Cancel first, no key confirms (003's safety rule). `onConfirm(method)` sends the echo.
- **Progress** (new `CleanupProgress` in `product/`): "Moving 12 of 128 to the Trash…", a plain determinate bar, the
  item in hand, **Stop**. No counting animation, no glow: the safety rule covers the removal too.
- **`ResultSummary`**: a Trash cleanup says "Moved to the Trash", its number the size moved, then "You get the space
  back when you empty the Trash." and "Space freed on the disk now: about 0.1 GB (measured)"; a permanent one says
  "Freed" with the measured space next to the estimate. Then what was skipped and why, **Show in the Trash** (reveals a
  moved item), a link to the cleanup in the history, **Done**.
- **Cleanups screen** (new, in the sidebar after Review): every cleanup — date, method, count, size, its state ("In
  the Trash: 12 of 128", "Deleted", "Interrupted") — and above them "Waiting in the Trash from cleanups: 20 GB". A
  cleanup opens to its items, each with its outcome or skip reason; **Put back** on an item or on all still in the
  Trash; **Show in the Trash**.
- Gallery stories and screenshots for every new state; keyboard, axe and zoom specs extended; the e2e flows run on the
  mocked core.

### 12. Where the code lives

- **`fresher-core`** — `cleanup/`: the plan, confirmation and outcome types, `Protection`, the journal records and
  port, the `Cleaner` port; `Trash` returns the landed path and puts back; the `OpenFiles` port;
  `Volumes::available`; the events, commands, errors and texts.
- **`fresher-macos`** — `MacTrash` (the system Trash call; `renamex_np` for put back), `MacOpenFiles` (the process
  table), the protection list for a home folder, the available space, the current user; all FFI in `mod ffi`.
- **`fresher-cleaner`** — `DiskCleaner`, which implements `Cleaner`: reach, paths, the guard, the removals, the single
  deletion point, rehearse, execute, locate, put back.
- **`fresher-store`** — `FileCleanupJournal`: the file, rotation, reading, migration.
- **`fresher-engine`** — the plan builder, the pending plan and its expiry, the clean operation, history and put back,
  the measured space; its `Ports` gain the cleaner, the journal and the volumes.
- **`fresher-testing`** — `FakeTrash` (landed paths, put back, a volume without a Trash), `FakeOpenFiles`,
  `MemoryCleanupJournal`, `FakeCleaner`; the sandbox marker in `FixtureTree`.
- **`fresher-app`** — the commands, the composition (`Reach` by build), the close guard, the capability, the bindings.

The cleaner depends on the core alone: it reaches the disk through `std` (handles opened with `O_NOFOLLOW_ANY`, their
metadata, the folder walk, the two `std::fs` removals) and through the ports it is given (`Trash`, `OpenFiles`,
`CommandRunner`, `Clock`), so its tests run on fixture trees with fakes. Only `fresher-cleaner` is given a `Trash`: the
app passes `MacTrash` to `DiskCleaner::new` and to nothing else.

**New dependencies**, pinned exactly in the step that adds them and listed in
[`dependencies.md`](../../dependencies.md):

| Crate              | Licence                   | Where               | Why                                                             |
| ------------------ | ------------------------- | ------------------- | --------------------------------------------------------------- |
| `objc2`            | MIT                       | `fresher-macos`     | calling `NSFileManager` from Rust, inside `mod ffi`             |
| `objc2-foundation` | Zlib OR Apache-2.0 OR MIT | `fresher-macos`     | typed `NSFileManager`, `NSURL`, `NSError` (those features only) |
| `trybuild`         | MIT OR Apache-2.0         | core, cleaner (dev) | the compile-fail tests                                          |

### 13. Testing

- **Nothing real is ever removed by a test**: every cleaner test runs on a `FixtureTree` (a temporary folder with the
  sandbox marker) with `FakeTrash` (moves into a folder of the tree, returns the landed path, puts back without
  replacing), `FakeOpenFiles` and `FakeCommandRunner`. `MacTrash` is tested on a disk image the test mounts with
  `-nobrowse`, so its items land in the image's own Trash and vanish with it; a test skips itself without `hdiutil`.
- **Negative cases first**, one per skip reason and per left-out reason, each on a fixture built for it: a link in the
  middle of the path, a link as the item, an item swapped for another between prepare and execute, a folder that grew,
  a mount point inside a folder (a disk image mounted there), an open file, a locked file, a root itself, `..`, a
  hard-blocked folder, an ancestor of a protected place, a sandbox marker missing or in the home.
- **Property tests** (`proptest`, `crates/cleaner/tests/never.rs`): random trees with links out and in, nested
  folders, a fake protected set and hard-blocked folders inside the tree, and random plans with paths in and out of the
  roots. For every plan, method and depth: every moved or deleted original path was in the plan, strictly inside a root
  physically, not protected, not inside nor an ancestor of a protected or ignored path, not a hard-blocked folder; the
  tree outside the roots and every protected subtree list exactly as before; a rehearsal changes nothing at all. Failing
  seeds are kept (`never.proptest-regressions`).
- **The engine**: with `FakeCleaner`, the plan builder's expansions, left-out reasons and cap; every field of the echo
  refused on a difference (property: any change of any field refuses); a plan confirms at most once and dies on each
  event of section 3; the journal written before the first item; a failing append stops after the current item; the
  events in order.
- **Compile-fail tests** (`trybuild`): the seven of section 2, in `crates/core/tests/compile_fail/` and
  `crates/cleaner/tests/compile_fail/`.
- **The store**: the journal on a temporary data folder — records written and read back, a torn last line, rotation at
  the threshold, a newer version never written, a broken header moved aside, non-UTF-8 paths round-trip; the footprint
  test gains a hundred cleanups and stays within the budget.
- **Core e2e** (`crates/e2e/tests/cleanup.rs`, golden `fixtures/golden/cleanup.json`): the dev-only `trial` module
  claims folders of a fixture home; scan with the reference walk, prepare, confirm, execute with the real `DiskCleaner`
  and `FakeTrash`, the journal on a temporary data folder, put back; a second case cancelled mid-way; a third refused by
  the echo.
- **The UI**: unit tests of the new state and components (negative first); the e2e flows on the mocked core: select →
  review → confirm → progress → result → history → put back; the permanent path; a refused echo; an expired plan;
  left-out items; screenshots of every new state; keyboard, axe and zoom.
- **The `trial` module**: no real module exists yet and the `demo` module's findings have no file behind them, so a
  dev-only module, registered in debug builds only like `demo`, claims folders that only a developer makes
  (`fresher-trial-cache`, `fresher-trial-kept`). It gives a development build something real to clean in a sandbox
  folder, and the core e2e its findings.
- **By hand** (step 08/02), on a development build with a sandbox folder of trial folders, through the real Trash, the
  real process table and the real window: move, put back (and Finder's own Put Back), a disk image without a Trash, a
  held folder, quitting mid-cleanup. Never on real data.

### 14. What plan 007 provides, and what this plan takes

Plan 007 (draft) promised guards "built before the cleaner". This plan cannot be tested safely without some of them, so
it takes the minimum into itself and leaves the rest to 007 (open question 1):

**Taken into 008:**

- the sandbox marker and `Reach` in the cleaner; `RealDiskToken` only in the release composition, with its source
  guard — so a debug build cannot remove anything outside a sandbox (007's "dry run by default" for the real disk);
- `FakeTrash` completed (landed path, put back, a volume without a Trash), `FakeOpenFiles`, `FakeCleaner`,
  `MemoryCleanupJournal`; `FixtureTree` writes the sandbox marker;
- the dry run itself (`rehearse` with `Depth::Full`).

**Left to 007:** scenarios and their change script; the separate app id and data folder of development builds; the
remaining fakes (apps, processes, permissions) and their scenarios; `HOME` set to a temporary folder for every test
run; the lints on looking up the home folder and starting processes; the debug panel and the developer CLI that call
`rehearse`; the banner; the release guard of the dev tools.

### 15. Interfaces with other plans

| Plan | 008 needs from it                                                              | 008 offers it                                                       |
| ---- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| 002  | `Removal`, `ToolCommand`, `FindingId`, `Operation::Clean`, the deletion lint   | the `Clean` operation and the cleaner crate, built                  |
| 003  | `ConfirmDialog`, `ResultSummary`, `SelectionBar`, the review screen, selection | the core's plan behind them; its hack removed                       |
| 004  | the walk, `Volumes`, claims, the rescan                                        | `changedSince` on the last scan                                     |
| 005  | the ignore list (empty until it exists); the settings file's state             | the guard that refuses ignored paths and fails closed               |
| 006  | removability and entry facts, capabilities, guarded places, `SystemUi::reveal` | their use right before each item                                    |
| 007  | — (section 14)                                                                 | `rehearse` for its CLI and debug panel; `Reach`                     |
| 009  | later: `RunningProcesses`, "a running app owns this folder"                    | the `OpenFiles` port and check 10 to extend                         |
| 010  | —                                                                              | the journal: what was cleaned and when, what waits in the Trash     |
| 011  | —                                                                              | every refusal and outcome as a typed error with its text            |
| 012  | —                                                                              | a plan builder input a storage node can join, behind the same guard |

**Plan 006 must be merged first**: this plan uses its `Removability`, `EntryFacts`, `AccessStatus::check`,
`Capability`, `MissingCapability`, guarded places and `SystemUi`, and needs the walk to fill `Match::facts` (otherwise
every path finding is `NeedsAdministrator` and nothing can be planned).

## Measured (01)

Measured on 2026-11-16 by plan 01's throwaway probes (`NO_COMMIT/probes/008/`, never committed) on a Mac15,7 (Apple M3
Pro), macOS 27.2.1 (26B131), internal SSD, APFS, from a terminal without Full Disk Access. Older macOS versions were
not available. Every item was the probes' own, in a temporary folder or on a disk image they made and mounted with
`-nobrowse`; every item moved to the Trash was put back by the probe, and nothing of the probes is left in the Trash.

**The Trash call** (`trashItemAtURL:resultingItemURL:error:`, 01/01):

1. **Links**: a link is moved as the link, its target untouched; a folder that holds a link moves as one, the link
   inside stays a link. Nothing of the item is followed. A link in a _parent_ component is followed (question 8).
2. **Identity and place**: the landed item has the same device and file id on the startup volume (APFS), an APFS image
   and an HFS+ image, also for a volume mounted at a folder outside `/Volumes`. It lands in `~/.Trash/<name>` on the
   startup volume and in `<volume>/.Trashes/<uid>/<name>` on any other. A name the Trash already holds becomes
   `<stem> HH-MM-SS-mmm.<ext>` (local time with milliseconds), so the landed path must come from the call, never be
   guessed.
3. **No Trash**: a read-only volume answers `NSCocoaErrorDomain` **3328** ("the volume doesn't have one"; underlying
   `OSStatus` −120). A writable volume whose root refuses a `.Trashes` folder, a read-only parent folder and a locked
   (`uchg`) item all answer **513** (no permission; underlying −5000). A missing item answers **4** (underlying −43).
   In every case the item stays in place, the same item: nothing is deleted. exFAT and FAT32 images do not mount on
   this macOS ("exfat filesystem is not available"), and no network share was at hand: not measured (needs the user).
4. **Finder's Put Back**: the call writes the records Finder's Put Back reads (`ptbL`, `ptbN` in the Trash folder's
   `.DS_Store`, checked on the probe's own image). That the menu works is checked on screen in 08/02.
5. **Privacy, without Full Disk Access**: `lstat` of an item the app moved into `~/.Trash` works, so does reading it
   and renaming it out (put back); only listing `~/.Trash` is refused (`EPERM`). No prompt appeared and no call waited.
   A volume's `.Trashes/<uid>` can be listed. With Full Disk Access: not run (needs the user); nothing in the design
   lists `~/.Trash`, so put back and `locate` need no extra permission and the Cleanups screen says nothing about it.
6. **Side effects**: no dialog, no prompt, no permission. A sound cannot be heard by a probe (08/02, by hand); a busy
   volume was not reproduced.
7. **Time**: the first call in a process or on a newly mounted volume 17–311 ms; then 0.4–2 ms per item; a folder of
   10 000 files 3–8 ms (a rename: its size does not matter); an app-like package 1–9 ms. A file held open by the
   probe moves and its open handle keeps working.
8. **A parent swapped for a link**: swapped before the call, the call moved the item _behind the link_. With a second
   thread swapping the parent back and forth, 2 × 1 000 calls: the item 410 times, the decoy behind the link 229, a
   refusal 1 361 — and **144 successes returned no landed URL at all**. A call takes 0.4–1.1 ms (median). The path of
   a handle opened with `O_NOFOLLOW_ANY` before the call (`fcntl(F_GETPATH)`) named where the verified item was after
   it in 2 000 of 2 000 calls, whatever moved; after an ordinary move it is the landed path for a file, a folder, a
   link opened with `O_SYMLINK` and a hard-linked file.
9. **`renamex_np` with `RENAME_EXCL`**: onto a taken name `EEXIST` (17), both items untouched; into a missing parent
   `ENOENT` (2); across volumes `EXDEV` (18); the same on APFS and HFS+. With `RENAME_NOFOLLOW_ANY` (0x10, defined
   in the SDK's `sys/stdio.h`, not in `libc` 0.2.191) a link anywhere in either path fails `ELOOP` and nothing moves.
10. **Freed space**: 1 GiB moved to the Trash changes `f_bavail` by 0 at 0, 100 ms, 1 s and 5 s. 1 GiB deleted with
    `remove_dir_all` (15–64 ms) shows as +1.000 GiB already at 0 ms, on the startup volume and on a quiet APFS image,
    two runs each: **no delay is needed** before measuring the space at the end of a cleanup. Local snapshots and
    clones that keep blocks were not measured.

**Links, mounts and open files** (01/02):

1. **`O_NOFOLLOW_ANY`** = `0x20000000` in `libc` 0.2.191 (`O_SYMLINK` = `0x200000`): `ELOOP` (62) for a link at the
   end and in the middle; a folder opens with `O_DIRECTORY` for `fstat`; `openat` beneath a held folder works and
   `../<link>/…` fails `ELOOP`. `O_SYMLINK | O_NOFOLLOW_ANY` opens a last-component link (to a file or a folder) as the
   link, and still fails `ELOOP` for a link in the middle: the item that is a link needs no fallback.
   `AT_SYMLINK_NOFOLLOW_ANY` (0x800) does the same for `fstatat`. Only macOS 27.2.1 was run.
2. **Temporary folders**: `tempfile`'s root and `std::env::temp_dir()` are `/var/folders/…`, which fails `ELOOP`
   (`/var` is a link of the system); the canonical `/private/var/folders/…` opens. Likewise `/tmp` fails and
   `/private/tmp` opens. Firmlinks pass (the home folder under `/Users`, `/Applications`).
3. **File flags** are read with `symlink_metadata`, without opening: `st_flags` (`uchg` 0x2; restricted 0x80000 on
   `/usr/bin` and `/bin/ls`; 0x100000, the firmlink flag, on `/usr/local`, `/Applications`, `/private/var/db`),
   `st_uid` and the mode. `remove_file` of a `uchg` file fails `EPERM`.
4. **`remove_dir_all` crosses mount points**: over a folder with an APFS image mounted inside, it deleted every file
   on the mounted volume and the folder's own files, then failed `EBUSY` (16) on the mount point. A walk comparing
   `st_dev` refused before deleting anything.
5. **The open-files snapshot**: 542–553 processes, 357–369 of the user, about 4 550 descriptors (3 790 vnodes, 3 720
   with a path); **one snapshot takes 7–19 ms** (median; max 46 ms), so a refresh every 2 s holds. Other users'
   processes answer `EPERM` already to `PROC_PIDTBSDINFO` (so even their owner is unknown) and to `PROC_PIDLISTFDS`,
   nothing else. Within the user's own processes, 65 descriptors of 7 processes answer `EPERM` for their path. A
   child's open file and working folder are seen. A file **mapped without a descriptor is not seen** by the
   descriptors: in the home folder 348 such files in 48 processes (1 628 held by descriptors); under
   `~/Library/Caches` 8 (496 by descriptors, a browser running). A walk of every mapped region
   (`PROC_PIDREGIONPATHINFO`) sees them: 275 000–297 000 regions, 0.4–0.9 s. **Names**: of 73 processes inside an
   app bundle, 45 have a short name that is not the app's ("… Helper (Renderer)"); 284 are not in an app bundle.
6. **Prompts**: none appeared and every call returned in milliseconds. With Full Disk Access: not run (needs the
   user); the process table is not under privacy protection.

**Calls touched**: 17 changed (the landed path comes from the kept handle, call 37); 18 confirmed; 19 confirmed, with
mapped files added (call 38); 23 changed (`RENAME_NOFOLLOW_ANY` too); 24 confirmed (no delay); 35
changed (the name comes from the app bundle).

**Needs the user**: Finder's own Put Back on screen, the sound, a run with Full Disk Access, an exFAT or FAT32 drive
and a network share (08/02 by hand).

## Open questions

None left: both were answered by the user (below).

## Decisions

The user's answers of 2026-11-16 (approving the plan):

1. **The design is approved**, with the agent's calls below still to check.
2. **From plan 007, the minimum** (option a): the sandbox marker, `Reach` and the fakes the cleaner's tests need; 007
   keeps scenarios, the separate identity, the debug panel and the CLI, and is no longer a precondition of 008.
3. **An unconfirmed plan expires after 10 minutes** (option a): "This list is older than 10 minutes. Review it again."
4. **Built on top of plan 006**: the branch `feat/008-safe-removal` starts from `feat/006-permissions` and is
   squash-merged into `main` after 006. The draft's six agent's calls are carried below as calls 1–6.

## Agent's calls (to check)

The user asked for the recommended decisions to be taken; each waits for a confirm or an overrule.

1. **Confirmation enforced by types**: `ConfirmedPlan` is built only by `CleanupPlan::confirm` from a `Confirmation`
   that can only be deserialized; it is not `Clone` and is executed by value, once.
2. **Sequential removal**, in the order shown, cancellable between items (and between an emptied folder's entries).
3. **The journal is an append-only file** (plan 002, decision 1), with put back for items still in the Trash.
4. **Tool commands are fixed argument lists**, run without a shell; a removal's arguments may not hold `-f`,
   `--force` or `--force=…`.
5. **A plan dies** on a scan or restore, a quit, a newer plan, a discard, a change of what is scanned or ignored, and
   after 10 minutes unconfirmed (open question 2).
6. **Target version 0.5.0.**
7. **The UI echoes the summary it showed** (count, allocated bytes, highest level, method); any difference refuses
   (`PlanMismatch`) and the plan dies: the user prepares again.
8. **`PermanentConsent`**: a permanent delete exists only as the confirmation's own method; no module, setting or
   default can produce one; the dialog's checkbox is off at every opening.
9. **The cleaner removes a `Verified`**, consumed by the removal, and every removal function also takes an `Act`
   that only a `ConfirmedPlan` lends; the identity is read through a handle opened with `O_NOFOLLOW_ANY` and kept open
   until the act.
10. **The plan freezes identities at prepare time** (a rehearsal), not at scan time: a restored scan has none, and what
    the user confirms is what was on the disk when the dialog opened.
11. **One pending plan at a time**, at most **10 000 items**.
12. **Nested items collapse into their parent** at prepare time (`Covered`), so nothing is counted or acted on twice.
13. **The guard's order and margins**: the eleven checks of section 5 in that order; "grown" means over the confirmed
    allocated size by more than 25 % and more than 64 MiB; the folder walk runs only right before the act.
14. **The protection list** of section 5, built by the macOS adapter for a home folder: system folders, other users'
    homes, `/Applications` and app bundles, the app's own bundle and data folder, the Trash folders, iCloud Drive and
    cloud storage folders never; the home, its standard folders and the Library's standard folders never as a whole.
15. **Fail closed** on an unknown ignore list (the settings file not `Loaded`), an unwritable journal, or a failed
    open-files snapshot: the whole cleanup is refused before any item.
16. **No fallback from the Trash**: a volume without a Trash skips its items (`NoTrash`); the user may choose a
    permanent delete in a new confirmation.
17. **The system Trash call** (`trashItemAtURL`) through `objc2` and `objc2-foundation` in `fresher-macos`'s `mod ffi`,
    rather than our own move into `~/.Trash` (per-volume Trash rules, names on collision) or Finder scripting (needs
    Automation, plan 006's call 13). The time between our check and its path-based call is not closed, but covered:
    the landed item's identity is checked after the move, so a wrong item is detected, reported and can be put back.
    **Changed by plan 01** (measured: with a parent swapped for a link the call moves the item behind it, and some
    successes return no landed path): the post-check no longer relies on the returned path alone — see call 37.
18. **Permanent deletion with `std::fs::remove_file` and `remove_dir_all`** (descriptor-relative, following no link
    since Rust 1.58.1) at one function, the only `expect` of the deletion lint; the folder walk before it refuses a
    folder with another volume inside, because `remove_dir_all` does not check devices. **Confirmed by plan 01**:
    `remove_dir_all` emptied a volume mounted inside the folder; a walk comparing devices refused first.
19. **In use = an open file of the user's own processes**, from a snapshot of the process table (`libproc`) refreshed
    every 2 s; processes of other users are out of reach without administrator rights and are named as a limit; plan
    009 adds app ownership to the same check. **Confirmed by plan 01** (7–19 ms per snapshot; other users' processes
    answer `EPERM`; no prompt), with mapped files added (call 38).
20. **The cleaner depends on the core alone**, reaching the disk through `std` and its ports; only it is given a
    `Trash`.
21. **The journal**: two generations of an append-only JSON-lines file, rotated by rename at 4 MiB, 8 MiB at most;
    write-ahead `begin`, outcomes synced every 50 items or second; a failed append stops after the current item;
    paths kept losslessly; nothing written at launch after a crash.
22. **A full disk refuses a cleanup** whose journal cannot be written, with a text that says how to free a little space
    first, rather than removing without a record.
23. **Put back never replaces anything** (`renamex_np` with `RENAME_EXCL`), refuses when the original place is taken or
    gone, runs as a short operation without a dialog, and is journalled. **Changed by plan 01**: also
    `RENAME_NOFOLLOW_ANY`, so a link in either path refuses (`ELOOP`); it works without Full Disk Access.
24. **Measured space** per affected volume at the start and the end of a cleanup, shown next to the estimate; "moved to
    the Trash", never "freed", for what waits there. **Confirmed by plan 01**: freed space shows at once, no delay.
25. **No rescan after a cleanup**: the removed findings leave the results, the last scan is marked out of date.
26. **The confirmation already says** "You get the space back when you empty the Trash."
27. **A Cleanups screen** in the sidebar, with "Waiting in the Trash from cleanups"; plan 010's history of scans stays
    separate.
28. **Quitting during a cleanup asks first**, then stops after the current item.
29. **`Reach::Sandbox` in every debug build**, with the `.fresher-sandbox` marker at or below a scan root and never in
    a hard-blocked folder; `RealDiskToken` only under `cfg(not(debug_assertions))` and only in the app's composition,
    checked by a source test.
30. **`EmptyFolder` moves each entry of the folder separately**, each verified; the folder itself stays.
31. **Probes first** (plan 01), in `NO_COMMIT/probes/008/`, as plan 004 did; their numbers go into "Measured (01)"
    before plans 03 and 04.
32. **The core is frozen after plan 02**, as in plans 002 and 004: a missing contract stops the plan that needs it, and
    the change goes into `crates/core` in the main session.
33. **This plan starts after plan 006 is merged**; the branch `feat/008-safe-removal` is rebased on `main` then.
34. **A tool's standard error is shown, not journalled**: the journal keeps the exit status only.
35. **The reason a process holds an item names the process** ("Quit Safari and try again"), from the process table.
    **Changed by plan 01**: 45 of 73 processes inside an app bundle have a helper's short name, so the name is the
    outermost app bundle's in the executable path (`proc_pidpath`), else the short name.
36. **A dev-only `trial` module** (debug builds only, like `demo`) claims folders a developer makes, so a cleanup can be
    tried for real in a sandbox folder and the core e2e has findings with files behind them.
37. **The landed path comes from the kept handle** (from plan 01): `Trash::move_to_trash` also takes the verified
    item's open handle and returns `Landed { at, reported }` — `at` is where that handle's item is after the call
    (`fcntl(F_GETPATH)` in `fresher-macos`'s `ffi`), `reported` the call's own landed path, if any. The item is
    `Moved` to `reported` when that path holds the verified identity, else to `at` when `at` lies in a Trash folder;
    otherwise the call moved something else: `Failed` with "Another item was moved to the Trash in its place", the
    `reported` path journalled when there is one. This changes plan 02's `Trash` port (02/04), which is frozen after
    02: it goes into the core in the main session. The alternative — keep the port, journal "landed: unknown" when the
    call gives no path — leaves a moved item that the result cannot name.
38. **Mapped files count as in use** (from plan 01): `MacOpenFiles` also walks the user's processes' mapped regions
    (`PROC_PIDREGIONPATHINFO`, 0.4–0.9 s for every process) and reuses that walk for 30 s, while the descriptors
    (7–19 ms) are read at every snapshot; so a database mapped without a descriptor is in use. The `OpenFiles` port is
    unchanged. The alternative — descriptors only — misses mapped files and names that as a limit.
