# 018/02/01 — The crate, the table and the texts

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, docs

The module's frame: its descriptor, its table and its interests, every text and the catalogue's nine files. Nothing is
decided yet: `on_match` and `finish` return `Ok(())` until 02/02.

## Files

- `modules/dev-caches/Cargo.toml` (new) — `fresher-module-dev-caches`, description "The developer caches module of
  Fresher: the download caches of package managers, Go's build cache and Xcode's derived data."; the workspace's
  `version`, `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`,
  `serde`, `thiserror`, `toml`; dev-dependencies `fresher-testing`, `proptest`, `serde_json`. Root `Cargo.toml`: the
  member.
- `modules/dev-caches/tools.toml` (new) — the ten rows of the design's section 2, in its order (`npm`, `yarn`,
  `homebrew`, `cargo-cache`, `cargo-src`, `cocoapods`, `gradle`, `pip`, `go-build`, `xcode-derived-data`), with a head
  comment saying what each field means and that the signs marked "to verify" are checked by hand in 04/02.
- `modules/dev-caches/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 9]`, in the
  design's section 4 order: `dev-caches.npm` Low, `.yarn` Low, `.homebrew` Low, `.cargo` Low, `.cocoapods` Low,
  `.gradle` Low, `.pip` Low, `.go-build` Safe, `.xcode-derived-data` Safe.
- `modules/dev-caches/src/tools.rs` (new) — `Tools::shipped()` (`include_str!("../tools.toml")`) and
  `Tools::parse(text, categories) -> Result<Tools, ToolsError>` with every refusal of the design's section 2, one
  `ToolsError` variant each (`Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`, `CategoryWithoutRow`,
  `WalkedPlace`, `NotRelative`, `SignNotAbove`, `SamePlace`, `SignsDiffer`, `EmptySign`, `NotAName`, `TooManySigns`,
  `BundleId`, `ShapeWithoutStar`); a `Cache` row (`id`, `category`, `tool`, `folder: KnownFolder`, `path`, `sign_at`,
  `sign`, `comes_back: ComesBack { Download, Build }`, `app: Option<BundleId>`, `name: Option<NameShape>` with the
  one shape `ProjectHash`), read with `deny_unknown_fields` and kebab-case names. `Tools::interests()` builds, in the
  table's order, one `Under(folder, path)` `Claim::Whole` per row (`.holding(sign)` when the row has no `sign-at`),
  then one `Under(folder, sign-at)` `Claim::Item` `.holding(sign)` per distinct `(folder, sign-at)`; and the map from
  an interest index to `Role::Cache(row)` or `Role::SignHolder(index)`.
- `modules/dev-caches/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what it
  never does (no tool's command, default places only); `DevCachesError` (`Id`, `TextKey`, `Descriptor`, `Interest`,
  `Tools`); `DevCachesModule::new()`: `ModuleId` `dev-caches`, `ModuleGroup::Development`, the nine categories, no
  setting, the dictionary, `.with_problems(…)` with the four left-out keys in the order `not-yours`, `special`,
  `app-open`, `no-system`; `interests()` from `Tools`; `on_match` and `finish` return `Ok(())`.
- `modules/dev-caches/locales/en.json` (new) — every text of the design's section 4: the nine categories' five texts
  each (`dev-caches.<name>.{name,what,made-by,if-removed,comes-back}`), the switch's label and description, the ten
  titles (`dev-caches.title.<row id>`), the two reasons, the four left-out reasons. Plain English, no jargon (rule 7).
- `docs/categories/dev-caches-npm.md`, `-yarn.md`, `-homebrew.md`, `-cargo.md`, `-cocoapods.md`, `-gradle.md`,
  `-pip.md`, `-go-build.md`, `-xcode-derived-data.md` (new) — in the catalogue's format (`**Danger level:** Low` or
  `Safe` · `**Rule:** none (the module's code, `modules/dev-caches`)` · `**Since:** 0.1.0`): what it is; where it lives
  (the default place, and that a cache moved by an environment variable or the tool's config is not found); the
  tool's sign; what happens if removed and whether it comes back (the texts); how the app decides (the decision table
  of the design's section 3, what is never offered beside it, the Trash rather than the tool's command).
  `docs/categories/README.md`: their nine rows.
- `docs/dependencies.md` — the new crate's rows, as for `fresher-module-temp`.

## Produces

`DevCachesModule::new()`, `Tools` with its interests and roles, and every text, for 02/02, 02/03 and 03/03.

## Checks

- `tools.rs`, negatives first: a table made for each `ToolsError` variant is refused with it; then the shipped table
  is read: ten rows, every category of `CATEGORIES` used, twelve interests (ten whole, two items: `Homebrew`,
  `.cargo/registry`), each role as the design says.
- `tests/contract.rs`: `assert_module_contract(&module, &Catalogue::Required(docs/categories))` passes (the nine files,
  their dangers, every text key the descriptor implies).
- `cargo nextest run -p fresher-module-dev-caches`; clippy; the crate guard and the dependency guard (no other module,
  `#![forbid(unsafe_code)]`); `cargo deny check`, `cargo machete`.

## Conclusions

Built (9d6cf7f): the crate `modules/dev-caches` (`fresher-module-dev-caches`, no root `Cargo.toml` edit: the members
are the glob `modules/*`), `tools.toml` with the ten rows of the design in order and a head comment on every field,
`Tools::shipped()` and `Tools::parse(text, categories)`, the nine categories (seven Low, two Safe) in the design's
order, `Tools::interests()` (one whole claim per row, one item claim with the held sign per distinct sign place: twelve
interests) with `InterestMap { interests, roles }`, the descriptor (group Development, no setting, the four left-out
reasons in the order `not-yours`, `special`, `app-open`, `no-system`), all 63 texts of the design in `locales/en.json`,
the nine `docs/categories/dev-caches-*.md` and their rows, the crate in `docs/dependencies.md`'s used-by columns, and
`tests/contract.rs` with `Catalogue::Required` (created here, extended in 02/03). `on_match` and `finish` do nothing
yet.

Deviations and calls:

- `ToolsError::DuplicateSign { id, name }` is added to the brief's list: `Interest::holding` accepts a duplicate name
  (a minor parked in 01/01), so the table refuses a sign that names one name twice instead of passing it on.
- `InterestMap::holder_of(row)` (a row to its sign holder's index) is added for 02/02, which needs the holder's match.
- The descriptor lists the categories in the design's order, not sorted by id.
- `proptest` and `serde_json` are not declared here but in the steps that first use them (F9); `docs/dependencies.md`
  has no per-crate rows, so the crate is added to the used-by column of `serde`, `thiserror` and `toml` (F14).
- The categories' "moved by" sentences name the variables the tools document; CocoaPods and Xcode are described
  generically.

Tests: `tools.rs` refusals first (every variant by a table made for it, including a sign prefix that is not a path
prefix and 65 signs), then the shipped table pinned field by field and its twelve interests; the descriptor's tests;
the contract test. RED against stubs, then `cargo nextest run -p fresher-module-dev-caches`: 30 passed; clippy,
`cargo machete`, `cargo deny` and the crate guard clean. Parked minors: `ToolsError::BundleId` lacks the row id;
overlapping places (a row nested in another, a path equal to another's `sign-at`) are not refused.
