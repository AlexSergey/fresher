# 021/02/01 — The crate, the table and the texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules, docs

The module's frame: its descriptor, its table, its interests, its setting, every text and the catalogue's four files.
Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 02/02. This step creates `src/lib.rs`,
`locales/en.json` and `tests/contract.rs`; 02/02 and 02/03 extend them. The design:
[`../00-design.md`](../00-design.md), sections 3.1, 3.3 and 3.5 (agent's calls 6 and 12).

## Files

- `modules/stale-projects/Cargo.toml` (new) — `fresher-module-stale-projects`, description "The stale projects module
  of Fresher: the installed packages and build output of projects nobody changed for a long time, and Rust's build
  cache."; the workspace's `version`, `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`;
  dependencies `fresher-core`, `serde`, `thiserror`, `toml`; dev-dependencies only what this step's tests use
  (`fresher-testing` for the contract test). Root `Cargo.toml`: the member.
- `modules/stale-projects/artifacts.toml` (new) — the ten rows of the design's section 3.3, in the core's order
  (`ProjectKind::ALL`, then each kind's `artifacts()`), with a head comment: what each field means; that every artifact
  of the core needs its row and a row the core does not list is refused; that `command` is what the reason names, not
  something the app runs; that framework output (`dist`, `.next`, …) is never a row (the user's decision of
  2026-11-19).
- `modules/stale-projects/src/categories.rs` (new) — `pub(crate) enum Category { Dependencies, BuildOutput,
  PythonEnvironments, RustBuildCache }` with `ALL`, `name()` (`dependencies`, `build-output`, `python-environments`,
  `rust-build-cache`: the start of a finding's identity), `id()` (`stale-projects.<name>`) and `danger()`
  (`PythonEnvironments` Medium, the others Low), as `modules/downloads/src/categories.rs`; `Category::of_table(name)`
  for the three a row may name (`rust-build-cache` is no row's).
- `modules/stale-projects/src/table.rs` (new) — `Artifacts::shipped()` (`include_str!("../artifacts.toml")`),
  `Artifacts::parse(text) -> Result<Artifacts, ArtifactsError>`, a `Row { kind: ProjectKind, folder: String,
  category: Category, command: String }` read with `deny_unknown_fields`; `Artifacts::row(kind, folder) ->
  Option<&Row>`. The kind's names (`node`, `rust`, `gradle`, `maven`, `python`, `swift-package`, `go`, `dart`) are the
  module's (a local `match` over `ProjectKind::ALL`; the core's enum has no names). `ArtifactsError`: `Format`
  (`#[from] toml::de::Error` kept as its message), `Empty`, `UnknownKind { kind }`, `UnknownCategory { category }`,
  `NotAnArtifact { kind, folder }` (not in the kind's `artifacts()`), `Duplicate { kind, folder }`, `Missing { kind,
  folder }` (an artifact of the core with no row), `BadCommand { kind, folder }` (empty, more than one line, or a
  character outside printable ASCII).
- `modules/stale-projects/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what it
  never does (offer source, the project folder, `.git` or framework output; run a tool; claim anything whole; look at a
  project in `~/Library`, the apps folders, an app bundle, a hidden folder of the home or the home itself);
  `StaleProjectsError` (`Id`, `TextKey`, `Descriptor`, `Interest`, `Setting`, `Artifacts`); `const UNUSED_FOR:
  DaySetting = DaySetting::new("unused-for-days", 30, 3650, 365);` (comment: never fewer than a month);
  `const CACHE_UNUSED_DAYS: u32 = 7;` (comment: the user's decision of 2026-11-19, not a setting);
  `StaleProjectsModule::new()`: `ModuleId` `stale-projects`, `ModuleGroup::Development`, the four categories (no
  needs), the setting, the dictionary, `.with_problems(…)` with the four left-out keys in the design's order
  (`scan-again`, `unread`, `recent`, `cache-recent`), the table (a bad one fails here); `interests()`: one
  `Interest::new(Pattern::FolderName(name), Claim::Item, Vec::new())` per name of the set (a `BTreeSet`, so in name
  order) of every kind's `artifacts()` and of every kind's `caches()` last segments — ten names, none hard-coded;
  `on_match` and `finish` return `Ok(())`.
- `modules/stale-projects/locales/en.json` (new) — every text of the design's section 3.5: the four categories' five
  texts each (`stale-projects.<name>.{name,what,made-by,if-removed,comes-back}`), the switch's and the setting's label
  and description, `stale-projects.title`, the four reasons (`stale-projects.reason.<category name>`), the four
  left-out reasons. Plain English (rule 7).
- `modules/stale-projects/tests/contract.rs` (new) — `assert_module_contract(&module,
  &Catalogue::Required(docs/categories))`.
- `docs/categories/stale-projects-dependencies.md`, `-build-output.md`, `-python-environments.md`,
  `-rust-build-cache.md` (new) — in the catalogue's format (`**Danger level:** Low`, `Low`, `Medium`, `Low` ·
  `**Rule:** none (the module's code, `modules/stale-projects`)` · `**Since:** 0.1.0`): what it is; where it lives
  (beside a project's marker file, the kinds and folders of the table; the cache paths); what happens if removed and
  whether it comes back (the texts); how the app decides (the design's sections 3.2 and 3.4 for that category: which
  projects are left out, the last activity and what it does not see — a file edited in place, a removed artifact
  making the project look active —, the days, the Trash; for the cache, the 7 days and that a project built every day
  keeps it). `docs/categories/README.md`: their four rows.
- `docs/dependencies.md` — the new crate's rows, as for `fresher-module-downloads`.

## Produces

`StaleProjectsModule::new()`, `Artifacts`, `Category`, the interests and every text, for 02/02, 02/03 and 03/03.

## Checks

- `table.rs`, negatives first: a table made for each `ArtifactsError` variant is refused with it (`kind = "cobol"`,
  `category = "rust-build-cache"`, `folder = "dist"` for node, a duplicate `node_modules`, the shipped table without
  its `venv` row, `command = ""` and `"npm\ninstall"`); then `the_shipped_table_has_a_row_for_every_artifact_of_the_core`
  and `python_environments_are_the_only_medium_rows` (through `Category::danger`).
- `lib.rs` tests: `no_interest_claims_anything_whole`, `the_interests_are_every_artifact_and_cache_name_once`
  (`node_modules`, `target`, `build`, `.gradle`, `.venv`, `venv`, `.build`, `.dart_tool`, `incremental`,
  `llvm-cov-target`), `no_category_needs_a_capability`.
- `tests/contract.rs` passes (the four files, their dangers, every text key the descriptor implies).
- `cargo nextest run -p fresher-module-stale-projects`; clippy; fmt; the crate guard and the dependency guard
  (`npm run check:crate-deps`: only `fresher-core`, `fresher-testing` as a dev-dependency, `#![forbid(unsafe_code)]`);
  `npm run check:deny`; `npm run check:machete`.

## Conclusions

The crate frame, the ten-row table, the categories, every text and the four catalogue files are in place; `on_match` and
`finish` do nothing until 02/02. `ArtifactsError::Format` holds the message as a `String`, as in `downloads`. A test
keeps `dist`, `.next`, `.nuxt`, `.turbo` and `.parcel-cache` out of the interests, the table, the core's artifacts and
the cache segments.
