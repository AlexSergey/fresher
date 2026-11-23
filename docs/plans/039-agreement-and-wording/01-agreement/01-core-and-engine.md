# 039/01/01 — The core and the engine

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** core, engine, e2e, app/tauri, docs

The agreement's text and version in the core, its acceptance as two core settings, and the engine's refusal of a
cleanup without it. After this step no cleanup can start in an engine whose settings do not hold the current
acceptance; every test helper that cleans accepts it. The design: [`../00-design.md`](../00-design.md), section 1.

## Read first

`crates/core/src/cleanup/confirm.rs`, `crates/core/src/cleanup/outcome.rs` (`BlockedReason`),
`crates/core/src/settings.rs` and `settings/core_schema.rs`, `crates/core/src/command.rs` (`SettingsView::new`),
`crates/core/src/error.rs` (`CommandError`, `CODES`, the `one_of_each` test), `crates/core/src/texts.rs`
(`core_keys`: the dictionary must hold exactly the keys the core can emit), `crates/engine/src/settings.rs`
(`update`, `reset`, `IGNORE_LISTS`, `is_ignore_list`), `crates/engine/src/cleanup/blocked.rs` and its two callers on
740cf3c3: `rehearse_and_keep` in `cleanup/pending.rs` (the tail of every plan: `prepare` and
`apps::uninstall::prepare` both end in it) and `confirm` in `cleanup/operation.rs`; `crates/engine/src/engine.rs` (the
command methods), `crates/engine/src/testing.rs` (`MemorySettings`, `Fixture` and its `engine()`, the free
`engine(walk)`), `crates/engine/src/apps/uninstall_tests.rs` (how an uninstall is prepared in a test).

## Files

- `crates/core/src/agreement.rs` (new), exported from `lib.rs`:
  - `pub const AGREEMENT_VERSION: u32 = 1;`
  - `pub const AGREEMENT_LINES: [&str; 5] = ["no-warranty", "your-decision", "best-judgement", "backup", "trash"];`
    each the key `core.agreement.line.<id>`.
  - `pub struct Acceptance { pub version: u32, pub at: Timestamp }` (`Clone, Copy, Debug, PartialEq, Eq, Serialize,
    TS`, camelCase).
  - `pub struct AgreementView { pub version: u32, pub title: Text, pub intro: Text, pub lines: Vec<Text>, pub accepted:
    Option<Acceptance>, pub changes: Option<Text> }` (`Clone, Debug, PartialEq, Eq, Serialize, TS`, camelCase).
  - `impl AgreementView`: `pub fn new(accepted: Option<Acceptance>) -> Self` = `of(AGREEMENT_VERSION, accepted)`;
    `pub(crate) fn of(current: u32, accepted: Option<Acceptance>) -> Self`; `changes` is
    `Some(core.agreement.changes.v<current>)` only when `accepted` is `Some` with `0 < version < current`;
    `pub fn is_accepted(&self) -> bool` is `accepted.version == self.version`.
  - Doc comment: changing a line means raising `AGREEMENT_VERSION` and adding `core.agreement.changes.v<n>`.
- `crates/core/locales/en.json` — the keys, texts exactly:
  - `core.agreement.title`: "Before your first cleanup"
  - `core.agreement.intro`: "Five things to know before Fresher removes anything:"
  - `core.agreement.line.no-warranty`: "Fresher is free and open source, and it comes with no warranty."
  - `core.agreement.line.your-decision`: "It removes only what you confirm, so each removal is your decision."
  - `core.agreement.line.best-judgement`: "A danger level is Fresher's best judgement, not a guarantee."
  - `core.agreement.line.backup`: "Keep a backup of anything you cannot afford to lose."
  - `core.agreement.line.trash`: "Items go to the Trash, where you can put them back until the Trash is emptied. A
    permanent delete, and what an app's own tool removes, cannot be undone." (refreshed 2026-11-23, design call 19)
  - `core.cleanup.blocked.agreement`: "Fresher removes nothing until you accept the short agreement. Read it, then try
    again."
  - `core.error.agreement-changed`: "The agreement changed while it was open. Read it again before you accept it."
- `crates/core/src/settings/core_schema.rs` — `pub const AGREEMENT_VERSION_KEY: &str = "core.agreement.version";`
  `pub const AGREEMENT_ACCEPTED_AT: &str = "core.agreement.accepted-at";` two descriptors at the end of
  `core_settings()`: `Number { min: 0, max: 1000, unit: None }` default `Number(0)`, and
  `Number { min: 0, max: i64::MAX, unit: None }` default `Number(0)`; both `SettingGroup::Agreement`,
  `SettingEffect::Now`. Export both from `settings.rs` and `lib.rs`.
