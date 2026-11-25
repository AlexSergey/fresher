# Removal

How Fresher removes anything from the disk, for a developer about to touch it. Everything here lives in
`fresher-cleaner` (`crates/cleaner/`), the only code of the workspace that removes, moves to the Trash or puts back.
The design and its reasons: plan [008](../plans/008-safe-removal/00-design.md), sections 2 and 5 to 8; the rules it
enforces: [`restrictions/deletion-safety.md`](../restrictions/deletion-safety.md).

## The crate and its one entry point

- `DiskCleaner` implements the core's `Cleaner` port: `rehearse`, `execute`, `locate`, `put_back`. Nothing else of
  the crate is public but its constructor's inputs (`Reach`, `RealDiskToken`, in debug builds `DryRunToken`,
  `CleanerPorts`) and the `Verified` type, which only the guard can make.
- `DiskCleaner::new(reach, user, home, protection, ports)`: the reach (below), the uid the app runs as, the home folder
  (a root written `~` or `~/…` is read under it), the system's protected places, and the ports it acts through
  (`Trash`, `OpenFiles`, `RunningApps`, `CommandRunner`, `Clock`). The app gives the system's Trash to the cleaner and
  to nothing else.
- It depends on `fresher-core` and, on macOS, on `fresher-macos`'s `beneath` module (the calls that name one entry of
  a held folder, below); it reaches the disk through `std`, those calls and the ports, so every test runs on a
  temporary fixture tree with the fakes of `fresher-testing`. Off macOS nothing opens and nothing is acted on.

## The flow

1. **Rehearse** (`pipeline.rs`): the engine's plan builder (its inputs are the scan's findings and the picked own
   files of the Largest screen, plan 035: a picked file joins as one more finding of `core.own-file`, nothing else
   changes) asks for every check of every item (`Depth::Quick`, no folder walk) and gets each item's identity or why
   it would be skipped. A full rehearsal is the same with `Depth::Full`. An item of several paths runs every check on each part, in order: the first part refused skips the
   item with that part's reason, else it answers every part's identity (`PassedTogether`).
   A rehearsal holds an unconfirmed plan's items and no `Act`, so it cannot call a removal: the types say so.
2. **Execute** (`pipeline.rs`): only a `ConfirmedPlan`, taken by value, so it runs once. Item by item in plan order,
   never in parallel: the cancel token (once set, every item left is `NotReached`), the snapshot of open files taken
   again when two seconds have passed on the clock, every check with the folder walk (`Depth::Full`), the removal its
   action and the plan's method call for, then the item's `ItemDone` to the engine before the next item starts. An
   item of several paths passes every check on every part, each against its own confirmed identity and held open,
   before its first part is acted on (**Together**, below).
3. **Locate and put back** (`put_back.rs`): where each item moved to the Trash is now, and moving one back. What waits in the Trash after a move is shown by the Trash line
   ([`README.md`](./README.md), "Data"); nothing here empties it.

## Reach and the sandbox

