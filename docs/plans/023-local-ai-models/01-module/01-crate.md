# 023/01/01 — The crate, the table and the texts

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, docs

The module's frame: its descriptor, its table and its interests, every text and the catalogue's three files. Nothing
is decided yet: `on_match` and `finish` return `Ok(())` until 01/02. The design: [`../00-design.md`](../00-design.md),
sections 1, 2, 5 and 6.

## Files

- `modules/local-ai-models/Cargo.toml` (new) — `fresher-module-local-ai-models`, description "The local AI models
  module of Fresher: the models Hugging Face and LM Studio downloaded."; the workspace's `version`, `edition`,
  `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`, `serde`, `thiserror`,
  `toml`; dev-dependency `fresher-testing` (the others in the steps that first use them). The workspace's members are
  the glob `modules/*`: no root edit.
- `modules/local-ai-models/stores.toml` (new) — the three rows of the design's section 1, in its order
  (`hugging-face`, `lm-studio`, `lm-studio-old`), with a head comment saying what each field means (`id`, `tool`,
  `folder`, `path`, `shape`, `sign`, `app`, `category` or `[[store.kind]]` with `kind` and `category`) and that
  LM Studio's bundle id and older place are "to verify" in 03/02.
- `modules/local-ai-models/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 3]`:
  `local-ai-models.hugging-face-models` Low, `.hugging-face-datasets` Low, `.lm-studio` Medium.
- `modules/local-ai-models/src/stores.rs` (new) — `Stores::shipped()` (`include_str!("../stores.toml")`) and
  `Stores::parse(text, categories) -> Result<Stores, StoresError>` with every refusal of the design's section 1, one
  `StoresError` variant each (`Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`, `CategoryWithoutRow`,
  `NotRelative`, `NoStar`, `ShapeMismatch`, `KindsMissing`, `KindsNotAllowed`, `DuplicateKind`, `NotAName`,
  `BundleId`); a `Store` row (`id`, `tool: Tool { HuggingFace, LmStudio }`, `folder: KnownFolder`, `path`,
  `shape: Shape { HubRepo, PublisherModel }`, `sign: Vec<String>`, `app: Option<BundleId>`, `categories:
  Categories` — one category, or the kinds' map), read with `deny_unknown_fields` and kebab-case names.
  `Stores::interests()` builds, in the table's order, one `Under(folder, path)` `Claim::Whole` per row,
  `.holding(sign)` when the sign is not empty.
- `modules/local-ai-models/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what
  it never does (no tool command, no Ollama, default places only, no file read); `LocalAiModelsError` (`Id`,
  `TextKey`, `Descriptor`, `Interest`, `Stores`); `LocalAiModelsModule::new()`: `ModuleId` `local-ai-models`,
  `ModuleGroup::Development`, the three categories, no setting, the dictionary, `.with_problems(…)` with the four
  left-out keys in the order `not-yours`, `special`, `app-open`, `no-system`; `interests()` from `Stores`;
  `on_match` and `finish` return `Ok(())`.
- `modules/local-ai-models/locales/en.json` (new) — every text of the design's section 6: the three categories' five
  texts each, the switch's label and description, the title, the two reasons, the four left-out reasons. Plain
  English, no jargon (rule 7).
- `modules/local-ai-models/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(…))`
  (extended in 01/03 with the fixture runs).
- `docs/categories/local-ai-models-hugging-face-models.md`, `-hugging-face-datasets.md`, `-lm-studio.md` (new) — in
  the catalogue's format (`**Danger level:** Low` or `Medium` · `**Rule:** none (the module's code,
  `modules/local-ai-models`)` · `**Since:** 0.1.0`): what it is; where it lives (the default place, and that a store
  moved by `HF_HOME`, `HF_HUB_CACHE` or LM Studio's settings is not found); the sign; what happens if removed and
  whether it comes back (the texts, the links into the cache, an imported model); how the app decides (the design's
  section 4 table, what is never claimed beside it, the Trash rather than a tool command). `docs/categories/README.md`:
  their three rows; the danger table's examples gain "a downloaded AI model" (Low) if it fits.
- `docs/dependencies.md` — the new crate in the used-by columns of `serde`, `thiserror`, `toml` and `fresher-testing`'s
  rows, as for `fresher-module-dev-caches`.

## Produces

`LocalAiModelsModule::new()`, `Stores` with its interests, and every text, for 01/02, 01/03 and 02/03.

## Checks

- `stores.rs`, negatives first: a table made for each `StoresError` variant is refused with it; then the shipped table
  is read: three rows, every category of `CATEGORIES` used, three whole interests, the hub's holding `blobs` and
  `snapshots`, LM Studio's two with no holding and the app's bundle id.
- The descriptor's tests: the group, the categories in order with their dangers, the four problems.
- `tests/contract.rs` passes.
- `cargo nextest run -p fresher-module-local-ai-models`; clippy; fmt; the crate guard and the dependency guard (no
  other module, `#![forbid(unsafe_code)]`); `cargo deny check`, `cargo machete`.

## Conclusions

Built as written: the crate `fresher-module-local-ai-models` (group Development), `stores.toml` with its refusals, three
whole-claim interests, every text, the three category files. The table is checked when the module is built.
Review notes carried to 01/02: the refusal variants of the table (a publisher row with no category, a hub row with
a category) and the `stores()` accessor's dead-code attribute, both fixed there.
