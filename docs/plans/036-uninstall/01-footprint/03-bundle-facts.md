# 036/01/03 — The bundle's facts

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** system, testing, app

`InstalledApps::bundle` answered for real (`MacInstalledApps`), for tests (`FakeInstalledApps`) and for the sandbox
(`StatedInstalledApps`): the bundle's sizes, its entry facts, Spotlight's last-used date and whether Homebrew installed
it. Metadata only: no file's contents are read, nothing is written, nothing is followed. The design:
[`../00-design.md`](../00-design.md), sections 3 and 4, and the facts table.

## Before you start

Read `crates/system/src/apps.rs` (`MacInstalledApps`, `AppSearch`), `crates/system/src/bundle.rs` (the existing
`Info.plist` readers; the new code goes in a **new** module `bundle_facts/`, not there), `crates/system/src/spotlight.rs`
(how a command runs through `CommandRunner`), `crates/system/src/process.rs`, `crates/core/src/removability.rs`
(`EntryFacts`), `crates/testing/src/system_fakes.rs` (`FakeInstalledApps`), `crates/testing/src/stated_apps.rs` and
`crates/testing/src/facts_lens.rs` (`Stated`), and `app/tauri/src/compose.rs` and `app/tauri/src/sandbox.rs` where the
installed apps are composed. Never run anything on the real `/Applications` from a test: every tree is a temporary
fixture, every command a `FakeCommandRunner`.

## Files

- `crates/system/src/bundle_facts/mod.rs` (new) — `pub struct BundleReader { runner: Option<Arc<dyn CommandRunner>>,
  casks: Vec<PathBuf> }` with `BundleReader::mac(runner)` (Spotlight on, the casks of `managers.rs`) and
  `BundleReader::stated()` (no Spotlight, no cask: the sandbox and tests), and `fn read(&self, app: &InstalledApp,
  user: Uid, cancel) -> Result<BundleFacts, SystemError>`: `measure`, `entry_facts`, `last_used`, `installed_by`.
  The bundle that is a link, or that `lstat` cannot read, is an error ("the bundle cannot be read"), never a path in
  the text.
- `crates/system/src/bundle_facts/measure.rs` (new) — `fn measure(bundle: &Path, cancel: &CancelToken) -> io::Result<Sizes>`:
  depth first, `symlink_metadata` only (a link counts as itself, never followed), an entry on another device than the
  bundle not entered and not counted, a hard-linked file counted once by `(dev, ino)` (its `reclaimable` share only when
  `nlink == 1`), `logical` the files' `len()`, `allocated` `blocks() * 512`; a folder that cannot be listed or an entry
  that cannot be read makes the result `Partial` and is skipped; `MAX_BUNDLE_ENTRIES = 2_000_000`, then `Partial`;
  `cancel` read between folders (`Err(Interrupted)` once set).
- `crates/system/src/bundle_facts/entry.rs` (new) — `fn entry_facts(meta: &Metadata, parent: &Metadata, user: Uid,
  groups: &[Gid]) -> EntryFacts`: pure on plain values; owned when `uid == user`; `parent_writable` and
  `folder_writable` from the mode bits for the user (owner bits when the user owns it, group bits when the user is in
  its group, else others'); `user_immutable` (`UF_IMMUTABLE`), `system_immutable` (`SF_IMMUTABLE`), `restricted`
  (`SF_RESTRICTED`) from `st_flags` (`MetadataExt::st_flags`, no unsafe). The user's groups come from the scanner's
  own source if one exists (look in `crates/scanner` and `crates/macos` for how the walk reads the user's access byte
  and reuse it); otherwise `fresher-macos`'s existing safe function. No `unsafe` in `fresher-system`.
- `crates/system/src/bundle_facts/last_used.rs` (new) — `fn last_used(runner, bundle, cancel) -> Option<Timestamp>`:
  runs `mdls` with the fixed arguments `["-raw", "-name", "kMDItemLastUsedDate", <bundle>]` (the path as one `OsStr`
  argument, no shell, the runner's 60 s limit); parses `YYYY-MM-DD HH:MM:SS +0000` into a `Timestamp`; `(null)`, an
  empty answer, a non-zero exit, a timeout or any other text is `None` (logged at debug, never the path).
- `crates/system/src/bundle_facts/managers.rs` (new) — `const CASKROOMS: [&str; 2] = ["/opt/homebrew/Caskroom",
  "/usr/local/Caskroom"]` ("Homebrew's records of the apps it installed: `<token>/<version>/<name>.app` is a link to
  the installed bundle"); `fn installed_by(casks: &[PathBuf], bundle: &Path) -> Option<InstalledBy>`: two levels of
  `read_dir` (never following a link), then `read_link` on each `*.app` entry; a target equal to the bundle's path (both
  made absolute; a relative target is joined to its folder, `..` resolved lexically, never through the disk) gives
  `InstalledBy { manager: Homebrew, token }` with the first level's name as the token (`InstalledBy::new` refuses a
  token off its shape: then `None`). A missing or unreadable Caskroom is `None`. The caskroom's index (target →
  token) is read once per `BundleReader` for every bundle of a listing (`OnceLock`), not once per bundle.