`Reach::RealDisk(RealDiskToken)` lets the cleaner act anywhere the other checks allow. `RealDiskToken::new` exists only
under `cfg(not(debug_assertions))`, and `tests/reach.rs` fails when the token is named anywhere but its definition, the
cleaner's re-export and `app/tauri/src/compose.rs`; only a release build's composition makes it. A test is
`Reach::Sandbox`, a sandbox folder (below) or the dry run (a development build on the real Mac answers "a dry run"): an item counts only when the scan root, or a folder between the root and
the item (never the item itself), holds the sandbox marker `.fresher-sandbox` — a regular file of the user, opened
through no link, whose first line is the core's `SANDBOX_SIGNATURE` — in a folder the system's protection does not
refuse (never the home, a hard-blocked folder such as a mounted volume's root, or an ancestor of a protected place). A
tool acts on the real system, so it never runs in the sandbox.

The third form, `Reach::SandboxFolder(SandboxFolder)`, is sandbox mode (plan 007): one folder made by a sandbox
generator. `SandboxFolder::new` takes the folder only when it is no link itself, made canonical, holds a valid marker of
the user, and is neither a protected place, an ancestor of one nor a whole folder by the real Mac's protection list.
An item then counts only when it is strictly inside that folder by components (never the folder itself) and the
folder still holds its marker right then. The protection the cleaner gets in this mode is the real Mac's list joined
with the sandbox home's (`Protection::joined`: a place either list refuses stays refused, and a carve-out opens only
its own list's protected places). That joined list makes the sandbox's `home/` a whole folder, so the folder's marker,
above it, would not count under `Reach::Sandbox`; `SandboxFolder` checks the folder against the real list once, when it
is made. It never runs a tool, and it grants nothing a release build lacks.

The fourth form, `Reach::DryRun(DryRunToken)`, exists in debug builds only: it is how far a development build outside a
sandbox reaches. It admits what `RealDisk` admits (check 1), every later check runs on the real disk, and `act_on` then
ends each item that passed them `SkipReason::DryRun` ("a dry run") before its act, for every action and method: nothing
is moved, emptied or deleted, and no tool runs. Put back of an item in the Trash is refused `dry-run` before any check
of its original path or any port. `DryRunToken::new` is made only by `app/tauri/src/compose.rs` and the cleaner's own
tests (`tests/reach.rs` fails on any other place). Behind it the app gives such a build `DryRunTrash`
(`app/tauri/src/dry_run.rs`), a Trash that refuses every call: the second wall. What a dry run cannot rehearse, since it
stops before the act: an emptied folder's checks of its place and of each entry (they run inside the act), and the Trash
port's own answers such as `NoTrash`.

The app enters sandbox mode in its setup (`app/tauri/src/sandbox.rs`, compiled in debug builds only): a debug build
started with `FRESHER_SANDBOX=<absolute path>` lives entirely in that folder — its `home/` is the home of the scan roots
(`~`) and of the known folders, `.fresher-data/` holds the settings, the scan cache and the cleanup journal, and the
cleaner's Trash is the sandbox's `home/.Trash` (`FRESHER_SANDBOX_TRASH=real` chooses the system's Trash; any other
value is refused). A folder that is a link, is protected on the real Mac, holds no valid marker or has no `home/`
stops the start with the reason. Clock, volumes, open files, running apps, commands and access stay real and are only
read; `sandbox_engine` takes them as `SandboxPorts`, so the sandboxes' autotests run the same composition on fakes. A
release build has no such branch and reads neither variable.

The marker is read by one function, `fresher_macos::sandbox_marker::holds_marker`, shared with the sandbox generator:
the marker is opened with `O_NOFOLLOW_ANY` (no link in any component) and without waiting on a named pipe, must be the
entry the folder lists (same volume and file id as its `lstat`), a regular file of the user, and start with the
signature line. Off macOS no marker counts.

**To clean in a development build**, use sandbox mode (`npm run dev:sandbox -- <feature>`, above): it keeps the real
removal inside the sandbox. `npm run dev` on the real Mac makes every cleanup a dry run, a folder with a valid marker
included: each item is checked and ends "a dry run", and nothing on the disk changes.

## The guard

`guard/mod.rs` runs the same checks in the rehearsal and right before each act, in this order; the first that fails
skips the item with its reason. It reads the disk only through paths and handles that follow no link.

| #   | Check                                                                                       | Where              | Skip reason                         |
| --- | ------------------------------------------------------------------------------------------- | ------------------ | ----------------------------------- |
| 1   | Reach: a valid marker at or below the root, or strictly inside the sandbox folder           | `reach.rs`         | `NotInSandbox`                      |
| 2   | Inside the roots (the scan roots and the places the last scan walked): absolute, plain, strictly inside a root by components, never a root | `paths.rs` | `OutsideRoots` (`CheckFailed`) |
| 3   | Protection, both lists, both forms of the path                                              | `paths.rs`         | `Protected`, `WholeFolder`          |
| 4   | Not ignored                                                                                 | `paths.rs`         | `Ignored`                           |
| 5   | Opened beneath the canonical root with `O_SYMLINK \| O_NOFOLLOW_ANY`, kept open until the act | `paths.rs`       | `ThroughLink`, `Vanished`           |
| 6   | On its root's volume                                                                        | `paths.rs`         | `OtherVolume`                       |
| 7   | Still the confirmed item: device, file id, kind; a socket, a pipe or a device (`EntryKind::Other`) is refused whatever was confirmed | `guard/identity.rs` | `Replaced`, `KindChanged` |
| 8   | Removable now: owner, write access, flags, read again from the handle and its parent        | `guard/facts.rs`   | `NotRemovable { removability }`     |
| 9   | Capability of each guarded place it lies in or holds                                        | `guard/access.rs`  | `MissingCapability { capability }`  |
| 10  | Not in use: no open file of the user's processes is the item or inside it, and no running app owns it by bundle id (plan 009) | `guard/in_use.rs`  | `InUse { app }`                     |
| 11  | Not grown (`Depth::Full`): over by more than 25 % and 64 MiB; no other volume inside, walked beneath the handle | `guard/walk.rs` | `Grown`, `OtherVolume` |

- A check that cannot tell (an IO error other than "not found", a root that cannot be made canonical, an act on an
  item with no identity) skips with `CheckFailed`.
- The guard opens an item without waiting on a named pipe (`O_NONBLOCK`, `paths.rs`): a pipe with no writer never
  blocks it, and check 7 then skips it (`a_pipe_with_no_writer_does_not_block_the_guard`).
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
  now when that is not its original place and lies in a Trash folder; when the handle's place is unknown, an item
  that left its place is `Moved` with nothing to put back; otherwise the call moved something else: `Failed`
  ("Another item was moved to the Trash in its place"). What it moved instead is never journalled: where it came from
  is unknown, and put back would bring it to the verified item's place (Finder's own Put Back knows its place). A
  volume without a Trash skips the item: never a permanent delete instead.
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
- **Together** (`remove/together.rs`), an item of several paths (`Action::RemoveTogether`): `verify_together` runs
  checks 1 to 10 on every part in order, each against its own confirmed identity, keeping each part held open; with
  `Depth::Full` it measures every part and checks growth (check 11) once, on the sum of the parts against the item's
  confirmed size. The first refusal skips the whole item with that part's reason and nothing has moved; a dry run ends
  it there. Then the parts go in order, the subject first, each with its own `Verified` and the plan's way (to the
  Trash, or deleted at the single deletion point); the cancel token is read before each part. Every part gone:
  `Moved` (or `Deleted`) with every part in `trashed`. A part refused, failed or cancelled after an earlier one went
  ends the item `Failed` ("only part of it went", `Failure::Partly { done, of, way }`, the way `Trash` or `Deleted`
  with its own text, so a delete never says its files are in the Trash) and nothing more is tried, with every part
  that went in `trashed`; the first part failing is that part's own outcome, as a single item's. It never
  falls back from the Trash to a delete.
