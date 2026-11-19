# 021/02/03 — Fixtures, golden and registration

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules, testing, app

The module run on real fixture trees with `run_module` (the real walk of the tree, the projects found as the engine
finds them since 01/03), its golden report, and its registration in every build after `downloads`. The design:
[`../00-design.md`](../00-design.md), section 4, "Fixture".

## Files

- `fixtures/homes/stale-projects-not-claimed.toml`, `stale-projects-kept.toml`, `stale-projects-offered.toml` (new) —
  the homes of the design's fixture table. **Every folder of a project that must look stale or active is a `[[dir]]`
  with an `age`**, the parents of its files and `.git/refs/heads` included: a folder the tree makes at the test's time
  is "changed now", and a stale case that forgets one folder turns active (the test then fails on its missing offer,
  never on a false one). A head comment in each home says so. Positives' files `size` small with `fill = "data"`; no
  path written twice in one home.
  - not claimed: `Code/site/` (`package.json`, `src/index.ts`, `dist/app.js`, `.next/cache/x`, `.turbo/x`,
    `.git/refs/heads/main`, every folder 400 days) without `node_modules`; `Code/notes/node_modules/x/index.js` with no
    marker; `Code/linked/package.json` (400 days) and a link `Code/linked/node_modules` → `elsewhere/modules`;
    `package.json` and `node_modules/left-pad/index.js` at the home itself (400 days); `.config/tool/package.json`
    with `node_modules/`; `Library/Application Support/Example/package.json` with `node_modules/`;
    `Applications/Tool.app/Contents/Resources/app/package.json` with `node_modules/`; `Code/svc/go.mod` with
    `target/x`; `Code/web/package.json` with `build/x` (Node lists no `build`) — all 400 days.
  - kept: `Code/active/` (`package.json`, `node_modules/`, folders 3 days) → `recent`; `Code/committed/` (folders 400
    days, `.git/refs/heads` 3 days) with `node_modules/` → `recent`; `Code/mono/` (`package.json`, `packages/web/` with
    its own `package.json` and `node_modules/`, `packages/web/src` 3 days, the rest 400 days) with `node_modules/` →
    `recent` twice; `Code/engine/` (`Cargo.toml`, `src` 3 days, `target/debug/incremental/x-1/s-1/` 3 days) →
    `target` `recent` and the cache `cache-recent`; `Code/hidden-part/` (400 days, `node_modules/`, `private/x`
    avoided through `Ports.avoid`) → `unread`.
  - offered: stale (400 days) `Code/web-old/` Node `node_modules/`; `Code/tool-old/` Rust `target/` holding
    `debug/incremental/x-1/s-1/` (one finding: `target`); `Code/java-old/` Maven `pom.xml` and `target/`;
    `Code/android-old/` Gradle `build.gradle.kts`, `build/` and `.gradle/`; `Code/swift-old/` `Package.swift` and
    `.build/`; `Code/dart-old/` `pubspec.yaml`, `.dart_tool/` and `build/`; `Code/py-old/` `pyproject.toml` and
    `.venv/`, `Code/py-venv/` `setup.py` and `venv/`; `Code/shop/` with `node_modules/` and `packages/web/` (its own
    `package.json` and `node_modules/`), all 400 days; and the active (3 days) `Code/fresher/` Rust project whose
    `target/debug/incremental/`, `target/aarch64-apple-darwin/debug/incremental/` and `target/llvm-cov-target/` are 10
    days old, its `target/debug/deps/x` 3 days old.
- `modules/stale-projects/tests/support/mod.rs` (new) — the fixtures' shared setup, as
  `modules/downloads/tests/support/mod.rs`: the ports at a fixed `now`, the setting, restored, the order, the avoided
  folders.
- `modules/stale-projects/tests/fixtures.rs` (new) — `mod negative`:
  `nothing_in_the_not_claimed_home_is_claimed_or_left_out`, `each_kept_item_is_left_out_with_its_reason` (a count and
  the sizes per reason), `a_restored_run_keeps_everything_to_scan_again` (the offered home, `Ports.restored`);
  `mod positive`: `every_category_is_offered_at_its_danger` (each title, reason, size; `shop`'s two `node_modules`
  under `shop`; `fresher`'s `target` left out `recent` with its sizes less the three caches'), the golden
  `fixtures/golden/stale-projects.json` (`UPDATE_GOLDEN=1`, reviewed), `the_setting_moves_the_days`
  (`{"stale-projects.unused-for-days": 500}` keeps the 400-day projects `recent`).
- `modules/stale-projects/tests/contract.rs` — the contract over the fixture runs (every declared left-out reason and
  every text used by a run, every removal a matched path), as `downloads`' does.
- `app/tauri/Cargo.toml` — `fresher-module-stale-projects`; `app/tauri/src/compose.rs` — `modules()` registers
  `StaleProjectsModule::new()?` after `DownloadsModule`; `ComposeError::StaleProjects(#[from] …)` as the others; every
  module list of `compose.rs` and `app/tauri/src/commands.rs` that names the registered modules gains it.

## Produces

The registered module and its golden, for 03.

## Checks

- `cargo nextest run -p fresher-module-stale-projects -p fresher-app` (the compose tests list the registered
  modules); the golden reviewed by hand: no path outside `Code/` in an offer, no `src`, `.git`, `dist`, `.next`, no
  project folder, every category present.
- Registration makes `npm run sandbox:test` require `fixtures/sandboxes/stale-projects.toml` (rule 8): 03/01 follows
  this step before any push, and this step's conclusions say that the autotest is red until then.
- Coverage of `modules/stale-projects` at or above 90 % (`cargo llvm-cov nextest -p fresher-module-stale-projects`);
  clippy; fmt.

## Conclusions

The three homes, the module's fixture tests, the golden `fixtures/golden/stale-projects.json` and the registration
are in place. Every folder of every project is a dated `[[dir]]` (400 days, 3 for the active and recent ones, 10 for
`fresher`'s caches); the avoided `Code/hidden-part/private` is dated 400 days so the kept case reads `unread`, and
`web-old` has a 400-day `.git/refs/heads/main` so the kept `committed` and `mono` cases pass for their own signal only.
The offered home gives 15 real offers through the real walk (three of them `fresher`'s caches, `tool-old`'s
cache folded into its offered `target`); `fresher`'s `target` is left out `recent` with the 8192 bytes of its
`deps` (its three caches subtracted). The golden holds only paths under `Code/`, no `src`, `.git`, `dist` or project
folder. The restored run is tested on both the kept home (the design's) and the offered one, and both, with the
offered home reversed, are contract runs. Registration: `StaleProjectsModule` after `DownloadsModule` in
`compose.rs` (`ComposeError::StaleProjects`), in the module lists of `compose.rs` and `commands.rs`. At 500 days the
400-day projects are `recent` and only the caches (`fresher`'s three and the 400-day `tool-old` one, which the
module offers on its own age) are offered. Coverage of the crate is 97 %.

**`npm run sandbox:test` and `fresher-app`'s `every_registered_module_has_a_case_file` are red until 03/01 adds
`fixtures/sandboxes/stale-projects.toml`** (rule 8): 03/01 follows before any push.