- `crates/system/src/apps.rs` — `MacInstalledApps` gains `with_bundle_reader(self, reader: BundleReader, user: Uid)`;
  `bundle()` uses it, else the trait's default error. `crates/system/src/lib.rs` exports `BundleReader`.
- `crates/testing/src/system_fakes.rs` — `FakeInstalledApps::with_bundle(self, path, Result<BundleFacts,
  SystemError>)`; `bundle()` answers the stated facts for that path, else the default error. A helper
  `bundle_facts(logical: u64) -> BundleFacts` (complete sizes, `EntryFacts::REMOVABLE`, no date, no manager).
- `crates/testing/src/stated_apps.rs` — `StatedInstalledApps::bundle`: the inner port's facts with the stated
  `owner` of the bundle's path applied to `entry.owned_by_user`, and the stated `last_used` and `installed_by` of the
  app (new optional fields on `Stated`: `last_used: Option<Timestamp>`, `installed_by: Option<InstalledBy>`; the
  sandbox writes them in 04/01). With nothing stated, the inner answer as it is.
- `app/tauri/src/compose.rs` — the real composition: `MacInstalledApps::new(…).with_bundle_reader(BundleReader::mac(
  runner), user)` with the existing `ProcessCommandRunner` and the user's uid already used for the cleaner.
  `app/tauri/src/sandbox.rs` — `BundleReader::stated()`: a sandbox never asks the Mac's Spotlight or Caskroom.

## Produces

Every installed app's `BundleFacts` for 01/04, in the app, the sandbox and the fakes.

## Checks

Fixture trees in temporary folders only; commands only through `FakeCommandRunner`.

- `measure.rs`, `mod negative`: `a_link_inside_is_counted_as_itself_not_followed` (a link to a large file outside),
  `a_hard_link_is_counted_once`, `an_unreadable_folder_makes_the_sizes_partial` (mode `000`, skipped when run as root),
  `the_entry_cap_makes_the_sizes_partial` (a cap lowered through a test-only constructor), `a_cancelled_measure_stops`;
  `mod positive`: `a_bundle_s_files_add_up` (logical and allocated), `an_empty_bundle_is_zero_and_complete`.
- `entry.rs`, `mod negative`: `a_bundle_of_another_owner_is_not_owned`, `a_folder_the_user_cannot_write_to_is_not_
  writable` (owner, group, others), `each_flag_is_read`; `mod positive`: `the_user_s_bundle_in_a_writable_folder_is_
  removable`.
- `last_used.rs`, `mod negative`: `null_is_no_date`, `garbage_is_no_date`, `a_failed_run_is_no_date`; `mod positive`:
  `a_spotlight_date_is_read`, `the_arguments_are_fixed` (the fake runner records `mdls -raw -name kMDItemLastUsedDate
  <path>`).
- `managers.rs`, `mod negative`: `no_caskroom_is_no_manager`, `a_link_to_another_bundle_is_no_manager`,
  `a_broken_link_is_no_manager`, `a_token_off_its_shape_is_no_manager`, `a_caskroom_link_is_never_followed` (the
  target is a folder of the fixture; nothing inside it is read); `mod positive`: `a_cask_link_to_the_bundle_names_
  homebrew_and_its_token`, `a_relative_cask_link_is_resolved_lexically`.
- `apps.rs`: `without_a_reader_the_bundle_is_unknown`; `stated_apps.rs`: `a_stated_owner_date_and_manager_overlay_
  the_measure`; `system_fakes.rs`: `a_fake_answers_the_bundle_it_was_given`.
- `cargo nextest run -p fresher-system -p fresher-testing -p fresher-app`; clippy; fmt; `npm run check:crate-deps`
  (no new edge: `fresher-system` already depends on core and macos).

## Conclusions

- Built in `crates/system/src/bundle_facts/` (`mod.rs`, `measure.rs`, `entry.rs`, `groups.rs`, `last_used.rs`,
  `managers.rs`): the measure by `lstat` (no link followed, no other device, hard links once, 2,000,000 entries at most,
  cancelled between folders), the entry's owner and flags, Spotlight's last-used date (`/usr/bin/mdls`), Homebrew's
  cask links (read once per listing), and `BundleReader::stated()` for the sandbox. Only a real folder with a real
  `Contents/Info.plist` is read. 1657 tests across the touched crates; line coverage 94 to 100 % per file.
- The user's groups come from `/usr/bin/id -G` through the command runner (read-only, fails closed, kept only on
  success): neither the scanner nor `fresher-macos` has a groups source, and without the group bit every app in
  `/Applications` (root and admin, 775) would need an administrator.
- A parent folder with a lock flag makes the bundle not removable; a cancelled measure is never kept.
