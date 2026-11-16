# Removal

How Fresher removes anything from the disk, for a developer about to touch it. Everything here lives in
`fresher-cleaner` (`crates/cleaner/`), the only code of the workspace that removes, moves to the Trash or puts back.
The design and its reasons: plan [008](../plans/008-safe-removal/00-design.md), sections 2 and 5 to 8; the rules it
enforces: [`restrictions/deletion-safety.md`](../restrictions/deletion-safety.md).

## The crate and its one entry point

- `DiskCleaner` implements the core's `Cleaner` port: `rehearse`, `execute`, `locate`, `put_back`. Nothing else of
  the crate is public but its constructor's inputs (`Reach`, `RealDiskToken`, `CleanerPorts`) and the `Verified` type,
  which only the guard can make.
- `DiskCleaner::new(reach, user, home, protection, ports)`: the reach (below), the uid the app runs as, the home folder
  (a root written `~` or `~/…` is read under it), the system's protected places, and the ports it acts through
  (`Trash`, `OpenFiles`, `CommandRunner`, `Clock`). The app gives the system's Trash to the cleaner and to nothing else.
- It depends on `fresher-core` and, on macOS, on `fresher-macos`'s `beneath` module (the calls that name one entry of
  a held folder, below); it reaches the disk through `std`, those calls and the ports, so every test runs on a
  temporary fixture tree with the fakes of `fresher-testing`. Off macOS nothing opens and nothing is acted on.

## The flow

1. **Rehearse** (`pipeline.rs`): the engine's plan builder asks for every check of every item (`Depth::Quick`, no
   folder walk) and gets each item's identity or why it would be skipped. The dry run is the same with `Depth::Full`.
   A rehearsal holds an unconfirmed plan's items and no `Act`, so it cannot call a removal: the types say so.
2. **Execute** (`pipeline.rs`): only a `ConfirmedPlan`, taken by value, so it runs once. Item by item in plan order,
   never in parallel: the cancel token (once set, every item left is `NotReached`), the snapshot of open files taken
   again when two seconds have passed on the clock, every check with the folder walk (`Depth::Full`), the removal its
   action and the plan's method call for, then the item's `ItemDone` to the engine before the next item starts.
3. **Locate and put back** (`put_back.rs`): where each item moved to the Trash is now, and moving one back.

## Reach and the sandbox

`Reach::RealDisk(RealDiskToken)` lets the cleaner act anywhere the other checks allow. `RealDiskToken::new` exists
only under `cfg(not(debug_assertions))`, and `tests/reach.rs` fails when the token is named anywhere but its
definition, the cleaner's re-export and `app/tauri/src/compose.rs`. Every debug build and every test is
`Reach::Sandbox`: an item counts only when the scan root, or a folder between the root and the item (never the item
itself), holds the sandbox marker `.fresher-sandbox` — a regular file of the user, opened through no link, whose first
line is the core's `SANDBOX_SIGNATURE` — in a folder the system's protection does not refuse (never the home, a
hard-blocked folder such as a mounted volume's root, or an ancestor of a protected place). A tool acts on the real
system, so it never runs in the sandbox.

**To clean in a development build** (`npm run tauri dev`): make a folder of its own (for example
`~/fresher-sandbox`, never the home folder itself), write the signature as the first line of a `.fresher-sandbox`
file in it, put the trial module's folders (`fresher-trial-cache`, `fresher-trial-kept`) right next to that file (the
module claims them only there) and make the folder a scan root. The scan roots are read-only in the interface until
plan 005, so they are set in `settings.json` (`core.scan.roots`) while the app is closed. Against any other folder the
same build leaves every item out ("Development build: removal works only in a sandbox folder."); a marker without the
signature counts as none.

## The guard

`guard/mod.rs` runs the same checks in the rehearsal and right before each act, in this order; the first that fails
skips the item with its reason. It reads the disk only through paths and handles that follow no link.

| #   | Check                                                                                       | Where              | Skip reason                         |
| --- | ------------------------------------------------------------------------------------------- | ------------------ | ----------------------------------- |
| 1   | Reach: in the sandbox, a valid marker at or below the root                                  | `reach.rs`         | `NotInSandbox`                      |
| 2   | Inside the roots: absolute, plain, strictly inside a root by components, never a root       | `paths.rs`         | `OutsideRoots` (`CheckFailed`)      |
| 3   | Protection, both lists, both forms of the path                                              | `paths.rs`         | `Protected`, `WholeFolder`          |
| 4   | Not ignored                                                                                 | `paths.rs`         | `Ignored`                           |
| 5   | Opened beneath the canonical root with `O_SYMLINK \| O_NOFOLLOW_ANY`, kept open until the act | `paths.rs`       | `ThroughLink`, `Vanished`           |
| 6   | On its root's volume                                                                        | `paths.rs`         | `OtherVolume`                       |
| 7   | Still the confirmed item: device, file id, kind                                             | `guard/identity.rs` | `Replaced`, `KindChanged`          |
| 8   | Removable now: owner, write access, flags, read again from the handle and its parent        | `guard/facts.rs`   | `NotRemovable { removability }`     |
| 9   | Capability of each guarded place it lies in or holds                                        | `guard/access.rs`  | `MissingCapability { capability }`  |
| 10  | Not in use: no open file of the user's processes is the item or inside it                   | `guard/in_use.rs`  | `InUse { app }`                     |
| 11  | Not grown (`Depth::Full`): over by more than 25 % and 64 MiB; no other volume inside, walked beneath the handle | `guard/walk.rs` | `Grown`, `OtherVolume` |