- **By its tool** (`remove/tool.rs`): the program and its fixed arguments through `CommandRunner`, no shell; exit 0
  is done, anything else fails with a plain sentence and the first line of its error as the text's `detail` (at
  most 200 characters, shown only on demand, not journalled).

## Fail closed

- Before the cleaner is reached, the engine checks the preconditions of a cleanup when a plan is prepared (a cleanup's
  and an uninstall's alike) and again when it is confirmed, in this order, and refuses with `CleaningBlocked` and the
  first that fails (`crates/engine/src/cleanup/blocked.rs`): the settings file is loaded, so the ignore list is known
  (`Settings`); the current version of the agreement is accepted (`Agreement`,
  [`deletion-safety.md`](../restrictions/deletion-safety.md), "The agreement"); the cleanup journal can be written
  (`Journal`). Put back checks none of them but the journal's. The agreement is accepted by `accept_agreement`, which
  never replaces a settings file that was not read.
- A confirmation carries two consents beside the echo, both checked by the core and neither remembered: the permanent
  choice (`PermanentConsent`) and, for a plan whose highest level is High, `highRiskUnderstood`
  (`ConfirmError::HighRiskNotUnderstood`, the plan consumed;
  [`deletion-safety.md`](../restrictions/deletion-safety.md), "A High plan needs its own tick"). The UI gates both in
  its action layer (`state/cleanup-actions.ts`); the sheet and the dialog are not the guard.
- No snapshot of open files (at the start or at a refresh): every item from then on is skipped (`CheckFailed`); a
  rehearsal without one skips every item. The engine has no other line for this.
- A root that is not absolute and plain after `~` is read, or cannot be made canonical: nothing under it is acted on.
- The cleaner never forces, never retries with more rights, never quits an app, never falls back from the Trash to a
  delete.

## Put back

The engine trusts a journal entry only when it landed right inside one of the user's Trash folders and its original
place is the item's own as the history shows it; any other entry is `Gone` before the cleaner is asked
(`Located::of`, `TrashFolders`; [deletion safety](../restrictions/deletion-safety.md), "Trash first"). The journal is
a plain file, so a forged entry can name neither a file outside the Trash nor a place other than the one shown. The
home's Trash is compared as the disk spells it: the app resolves a home reached through a link once at its start
(`compose::disk_home`), as the system's Trash reports where an item landed.

`locate` looks at each landed path without following a link: `InTrash` (its device, file id and kind), `Replaced`,
`Gone`. `put_back` needs `InTrash`, then the original path must pass checks 1 to 4 today and its folder must open
through no link on the root's volume; the `Trash` port moves it back and never replaces anything (`Occupied`,
`NoParent`); afterwards the original path must hold the item. It is refused `dry-run` in a development build outside
a sandbox.