- `crates/core/src/settings.rs` — `SettingGroup::Agreement` (doc: "the acceptance of the agreement: never shown as
  rows, written only by `Engine::accept_agreement`").
- `crates/core/src/command.rs` — `SettingsView::new` leaves out the descriptors and the values of
  `SettingGroup::Agreement` (the screen never shows them; the UI reads `get_agreement`).
- `crates/core/src/texts.rs` — `core_keys`: the agreement's keys (title, intro, five lines, `changes.v<n>` for
  `n in 2..=AGREEMENT_VERSION`), and no `label`/`description` for the settings of `SettingGroup::Agreement`.
- `crates/core/src/cleanup/outcome.rs` — `BlockedReason::Agreement` (doc: "the current agreement is not accepted"),
  added to `ALL` and `text()` (`"agreement"`).
- `crates/core/src/error.rs` — `CommandError::AgreementChanged` (`#[error("the agreement is not of the current
  version")]`), in `CODES` (`agreement-changed`) and in `one_of_each`.
- `crates/engine/src/settings.rs`:
  - `update` refuses the two agreement keys with `CommandError::UnknownSetting { key }`, before any save.
  - `reset` keeps them, as it keeps `IGNORE_LISTS` (one `kept` list; its doc comment says so).
  - `pub(crate) fn agreement(&self) -> Option<Acceptance>`: `Some` when the stored version is above 0.
  - `pub(crate) fn accept_agreement(&mut self, version: u32, at: Timestamp, store: &dyn SettingsStore) ->
    Result<(), CommandError>`: saves both values in **one** save (one `StoredSettings` with both keys), then takes
    them; refused on a read-only file.
- `crates/engine/src/cleanup/blocked.rs` — `check(settings: &Settings, ports: Option<&CleanupPorts>)` (today it takes
  `SettingsFile`): the settings file first (`Settings`), then `Agreement` when `settings.agreement()` is not of
  `AGREEMENT_VERSION`, then the journal. Its two callers already hold the settings' lock
  (`lock(&shared.settings)?.file()`): they pass `&*lock(&shared.settings)?` instead, in `rehearse_and_keep`
  (`cleanup/pending.rs`) and `confirm` (`cleanup/operation.rs`). The uninstall's `prepare`
  (`crates/engine/src/apps/uninstall.rs`) reaches the check through `rehearse_and_keep`: no new call site. The module
  doc lists the preconditions in order.
- `crates/engine/src/engine.rs` — `pub fn agreement(&self) -> Result<AgreementView, CommandError>` and
  `pub fn accept_agreement(&self, version: u32) -> Result<AgreementView, CommandError>`: `AgreementChanged` when
  `version != AGREEMENT_VERSION`; the time from `shared.ports.clock`; allowed while a scan or a cleanup runs (it
  changes nothing on disk). Both documented.
- `crates/engine/src/testing.rs` — every engine `Fixture::engine()` builds (and the free `engine(walk)`) holds the
  acceptance by default, so the 140 call sites stay as they are: `Fixture` gains `agreement: bool` (default `true`)
  and `unaccepted(self) -> Self`; `engine()` seeds the two values into the settings it was given before the engine is
  built (a `MemorySettings` method that inserts them into its stored values without counting a save), so a test's own
  `.settings(MemorySettings::with(..))` keeps them. Tests that assert `stored()` (8) or `saves()` (14) are read: a
  `stored()` equality gains the two keys or compares only the keys it is about. Never weaken the check to make a test
  pass.
- Every other test that cleans keeps passing in this same step, so the workspace stays green. The engines that clean,
  on 740cf3c3, are built in exactly these places; each accepts once after it is built, through
  `Engine::accept_agreement(AGREEMENT_VERSION)` (or by seeding its memory settings):
  - `crates/e2e/tests/support/home_engine.rs` (used by every `*_home/` fixture and the per-module e2e tests:
    `app_caches*.rs`, `app_leftovers.rs`, `dev_caches.rs`, `docker.rs`, `downloads.rs`, `ignore.rs`, `largest.rs`,
    `local_ai_models*.rs`, `logs.rs`, `stale_projects.rs`, `temp.rs`, `trash.rs`, `uninstall.rs`,
    `unused_toolchains.rs`, `xcode.rs`), `crates/e2e/tests/cleaning/mod.rs`, `crates/e2e/tests/places.rs`,
    `crates/e2e/tests/jetbrains_home/mod.rs`;
  - `app/tauri/src/testing.rs` (the `Shell`, also used by `shell.rs` and `dev_runs/shell_tests.rs`),
    `app/tauri/src/sandbox/tests.rs` (its `compose` calls);
  - the sandbox autotest: `app/tauri/tests/sandboxes/harness.rs` (`World::engine`, used by `run` and by
    `sandboxes/uninstall.rs`). The harness only accepts here; its assertion of the refusal is 01/02's.
  `crates/e2e/tests/demo.rs` builds `Engine::new` (no cleanup ports) and needs nothing. Check the list again with
  `grep -rn 'with_cleanup(\|sandbox_engine(\|compose(' crates app modules`.
