# 030/01/03 — Fixtures, golden and the contract

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, fixtures

The module on fixture trees through `run_module` (the reference walk), its golden report, the contract test over the
fixture runs. The app's registration is made in 02/01, not here: the app's test that every registered module has a case
file would fail until then. The design: [`../00-design.md`](../00-design.md),
section 7.

## Files

- `fixtures/homes/app-caches-not-claimed.toml` (new) — under the fixture home's `Library/Caches/`: `SomeTool/x` (not
  an id), `com.apple.Safari/x` (Apple's), `io.github.alexsergey.fresher/x` and `io.github.alexsergey.fresher.dev/x`
  (Fresher's), `com.example.Gone/Cache.db` (no installed app: `app-leftovers`' to decide), `com.example/x` (the vendor
  prefix of an installed app), `com.example.Editorx/x` (a sibling id), a link `com.example.Editor.Linked` to a folder
  outside, a file `com.example.Editor.plist`, an empty folder `com.example.Editor.Empty`. Scanned with
  `com.example.Editor` installed (helper `com.example.Editor.Helper`). Nothing is claimed and nothing left out.
- `fixtures/homes/app-caches-kept.toml` (new) — `com.example.Busy/` with a socket inside (`special`),
  `com.example.Locked/` with a file `flags = ["user-lock"]` (`not-yours`), `com.example.Open/` run with its app among
  the running apps (`app-open`), `com.example.Agent/` run with only its helper `com.example.Agent.Menu` running
  (`app-open`; the installed Agent declares that helper), and once every folder with the running apps failing
  (`no-system` for the folders that reach row 9). Every app installed.
- `fixtures/homes/app-caches-offered.toml` (new) — `com.example.Editor/Cache.db` (own),
  `com.example.Editor.Helper/Cache.db` (a helper's), `com.example.Editor.ShipIt/update.zip` (an id under the app's),
  `com.example.Suite/a` and `com.example.Suite.Pro/b` with both `com.example.Suite` and `com.example.Suite.Pro`
  installed (each its own owner). Small files, real sizes.
- `fixtures/golden/app-caches.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (titles, reasons, sizes, apps,
  dangers, removals), the left-outs.
- `modules/app-caches/tests/fixtures.rs` (new) and `tests/support/mod.rs` — the three homes through `run_module`
  with the installed and running apps, as `modules/app-leftovers/tests/fixtures.rs` passes them; the golden comparison.
- `modules/app-caches/tests/contract.rs` — the fixture runs added (seven runs, among them the running and the installed
  apps failing), so every text and every declared left-out is used and every removal is an `EmptyFolder` of a matched
  path.
- `modules/app-caches/Cargo.toml` — `serde_json` as a dev-dependency (the golden); its row in `docs/dependencies.md`.

## Produces

The module's fixtures, golden and contract, for 02.

## Checks

- `tests/fixtures.rs`, negatives first: `mod negative` `nothing_in_the_not_claimed_home_is_offered_or_left_out`,
  `the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason`; `mod positive`
  `the_offered_home_offers_every_cache_in_its_app_s_name`, `nested_ids_go_to_the_most_specific_app`,
  `every_offer_empties_its_folder`, `the_golden_report_matches`.
- `tests/contract.rs` passes with the fixture runs.
- `cargo nextest run -p fresher-module-app-caches`; coverage of the crate at or above 90 %.

## Conclusions

Built with one change: the app registration moved to step 02/01 (preflight F1), so `fresher-app` stayed green between steps. Three fixture homes (not-claimed, kept, offered), the golden (5 offered, 4 kept, 0 not-claimed), and a contract test over seven fixture runs: every text is used, every declared left-out reason is given by a run, every removal names a matched path and is an `EmptyFolder`. The first round did not contain the contract test (the report said it did); the fix round added it. `serde_json` became a dev-dependency (the golden).
