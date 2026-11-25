# 042/01/01 — The High switch in the engine

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, engine, e2e, app/tauri, docs

The engine takes a switch, `HighLevel::{Offered, Withheld}`; withheld, no High finding leaves it and every way to
remove one is refused. The design: [`../00-design.md`](../00-design.md), section 1; agent's calls 1 and 12. Model: none
of the mockups (no screen changes).

## Read first

`crates/core/src/{danger.rs,error.rs,finding.rs,category.rs,module.rs}`, `crates/core/src/texts.rs` (`core_keys`),
`crates/engine/src/{engine.rs,results.rs,own.rs,registry.rs,session.rs,storage/shares.rs,cleanup/pending.rs,
apps/page.rs,apps/uninstall.rs,settings.rs (Ignored),testing.rs}`, the 14 constructor call sites (`rg
"Engine::(new|with_cleanup)\("`), `crates/e2e/tests/high_risk.rs` and `crates/e2e/tests/support/home_engine.rs` (how
an e2e test builds a home), `docs/restrictions/deletion-safety.md`.

## Files

- `crates/core/src/danger.rs` — `pub enum HighLevel { Offered, Withheld }` (`Clone, Copy, Debug, PartialEq, Eq, Hash,
  Serialize, Deserialize, TS`, `#[serde(rename_all = "kebab-case")]`) with `pub const fn offers(self, level:
  DangerLevel) -> bool` (false only for `Withheld` and `High`); exported from `lib.rs`. A doc comment: the first release
  withholds High; the app sets it at start (plan 042).
- `crates/core/src/error.rs` — `CommandError::HighWithheld` (its code `high-withheld`, a `Refusal` like the others);
  `crates/core/locales/en.json` `core.error.high-withheld`: "Items that may hold the only copy of your data are not
  offered in this version. Nothing was removed."; the key in `core_keys`.
- `crates/engine/src/engine.rs` — `Engine::new(registry, ports, high: HighLevel)` and `Engine::with_cleanup(registry,
  ports, cleanup, high: HighLevel)`; `Shared.high`; `pub fn high_level(&self) -> HighLevel`. No default.
- `crates/engine/src/results.rs` — a `Withheld` value set once at build from the switch and the registry: the ids of
  the registered categories whose descriptor is High (empty when offered). `shown()` leaves out a finding that the
  ignore lists hide **or** whose own `danger()` is High while withheld **or** whose category is in the set; `finding(id)`
  and the picks follow the same rule. `Committed` carries the withheld set (and the paths of the state's findings
  raised to High, gathered in `keep_state`) to the shares.
