# 019/02/01 — The crate, the tables and the texts

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, docs

The module's frame: its descriptor, its two tables and its interests, its setting and choices, every text and the
catalogue's three files. Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 02/02. This step creates
`src/lib.rs`, `locales/en.json` and `tests/contract.rs`; 02/02 and 02/03 extend them.

## Files

- `modules/app-leftovers/Cargo.toml` (new) — `fresher-module-app-leftovers`, description "The uninstalled app
  leftovers module of Fresher: the caches, settings and data of apps that are no longer installed."; the workspace's
  `version`, `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`,
  `serde`, `thiserror`, `toml`; dev-dependencies only what this step's tests use (`fresher-testing` for the contract
  test, `serde_json` if the settings test reads stored values); `proptest` waits for 02/02 (`cargo machete`). Root
  `Cargo.toml`: the member.
- `modules/app-leftovers/places.toml` (new) — the ten rows of the design's section 2, in its order (`caches`, `logs`,
  `saved-state`, `preferences`, `http-storage`, `http-cookies`, `web-data`, `cookies`, `support`, `container`), with a
  head comment saying what each field means, why `saved-state` and `container` are proofs, why `container` is optional
  (the app does not open `Library/Containers` until plan 006; a sandboxed app keeps everything inside its container),
  and that group containers are left out (the user's decision 1).
- `modules/app-leftovers/exclude.toml` (new) — `[[exclude]] prefix = "com.apple."` with its `why`, and a head comment:
  an entry only ever keeps; the ids shared frameworks write are added from the run by hand (04/02).
- `modules/app-leftovers/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 3]`:
  `app-leftovers.caches` Safe, `.settings` Medium, `.data` High.
- `modules/app-leftovers/src/places.rs` (new) — `Places::shipped()` (`include_str!("../places.toml")`) and
  `Places::parse(text, categories) -> Result<Places, PlacesError>`, one variant per refusal of the design's section 2
  (`Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`, `CategoryWithoutRow`, `WalkedPlace`,
  `GroupContainers`, `NotAStarPath`, `SameForm`, `BadSuffix`, `OptionalDiffers`, `NoProof`); a `Row` (`id`,
  `category`, `folder: KnownFolder`, `path`, `kind: Form { Folder, File }`, `suffix: Option<String>`, `proof`,
  `optional`) read with `deny_unknown_fields` and kebab-case names. `Places::interests()`: one `Under(folder, path)`
  `Claim::Whole` `.only_bundle_ids()` per distinct `(folder, path)`, in the table's order of first use (nine);
  `Places::rows_of(interest) -> &[usize]`; `Places::row(index)`.
- `modules/app-leftovers/src/exclude.rs` (new) — `Exclude::shipped()`, `Exclude::parse(text) -> Result<Exclude,
  ExcludeError>` (`Format`, `Empty`, `BadPrefix` — a prefix that does not end with `.` or whose part before it is not
  a bundle id —, `BadId`, `Duplicate`, `AppleMissing`; an entry has exactly one of `prefix` and `id`, and a `why`), `Exclude::covers(&BundleId)
  -> bool` (an exact `id`, or the id starting with a `prefix`, case ignored).
- `modules/app-leftovers/src/settings.rs` (new) — `app-leftovers.uninstalled-after-days`, 7 to 3650, 30 by default,
  as `modules/jetbrains/src/settings.rs` (the same functions, the module's own error).
- `modules/app-leftovers/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers (an
  id-named leftover of an app that is proven, removed, not running, not found by Spotlight, unchanged for the days)
  and what it never does (a name that is not an id, `com.apple.`, group containers, launch agents, any command);
  `AppLeftoversError` (`Id`, `TextKey`, `Descriptor`, `Interest`, `Places`, `Exclude`, `Setting`);
  `AppLeftoversModule::new()`: `ModuleId` `app-leftovers`, `ModuleGroup::Apps`, the three categories, the setting, the
  dictionary, `.with_problems(…)` with the nine left-out keys in the design's order (`no-system`, `apps-unread`,
  `running`, `scan-again`, `used-recently`, `spotlight-unread`, `installed-elsewhere`, `special`, `not-yours`),
  `.with_choices(…)` with `everything` (the three categories) and `keep-settings` (caches); `interests()` from
  `Places`; `on_match` and `finish` return `Ok(())` (each with an `#[expect(…, reason = "…")]` only if clippy needs
  one, removed by 02/02).
- `modules/app-leftovers/locales/en.json` (new) — every text of the design's section 4: the three categories' five
  texts each (`app-leftovers.<name>.{name,what,made-by,if-removed,comes-back}`), the switch's label and description,
  the setting's, the two choices' (`app-leftovers.choice.<name>.{label,description}`), the ten titles
  (`app-leftovers.title.<row id>`, each with `{id}`), the finding's reason (with `{days}`), the nine left-out reasons.
  Plain English, no jargon (rule 7).
- `modules/app-leftovers/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(docs/categories))`.
- `docs/categories/app-leftovers-caches.md`, `-settings.md`, `-data.md` (new) — in the catalogue's format
  (`**Danger level:** Safe`, `Medium`, `High` · `**Rule:** none (the module's code, `modules/app-leftovers`)` ·
  `**Since:** 0.1.0`): what it is; where it lives (the rows of that category, with their names); what happens if
  removed and whether it comes back (the texts); how the app decides (the design's section 3, the proof, what is never
  offered: names that are not ids, Apple's own, group containers, launch agents, an installed, running or elsewhere
  app; the Trash). `docs/categories/README.md`: their three rows.
- `docs/dependencies.md` — the new crate's rows, as for `fresher-module-dev-caches`.

## Produces

`AppLeftoversModule::new()`, `Places` with its interests and rows, `Exclude`, the setting, the choices and every text,
for 02/02, 02/03 and 03/03.

## Checks

- `places.rs`, negatives first: a table made for each `PlacesError` variant is refused with it; then the shipped
  table: ten rows, every category of `CATEGORIES` used, nine interests all shaped and whole, `saved-state` and
  `container` proofs, `container` alone optional, `http-storage` and `http-cookies` on one interest.
- `exclude.rs`, negatives first: each `ExcludeError`; `com.applex.App`, `com.example.App` and `apple.com.x` not
  covered; then `com.apple.Safari` and `COM.APPLE.finder` covered.
- `settings.rs`: as `jetbrains`' tests (unresolved or out of range → the default or the nearest limit; then 7, 30,
  3650).
