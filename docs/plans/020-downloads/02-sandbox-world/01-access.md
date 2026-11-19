# 020/02/01 — Access in the world and the banner

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** sandbox, core, app, app/ui, docs

A case states what the guarded folders allow (`access = { downloads = "granted" }`); sandbox mode and the autotest
answer access from it instead of `UnknownAccess`, and the standing avoid list stops hiding the sandbox's own stated
folders, which the engine's coverage then decides as on a Mac. The real home's guarded places stay avoided whatever
the world says. The banner lists what is simulated. The design: [`../00-design.md`](../00-design.md), section 2
(agent's calls 13 and 14).

## Files

- `crates/sandbox/src/case.rs` — `RawCase` gains `access: Option<BTreeMap<String, RawAccess>>`
  (`RawAccess::{Granted, Denied, NotAsked}`, kebab case); a key must be `desktop`, `documents` or `downloads`
  (`Capability::Folder(_).id()`), anything else refused with the case's name ("`access` names `<key>`: only the guarded
  folders desktop, documents and downloads"); `Case::access() -> &[(GuardedFolder, Access)]`. The comment above
  `RawCase` names `access` among the keys that are not files.
- `crates/sandbox/src/world.rs` — `World` gains `access: Vec<(GuardedFolder, Access)>`, joined in `World::of`: for a
  folder stated twice the most restrictive wins (`Denied` over `NotAsked` over `Granted`); `RawWorld` gains
  `access: BTreeMap<String, RawAccess>` written as `[access]` only when not empty (a world without it is written as
  before), with a comment line in `text()`; `parse` refuses an unknown key or value. New methods:
  - `pub fn declares_access(&self) -> bool`;
  - `pub fn access(&self, home: &Path) -> Option<FakeAccess>`: `None` when nothing is stated; otherwise
    `FakeAccess::with` every capability of `Capability::SYSTEM` (Full Disk Access `Unknown`, a stated folder as stated,
    every other `NotAsked`), `.home(home)`, each stated folder guarded at its name (`Desktop`, `Documents`,
    `Downloads`) and `.answering` `Granted` for `Granted` and `NotAsked`, `Denied` for `Denied`;
  - `pub fn stated_places(&self, home: &Path) -> Vec<PathBuf>`: `home/<Folder>` of each stated folder.
- `crates/sandbox/src/expected.rs` — `EXPECTED.md` says which folders the sandbox allows, refuses or has not been asked
  for, and for `not-asked` that the switch in Settings → Permissions must be turned on first.
- `crates/core/src/app_info.rs` — `Build::Sandbox { path, simulated: Vec<Simulated> }` with `pub enum Simulated {
  Running, Spotlight, Access, Installs }` (kebab case, `TS`), replacing `simulated_running` and
  `simulated_spotlight`; `Installs` is set by 02/02. The serialization test updated.
- `app/tauri/src/sandbox.rs` — `SandboxPorts::real`: `access` is `world.access(&sandbox.home())` when stated, else
  `UnknownAccess`; `compose`: the standing avoid list is `places_that_prompt(&home)` without
  `world.stated_places(&home)`, plus `places_that_prompt(real_home)` as today; `build()` lists `Running` always,
  `Spotlight` when `declares_spotlight`, `Access` when `declares_access`.
- `app/tauri/src/commands.rs` — the two `Build::Sandbox` literals of its tests take `simulated`.
- `app/tauri/src/adapters.rs` — `UnknownAccess`'s doc: the stand-in when the world states no access.
- `app/tauri/tests/sandboxes/harness.rs` — `fakes()`: `access` is `world.access(sandbox_home)` when stated, else the
  `FakeAccess` it uses today.
- `app/ui/src/ipc/bindings.generated.ts` — regenerated. `app/ui/src/product/build-banner/build-banner.tsx` — the text
  is `ui.build-banner.sandbox` when nothing is simulated, else `ui.build-banner.sandbox-simulated` with `{parts}`: the
  parts' texts (`ui.build-banner.simulated.running` "open apps", `.spotlight` "Spotlight", `.access` "access",
  `.installs` "installs") joined with ", " in the core's order and the first letter capitalized; the three old
  combination keys removed from `app/ui/src/locales/en.json`. `build-banner.test.tsx`, `build-banner.stories.tsx`,
  `app/ui/src/screens/app-frame.test.tsx` updated.
