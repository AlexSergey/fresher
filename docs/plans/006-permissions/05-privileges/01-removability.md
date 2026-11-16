# 006/05/01 — Removability of findings

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, engine

Needs plan 004/03: each `Match` carries what the walk read for the entry — whether it belongs to the current user,
whether the app may write to its parent folder (from the effective-access attribute of the bulk read), and its file
flags (user-immutable, system-immutable, restricted). If 004 carries them under other names, this step maps them.

## Files

- `crates/core/src/removability.rs` (new) — `Removability` (`Removable`, `NeedsAdministrator`, `Locked`,
  `ProtectedBySystem`), exported to TypeScript; `EntryFacts` (the OS-neutral facts above, as plain booleans) and the
  pure `Removability::of(&EntryFacts)`. Order of precedence: protected by the system, then locked, then needs an
  administrator, then removable. A tool removal (`ToolCommand`) is always `Removable` here; its need is the tool.
- `crates/core/src/finding.rs` — `Finding.removability`, set by the engine, never by a module (a module cannot make an
  item removable).
- `crates/engine/src/session.rs` — fills `removability` from the facts of the match whose path the finding's removal
  names (the engine keeps the facts of this scan's matches by path, so a finding made in `finish` — a stale version —
  gets them too). A path removal with no such match is `NeedsAdministrator` — fail closed — and the module kit's
  contract test flags it.
- `crates/core/locales/en.json` — `core.removability.<state>` (a short label) and `.how` (the instruction of the
  design, section 9; for `ProtectedBySystem` an explanation with no instruction).

## Produces

`Removability` for plan 008: its plan builder refuses an item that is not `Removable`, and its guard re-reads the facts
right before acting. The UI of step 02.

## Checks

- Negative (first): `an_item_of_another_user_needs_an_administrator`, `an_item_in_a_folder_the_app_cannot_write_needs_an_administrator`,
  `a_locked_item_is_locked`, `a_restricted_item_is_protected_by_the_system_even_if_writable`,
  `a_module_cannot_set_removability`, `a_path_finding_without_a_match_is_not_removable`.
- Positive also: `a_finish_finding_takes_the_facts_of_its_match`.
- Positive: `a_users_own_writable_item_is_removable`, `a_tool_removal_is_removable`,
  `removability_serializes_in_kebab_case`, property test: `Removability::of` is `Removable` only when all four facts
  allow it.

## Conclusions

- `crates/core/src/removability.rs` holds `Removability` (kebab-case, exported to TypeScript) with the pure
  `Removability::of(EntryFacts)` in the order of the step (protected by the system, locked, needs an administrator,
  removable) and `Removability::of_removal(&Removal, Option<EntryFacts>)` (a tool's removal is `Removable`, a path
  with no facts `NeedsAdministrator`). `Finding` carries `removability` (serialized next to `id` and `module`),
  passed to `Finding::new` by the engine only; `NewFinding` has none, so a module cannot set it. The session keeps the
  facts of every match of the scan by path and fills each finding from the entry its removal names, also in
  `finish`. Texts: `core.removability.<state>.name` and `.how` (none for `removable`).
- **Core change beyond the step's file list (recorded as asked):** plan 004 carries no entry facts, so
  `Match` gained `facts: EntryFacts` (`interest.rs`). `EntryFacts` is OS-neutral plain booleans whose default is
  "nothing known" and not removable, so a producer that cannot tell fails closed; `EntryFacts::REMOVABLE` names the
  user's own writable item. **Plan 004/05's walker must fill it** (the owner, `ATTR_CMN_USERACCESS` for the parent and
  the folder, `ATTR_CMN_FLAGS`; `RawEntry` has none of them yet): until it does, every path finding of the real app is
  `NeedsAdministrator`, never wrongly removable.
