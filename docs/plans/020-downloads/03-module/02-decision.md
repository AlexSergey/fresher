# 020/03/02 — The decision

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules

What the module does with the first-level entries of Downloads: gather them for the scan, then decide each in
`finish` by the rules of the design's section 3.4 — offered, left out with its reason, or not claimed. Pure decisions
on plain values, so every row is a unit test; the module only gathers and emits. The design:
[`../00-design.md`](../00-design.md), section 3.4 (agent's calls 1 to 4, 10 and 11).

## Files

- `modules/downloads/src/gathered.rs` (new) — the matches of one scan keyed by its cancel token, as
  `modules/app-leftovers/src/gathered.rs`: interest 0's files and interest 1's folders, each with its name, kind,
  sizes, `modified`, `changed`, `inside` and `holds`; only first-level entries of the place (a match whose parent is
  not the Downloads folder is dropped, defensively).
- `modules/downloads/src/name.rs` (new) — `pub(crate) fn names_app(stem: &str, apps: &[InstalledApp], words: &Words)
  -> Named` with `enum Named { None, One { app: usize, version: Option<Version> }, Several }`, by the name rule of the
  design (forms of each non-Apple app, the boundary, the rest's tokens); `pub(crate) struct Version(Vec<u32>)` with
  `Version::parse(token)` (digits and dots with at least one dot, an optional leading `v`) and `Version::of_app(text)`
  (an installed app's `version`; `None` when not only digits and dots); comparison by parts, a missing part as 0.
- `modules/downloads/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) struct Scan { now: Timestamp, days: u32, restored: bool, place_complete: bool }`;
  - `pub(crate) fn unfinished(file, row, running: Result<&[RunningApp], ()>, scan) -> Verdict`;
  - `pub(crate) fn placeholder(part_offered: bool, sibling: Option<&Entry>, scan) -> bool`;
  - `pub(crate) fn resumable(bundle, row, running, scan) -> Verdict`;
  - `pub(crate) fn macos_installer(bundle, version: SystemVersion, running_os: Result<SystemVersion, ()>, scan) ->
    Verdict`;
  - `pub(crate) fn installer(file, proof: Proof, apps: AppsState, scan) -> Verdict`, with `enum Proof { Receipt
    { installed: Timestamp }, Name(Named) }` (a receipt proof found first) and `enum AppsState { Unlisted, Partial,
    Complete(&[InstalledApp]) }`;
  - `pub(crate) fn large(file, scan) -> Verdict`;
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered { reason: OfferReason } }` with `Reason` one
    variant per left-out key and `OfferReason` the data each reason text needs (`days`, `app` name and version,
    `date`, `running` version, `size`).
  "Newest change" is the newest of `modified` and `changed`; unknown or after `now` counts as within the days.
- `modules/downloads/src/lib.rs` — `on_match` gathers; `finish` takes the gathered entries for the scan, sorts them by
  name and decides each: an unfinished file by its row; then a bundle of interest 1 (resumable, macOS installer, or
  not claimed); then `.dmg` / `.pkg` (receipt proof against `cx.system().receipts()` — an error proves nothing and is
  logged —, then the name proof against `installed_apps()` / `installed_apps_complete()`); then a large file. Asks the
  running apps only when an unfinished file or a bundle is gathered, the apps only when an installer is, the receipts
  only when a `.pkg` is, the version only when a macOS installer is. Emits each offered item as a `NewFinding` (the
  design's finding table; `for_app(app.bundle_id)` for an installer proven by name) and each kept one as a
  `LeftOutItem` in its category with its sizes; checks `cx.cancel()` between entries and returns
  `ModuleError::Cancelled`.
- `modules/downloads/src/tests.rs` (new) — the module run on matches built by hand (as `modules/app-leftovers/src/
  tests.rs`), with `FakeInstalledApps`, `FakeRunningApps`, `FakeInstallRecords`.
- `modules/downloads/Cargo.toml` — `proptest` as a dev-dependency.

## Produces

The module's answers, for 03/03 (fixtures and golden) and 04.

## Checks

- `name.rs`, negatives first:
  - `mod negative`: `a_name_with_another_word_names_no_app` (`notes-backup` with Notes not Apple's in the test),
    `apples_own_apps_are_never_named` (`Safari 18.0` with `com.apple.Safari`),
    `a_form_shorter_than_three_letters_is_dropped` (app `Go`, `go-1.2`), `a_name_running_on_into_letters_is_not_named`
    (`Firefoxsync.dmg`), `a_copy_mark_names_no_app` (`Firefox (1)`), `two_apps_named_is_several` (an app named `Docker`
    and another whose bundle id ends `.docker`, stem `docker`), `a_version_with_letters_is_not_comparable` (`1.0b2`);
  - `mod positive`: `the_designs_examples_are_named` (`Firefox 133.0`, `docker`, `GoogleChrome`, `Example-2.1-arm64`,
    `firefox-133.0-universal`), `the_last_part_of_the_bundle_id_names_the_app` (`org.mozilla.firefox` → `firefox`),
    `versions_compare_by_parts` (`133.0` ≤ `133.0.1`, `2.10` > `2.9`, `1` = `1.0`).
- `decide.rs`, negatives first, one test per row of each table of the design's section 3.4, keep before offer, each
  named after its row (`an_unfinished_file_of_an_open_browser_is_kept`,
  `an_unfinished_file_changed_within_the_days_is_kept`, `an_unfinished_file_changed_in_the_future_is_kept`,
  `a_placeholder_that_is_not_empty_is_never_claimed`, `no_placeholder_when_the_place_was_not_seen_in_full`,
  `a_resumable_download_without_its_info_plist_is_not_claimed`, `a_bundle_read_in_part_is_kept_to_scan_again`,
  `a_macos_installer_for_a_newer_macos_is_kept`, `an_installer_whose_app_changed_before_it_is_kept`,
  `an_installer_newer_than_its_app_is_kept`, `an_installer_when_the_app_list_is_partial_is_kept`,
  `an_installer_naming_no_app_is_not_claimed`, `a_restored_scan_keeps_everything_it_would_offer`,
  `a_dmg_never_falls_to_large_untouched`, `a_big_file_changed_within_the_days_is_kept`,
  `a_file_under_one_gigabyte_is_not_claimed`); then the offers
  (`an_old_unfinished_download_of_a_closed_browser_is_offered`, `its_empty_placeholder_is_offered_with_it`,
  `a_proven_installer_is_offered_for_its_app`, `a_package_with_its_receipt_is_offered_without_an_app`,
  `an_old_macos_installer_is_offered`, `a_big_untouched_file_is_offered_at_high`); the precedence
  (`restored_comes_before_everything`, `an_open_browser_comes_before_the_age`, `a_receipt_proof_needs_no_app_list`).
  Properties (proptest): `adding_a_running_browser_never_turns_a_keep_into_an_offer`,
  `a_later_change_never_turns_a_keep_into_an_offer`,
  `a_later_app_change_never_turns_an_offer_into_a_left_out_of_another_reason`.
- `tests.rs`: `the_matches_in_two_orders_give_the_same_answers`, `nothing_is_asked_of_the_system_without_a_candidate`
  (the fakes' `calls()` are 0 for a Downloads of documents only), `a_cancelled_scan_stops_between_entries`.
- `cargo nextest run -p fresher-module-downloads`; clippy; fmt; `npm run check:machete`.

## Conclusions

- Built as written, with three small additions: `src/report.rs` builds the findings and left-out lines (the text keys
  made once with the module), `categories.rs` names the categories as an enum (`Category`) the decision returns, and
  `decide.rs` gains the pure helpers `package_of`, `receipt_proof` and `placeholder_name`.
- The pre-flight rulings are applied: `tracing` logs unreadable receipts once per scan, by their reason only; a
  restored scan follows the 3.4 tables (installer rows 1 to 3 before `scan-again`); the third property is
  `a_later_app_change_never_turns_an_offer_into_a_keep`; `large()` itself refuses a `.dmg` or `.pkg`.
- An entry whose name is not text, a link, an `Other` entry, a folder of interest 0 and a match below the first level
  are never gathered. A `.part`'s placeholder is skipped in the walk by name and told right after its `.part`.
- A name's plain number is no version (the design's rule): `Firefox 140.dmg` beside Firefox 133 is held back only by
  the app's change time.