- `docs/testing.md` — the case-file vocabulary: `access` (keys, values, the most restrictive wins, `not-asked` answers
  `granted` once the switch is on); `world.toml` gains `[access]`.
- `docs/hacks/README.md` ("Sandbox mode avoids the places that prompt") — the sandbox home's places of the folders the
  world states are not avoided; its access is the world's when stated. `docs/architecture/permissions.md`
  ("Developer builds") — the same.
- `docs/deferred/sandbox-world.md` — "Partly taken by plan 020": the access of the guarded folders (Full Disk Access
  and other apps' data stay `Unknown` / `NotAsked`); its row in `docs/deferred/README.md` updated.

## Produces

`access` in the case files and `world.toml`, `World::{access, stated_places, declares_access}`, `Simulated`, for
04/01 (`downloads.toml` granted, `downloads-denied.toml` denied) and 04/03 (the banner in the gallery).

## Checks

- `case.rs`, negatives first: `an_access_key_other_than_a_guarded_folder_is_refused` (`all-files`,
  `other-apps-data`, `x`), `an_access_value_other_than_the_three_is_refused` (`"yes"`, `true`); then
  `access_parses_per_folder`, `a_case_that_states_no_access_has_none`.
- `world.rs`, negatives first: `a_world_without_access_writes_no_access_table` (the text is byte for byte today's for
  a world with running apps only), `an_unknown_access_key_in_world_toml_is_refused`; then
  `the_most_restrictive_statement_wins` (granted + denied → denied; granted + not-asked → not-asked),
  `access_round_trips_through_world_toml`, `access_answers_every_capability` (Full Disk Access `Unknown`, Documents
  `NotAsked`, Downloads as stated, the place guarded at `home/Downloads`, `try_place` answering as the design says),
  `stated_places_are_the_stated_folders_under_the_home`.
- `app/tauri/src/sandbox/tests.rs`, negatives first: `the_real_homes_guarded_places_stay_avoided_whatever_the_world`
  (a world granting Downloads: the avoid list still holds `<real home>/Downloads`),
  `a_world_without_access_keeps_the_sandbox_homes_downloads_avoided`; then
  `a_world_granting_downloads_lets_the_walk_into_the_sandbox_downloads` (a scan of a sandbox with
  `home/Downloads/x` and no module but the engine's coverage: the coverage does not name Downloads),
  `the_banner_lists_access_when_the_world_states_it`.
- UI: `npx vitest run app/ui/src/product/build-banner/build-banner.test.tsx app/ui/src/screens/app-frame.test.tsx`
  (negatives first: no part → the plain text; then one part, then all four in order).
- `cargo nextest run -p fresher-sandbox -p fresher-core -p fresher-app`; `npm run sandbox:test` (every existing case
  file passes unchanged: none states access); clippy; fmt; `npm run lint`.

## Conclusions

A case's `access` and `world.toml`'s `[access]` share one value type and one key check; the world joins them most
restrictive first. Sandbox mode and the autotest take `World::access` when stated, and the standing avoid list drops
the sandbox home's places of the stated folders only. The banner text is "Sandbox · {path} · Simulated: {parts}."
with the parts lowercase in the core's order, no capitalisation (preflight G3), and the e2e mock, the visual test and
the gallery images changed with it (G4). The walk-into-Downloads test asserts a node at `home/Downloads/x` and fails
without the avoid-list change (G5). Bindings regenerated (`Simulated`); `EXPECTED.md` names each stated folder.
