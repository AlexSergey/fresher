# 006/03/03 — The adapters

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** system, app

The real `AccessProbe` and `SystemUi`. Plan 009's `fresher-system` exists, so the two adapters live there and the app
only composes them (design, agent's call 2). From here on the normal app's walk is kept out of guarded places by the
capability state, not by the standing avoid list.

Needs 03/01, 03/02 and **03/05** (the `container` row of `app-leftovers` not offered): this step is the one that lets
the walk enter `Library/Containers` when data from other apps or Full Disk Access is granted.

## What exists already

- `app/tauri/src/adapters.rs` — the stand-ins `UnknownAccess` (Full Disk Access `Unknown`, every other permission
  `NotAsked`, no guarded place, `try_place` → `Unknown`) and `NoSystemUi` (`Unsupported` to everything), with one test.
- `app/tauri/src/compose.rs` — `engine_with` gives `Shared { avoid: places_that_prompt(home), access:
  Arc::new(UnknownAccess), commands, … }`; `assemble` sets `ScannerOptions::standing_avoid` from `avoid` and
  `Ports::system_ui: Arc::new(NoSystemUi)`.
- `app/tauri/src/sandbox.rs` — `SandboxPorts::real` gives `access: Arc::new(UnknownAccess)`; `compose` sets the
  standing avoid list to `places_that_prompt` of both the sandbox's home and the real home.
- `app/tauri/src/testing.rs` — the app's test engine with `UnknownAccess` and `NoSystemUi`.
- `crates/engine/src/coverage.rs` — `before_walk` already decides every place `guarded_places(roots)` returns and
  feeds the request's `avoid`; a place whose capability is `AllFiles` is never tried, only skipped until granted.
- `crates/scanner/src/scanner.rs` — `ScannerOptions::standing_avoid`, documented as "the app's stand-in until plan
  006"; the walk skips an avoided root as well as an avoided folder.
- `crates/system/src/plist/` — the plist reader of `fresher-system` (XML and binary).
- `docs/hacks/README.md` — three entries: "Places that prompt are avoided until plan 006", "Access answers Unknown
  until plan 006/03", "No system interface until plan 006/03".

## Files

- `crates/system/src/macos_version.rs` (new) — `MacosVersion { major: u16, minor: u16 }` and
  `macos_version(root: &Path) -> Option<MacosVersion>`: `ProductVersion` of
  `<root>/System/Library/CoreServices/SystemVersion.plist` through the crate's plist reader; the app passes `/`,
  tests a temporary tree. No FFI (agent's call 29).
- `crates/system/src/access.rs` (new) — `MacAccess` implements `AccessProbe`, built with
  `MacAccess::new(home: PathBuf, version: Option<MacosVersion>)`:
  - `status()`: `AllFiles` from `full_disk_access(&fda_candidates(home))` (`Allowed` → `Granted`, `Refused` →
    `Denied`, otherwise `Unknown`). With `AllFiles` granted, the three folders and other apps' data are `Granted`
    without a touch. Otherwise each is the last `try_place` answer for its capability in this session (agent's call
    28: kept in a `Mutex`, no new open), or `NotAsked`. Other apps' data is `NotApplicable` when `version` says macOS
    13; an unknown version → applicable. Both volumes are `NotAsked` until tried, then their last answer.
  - `guarded_places(roots)`: `privacy::guarded_places(home)` mapped onto `GuardedPlace` (`Guard::FullDiskAccess` →
    `Capability::AllFiles`, `OtherAppsData` → `OtherAppsData`, the folders → `Folder(..)`), kept when the place lies
    inside a root; a root that lies inside a guarded place is reported itself with that place's capability (agent's
    call 27); and each root's `volume_guard` as `Volume(Removable | Network)` for the root. Roots and places compared
    as paths (`Path::starts_with`), never strings.
  - `try_place(place)`: `privacy::try_inside(&place.path)` for `OtherAppsData`, `privacy::try_place(&place.path)`
    for every other capability → `Allowed` → `Granted`, `Refused` → `Denied`, otherwise
    `Unknown`; the answer is remembered for its capability.
- `crates/system/src/system_ui.rs` (new) — `MacSystemUi { commands: Arc<dyn CommandRunner> }` implements
  `SystemUi`: `open_access_settings` maps the capability to its `Guard`, then `pane_of`, and runs
  `privacy::OPEN` with `open_args(pane)`; a `Tool` capability → `Unsupported`. `reveal` runs `OPEN` with
  `reveal_args(path)` (a path that is not UTF-8 → `Failed`, 03/02's interface note). A non-zero exit or a runner error
  → `Failed { reason }`. Runs with a fresh `CancelToken` (the call is the user's click; `/usr/bin/open` returns at
  once).
- `crates/system/src/lib.rs` — `mod access; mod macos_version; mod system_ui;` and `pub use` of `MacAccess`,
  `MacSystemUi`, `MacosVersion`, `macos_version`; the crate's doc line names them.
- `app/tauri/src/compose.rs` — the normal engine: `access: Arc::new(MacAccess::new(home.to_path_buf(),
  macos_version(Path::new("/"))))`, no standing avoid list (`avoid: Vec::new()`: the engine's request carries the
  guarded places); `assemble` sets `system_ui: Arc::new(MacSystemUi::new(Arc::clone(&commands)))` for both paths. The
  comment "Until plan 006 decides…" goes.
- `app/tauri/src/sandbox.rs` — unchanged in behaviour (agent's call 30): `SandboxPorts::real` keeps a stand-in that
  answers `Unknown`; the standing avoid list of both homes stays, now through `places_that_prompt` as 03/01 redefined
  it. Its comment names the reason (the sandbox's world is not the Mac's; a scan root outside the sandbox raises no
  prompt).
- `app/tauri/src/adapters.rs` — `NoSystemUi` removed; `UnknownAccess` stays for sandbox mode and the app's tests, its
  doc saying so; the test keeps only its `UnknownAccess` half.
- `app/tauri/src/testing.rs` — `system_ui: Arc::new(FakeSystemUi::default())` (`crates/testing/src/fake_access.rs`)
  instead of `NoSystemUi`.
- `crates/scanner/src/scanner.rs` — the doc of `standing_avoid`: "places no walk opens, added to every request's own
  `avoid`: sandbox mode's list (docs/hacks/)". No code change.
- `crates/core/locales/en.json` — `core.storage.not-scanned.avoided` becomes "Not opened: macOS guards it, and Fresher
  looks there only with your permission." (an `AllFiles` place refuses rather than asks; the old text promised a
  prompt).
- `docs/hacks/README.md` — "Access answers Unknown until plan 006/03" and "No system interface until plan 006/03"
  removed; "Places that prompt are avoided until plan 006" rewritten as "Sandbox mode avoids the places that prompt":
  what (the standing list of both homes, and `UnknownAccess`), where (`app/tauri/src/sandbox.rs`,
  `crates/macos/src/privacy/places.rs`), why (agent's call 30), if cleaned up (a sandbox scan of a root outside the
  sandbox raises real prompts). Its paragraph "What it costs `app-leftovers`" goes (03/05 handled it).
- `docs/architecture/README.md` — the `AccessProbe` and `SystemUi` rows of the ports table name `MacAccess` and
  `MacSystemUi` in `fresher-system` (05/03 writes the rest of the docs).

## Produces

The real access of the app: `MacAccess`, `MacSystemUi`, `MacosVersion`, `macos_version`. From here on the window shows
what macOS allows, and a scan enters a guarded place only when its capability allows it or the user opted in.

## Checks

On temporary trees (a fake home with the candidates and the places) and `FakeCommandRunner`; no test opens System
Settings, reads the real home folder or tries a real guarded place.

- Negative (first): `a_home_without_any_candidate_gives_unknown_full_disk_access`,
  `without_full_disk_access_the_folders_are_not_asked`, `a_guarded_place_outside_every_root_is_not_reported`,
  `other_apps_data_is_not_applicable_on_macos_13`, `an_unreadable_system_version_keeps_other_apps_data_guarded`,
  `a_tool_has_no_settings_pane`, `a_failed_open_is_reported_as_a_system_ui_error`,
  `a_path_that_is_not_utf8_is_not_revealed`, `the_normal_engine_has_no_standing_avoid_list` (compose),
  `a_sandbox_engine_still_avoids_the_places_of_both_homes` (sandbox tests).
- Positive: `a_readable_candidate_grants_all_files_and_every_folder`,
  `a_tried_folder_reports_its_answer_in_the_status`, `a_root_inside_a_guarded_place_is_reported_itself`,
  `guarded_places_of_the_home_root_are_reported_with_their_capability` (Pictures under `AllFiles`, `Library/Containers`
  under `OtherAppsData`), `open_access_settings_runs_open_with_the_pane_address` (the exact program and arguments
  recorded by the fake), `reveal_runs_open_with_dash_r_and_the_path`, `the_version_is_read_from_the_plist`.
- `cargo nextest run -p fresher-system -p fresher-app -p fresher-e2e`; the sandbox autotests (`npm run sandbox:test`)
  pass with their golden reports unchanged (sandbox mode's access is still the stand-in; its system interface is now
  `MacSystemUi` over the sandbox ports' `commands`, a fake in the autotests); Clippy clean.

## Conclusions

Built as the step says. `fresher-system` gains `MacAccess` (`access.rs`), `MacSystemUi` (`system_ui.rs`) and
`MacosVersion` / `macos_version` (`macos_version.rs`); the normal engine composes `MacAccess` on the user's home and
the version read from `/`, with no standing avoid list, and both engines get `MacSystemUi` over their `commands`.
Sandbox mode keeps `UnknownAccess` and the standing list of both homes. `NoSystemUi` is gone; the app's tests use
`FakeSystemUi`.

- Decision: `macos_version` reads the file through `bundle.rs`'s capped reader (`read_capped`, now `pub(crate)`: a
  regular file of at most 1 MiB, no link followed) and the crate's plist reader; `ProductVersion` is parsed as
  `<major>[.<minor>[.…]]`, a missing minor is 0, anything else is `None` (other apps' data then stays guarded).
- Decision: other apps' data is `NotApplicable` when the version's major is below 14 (the design's agent's call 29,
  "before macOS 14"); this wins over Full Disk Access, since the permission does not exist there. With Full Disk Access
  granted, the three folders and other apps' data are `Granted`; the two volumes keep their own state (`NotAsked`
  until tried), as the step lists only the folders and other apps' data — the engine skips every guarded place once
  every file is allowed anyway.
- Decision: a try is remembered per capability whatever it answered, `Unknown` included (a missing place or one the
  POSIX permissions refuse); the next try of the capability replaces it.
- Decision: `guarded_places` reports each place once (overlapping roots such as `~` and `~/Library` give 12, not 18
  places); a root equal to a guarded place is the place itself.
- Decision: `the_normal_engine_has_no_standing_avoid_list` needs to see the composed parts, so `engine_with` is split
  into `normal` (the `Shared` of the normal engine) and `assemble`; the test checks `normal(..).avoid` is empty.
  `a_sandbox_engine_still_avoids_the_places_of_both_homes` is a scan: a trial cache with its marker under the
  sandbox home's `Library/Containers` and the real home's `Desktop` (also a scan root) is never found, while the
  sandbox's own one is; with the standing list cleared it fails, checked by hand.
- Decision: the architecture's sandbox paragraph said access "stays real"; it is the stand-in `UnknownAccess`, now
  written so (the rest of the docs is 05/03's).
- Extra: an e2e check in `crates/e2e/tests/app_leftovers.rs` — an engine with `app-leftovers` only and a fake access
  answering as `UnknownAccess` (Full Disk Access `Unknown`, every other permission `NotAsked`) reports its three
  categories `Limited { missing: [OtherAppsData] }` (`without_data_from_other_apps_every_category_is_limited`), and
  `Ready` once other apps' data is granted (`with_data_from_other_apps_granted_every_category_is_ready`).
- The text `core.storage.not-scanned.avoided` changed in `crates/core/locales/en.json` and in the UI's copies of it
  (`app/ui/src/test/storage-map.ts`, `place-list.test.tsx`).
- Tests: `fresher-system` 172 passed (the step's fifteen named tests of `access`, `system_ui` and `macos_version`,
  plus `a_sibling_with_a_longer_name_is_not_inside_a_guarded_place`, `a_place_that_cannot_be_told_is_remembered_as_unknown`,
  `other_apps_data_is_tried_inside_its_folder`, `overlapping_roots_report_each_place_once`, and the version parser's
  edge cases); none opens System Settings, reads the real home folder or tries a real guarded place (every home is a
  temporary tree, every command a `FakeCommandRunner`).
- Checked: `cargo nextest run -p fresher-system -p fresher-app -p fresher-e2e -p fresher-core -p fresher-scanner`
  1119 passed; `npm run sandbox:test` 42 passed with the golden reports unchanged; the UI's tests of `src/product`,
  `src/state`, `src/screens/overview` and `src/ipc` 430 passed; `npm run lint:ts` clean; `cargo fmt` and
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Fix round 1 (review): the engine passes the scan roots as the settings write them (default `~`), so
  `MacAccess::guarded_places` compared the home's places with a literal `~`, found none, and the default scan would
  have walked into the guarded places. Fixed with one shared expansion: `fresher_core::expand_home(root, home)` (a
  leading `~` component is the home; `~user` and a `~` further in are not expanded) in `known_folder.rs`, used by the
  scanner's `expand_home` (now a call to it, behaviour unchanged) and by `MacAccess`, which expands every root before
  comparing and reports a root inside a guarded place as the expanded path the walk opens. The core is the one crate
  both reach without a new dependency. New tests: `only_a_leading_tilde_component_is_expanded`,
  `a_leading_tilde_is_the_home` (core); `the_home_written_as_a_tilde_reports_every_place_under_the_home`,
  `a_tilde_root_inside_a_guarded_place_is_reported_as_the_path_the_walk_opens` (system);
  `the_normal_access_keeps_the_default_root_out_of_every_guarded_place` (compose: the normal parts on a temporary home
  with no candidate name Desktop and `Library/Containers` under `~`, none allowed; nothing tried). Removing the
  expansion fails all three of the last. The test that takes a folder's permissions gives them back through a drop
  guard. Decision: the `#[cfg(test)]` on the e2e helper module stays: it is what lets Clippy's in-test allowances
  (`unwrap`) apply to an integration test's helpers, as `coverage.rs` notes; without it Clippy fails.
- Checked after the fix: `cargo nextest run -p fresher-system -p fresher-app -p fresher-e2e -p fresher-core
  -p fresher-scanner` 1124 passed; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Final review fix: a root written in another case (`~/desktop`) or reached through a link (`~/Link` → `~/Desktop`,
  or into `~/Documents/X`) met no guarded place, since the roots were compared with `starts_with` on the written form
  while the walk opens the canonical root; before this branch the standing list, canonicalized by the walk's
  boundary, caught it. `MacAccess::guarded_places` now compares names without regard to ASCII case
  (`guarded_root::is_within`, as the settings' folder rules do), and a root that meets no place as written is followed
  through its links (`guarded_root::resolve`: `lstat` and `readlink` only, never a name inside a guarded place, at most
  32 links) and compared with the places of the canonical home; a root reported is the resolved path. New tests:
  `a_root_written_in_another_case_is_inside_its_guarded_place`,
  `a_root_linked_onto_a_guarded_place_is_reported_as_the_place_it_opens`,
  `a_root_linked_into_a_guarded_place_is_reported_as_the_folder_it_opens`,
  `a_root_linked_outside_every_guarded_place_is_not_reported`, and the helper's own five; every home is a temporary
  tree and nothing is tried.
- Re-review residual: a root on a removable or network volume (`volume_guard` set) gets only its volume's guard and
  its links are not followed, and `resolve` takes a stop rule (a guarded place of the canonical home, or a path with a
  volume guard), sticky once met, so a link that leads onto a volume is not read past its mount point. The canonical
  home and its places are worked out once per call, and only when a root needs its links followed. New test
  `a_link_that_leads_onto_a_volume_is_not_read_past_its_mount_point` (the stop rule given a stand-in mount point; no
  real volume is mounted).
