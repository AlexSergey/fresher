# 034/01/01 — The table and the two categories

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, docs

The table `apps.toml`, its parser with every refusal, its interests appended to the module's, and the two categories
with their five texts each and their catalogue files. Nothing is decided for a table row yet: the module ignores a
match of a table interest until 01/02. The design: [`../00-design.md`](../00-design.md), sections 1, 5 and 6.

## Context for the implementer

- The module lives in `modules/app-caches` (crate `fresher-module-app-caches`, plan 030). Read `src/lib.rs`,
  `src/places.rs`, `src/places/tests.rs`, `src/categories.rs`, `places.toml`, `exclude.toml` and `locales/en.json`
  first: the new table follows `places.rs`' shape (a `RawTable` read with `deny_unknown_fields` and kebab-case names,
  a checked `Row`, one error variant per refusal, `shipped()` with `include_str!`).
- No core change (`crates/core`, `crates/scanner`, `crates/cleaner`, `crates/engine` stay untouched), no new
  dependency. Rules of the repository: `CLAUDE.md` ("Rust Rules", "Testing Rules"); negative tests first in
  `mod negative`, then `mod positive`.
- Build with `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target`.

## Files

- `modules/app-caches/apps.toml` (new) — a head comment saying what each field means (`id`, `category`, `bundle-id`,
  `folder`, `path`, `caches`), that the one `*` of a `path` stands for a profile and is never its first name, that a
  name not in `caches` is never touched (a profile's history, cookies and passwords sit beside the caches), and that a
  row is added only after it was checked on a real install. The four rows, each with a one-line comment of what was
  checked:

  ```toml
  [[app]]
  id = "chrome"
  category = "app-caches.browsers"
  bundle-id = "com.google.Chrome"
  folder = "app-caches"
  path = "Google/Chrome/*"
  caches = ["Cache", "Code Cache"]

  [[app]]
  id = "chromium"
  category = "app-caches.browsers"
  bundle-id = "org.chromium.Chromium"
  folder = "app-caches"
  path = "Chromium/*"
  caches = ["Cache", "Code Cache"]

  [[app]]
  id = "firefox"
  category = "app-caches.browsers"
  bundle-id = "org.mozilla.firefox"
  folder = "app-caches"
  path = "Firefox/Profiles/*"
  caches = ["cache2", "startupCache"]

  [[app]]
  id = "redis-insight"
  category = "app-caches.web-engine"
  bundle-id = "org.RedisLabs.RedisInsight-V2"
  folder = "app-support"
  path = "RedisInsight"
  caches = ["Cache", "Code Cache", "GPUCache", "DawnGraphiteCache", "DawnWebGPUCache"]
  ```

- `modules/app-caches/src/categories.rs` — three constants:
  `pub(crate) const PLACE_CATEGORIES: [(&str, DangerLevel); 1]` (`app-caches.library-caches` Low),
  `pub(crate) const APP_CATEGORIES: [(&str, DangerLevel); 2]` (`app-caches.browsers` Low, `app-caches.web-engine`
  Low), and `pub(crate) const CATEGORIES: [(&str, DangerLevel); 3]`, the three in that order (written out; the
  descriptor's list). `Places::shipped()` checks against `PLACE_CATEGORIES` (its `CategoryWithoutRow` check must not
  see the table's categories).
- `modules/app-caches/src/apps.rs` (new):
  - `pub enum AppsError` (`thiserror`, `Clone, Debug, PartialEq, Eq`), one variant per refusal of the design's
    section 1 table, in its order: `Format(String)`, `Empty`, `BadId { id }`, `DuplicateId { id }`,
    `BadBundleId { id, bundle_id }`, `UnknownCategory { id, category }`, `CategoryWithoutRow { category }`,
    `WalkedPlace { id, folder: KnownFolder }`, `BadPath { id, path }`, `StarFirst { id, path }`,
    `IdShapedName { id, path }`, `NoCaches { id }`, `BadCache { id, cache }`, `OverlappingPlaces { first, second }`.
    Messages in the style of `PlacesError`'s.
  - `pub(crate) struct AppRow { pub(crate) id: String, pub(crate) category: &'static str, pub(crate) bundle_id:
    BundleId, pub(crate) folder: KnownFolder, pub(crate) path: String, pub(crate) profile: Option<usize>,
    pub(crate) caches: Vec<String> }` — `profile` is the index of the `*` among the path's names (`Some(2)` for
    `Google/Chrome/*`), `None` without one.
  - `pub(crate) struct Apps { rows: Vec<AppRow> }` with `shipped()` (`include_str!("../apps.toml")`, checked against
    `APP_CATEGORIES`), `parse(text, categories) -> Result<Apps, AppsError>`, `rows() -> &[AppRow]`, and
    `interests() -> Result<Vec<(Interest, usize, usize)>, InterestError>`: for each row in order, for each cache in
    order, `(Interest::new(Pattern::Under(folder, format!("{path}/{cache}")), Claim::Whole, Vec::new())?, row, cache)`.
  - Checks: the row id with the same `is_row_id` rule as `places.rs` (one kebab-case segment); `BundleId::new` and a
    dot for `bundle-id`; `KnownFolder::is_walked_place`; a path's names split on `/`, each not empty, not `.` or `..`,
    no NUL, no `*` inside a longer name, at most one `*` (`BadPath`), the first not `*` (`StarFirst`), the first not
    `fresher_core::is_bundle_id_shaped` (`IdShapedName`); `caches` not empty (`NoCaches`), each a single plain name
    without `*`, listed once (`BadCache`); two rows of one folder overlap when their paths have the same number of
    names and each pair of names is equal or one of them is `*` (`OverlappingPlaces`); `CategoryWithoutRow` after
    every row is read.
- `modules/app-caches/src/apps/tests.rs` (new) — as `src/places/tests.rs`.
- `modules/app-caches/src/lib.rs`:
  - `mod apps;` and `pub use crate::apps::AppsError;`; `AppCachesError::Apps(#[from] AppsError)`.
  - The module keeps `apps: Apps` and, beside `interests`, `sources: Vec<Source>` of the same length:
    `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub(crate) enum Source { Place { row: usize }, App { row: usize,
    cache: usize } }` (put it in `gathered.rs`, where 01/02 needs it). `new()` builds the places' interests (each
    `Source::Place`), then the table's (each `Source::App`).
  - `on_match` gathers only a `Source::Place` match, as today (a table match is ignored until 01/02).
  - The module doc names the new table: the cache folders a table ties to an installed browser or web-engine app, by
    exact name, never the data beside them.