- `tests/contract.rs` passes (the three files, their dangers, every text key the descriptor implies, the choices').
- `cargo nextest run -p fresher-module-app-leftovers`; clippy; fmt; the crate guard and the dependency guard (no other
  module, `#![forbid(unsafe_code)]`); `cargo deny check`; `cargo machete`.

## Conclusions

Built (d10a073): `modules/app-leftovers` (`fresher-module-app-leftovers`) with `places.toml` (ten rows, nine distinct
places), `exclude.toml`, `Places::{shipped, parse, interests}` with its 13 `PlacesError` variants and `Exclude` with its
errors, the descriptor (group Apps, three categories, the days setting, nine problems, two choices), the texts
(`locales/en.json`) and `tests/contract.rs`; three category docs and their catalogue rows; `dependencies.md` rows.

Deviations and calls:

- **Nine interests, not eight.** The design's own ten rows give nine distinct `(folder, path)` pairs (only
  `http-storage` and `http-cookies` differ in kind on one place); the rows were kept verbatim. The indexes: 0 caches,
  1 logs, 2 saved state, 3 preferences, 4 HTTPStorages, 5 WebKit, 6 Cookies, 7 support, 8 containers. The design and the
  later steps now say nine.
- `Exclude::covers`, `Places::{rows_of, row}` and the settings reader moved to 02/02 (they are used only there; preflight
  F7), so no `#[expect(dead_code)]` was needed.
- `WalkedPlace` refuses every known folder outside `library`, `app-caches`, `app-support`, `preferences`, `logs`,
  `containers` and `saved-state` (F21); `GroupContainers` is its own variant.
- The day setting's descriptor is a copy of `jetbrains`' (about 30 lines; a module may not depend on another). Done later by plan 020
  ([01/01](../../020-downloads/01-core-prerequisites/01-day-setting.md)): the shared `DaySetting` in
  `crates/core/src/settings/days.rs` replaced the copies (F16).

Tests: 33, negatives first, one for each error variant; the contract test went red on the three missing category docs and
green after them. Clippy, fmt, `cargo machete` and `cargo deny` clean.

Parked minors (fixed in 02/02): the library path guard, a test in the wrong block, the `keep-settings` choice by id, and
`is_suffix` accepting empty names between dots.