- `crates/engine/src/storage/shares.rs` — `Shares::of(state, ignored, withheld)`: an owner of a withheld category or
  at a withheld path is not counted, exactly as an ignored one (its bytes are the user's folders, never an outer
  owner's). Its doc comment says so.
- `crates/engine/src/engine.rs` / `registry.rs` — `modules()` while withheld: a module's `categories` leave out the
  withheld ones; its `choices` leave them out of their `categories`, and a choice left with none is dropped.
- `crates/engine/src/session.rs` — the `left_out` of `Event::ModuleFinished` leaves out the counts of a withheld
  category before it is published.
- `crates/engine/src/cleanup/pending.rs` — `rehearse_and_keep`: after the machine is idle and before
  `blocked::check`, a selected finding whose `danger()` is High while withheld refuses the plan with
  `CommandError::HighWithheld` (nothing kept, nothing rehearsed). This covers every prepare, the uninstall's too.
- `crates/engine/src/own.rs` — `pick_own_item` refuses with `HighWithheld` right after the idle check while withheld (a
  pick is always High).
- `crates/engine/src/apps/uninstall.rs` and `apps/page.rs` — `prepare_uninstall(id, UninstallChoice::Everything)`
  refused with `HighWithheld` before anything is read; the app page's `choices` leave out `Everything` while withheld.
- Every call site passes `HighLevel::Offered` (the existing tests are unchanged in meaning), but
  `app/tauri/src/compose.rs::assemble`, which passes `HighLevel::Offered` in this step too: step 02 makes the app
  withhold. `crates/engine/src/testing.rs`'s builders take an optional `.high(HighLevel)` (default `Offered`) for the
  new tests.
- `crates/engine/src/high_tests.rs` (new, `#[cfg(test)] #[path = "high_tests.rs"] mod high_tests;` in `engine.rs`, as
  `own_tests.rs` is).
- `crates/e2e/tests/high_withheld.rs` (new) on a fixture home with `app-leftovers` data (High) and caches (Safe), and a
  large untouched download (High) if the module's fixture helper allows it in the same home.
- `docs/restrictions/deletion-safety.md` — a section after "A High plan needs its own tick":
  `## High is withheld in the first release` — Rule: while the app sets `HighLevel::Withheld` (every build of the
  first release, development included), no finding at High leaves the engine: no page, total, found size, "Can be
  freed", module category or choice, or left-out names one, and every prepare with a High item, every Largest pick and
  every uninstall of everything is refused (`HighWithheld`), acting on nothing. Why: the user's decision of
  2026-11-24: the first release offers only what comes back or what the user can check by eye. Enforced by: the
  places above and their tests (`crates/engine/src/high_tests.rs`, `crates/e2e/tests/high_withheld.rs`); the
  sandbox's `high-withheld.toml` (step 02).

## Tests (negatives first)

`high_tests.rs`, `mod negative` (withheld):
- `a_high_finding_is_in_no_page_total_or_found_size` (a module with a High and a Safe category: only the Safe one).
- `a_finding_raised_to_high_is_left_out` (a Low category, one finding raised).
- `a_withheld_finding_is_unknown_by_id`.
- `the_map_counts_no_high_owner_as_can_be_freed` (its bytes are in your folders; `explained` excludes them).
- `a_module_lists_no_high_category_and_no_empty_choice`.
- `a_left_out_of_a_high_category_is_not_published`.
- `a_selection_of_a_high_category_finds_nothing_to_prepare` (whatever `plan::expand` answers, no plan holds High).
- `a_prepare_with_a_high_item_is_refused_and_keeps_no_plan` (drive the tail directly, as the uninstall does).
- `a_largest_pick_is_refused`.
- `an_uninstall_of_everything_is_refused_and_the_page_offers_keep_data_only`.
- `the_refusal_comes_before_the_agreement_check` (agreement not accepted: still `HighWithheld`).

`mod positive`:
- `offered_high_is_as_before` (the same module: both categories, a High plan prepared).
- `withheld_keeps_every_lower_level` (Safe, Low, Medium findings, totals and a plan as offered).
- `the_engine_reports_its_switch` (`high_level()`).

`crates/core/src/danger.rs`: negative `withheld_does_not_offer_high`; positive `offered_offers_every_level`,
`withheld_offers_safe_to_medium`, `serializes_as_kebab_case`. `error.rs`: the new code and its text key (with the
existing refusal tests).

`crates/e2e/tests/high_withheld.rs`: negative `a_withheld_home_offers_no_high_finding_and_refuses_its_plan` (findings,
overview totals, storage map "Explained" against `Offered` on the same home, the refusal, the files untouched);
positive `the_same_home_offered_lists_the_high_finding`.

## Checks

- `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core -p fresher-engine -p fresher-e2e -p fresher-app`,
  `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`; the bindings test (`app/tauri/tests/bindings.rs`)
  regenerates `bindings.generated.ts` if a type changed (`HighLevel` is exported only when a later step serializes it:
  if ts-rs exports it now, commit the regenerated file and run `npm run lint:ts`).
- `rg "Engine::(new|with_cleanup)\(" crates app` — every site passes a `HighLevel`.

Commit: `feat(engine): withhold High findings behind a switch`.

## Conclusions

- `HighLevel::{Offered, Withheld}` in `danger.rs`, a required argument of `Engine::new` and `Engine::with_cleanup`;
  `crates/engine/src/withheld.rs` holds the rule (its own file rather than inside `results.rs`). Every test and bench
  call site passes `Offered`.
- `FindingsAdded` also leaves withheld findings out (a batch left empty is not sent): the event crosses the IPC and the
  run record, so "no High finding leaves the engine" needed it.
- A selection naming a High category finds `NothingToClean`, a High finding by id `UnknownFinding` (it is never
  visible); the direct refusal is `HighWithheld`, first in `rehearse_and_keep` (ruling C-8, before the agreement).
- The raised paths count in the shares only at exactly their path, consistent with `shown()`. The Largest query still
  answers its rows; only the pick is refused (the UI hides the screen, 03/03).
- The final review added insight parts at High and an app page's footprint items of a High kind; task 27 added an Apps
  row's parts of a High kind and their bytes (`crates/engine/src/apps/list.rs`). `high_tests.rs`: 14 negatives, 4
  positives; `crates/e2e/tests/high_withheld.rs` on files.