- `docs/restrictions/deletion-safety.md` — a section "The agreement": the five lines quoted; the engine refuses every
  prepare and confirm without the current acceptance (`BlockedReason::Agreement`); put back never needs it; changing
  a line raises `AGREEMENT_VERSION` and writes `core.agreement.changes.v<n>`; the acceptance is local and never sent.
- `docs/architecture/removal.md` — the preconditions of a cleanup gain the agreement, in their order.

## Tests (negatives first)

- `agreement.rs`: negative — "an unaccepted agreement is not accepted"; "an older acceptance is not accepted and names
  what changed" (`of(2, Some(v1))` → `changes` = `core.agreement.changes.2`); "a version 0 acceptance shows no
  changes"; positive — "the current acceptance is accepted"; "the lines are the five keys in order".
- `core_schema.rs`: negative — "the agreement keys are in the agreement group" and still pass the banned-words test;
  `command.rs`: negative — "the settings view leaves the agreement group out".
- `outcome.rs` / `error.rs`: the new reason and code have texts (the existing exhaustive tests).
- `engine/settings.rs`: negative — "update refuses the agreement keys and saves nothing"; "accepting on a read-only
  file is refused"; positive — "reset keeps the acceptance"; "acceptance saves both values once".
- `engine/cleanup` (beside the existing blocked tests in `pending.rs` and `operation.rs`): negative — "prepare without
  the agreement is blocked and holds no plan"; "confirm without the agreement is blocked and acts on nothing" (accept,
  prepare, withdraw by a store edit or a fresh unaccepted engine, confirm); "a broken settings file is reported before
  the agreement"; "an acceptance of an older version is blocked"; positive — "after acceptance the cleanup runs";
  "put back needs no acceptance".
- `apps/uninstall_tests.rs`: negative — "an uninstall without the agreement is blocked and holds no plan" (the
  `UninstallRefused` and `UnknownApp` refusals come first, as today: the check sits in `rehearse_and_keep`).
- `own_tests.rs`: positive — "picking a Largest row needs no acceptance" (`pick_own_item` changes nothing on disk).
- `engine.rs`: negative — "accept_agreement of another version is refused and saves nothing"; positive —
  "accept_agreement records the clock's time and answers the view".

## Checks

- `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core -p fresher-engine`, then
  `cargo nextest run --workspace` (every crate that builds an engine compiles and passes, `fresher-app`'s sandbox
  autotest and `fresher-e2e` included); `cargo clippy --workspace --all-targets -- -D warnings`.
- The UI still builds: `bindings.generated.ts` is regenerated only in 01/02 (the new types are not exported to it yet);
  if the bindings test regenerates it here because `SettingGroup`, `BlockedReason` or `CommandError` changed, run
  `npm run lint:ts` and `npm run test:ui` and commit the regenerated file; `app/ui/src/screens/settings/settings-groups.ts`
  then adds `'agreement'` to the groups `CoreRowsGroup` excludes (the view never holds it).
- `git diff -- crates/cleaner crates/scanner crates/macos crates/system crates/store crates/rules modules` is empty.

## Conclusions

- Built as designed: `AGREEMENT_VERSION = 1`, the five lines and their texts in `core.agreement.*`, the two hidden
  settings (`SettingGroup::Agreement`), `Engine::accept_agreement` (one save, the engine's clock), and
  `BlockedReason::Agreement` in `blocked::check` after the settings file and before the journal. The two call sites
  (`rehearse_and_keep`, which ends every prepare including the uninstall's, and `operation::confirm`) are the only
  ones; put back, picking a Largest row, the Trash view, scans and settings never check it.
- The "what changed" key is `core.agreement.changes.v<n>`: a text key may not have a digits-only segment. The design
  text now says so.
- With version 1 no older version exists, so "an older acceptance asks again" is proven by the core's view of
  `AgreementView::of(2, v1)` and by a stored version 0 and 2 being blocked.
- From the final review: `accept_agreement` is refused (`CleaningBlocked`, `Settings`) unless the settings file was
  read, so it never replaces a broken file and unblocks cleaning; the settings view leaves out the problems of the
  hidden keys too.
