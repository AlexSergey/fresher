# Deletion safety

The app deletes files from a real person's disk. These rules hold in every version.

## Explicit confirmation only

- **Rule:** nothing is deleted unless the user confirmed that exact set of items in the current session, after seeing
  their total size and the highest danger level among them. No background, scheduled or automatic deletion.
- **Why:** the user asked for it explicitly; a cleaner that deletes on its own cannot be trusted.
- **Enforced by:** the types of the core (plan 008/02). `Cleaner::execute` takes only a `ConfirmedPlan`, by value, so
  it runs once; its only constructor is `CleanupPlan::confirm`, which compares the user's `Confirmation` (the plan id,
  the item count, the allocated size and the highest danger level the dialog showed) with the plan the engine holds,
  and refuses one older than `PLAN_LIFETIME` (`crates/core/src/cleanup/confirm.rs`). A removal needs an `Act`, which
  only a confirmed plan lends. The engine keeps one pending plan, in memory only
  (`crates/engine/src/cleanup/pending.rs`). Tests: the compile-fail cases of `crates/core/tests/types.rs` and
  `crates/cleaner/tests/types.rs` (each crate's `tests/compile_fail/`); the property test
  `any_changed_field_of_the_echo_refuses` (`confirm.rs`); the core e2e `crates/e2e/tests/cleanup.rs` (a wrong count,
  an old plan: nothing moved, nothing journalled).
- **An item may be several paths** (a model of a local AI tool: its list and the layers only it uses): the user
  confirms the item, not its files one by one; the confirmation lists the other paths on demand ("and N more files
  that go with it"), the item count, size and highest danger are the item's, and every path of it is checked before the
  first moves (below). Tests: `app/ui/src/product/confirm-dialog/confirm-dialog.test.tsx` (the toggle, the list, the
  keyboard),
  `a_together_item_shows_its_subject_and_the_parts_after_it` (`crates/core/src/cleanup/view.rs`),
  `a_together_item_with_a_covered_part_is_left_out_whole` (`crates/engine/src/cleanup/plan.rs`).

## Scanning is read-only

- **Rule:** the scanner and the rules never write to, move or delete anything in a scanned root.
- **Why:** a scan must be safe to run at any time.
- **Enforced by:** Clippy `disallowed-methods` forbids the deleting functions (`std::fs`'s and `fresher-macos`'s
  `remove_at`) outside the cleaner's single deletion point (plans 001 and 008); the crate layout. The same lint
  forbids looking up the home folder; the only other files that may expect it are the two home lookups (the app's
  setup and the sandbox generator's binary), once each, for that lookup only (`HOME_LOOKUPS` in
  `crates/cleaner/tests/deletion_point.rs`).

## Trash first

- **Rule:** items are moved to the Trash by default; a permanent delete is a separate, explicit choice per confirmation.
- **Why:** a mistake must be recoverable.
- **Enforced by:** the cleaner's API (plan 008/04). A plan's `Method` is `Trash` unless the user ticked the permanent
  choice; every part of an item of several paths goes the same way, the subject first, and put back brings the subject
  back last and never alone (`remove/together.rs`; `put_back_together` in `crates/engine/src/cleanup/history.rs`); a
  delete takes the `PermanentConsent` that only `confirm` makes, and only for `Method::Permanent`
  (`crates/cleaner/src/pipeline.rs`, `remove/permanent.rs`). A volume with no Trash skips the item (`NoTrash`), never
  deletes it instead (`remove/trash.rs`). Tests: `permanent_consent_literal` (`crates/core/tests/compile_fail/`),
  `a_permanent_method_alone_gives_no_consent_for_the_trash` (`confirm.rs`), `remove/trash.rs`, `pipeline.rs`
  (`the_trash_method_moves_and_the_permanent_method_deletes`); for an item of several paths,
  `every_part_goes_to_the_trash_the_subject_first` and `the_permanent_way_deletes_each_part_through_the_deletion_point`
  (`remove/together.rs`), `a_together_item_comes_back_parts_first_and_the_subject_last` and
  `a_together_item_whose_layer_cannot_come_back_keeps_its_subject_in_the_trash` (`history.rs`).

## Re-check before delete

- **Rule:** right before an item is deleted, the cleaner checks it is still what was confirmed (same path, same kind, not
  replaced by a symlink, size not grown past the confirmed one by a large margin); otherwise it skips the item and
  reports it.
- **Why:** the disk can change between the scan and the confirmation.
- **Enforced by:** the cleaner's guard (`crates/cleaner/src/guard/mod.rs`), run in the rehearsal and again right
  before each act ([`../architecture/removal.md`](../architecture/removal.md), "The guard"): the item opened beneath
  its root through no link and kept open until the act (check 5), on its root's volume (6), still the confirmed device,
  file id and kind (7), and not grown by more than 25 % and 64 MiB, walked beneath the held handle (11); the first
  check that fails skips the item with its reason. Tests: `paths.rs`, `guard/mod.rs`, `guard/walk.rs`, the race tests
  of `remove/permanent.rs`, `tests/never.rs`; the core e2e
  `an_item_replaced_after_prepare_is_skipped_and_the_rest_done` (`crates/e2e/tests/cleanup.rs`). An item of several
  paths is re-checked in every part, each against its own confirmed identity and held open, and its growth on the sum
  of the parts, all before the first part moves: one part refused skips the whole item and nothing moves. Tests:
  `a_part_outside_the_roots_moves_no_part`, `a_protected_part_moves_no_part`, `a_replaced_part_moves_no_part`,
  `a_grown_part_moves_no_part`, `a_part_held_open_moves_no_part`, `a_mapped_part_moves_no_part` (`pipeline.rs`),
  `parts_that_grew_together_past_the_margin_are_grown` (`guard/mod.rs`), `a_together_item_goes_whole_or_not_at_all`
  (`tests/never.rs`); the core e2e `crates/e2e/tests/local_ai_models_ollama.rs`
  (`a_model_whose_layer_is_mapped_moves_nothing`, `a_layer_replaced_after_the_plan_moves_nothing`).

## Never follow symlinks out, never touch protected paths

- **Rule:** the scanner and the cleaner follow no links — neither symlinks nor Finder aliases — so a link can never
  point them somewhere unexpected; nothing under `/System`, `/usr` (except `/usr/local`), `/bin`, `/sbin`,
  `/private/var/db`, SIP-protected locations, or another user's home is ever offered for deletion. Docker objects are
  removed only through the Docker CLI, never by deleting its files.
- **Why:** deleting there breaks macOS or another user's data; Docker's files are its own database.
- **Enforced by:** the scanner lists a link as itself and never follows it (`crates/scanner/tests/reference.rs`). The
  cleaner's checks 2, 3 and 5: an item strictly inside a root by components, refused by the protection list
  (`fresher-macos`'s `protection`, `crates/macos/src/protection.rs`: `/System`, `/usr` but `/usr/local`, `/bin`,
  `/sbin`, `/private/var/db`, every other home, the Trash; `/private/var/folders` but the user's own temporary folder
  `<xx>/<hash>/T` (checked to be exactly that shape, `is_user_temp`), whose `TemporaryItems` stays protected;
  `/private/var/tmp`, `/private/var/vm`, `/.DocumentRevisions-V100` and `~/Library/Autosave Information`; the user's
  `T` and `/private/tmp` are whole, so only an item inside them can go, never the folder; both forms of the path),
  and opened with `O_NOFOLLOW_ANY`;
  a delete and an emptying act beneath held folders (`fresher-macos`'s `beneath`); check 8 refuses what the system's
  flags protect (SIP's `restricted`, the immutable flags). A tool removal runs its program with fixed arguments
  through `CommandRunner`, never `-f` or `--force` (`remove/tool.rs`, `guard/mod.rs`). Tests: `paths.rs`,
  `remove/permanent.rs`, `tests/never.rs` (random trees with links in, out and up, a protected and a carved-out
  folder), `crates/macos/src/protection.rs`. A folder setting (the scan roots) that names a protected folder, a folder
  inside it or above it (`/`, `/private`, `/Users`), or that is not `~`, `~/…` or a plain absolute path, is refused by
  the core's settings check (`crates/core/src/settings/folders.rs`). A development build has a third reach, a sandbox
  folder (`Reach::SandboxFolder`, [`../architecture/removal.md`](../architecture/removal.md)): the cleaner acts only
  strictly inside a folder that holds a valid marker, is no link (also named with a trailing slash) and is no
  protected or whole place of the real Mac, with the real and the sandbox protection lists joined, so it removes
  nothing the real list would keep. Tests: `crates/cleaner/tests/reach.rs`, `crates/core/src/cleanup/protection.rs`,
  `app/tauri/tests/sandboxes.rs`. A development build outside a sandbox has a fourth, the dry run (`Reach::DryRun`,
  debug builds only): every check runs on the real disk and no item is acted on; it ends "a dry run". What it cannot
  rehearse is what runs inside the act: an emptied folder's checks of its place and of each entry, and the Trash's own
  answers such as `NoTrash`. Tests: `crates/cleaner/src/pipeline.rs`, `crates/cleaner/tests/never.rs`,
  `app/tauri/src/compose.rs`.

## The user's own folders are hard-blocked

- **Rule:** the home folder and its standard folders — Desktop, Documents, Downloads, Movies, Music, Pictures, Public,
  Library — can never be removed as a whole, whatever a rule or a selection says. Items inside them can be, one by one.
- **Why:** a broken rule or a wrong selection must not be able to wipe a person's files; the guard sits below every
  rule.
- **Enforced by:** the protection list's `whole` folders (`crates/macos/src/protection.rs`: the home, its standard
  folders and the Library's, every mounted volume), refused by check 3 (`WholeFolder`, `crates/cleaner/src/paths.rs`);
  the context's protection and the system's both apply, whatever the engine passes. A sandbox marker never counts in
  the home itself. Tests: `the_home_and_its_standard_folders_are_whole` (`protection.rs`), `a_whole_folder_is_refused`
  (`paths.rs`), `tests/never.rs` (a hard-blocked folder and its ancestors are never removed).

## Items in use are skipped, never forced

- **Rule:** a file open by a running process, or a folder owned by a running app or service (a database, a container
  runtime), is skipped and reported with the reason; the cleaner never forces a removal and never quits an app on its
  own.
- **Why:** removing files under a running program corrupts its data or crashes it.
- **Enforced by:** the cleaner's check 10 (`crates/cleaner/src/guard/in_use.rs`) over the `OpenFiles` snapshot
  (`fresher-macos`'s `MacOpenFiles`: the descriptors, working folders and mapped files of the user's own processes),
  taken at the start and again every two seconds; a snapshot that cannot be taken skips every item from then on. The
  item is skipped `InUse` with the holder's name. The cleaner never forces, never retries with more rights and never
  quits an app; a tool removal never forces. Processes of other users are not seen
  ([`../known-issues/cleaner-other-users-processes.md`](../known-issues/cleaner-other-users-processes.md)). Tests:
  `guard/in_use.rs`, `guard/mod.rs`, `pipeline.rs` (`a_failed_snapshot_acts_on_nothing`, the refresh after two
  seconds).

## Unknown is not junk

- **Rule:** a rule claims a path only when it can tell what it is. Unrecognized space is shown, never offered as junk.
  **Two exceptions:**
  - (plan [017](../plans/017-temp-files/readme.md), decision 6) a top-level item of the user's own temporary folder
    (`$TMPDIR`) or the user's own item in `/private/tmp` is claimed by the place's contract whatever its name — macOS
    itself removes anything there after three days — and offered only after every row of the `temp` module's
    decision: not restored, no lock name, no socket or pipe, the user's and unflagged, not locked, the open apps
    listed and none named by it, read in full, and unchanged for more than the setting's days
    ([`../categories/temp-old-files.md`](../categories/temp-old-files.md));
  - (plan [033](../plans/033-logs/readme.md), agent's call 1) a regular file under the user's `~/Library/Logs`, up to
    six names deep, is claimed by the place's contract whatever its name — it is the place macOS gives apps for their
    logs — and offered, to the Trash, only after every row of the `logs` module's decision: not another module's
    place (`JetBrains/`, a bundle-id-shaped top folder), not a state or lock name, not restored, no socket, pipe or
    device, the user's and unflagged, not locked, and unchanged for more than the setting's days
    ([`../categories/logs-app-logs.md`](../categories/logs-app-logs.md),
    [`../categories/logs-crash-reports.md`](../categories/logs-crash-reports.md)). No folder is claimed.

  Nowhere else does a name the app cannot read count.
- **Why:** rule 1 of [`../rules.md`](../rules.md): a false positive is worse than a missed gigabyte. The exceptions
  hold because the place's contract, not the name, says what the item is: working files that the system clears, and
  the logs apps write to the place made for them.
- **Enforced by:** every rule's negative tests; for the first exception, the `temp` module's decision tests
  (`modules/temp/src/decide.rs`), its fixtures and its sandbox case file (`fixtures/sandboxes/temp.toml`), and the
  protection list above (nothing outside the two places); for the second, the `logs` module's decision tests
  (`modules/logs/src/decide.rs`, `names.rs`), its fixtures and its case file (`fixtures/sandboxes/logs.toml`).

## Settings never weaken deletion safety

- **Rule:** no setting turns off the confirmation, lowers a danger level or chooses how items are removed; a setting may
  only hide or filter. What the user ignores is never removed: an ignored category or path is left out of every answer
  and every plan, and the cleaner refuses an ignored path, what is inside it and what holds it. When the settings file
  cannot be read, nothing is cleaned (the ignore list cannot be known); scanning goes on with the defaults.
- **Why:** rule 2 of [`../rules.md`](../rules.md): the confirmation and Trash first are not preferences; and an
  "Ignore" that a cleanup could still reach would be a broken promise.
- **Enforced by:** the core's settings catalogue and its test that no core key names the confirmation, a danger level
  or the removal (`crates/core/src/settings/core_schema.rs`); a module's settings are always in the module's own group
  (`SettingDescriptor::new`). The engine leaves ignored findings out of pages, totals and plans
  (`crates/engine/src/results.rs`, `cleanup/pending.rs`) and gives the cleaner `Protection::with_ignored`
  (`cleanup/context.rs`; a list it refuses blocks the cleanup). Tests: `crates/engine/src/ignore.rs`,
  `crates/e2e/tests/ignore.rs`; cleaning blocked on an unloaded file: `crates/engine/src/cleanup/blocked.rs`.

## No administrator rights

- **Rule:** the app never elevates: no privileged helper, no launch daemon, no password prompt, no `sudo`. An item the
  app may not remove (owned by another user or the system, in a folder it cannot write to, locked, or protected by the
  system) is shown with its size and why, is never selected, and is left out of every plan; the user removes it by
  hand, in Finder.
- **Why:** rule 1 of [`../rules.md`](../rules.md): a deleter with administrator rights is the largest risk the product
  could carry, and a copied privileged command with a wrong path the worst outcome it could cause.
- **Enforced by:** `Removability`, worked out by the core from the walk's entry facts and never set by a module
  (`crates/core/src/removability.rs`); the plan builder's `SkipReason::NotRemovable`
  (`crates/engine/src/cleanup/plan.rs`); the `Info.plist` test, which allows no privileged-helper key
  (`app/tauri/tests/info_plist.rs`); review.
