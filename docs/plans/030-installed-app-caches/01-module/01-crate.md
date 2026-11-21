# 030/01/01 — The crate, the tables and the texts

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, docs

The module's frame: its descriptor, its two tables and its interest, every text and the catalogue's file. Nothing is
decided yet: `on_match` and `finish` return `Ok(())` until 01/02. The design: [`../00-design.md`](../00-design.md),
sections 1, 5 and 6.

## Files

- `modules/app-caches/Cargo.toml` (new) — `fresher-module-app-caches`, description "The installed app caches module
  of Fresher: the caches installed apps keep in the user's Library."; the workspace's `version`, `edition`,
  `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`, `serde`, `thiserror`,
  `toml`; dev-dependency `fresher-testing` (the others in the steps that first use them). The workspace's members are
  the glob `modules/*`: no root edit.
- `modules/app-caches/places.toml` (new) — the one row of the design's section 1 (`library-caches`), with a head
  comment saying what each field means (`id`, `category`, `folder`, `path`) and that the `*` matches only a
  bundle-id-shaped name, the same interest as `app-leftovers`' `caches` row.
- `modules/app-caches/exclude.toml` (new) — `com.apple.` (prefix) and `io.github.alexsergey.fresher` (an id and every
  id under it), each with its `why`; a head comment: an entry only ever keeps, entries for apps whose cache holds what
  does not come back are added from 03/02.
- `modules/app-caches/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 1]`:
  `app-caches.library-caches` Low.
- `modules/app-caches/src/places.rs` (new) — `Places::shipped()` (`include_str!("../places.toml")`) and
  `Places::parse(text, categories) -> Result<Places, PlacesError>` with every refusal of the design's section 1, one
  variant each (`Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`, `CategoryWithoutRow`, `WalkedPlace`,
  `NotPlainStar`, `DuplicatePlace`); a `Row` (`id`, `category`, `folder: KnownFolder`, `path`), read with
  `deny_unknown_fields` and kebab-case names. `Places::interests()` builds, in the table's order, one
  `Under(folder, path)` `Claim::Whole` `.only_bundle_ids()` per row.
- `modules/app-caches/src/exclude.rs` (new) — `Exclude::shipped()`, `Exclude::parse(text)` with its refusals
  (`Format` (also for an entry with both or neither of `id` and `prefix`, or no `why`), `Empty`, `BadPrefix`, `BadId`,
  `Duplicate`, `AppleMissing`), and `Exclude::keeps(&BundleId) -> bool`: a
  prefix compared ignoring ASCII case, an id by `BundleId::names`. Written for this module, in the shape of
  `modules/app-leftovers/src/exclude.rs` (a module never depends on another).
- `modules/app-caches/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what it
  never does (no name that is not an id, nothing of Apple's or Fresher's, no folder whose owner is not an installed
  app, nothing while its app runs; the folder is emptied, never removed; no command); `AppCachesError` (`Id`,
  `TextKey`, `Descriptor`, `Interest`, `Places`, `Exclude`), `ExcludeError` and `PlacesError` re-exported (the error
  types of its private tables are public); `AppCachesModule::new()`: `ModuleId` `app-caches`,
  `ModuleGroup::System`, the category, no setting, the dictionary, `.with_problems(…)` with the four left-out keys in
  the order `special`, `not-yours`, `no-system`, `app-open`; `interests()` from `Places`; `on_match` and `finish`
  return `Ok(())`.
- `modules/app-caches/locales/en.json` (new) — every text of the design's section 6: the category's five texts, the
  switch's label and description, the two titles, the reason, the four left-out reasons. Plain English, no jargon
  (rule 7).
- `modules/app-caches/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(…))`
  (extended in 01/03 with the fixture runs).
- `docs/categories/app-caches-library-caches.md` (new) — in the catalogue's format (`**Danger level:** Low` ·
  `**Rule:** none (the module's code, `modules/app-caches`)` · `**Since:** 0.1.0`): what it is; where it lives
  (`~/Library/Caches/<id>`, one finding per folder, the app's own, a helper's or an id under the app's); what happens
  if removed (emptied into the Trash, the folder stays; how it comes back, the offline-content sentence); how the app
  decides (the design's section 4 table; never offered: a name that is not an id, `com.apple.`, Fresher's own, a
  removed app's cache — `app-leftovers`' — a vendor prefix, a link; kept while the app or a helper runs; containers
  not read). `docs/categories/README.md`: its row; in the danger table, "an installed app's cache" among the Low
  examples and the Safe examples' "app caches" made "an IDE's caches" (agent's call 4).
- `docs/dependencies.md` — the new crate in the used-by columns of `serde`, `thiserror`, `toml` and `fresher-testing`'s
  rows, as for `fresher-module-local-ai-models`.

## Produces

`AppCachesModule::new()`, `Places` with its interest, `Exclude`, and every text, for 01/02, 01/03 and 02/03.

## Checks

- `places.rs`, negatives first: a table made for each `PlacesError` variant is refused with it; then the shipped
  table: one row, the category used, one whole interest under `AppCaches` matching only bundle ids.
- `exclude.rs`, negatives first: each refusal; `keeps` refuses `com.example.App` and `io.github.alexsergey.fresherx`;
  then keeps `com.apple.Safari`, `COM.APPLE.x`, `io.github.alexsergey.fresher` and `io.github.alexsergey.fresher.dev`.
- The descriptor's tests: the group, the category with its danger, the four problems in order.
- `tests/contract.rs` passes.
- `cargo nextest run -p fresher-module-app-caches`; clippy; fmt; the crate guard and the dependency guard (no other
  module, `#![forbid(unsafe_code)]`); `cargo deny check`, `cargo machete`.

## Conclusions

Built as written. The crate `modules/app-caches` with its places table, `exclude.toml` (Apple's and Fresher's own ids), the texts and the category (Low, no setting, four left-out reasons in order). The error types of both tables are re-exported (preflight F5); an `exclude.toml` entry with both or neither of `id` and `prefix`, or no `why`, is a format error. The `proptest` dependency row waited for the step that uses it (F6). Review: approved.