An item of several paths (`Action::RemoveTogether`) is put back as a whole, so it never comes back as a list that names
a file still in the Trash: the parts after the subject that went and are not back yet first, in the reverse of the
order they moved, then the subject, and only when every other part that went is back (brought back now, or by an
earlier attempt, which the journal's `back` records). Otherwise the subject stays in the Trash and the answer is the
first refusal. A part that went but whose landing the Trash did not report, and a part whose entry left the Trash
without being brought back, cannot come back and keep the subject there; a part that landed elsewhere than its own
place counts as one that went (`went`). A record that does not hold together (more parts went than the item has, an
entry of a path that is none of its parts) puts nothing back. Known limit: a part put back and then deleted by the user
counts as back, so a later attempt may bring the subject back without it (the journal records what was brought back,
not what the disk holds afterwards).

## The threat model

| Threat                                                  | Defence                                                                                        | Left                                                                                     | Proved by                                                                                                              |
| ------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| A link points elsewhere                                 | `O_NOFOLLOW_ANY` on the whole path; a link item is removed as the link                         | firmlinks, transparent by design and on the same volume                                  | `paths.rs` (link in the middle, to outside, as the item), `remove/permanent.rs` (a link inside a folder), `never.rs`  |
| A path outside the roots                                | the lexical check, the open beneath the canonical root, the same device                        | nothing                                                                                  | `paths.rs`, `never.rs` (made-up, dotted, root, canonical-form paths)                                                   |
| A protected path, a parent of one, a user's own folder  | both protections by components, check 8 for what macOS protects                                | a place the list misses                                                                  | `paths.rs`, `pipeline.rs` (the system's list alone), `never.rs` (never, carve-out, whole, ignored, their ancestors)    |
| The item changed between prepare and act                | the identity through the open handle; the folder walk right before the act                     | a same-size rewrite inside a folder that kept its identity                               | `guard/mod.rs` (replaced, kind changed, grown), `guard/walk.rs`                                                        |
| The item changed between the check and the act         | the handle kept open; the landed check after a move; deletes and the walk beneath held folders | the Trash call takes a path (detected after the move); an entry replaced inside a held folder between its check and `unlinkat` | `remove/trash.rs` (decoy, unlocated), `remove/empty.rs` (an entry or the folder swapped for a link), `remove/permanent.rs` (a parent and a folder inside swapped during the delete, 100 races each; a parent swapped or a folder replaced after the check), `never.rs` (the swap case) |
| A file in use                                           | the open-files snapshot and the running apps by bundle id, both refreshed every 2 s; a failed listing of either acts on nothing | other users' processes' open files where no running app owns the item by name; withheld paths; a file opened or an app started after the snapshot | `guard/in_use.rs`, `guard/mod.rs`, `pipeline.rs` (refresh after two seconds, not before)                               |
| Another volume inside a folder                          | the walk compares devices before any delete; the delete never descends into another volume     | nothing                                                                                  | `guard/walk.rs`, `paths.rs`, `pipeline.rs`, `remove/permanent.rs` (disk images mounted inside the fixture)             |
| A wrong selection or rule                               | only a confirmed plan's items, re-checked; tools' arguments never forced                       | a harmless-looking wrong item, recoverable from the Trash                                | `never.rs`, `guard/mod.rs`                                                                                             |
| A development accident | `Reach::DryRun` in a development build (every check, no act; `DryRunTrash` behind it), `Reach::Sandbox` or a sandbox folder in tests and sandbox mode; the real-disk token only in the release composition | a release build run against the real disk, which is the product working | `reach.rs` and `tests/reach.rs`, `pipeline.rs` (a dry run acts on nothing), `tests/never.rs` (`a_dry_run_changes_nothing_in_any_tree`), `app/tauri/src/compose.rs` (`the_development_engine_moves_nothing_on_the_real_disk`) |
| Reusing a check, or acting without a confirmation       | `Verified` consumed, not `Clone`, made only by the guard; removals private, each taking an `Act` | nothing                                                                                  | `tests/types.rs` with `tests/compile_fail/` (`verified_clone`, `verified_used_twice`, `verified_literal`, `remove_without_act`) |
| An item of several paths half done | every part through checks 1 to 10 against its own identity and held open, growth on the sum of the parts, all before the first part moves; the subject first; every moved part in `trashed`, so it is journalled | a part the Trash refuses, or a cancel, after earlier parts moved: reported "only part of it went", what went can be put back | `pipeline.rs` (a part outside the roots, protected, through a link, replaced, vanished, grown, held open, mapped, owned by a running app: no part moves; the dry run), `remove/together.rs` (partly, in the Trash way and the delete way; cancel between parts; the first part's own failure), `guard/mod.rs` (growth on the sum), `never.rs` (`a_together_item_goes_whole_or_not_at_all`) |
| An app bundle removed by a plan that is not an uninstall | the protection opens only a whole `.app` bundle in the apps folders (never inside one, never the folder or `<folder>/<sub>`), and the engine's context refuses both apps folders and the cleaner's own app folders for every plan but an uninstall (`Protection::with_refused`); an uninstall goes only to the Trash (a permanent confirmation is refused); the cleaner's roots need the plan to name the folder, and only an uninstall plan does; the uninstall is made from the listing and refused unless the bundle's canonical path equals the listed path byte for byte and is a real folder (at prepare and at confirm) | the protection is lexical, so a link or a name the volume folds to another passes it by name (the listing, the plan's canonical check and the cleaner's `O_NOFOLLOW_ANY` stop it, [known issue](../known-issues/core-protection-passes-a-bundle-shaped-link.md)); the cleaner compares names as written ([known issue](../known-issues/cleaner-protection-compares-names-as-written.md)); an app opened between the confirmation and the move, or during a move of many parts, is not asked again ([known issue](../known-issues/cleaner-app-opened-after-the-confirmation.md)) | `crates/core/src/cleanup/protection.rs` (`protection_tests.rs`), `crates/engine/src/cleanup/context.rs`, `crates/engine/src/apps/uninstall_tests.rs`, `crates/e2e/tests/uninstall.rs` (`an_ordinary_cleanup_never_moves_a_bundle_in_either_apps_folder`, mutation-checked; `an_ordinary_cleanup_never_moves_a_path_inside_a_bundle`; `a_permanent_uninstall_is_refused`); the cleaner's code is unchanged |
| A second deletion point                                 | the deletion lint (`std::fs`'s and `remove_at`), expected once; the system's calls in one file | nothing                                                                                  | `tests/deletion_point.rs`                                                                                              |

`tests/never.rs` holds it for any plan: random trees with links in, out and up and a protected, carved-out, hard-blocked
and ignored folder; random plans of real and made-up paths, actions (items of several paths among them) and methods; a
rehearsal at both depths changes nothing, and the execution removes only plan items (or what is inside them) that the
model allows; the outside folder and every protected subtree list exactly as before; an item of several paths acted on
went whole, and one skipped moved no part. A lone item of several paths, the Trash refusing one of its parts or none,
goes whole or not at all, unless the Trash refuses a part at its act: then exactly the parts before it went. The same
holds for a permanent cleanup while a folder of the root is swapped for a link to the outside folder, which holds a twin
of everything in it, and back on another thread (`fresher-testing`'s `Swapper`). A failing seed is kept in
`never.proptest-regressions`, written by the first failure (none so far); `PROPTEST_CASES=10000` runs it longer by hand.

## Adding a kind of removal

A new kind goes into `remove/` and is called only from `pipeline.rs`. It must:

1. take a `Verified` by value and an `Act`, and nothing else that names an item;
2. never delete itself: a permanent removal goes through `remove::delete` (or `delete_entry` beneath a held folder),
   the single deletion point;
3. say where everything went that can be put back (`TrashedItem`), and never fall back to a delete;
4. read the cancel token between the parts it acts on, if it acts on more than one;
5. bring its negative tests first, a case in `tests/never.rs`'s generator, and a row in the threat model above.