- A check that cannot tell (an IO error other than "not found", a root that cannot be made canonical, an act on an
  item with no identity) skips with `CheckFailed`.
- A tool's item has no path: check 1, its arguments (never `-f`, `--force` or `--force=…`: `ForcedTool`) and check 9,
  where the tool must be `Granted` by plan 006's tool check; then the tool's own refusal decides.
- **Both protections always apply**: the system's list given to `new`, and the context's (with the ignored paths).
- A move that finds no Trash is `NoTrash`. `Covered` and `NotTheSubject` are the plan builder's reasons (the engine),
  never the cleaner's.

## Held folders

`fresher-macos`'s `beneath` module (`crates/macos/src/beneath.rs`, its FFI in `ffi/beneath.rs`) acts on one entry of a
folder held open, never on a path, so a folder swapped for a link after it was opened cannot send a call elsewhere:

| Call             | What                                                                                             |
| ---------------- | ------------------------------------------------------------------------------------------------ |
| `open_folder`    | opens a folder by path with `O_DIRECTORY \| O_NOFOLLOW_ANY`: a link in any component is `ELOOP`  |
| `open_folder_at` | opens the folder `name` beneath a held folder (`openat`, `O_NOFOLLOW_ANY`)                       |
| `identity_at`    | the volume, file id and kind of the entry `name`, a link as itself (`fstatat`, `AT_SYMLINK_NOFOLLOW`) |
| `read_folder`    | every entry with its attributes (the scan's bulk read), from the first entry each time           |
| `remove_at`      | removes the entry `name` (`unlinkat`; `AT_REMOVEDIR` for an empty folder): the deleting call     |

A `name` must be one plain name (never empty, `.`, `..`, nor holding `/` or NUL), so nothing resolves through a path.
The cleaner's `beneath.rs` adds the two steps the walk and the deletion share: the entries of a held folder sorted by
name, and `descend`, which opens a folder beneath the one above and checks it is the listed one, on the root's volume.

## Kinds of removal

Each takes a `Verified` (consumed: one check serves one act) and an `Act`, which only a confirmed plan lends; none is
reachable from outside the crate (`remove/`).

- **To the Trash** (`remove/trash.rs`): the `Trash` port with the physical path and the held handle. The item is
  `Moved` to the reported landed path when that path holds the verified identity, else to where the handle's item is
  now when that lies in a Trash folder; when the handle's place is unknown, an item that left its place is `Moved`
  with nothing to put back; otherwise the call moved something else: `Failed` ("Another item was moved to the Trash in
  its place"), and the reported path is journalled so it can be put back. A volume without a Trash skips the item:
  never a permanent delete instead.
- **Emptied** (`remove/empty.rs`): the folder is verified and its place, opened through no link, must still hold it;
  its entries are listed from its held handle, each goes on its own and is checked against its listed identity: to
  the Trash after it is opened beneath the folder through no link (checks 5 to 7), or deleted beneath the held handle
  as below. The folder stays; the cancel token is read between entries.
- **Deleted** (`remove/permanent.rs`), with the plan's `PermanentConsent`: **the single deletion point**, the only
  `#[expect(clippy::disallowed_methods)]` of the workspace (`tests/deletion_point.rs` fails on any other, on a second
  one, or on silencing `clippy::style` or `clippy::all`). The deleting functions are `std::fs`'s and `remove_at`
  (all in `clippy.toml`); the deletion point calls only `remove_at`, from one function that checks the entry's
  identity right before it. The item's parent is opened through no link and its entry must be the verified item; a
  file or a link (as the link) is removed at once, a folder depth first with one open folder per level: each folder
  opened beneath the one above and checked against its listing and the root's volume (a volume mounted inside is
  never crossed), each entry checked against its listing right before it goes, then the emptied folder itself. The
  first refusal (a lock, a replaced entry, a link where a folder was, another volume) stops it and the item `Failed`:
  what went before the refusal stays gone, the rest stays in place. Afterwards the parent's entry must no longer be
  the item. The system's deleting calls are named only in `crates/macos/src/ffi/beneath.rs` (`tests/deletion_point.rs`),
  and no other crate may hold the unsafe code to call them.
- **By its tool** (`remove/tool.rs`): the program and its fixed arguments through `CommandRunner`, no shell; exit 0
  is done, anything else fails with the first line of its error (shown, not journalled).

## Fail closed

- No snapshot of open files (at the start or at a refresh): every item from then on is skipped (`CheckFailed`); a
  rehearsal without one skips every item. The engine has no other line for this.
- A root that is not absolute and plain after `~` is read, or cannot be made canonical: nothing under it is acted on.
- The cleaner never forces, never retries with more rights, never quits an app, never falls back from the Trash to a
  delete.

## Put back

`locate` looks at each landed path without following a link: `InTrash` (its device, file id and kind), `Replaced`,
`Gone`. `put_back` needs `InTrash`, then the original path must pass checks 1 to 4 today and its folder must open
through no link on the root's volume; the `Trash` port moves it back and never replaces anything (`Occupied`,
`NoParent`); afterwards the original path must hold the item.

