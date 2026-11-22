# 036/04/01 — The sandbox's uninstall cases

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, testing, app

The sandbox learns to state what an uninstall needs (a bundle's last use and its package manager) and to expect what
an uninstall does (`[case.uninstall]`); the autotest checks it through the app's own sandbox composition; two case
files, negatives first, with their goldens; `EXPECTED.md` names each line for the walk by hand. The design:
[`../00-design.md`](../00-design.md), section 9.

## Before you start

Read `docs/testing.md` ("Sandboxes"), `crates/sandbox/src/case.rs` (`[[case.app]]`, `take_receipts` as the model of a
taken-out table, the checks of a case file), `world.rs` and `generate.rs` (how a case's apps and stated facts reach
`world.toml`), `expected.rs` (the checklist), `crates/testing/src/app_bundle.rs`, `crates/testing/src/facts_lens.rs`
(`Stated`, 01/03's new fields), `crates/sandbox/src/largest.rs` (plan 035's `[case.largest]`: a typed expectation
of `RawCase`, the model of `[case.uninstall]`), `app/tauri/src/sandbox.rs` (how `world.toml` becomes the stated installed apps),
`app/tauri/tests/sandboxes.rs` and `sandboxes/harness.rs`, `sandboxes/map.rs` and `sandboxes/largest.rs` (an extra check beside the findings'),
and `fixtures/sandboxes/storage-map.toml` (a case file for an engine feature on `module = "trial"`) and
`app-caches-app-open.toml` (a running app). Plans 031, 032 and 035 added `[[case.tool]]`, `[[case.insight]]`,
`[case.largest]` and `[[world.tool]]` to the same files: add after their lines.

## Files

- `crates/testing/src/app_bundle.rs` — `AppBundle` gains `last_used: Option<String>` (an age, `"240d"`) and
  `installed_by: Option<String>` (`"homebrew:<token>"`, checked: a known manager and a token `InstalledBy::new`
  accepts) and `owner: Option<String>` (`"system"` or another stated owner, as a `[[case.file]]`'s `owner`, stated for
  the bundle's own path); `check` refuses anything else (`FixtureError::AppLastUsed`, `FixtureError::AppInstalledBy`,
  the existing owner error).
- `crates/sandbox/src/case.rs` — `[[case.app]]` reads `last-used`, `installed-by` and `owner`; `[case.uninstall]` (a typed field of
  `RawCase`, as `largest` is, so it is never read as a file): `app` (the bundle's path, one of the case's `[[case.app]]`), then exactly one of
  `listed = false`; `offered = false` with `reason` (a `NotOffered` code); or `choice` (`keep-data` | `everything`) with
  `trashed` (paths that must be in the sandbox's Trash afterwards, the bundle first) and `kept` (paths that must be in
  place afterwards). Refused (one `CaseError` each, with its message): an `app` no `[[case.app]]` of the case declares,
  two forms at once, a `choice` without `trashed`, a path of `trashed` or `kept` the case does not write, a path in
  both. A positive case needs `[case.expect]` **or** `[case.uninstall]` with a `choice`; a negative case's
  `[case.uninstall]` is `listed = false` or `offered = false`. The bundle of a positive uninstall is its own item (no
  other case writes under it).
- `crates/sandbox/src/world.rs` / `generate.rs` — the stated `last_used` (the age turned into a moment from the
  generator's clock, as `changed` is), `installed_by` and `owner` of each app written into `world.toml` and read back into
  `StatedFacts` for the bundle's path.
- `crates/sandbox/src/expected.rs` — one checklist line per expectation: "Apps: `<name>` is not listed.", "Apps:
  `<name>` is shown and not offered: <the reason's sentence>.", "Apps: uninstall `<name>` with \"<the choice's label>\":
  these go to the Trash: …; these stay: …. Then Cleanups → put back: all of them are back."
- `app/tauri/tests/sandboxes/uninstall.rs` (new) — after the scan: `engine.apps(&AppsQuery::default())`; for each
  `[case.uninstall]`: `listed = false` → no summary has the bundle's name and path; `offered = false` → the summary's
  offer carries the reason; `choice` → `engine.prepare_uninstall(id, choice)`, the plan has one item with no left-out,
  `confirm_cleanup` with the plan's own summary, wait for the end, every `trashed` path is gone from its place and in
  `home/.Trash`, every `kept` path is in place, the app is no longer listed; then `put_back` of the cleanup: every
  `trashed` path is back, byte for byte. A failure names the case. The report (golden) gains `apps` (each summary's
  name, logical and allocated size, parts by kind, offer code) and `uninstalls` (each choice's plan: category, danger,
  item count, paths).
- `app/tauri/tests/sandboxes/harness.rs` — calls it for a case file with an uninstall expectation; negatives' files
  stay untouched as today.
- `app/tauri/tests/sandboxes.rs` — `uninstall => "uninstall"`, `uninstall_open => "uninstall-open"` in
  `case_files!`, sorted; a negative test `a_wrong_uninstall_expectation_fails_naming_its_case` (a `trashed` path that
  stays).
- `fixtures/sandboxes/uninstall.toml` (new), `module = "trial"`, names apart from every other case file (`Chess`,
  `com.apple.Notes`, `Brewed`, `Rooted`, `Keeper`, `Purger`, `Twin`, `com.example.Keeperx`):
  - negative "an app of the system is not listed": `System/Applications/Chess.app` (`com.apple.Chess`);
  - negative "an app with Apple's id is not listed": `Applications/Notes.app` (`com.apple.Notes`);
  - negative "an app Homebrew installed is not offered": `Applications/Brewed.app`, `installed-by =
    "homebrew:brewed"`, with a cache;
  - negative "an app owned by the system is not offered": `Applications/Rooted.app` with `owner = "system"`
    (`reason = "needs-administrator"`);
  - negative "a look-alike id's files stay": `home/Library/Caches/com.example.Keeperx/blob.bin`;
  - positive "keep-data moves the app, its caches, logs and saved windows; settings and files stay":
    `Applications/Keeper.app` (`com.example.Keeper`, `last-used = "240d"`), `Caches/com.example.Keeper`,
    `Logs/com.example.Keeper`, `Saved Application State/com.example.Keeper.savedState`, `Preferences/com.example.Keeper.plist`,
    `Application Support/com.example.Keeper`; `trashed` the first four, `kept` the last two;
  - positive "everything moves every file Fresher found": `Applications/Purger.app` (`com.example.Purger`) with the same
    kinds and `LaunchAgents/com.example.Purger.helper.plist`; all `trashed`;
  - positive "the user's copy of a twin goes alone": `Applications/Twin.app` and `home/Applications/Twin.app` (same id
    `com.example.Twin`), `Application Support/com.example.Twin`; uninstall `home/Applications/Twin.app` with
    `everything`: `trashed` that bundle only, `kept` the other bundle and the support folder.
- `fixtures/sandboxes/uninstall-open.toml` (new): negative "an open app is not offered" (`Applications/Mailbox.app`,
  `running = true`, `offered = false`, `reason = "open"`); positive "a closed app goes while another is open"
  (`Applications/Sketch.app` with a cache, `keep-data`).
- `fixtures/golden/sandbox-uninstall.json`, `sandbox-uninstall-open.json` — written by `UPDATE_GOLDEN=1 npm run
  sandbox:test`, the diff read.

## Checks

- `crates/sandbox` unit tests, negatives first: each refusal of `[case.uninstall]` and of the three `[[case.app]]` keys;
  then a case file with each form parses and its checklist lines read as above.
- `npm run sandbox:test` passes for every case file (`cargo nextest run -p fresher-app --test sandboxes --locked`);
  `npm run sandbox -- all` leaves out no file it did not leave out before.
- **By hand, once, each reverted** (written in this step's conclusions): a `kept` path moved to `trashed` fails the
  autotest naming the case; an `offered = false` reason changed fails; a `listed = false` on `Keeper` fails.
- `npm run dev:sandbox -- uninstall` starts (the walk itself is the user's, 05/01).

## Conclusions

- Done as written. `[[case.app]]` states `last-used`, `installed-by` and `owner` (typed as a `[[case.file]]`'s owner);
  they reach `world.toml` as `[[entry]]` facts of the bundle and `StatedInstalledApps` lays them over the measured
  bundle. `[case.uninstall]` is a typed field of `RawCase` with one refusal per broken rule, and a file-level check that
  no other case writes into a bundle a positive uninstalls.
- The autotest (`sandboxes/uninstall.rs`) runs after the module's cleanup and the Trash view: it lists the apps,
  checks `listed = false` (no summary at the bundle's path) and `offered = false` (the reason), then for each choice
  prepares the plan (one item, nothing left out, its paths exactly `trashed`), confirms it, checks that exactly one new
  Trash entry per `trashed` path arrived holding the same bytes, that every `kept` path and every other path of the
  file is in place and that the app is no longer listed, then puts the cleanup back and compares every path byte for
  byte. The golden gains `apps` and `uninstalls`; no other golden changed, and `npm run sandbox -- all` leaves out the
  same seven files as before.
- The Notes case is at `Applications/Utilities/Notes.app` (one folder below, still a listed place):
  `Applications/Notes.app` is `app-caches-app-open.toml`'s app, and `all` would have left this file out.
- The scan folds a file under 1 MiB that no registered module has an interest in, and the trial module has none, so a
  small settings plist or launch agent is never a footprint part. The case files make those parts 1 MiB; the gap
  itself is accepted for this plan (it only takes less) and goes to `docs/deferred/` and the user's review in 05/01.
- By hand, once, each reverted: moving Keeper's `Application Support` from `kept` to `trashed` failed naming the case
  ("the plan moves …, the case expects …"); changing Rooted's reason to `locked` failed naming it ("not offered for
  `needs-administrator`, not for `locked`"); `listed = false` on Keeper (a positive) refused the case file naming the
  case, and `listed = false` on Brewed failed naming it ("listed: Brewed").
- `npm run dev:sandbox -- uninstall` generated the sandbox and started the app on it.