- Not spelled out by the step, taken the safe way:
  - **Six facts, not four**: `owned_by_user`, `parent_writable`, `folder_writable` (moving or emptying a folder
    writes inside it: its link to its parent, or its entries; `true` for a file or a link), `user_immutable` →
    `Locked`, and `system_immutable` or `restricted` → `ProtectedBySystem` (only the system can clear either; no
    instruction is offered). The property test checks that `of` is `Removable` exactly when all six allow it.
  - The keys are `core.removability.<state>.name` rather than `core.removability.<state>`, like
    `core.danger.<level>.name`, so no key is a prefix of another.
  - `NaiveWalk` reads the facts with `std::fs` (`entry_facts.rs`, its own file: `naive_walk.rs` is past 700 lines):
    the user is the owner of its home folder, write access comes from the mode bits (the owner's bit for the user's
    own entry, the others' bit otherwise; a group never counts), the flags from `st_flags` on macOS.
  - The module kit's flag is `check_removals_matched(&runs)` with `ContractProblem::UnmatchedRemoval { identity }`
    (`ModuleRun` records its matched paths). The `demo` module's made-up paths are all flagged, which its contract
    test asserts; its golden report shows its path findings as `needs-administrator` and its tool findings as
    `removable`. `docs/testing.md` and `docs/architecture/modules.md` name the check.
- Regenerated: the bindings (`Removability`, `Finding.removability`) and `fixtures/golden/demo.json`; the UI's demo
  findings carry `removability`.
- **The walk reads the entry facts** (the open item above, closed on its own branch after plan 004 was taken in):
  - **Core change, recorded as asked:** `RawEntry` gained `access: EntryAccess` — the owner's user id (`None` when
    the reader cannot tell), the permission bits (`0o7777` of the mode) and the three locks (user-immutable,
    system-immutable, restricted). Its default owns nothing, allows nothing and locks nothing. `FolderReader` gained
    `access(path)`: the same of a folder, for the scan roots and the home folder (one call each per walk).
  - macOS (`MacFolderReader`): the bulk read asks `ATTR_CMN_OWNERID` and `ATTR_CMN_ACCESSMASK` in the same call as
    before (it already asked `ATTR_CMN_FLAGS`): no call is added per entry; `access` reads `fstat` of the opened
    folder. Portable (`StdReader`): `symlink_metadata`'s `uid`, `mode` and, on macOS, `st_flags`. Both decode the
    flags alike; the reader test checks the bulk read against `std` entry by entry.
  - **Not `ATTR_CMN_USERACCESS`**, as the bullet above proposed: the portable reader cannot read it without an FFI
    call, and the facts must be the same from both readers and the reference walk. Write access is read from the
    mode bits as `NaiveWalk` reads it (the owner's bit for the user's own entry, the others' bit for any other; a
    group or an access list never counts). It fails closed: an item a group lets the user write (an admin user's
    `/Applications`) shows as needing an administrator. A call to check if such places get a module.
  - "The user" is the owner of the home folder, read through the reader when the walk is prepared (as `NaiveWalk`
    does); without one, nothing is owned. A restore reads nothing.
  - The scanner keeps a node's access for that user in one byte that was padding (`tree/access.rs`, `NodeAccess`:
    owned, writable, the three locks; the node stays 72 bytes), and works out a match's facts from its node and its
    parent's (`Tree::facts`), in the walk, the rescan and the restore alike. A rescan copies the access of a node
    whose folder was not read again; a root's is read again.
  - **The cache's tree format is 2** (was 1): every node's record ends its owner with its access byte. A cache of
    format 1 is refused and discarded: the first scan after the update is full. About 1 byte a node (24 → 25 on the
    SMALL tree).
  - The comparison of the scanner with `NaiveWalk` keeps the facts (`crates/scanner/tests/support`); new tests cover
    the user's own file, a read-only file (removable: its folder is writable), a file in a read-only folder, a
    read-only folder and a file locked with `chflags uchg`, from both readers, and that no read entry gets
    `EntryFacts::default()`; a rescan after a folder is made read-only and a restore give the facts of a full scan.
  - Measured on the SMALL bench tree (`npm run bench`, `scan/full`): the machine was loaded by other work, so the
    before and after binaries were run interleaved four times: 1 thread 395 → 383 ms, 12 threads 260 → 261 ms
    (medians), no difference beyond the noise. A quieter run against the saved baseline gave 68 → 73–76 ms at 12
    threads (+4 to +9 %); the cache's encode and decode +6 %.
  - Added by the main session: the append-only flags (`UF_APPEND`, `SF_APPEND`) and the no-unlink flag
    (`SF_NOUNLINK`) count as the user's or the system's lock, in both readers and the reference walk, so such an item
    is never shown as removable (fail closed).
  - A disk image of FAT no longer mounts on the reference Mac (`hdiutil attach`: "no mountable file systems", in every
    FAT variant); the journal test of a volume without a journal now uses an APFS image mounted with `-nobrowse`,
    which has no journal either.
  - The bench tree under `target/bench-trees/` of a cloned `target/` is made of clones (every file then costs a
    private-size call, and `the_macos_reader_and_the_std_reader_build_the_same_tree` fails): it was removed and
    built again before measuring.