## The threat model

| Threat                                                  | Defence                                                                                        | Left                                                                                     | Proved by                                                                                                              |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| A link points elsewhere                                 | `O_NOFOLLOW_ANY` on the whole path; a link item is removed as the link                         | firmlinks, transparent by design and on the same volume                                  | `paths.rs` (link in the middle, to outside, as the item), `remove/permanent.rs` (a link inside a folder), `never.rs`  |
| A path outside the roots                                | the lexical check, the open beneath the canonical root, the same device                        | nothing                                                                                  | `paths.rs`, `never.rs` (made-up, dotted, root, canonical-form paths)                                                   |
| A protected path, a parent of one, a user's own folder  | both protections by components, check 8 for what macOS protects                                | a place the list misses                                                                  | `paths.rs`, `pipeline.rs` (the system's list alone), `never.rs` (never, carve-out, whole, ignored, their ancestors)    |
| The item changed between prepare and act                | the identity through the open handle; the folder walk right before the act                     | a same-size rewrite inside a folder that kept its identity                               | `guard/mod.rs` (replaced, kind changed, grown), `guard/walk.rs`                                                        |
| The item changed between the check and the act         | the handle kept open; the landed check after a move; deletes and the walk beneath held folders | the Trash call takes a path (detected after the move); an entry replaced inside a held folder between its check and `unlinkat` | `remove/trash.rs` (decoy, unlocated), `remove/empty.rs` (an entry or the folder swapped for a link), `remove/permanent.rs` (a parent and a folder inside swapped during the delete, 100 races each; a parent swapped or a folder replaced after the check), `never.rs` (the swap case) |
| A file in use                                           | the open-files snapshot, refreshed every 2 s                                                   | other users' processes; withheld paths; a file opened after the snapshot                 | `guard/in_use.rs`, `guard/mod.rs`, `pipeline.rs` (refresh after two seconds, not before)                               |
| Another volume inside a folder                          | the walk compares devices before any delete; the delete never descends into another volume     | nothing                                                                                  | `guard/walk.rs`, `paths.rs`, `pipeline.rs`, `remove/permanent.rs` (disk images mounted inside the fixture)             |
| A wrong selection or rule                               | only a confirmed plan's items, re-checked; tools' arguments never forced                       | a harmless-looking wrong item, recoverable from the Trash                                | `never.rs`, `guard/mod.rs`                                                                                             |
| A development accident                                  | `Reach::Sandbox` in every debug build and test; the token only in the release composition      | a release build run against the real disk, which is the product working                  | `reach.rs` and `tests/reach.rs`, `pipeline.rs` (no tool runs in the sandbox)                                            |
| Reusing a check, or acting without a confirmation       | `Verified` consumed, not `Clone`, made only by the guard; removals private, each taking an `Act` | nothing                                                                                  | `tests/types.rs` with `tests/compile_fail/` (`verified_clone`, `verified_used_twice`, `verified_literal`, `remove_without_act`) |
| A second deletion point                                 | the deletion lint (`std::fs`'s and `remove_at`), expected once; the system's calls in one file | nothing                                                                                  | `tests/deletion_point.rs`                                                                                              |

`tests/never.rs` holds it for any plan: random trees with links in, out and up and a protected, carved-out,
hard-blocked and ignored folder; random plans of real and made-up paths, actions and methods; a rehearsal at both
depths changes nothing, and the execution removes only plan items (or what is inside them) that the model allows; the
outside folder and every protected subtree list exactly as before. The same holds for a permanent cleanup while a
folder of the root is swapped for a link to the outside folder, which holds a twin of everything in it, and back on
another thread (`fresher-testing`'s `Swapper`). A failing seed is kept in `never.proptest-regressions`, written by the
first failure (none so far); `PROPTEST_CASES=10000` runs it longer by hand.

## Adding a kind of removal

A new kind goes into `remove/` and is called only from `pipeline.rs`. It must:

1. take a `Verified` by value and an `Act`, and nothing else that names an item;
2. never delete itself: a permanent removal goes through `remove::delete` (or `delete_entry` beneath a held folder),
   the single deletion point;
3. say where everything went that can be put back (`TrashedItem`), and never fall back to a delete;
4. read the cancel token between the parts it acts on, if it acts on more than one;
5. bring its negative tests first, a case in `tests/never.rs`'s generator, and a row in the threat model above.