- `modules/app-caches/locales/en.json` — the ten category texts of the design's section 5 (`app-caches.browsers.*`,
  `app-caches.web-engine.*`, each `name`, `what`, `made-by`, `if-removed`, `comes-back`), word for word unless the
  rule 7 polish needs a change. Not `app-caches.title.profile` (01/03: a text must be used by a fixture run).
- `docs/categories/app-caches-browsers.md` (new) and `docs/categories/app-caches-web-engine.md` (new) — in the format of
  `docs/categories/app-caches-library-caches.md` (`**Danger level:** Low · **Rule:** none (the module's code,
  `modules/app-caches`, table `apps.toml`) · **Since:** 0.1.0`): what it is; where it lives (the rows' places, one
  finding per cache folder, per profile for a browser); what happens if removed (emptied into the Trash, the folder
  stays; what stays: history, bookmarks, passwords, cookies, open tabs / messages, files, settings, sign-ins); how the
  app decides (only a name in the row, only while the row's app is installed and it and its helpers are closed; the
  plan-030 rows 1 to 11 with rows 2 and 4 by the table; never a link, a file, an empty folder; the browsers'
  support-folder caches and Safari not read; a new row only after a check on a real install).
- `docs/categories/README.md` — the two rows after `app-caches.library-caches`.

## Produces

`Apps` with its rows and interests, `Source`, the three categories and their texts, for 01/02 and 01/03.

## Checks

- `src/apps/tests.rs`, `mod negative`, one test per `AppsError` variant, each refusing a table made for it:
  `a_table_that_is_not_toml_or_has_an_unknown_field_is_refused`, `a_table_with_no_row_is_refused`,
  `a_row_id_that_is_not_kebab_case_is_refused`, `a_row_id_twice_is_refused`,
  `a_bundle_id_that_is_not_one_or_has_no_dot_is_refused`, `a_category_not_of_the_table_is_refused` (including
  `app-caches.library-caches`), `a_category_without_a_row_is_refused`, `a_walked_place_is_refused`,
  `a_path_with_a_bad_name_or_two_stars_is_refused`, `a_path_starting_with_a_star_is_refused`,
  `a_path_starting_with_an_id_shaped_name_is_refused`, `a_row_without_caches_is_refused`,
  `a_bad_or_repeated_cache_name_is_refused`, `overlapping_places_are_refused` (`Google/Chrome/*` and
  `Google/*/Default`; not refused: the same paths under two folders, and paths of different lengths).
  `mod positive`: `the_shipped_table_has_its_four_rows` (ids, bundle ids, folders, `profile` `Some(2)`, `Some(1)`,
  `Some(2)`, `None`), `the_shipped_table_gives_one_whole_interest_per_cache` (eleven, in order, each
  `Claim::Whole`, none shaped to bundle ids, the first `Under(AppCaches, "Google/Chrome/*/Cache")`),
  `no_shipped_row_is_kept_by_the_exclusions` (`Exclude::shipped()` keeps none of the rows' ids).
- The descriptor's tests: three categories in order, all Low; the four problems unchanged.
- `tests/contract.rs` passes unchanged (the two new category files exist with `Low`).
- `cargo nextest run -p fresher-module-app-caches`; `cargo clippy -p fresher-module-app-caches --all-targets -- -D
  warnings`; `cargo fmt --check`; `npx prettier --check` and `npm run lint:md` (or the repository's Markdown lint) on
  the changed docs.

## Conclusions

Built as the step says: `apps.toml` with its four rows, `Apps` (parser with 14 refusals, one test each), the
interests of the table appended to the module's, the two categories after `library-caches`, their ten texts and their
catalogue files. `cargo nextest run -p fresher-module-app-caches`: 104 tests; `apps.rs` and `lib.rs` 100 % of lines,
every file of the crate at least 98 %.

Changes from the design: `AppRow.profile` is an `Option<usize>` (the index of the `*`), not a `bool`, as the step
says; `is_row_id` of `places.rs` became `pub(crate)` and is reused by `apps.rs` (one rule, no copy); two
`#[expect(dead_code)]` stood until step 02 used `Apps::rows` and the `apps` field, and step 02 removed them.
