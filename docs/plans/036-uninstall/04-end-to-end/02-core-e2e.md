# 036/04/02 — The core's end-to-end flows

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e

The uninstall through the real scanner, the real settings, scan cache and journal, and the real `DiskCleaner` acting
only inside a fixture sandbox, with a fake Trash: both choices move exactly their paths, put back brings the whole app
back, and every way the disk can change between the plan and the act moves nothing. The apps folder is the fixture's
own `Applications`. The design: [`../00-design.md`](../00-design.md), sections 6, 7 and 13.

## Before you start

Read `crates/e2e/tests/app_caches.rs` and `app_caches_home/mod.rs` (a flow's setup: the fixture sandbox, the composed
engine, the fakes a flow sets, the golden report), `crates/e2e/tests/local_ai_models_ollama.rs` (an item of several
paths: `a_layer_replaced_after_the_plan_moves_nothing`, `a_model_whose_layer_is_mapped_moves_nothing`),
`crates/e2e/tests/support/`, `docs/architecture/removal.md` (Together, put back) and 02/01. Name the new file
`uninstall.rs`: `footprint.rs` already exists and is the app's own data folder's.

## Files

- `crates/e2e/tests/uninstall_home/mod.rs` (new) — the fixture: a sandbox with its marker, `Applications/Editor.app`
  (`com.example.Editor`, helper `com.example.Editor.Helper`, team `TEAM123456`), its data under `home/Library` (caches,
  the helper's caches, logs, saved state, a launch agent, settings, support, web data, a group container
  `TEAM123456.com.example.shared`), `Applications/Viewer.app` with no data, `com.example.Editorx`'s cache, the known
  folders located inside the sandbox (`Applications` at the sandbox's `Applications`), the engine composed with the real
  scanner and cleaner (`Reach::SandboxFolder`, the protection of the real Mac joined with the sandbox's, the bundle
  reader `BundleReader::stated()` through `StatedInstalledApps`), `FakeRunningApps`, `FakeOpenFiles`, `FakeClock`.
- `crates/e2e/tests/uninstall.rs` (new) — the flows, negatives first:
  - `mod negative`:
    - `an_ordinary_cleanup_never_moves_a_bundle`: the scan roots set to the home and the sandbox's `Applications`
      (allowed by the settings check), and a module defined in the test that claims every `*.app` folder there; its
      finding of `Applications/Viewer.app`, selected and confirmed, is skipped `Protected` by the engine's second wall
      and the bundle is in place;
    - `a_part_replaced_after_the_plan_moves_nothing`: the support folder swapped for another folder after
      `prepare_uninstall`; the cleanup skips the item (`Replaced`) and every path is in place;
    - `a_part_held_open_moves_nothing` (`FakeOpenFiles` names a file of the caches);
    - `a_part_grown_moves_nothing` (a large file added to the support folder past the margin);
    - `the_app_opened_after_the_plan_moves_nothing` (`FakeOpenFiles` maps the bundle's executable);
    - `the_app_open_at_prepare_is_refused` (`FakeRunningApps` runs the helper);
    - `a_second_copy_keeps_the_data` (`home/Applications/Editor.app` with the same id: the plan of `everything` holds
      the bundle alone);
    - `the_look_alike_s_cache_stays` after every flow;
    - `a_new_scan_kills_the_plan` (`confirm_cleanup` answers `PlanGone`, nothing moved).
  - `mod positive`:
    - `keep_data_moves_the_app_and_its_caches_logs_saved_state_and_agent`: exactly those paths in the fake Trash, the
      bundle first in the journal's item; settings, support, web data and the group container in place; the app gone
      from `apps()`; `app-caches`' finding of the cache gone from the results;
    - `everything_moves_every_part`: every path of the fixture's app in the Trash, the group container included (other
      apps' data granted in the fake access) and kept when it is not (a second run with it denied);
    - `put_back_brings_the_whole_app_back`: after `everything`, `put_back` of the cleanup: every part back byte for byte,
      the bundle last (the journal's order), beside a file written into `Caches` since;
    - `an_app_with_no_data_is_one_path` (`Viewer`);
    - the report of the `keep_data` flow is the golden `fixtures/golden/uninstall-flow.json` (`UPDATE_GOLDEN=1 cargo
      nextest run -p fresher-e2e --test uninstall`).
- `crates/e2e/Cargo.toml` — a dependency only if one is missing (none expected).

## Checks

- `cargo nextest run -p fresher-e2e --test uninstall`; the golden's diff read; clippy; fmt.
- `git diff -- crates/cleaner modules` is empty.

## Conclusions

- Built: `crates/e2e/tests/uninstall.rs` (17 flows, 13 negative then 4 positive; 17 of 17 four times in a row),
  `uninstall_home/`, `fixtures/homes/uninstall-flow.toml`, `fixtures/golden/uninstall-flow.json`; `Extras` gained
  optional `apps` and `running`; `fresher-system` is a dev-dependency. Crate total 213 tests.
- The mandatory flows hold: an ordinary cleanup never moves a bundle in either apps folder (mutation-checked: with the
  wall off the test fails); the app opened between the plan and the confirmation; a bundle swapped for a link or
  renamed to another case; a bundle moved and scanned again.
- The golden sorts the journal's paths, so the bundle-first order is asserted in the flows. The sandbox's own
  composition opens the sandbox's apps folder with no `with_app_folders`: there the engine's wall is the only one.
