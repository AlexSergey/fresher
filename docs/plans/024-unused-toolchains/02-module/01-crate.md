# 024/02/01 — The crate, the table and the texts

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, docs

The module's frame: its descriptor (two categories with their needs, the Node switch), its table and its interests,
every text and the catalogue's two files. Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 02/03.
The design: [`../00-design.md`](../00-design.md), sections 3, 4, 6 (needs), 7 and 8.

## Files

- `modules/unused-toolchains/Cargo.toml` (new) — `fresher-module-unused-toolchains`, description "The unused
  toolchain versions module of Fresher: rustup toolchains and nvm Node versions nothing points at."; the workspace's
  `version`, `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies
  `fresher-core`, `serde`, `thiserror`, `toml`; dev-dependency `fresher-testing`. The workspace's members are the glob
  `modules/*`: no root edit.
- `modules/unused-toolchains/toolchains.toml` (new) — the two rows of the design's section 3 (`rustup`, `nvm`), with a
  head comment saying what each field means (`id`, `manager`, `versions`, `shape`, `pointers` with `place` and
  `holds`, `pins` with `name` and, for `.tool-versions`, `keys`, `category`, `switch`) and that only the default
  places are read.
- `modules/unused-toolchains/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 2]`:
  `unused-toolchains.rust` Low, `unused-toolchains.node` Medium; `pub(crate) fn needs() -> Vec<Need>`: `required` on
  `Folder(Desktop)`, `Folder(Documents)`, `Folder(Downloads)`, given to both.
- `modules/unused-toolchains/src/settings.rs` (new) — `node-versions`: a `Switch`, default `false`;
  `pub(crate) fn node_versions(settings: &ModuleSettings) -> bool` (an unresolved value counts as off).
- `modules/unused-toolchains/src/toolchains.rs` (new) — `Toolchains::shipped()` (`include_str!("../toolchains.toml")`)
  and `Toolchains::parse(text, categories, settings) -> Result<Toolchains, ToolchainsError>` with every refusal of the
  design's section 3, one variant each (`Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`,
  `CategoryWithoutRow`, `NotRelative`, `NoStar`, `NotAName`, `KeysMissing`, `DuplicateKey`, `UnknownSwitch`); a `Row`
  (`id`, `manager: Manager { Rustup, Nvm }`, `versions`, `shape: Shape { RustupToolchain, NodeVersion }`,
  `pointers: Vec<PointerPlace>`, `pins: Vec<Pin { name, keys }>`, `category`, `switch: Option<&'static str>`), read
  with `deny_unknown_fields` and kebab-case names. `Toolchains::interests()` builds them in the design's section 4
  order and keeps, for each interest index, what it is (`Versions(row)`, `Pointer(row, place)`, `Pin(name)`).
- `modules/unused-toolchains/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and
  what it never does (no tool command, default places only, reads only the files its matches showed, through
  `small_file`); `UnusedToolchainsError` (`Id`, `TextKey`, `Descriptor`, `Interest`, `Setting`, `Toolchains`);
  `UnusedToolchainsModule::new()`: `ModuleId` `unused-toolchains`, `ModuleGroup::Development`, the two categories with
  their needs, the switch, the dictionary, `.with_problems(…)` with the five left-out keys in the order `special`,
  `not-yours`, `partial-scan`, `default-unknown`, `pin-unknown`; `interests()` from `Toolchains`; `on_match` and
  `finish` return `Ok(())`.
- `modules/unused-toolchains/locales/en.json` (new) — every text of the design's section 8: the two categories' five
  texts each, the two switches' labels and descriptions, the two titles, the two reasons, the five left-out reasons.
- `modules/unused-toolchains/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(…))`
  (extended in 02/04 with the fixture runs).
- `docs/categories/unused-toolchains-rust.md`, `unused-toolchains-node.md` (new) — in the catalogue's format
  (`**Danger level:** Low` or `Medium` · `**Rule:** none (the module's code, `modules/unused-toolchains`)` ·
  `**Since:** 0.1.0`): what it is; where it lives (the default place, and that `RUSTUP_HOME` or `NVM_DIR` elsewhere is
  not found); what happens if removed and how it comes back (the command, components and targets, global packages,
  an editor's path); how the app decides (what points at a version, the design's section 6 table, what is never
  offered, the needs, the Node switch off by default, the Trash rather than a tool command, a scan root narrower than
  the home). `docs/categories/README.md`: their two rows; the danger table's examples gain "an unused Rust toolchain"
  (Low) and "an unused Node version" (Medium).
- `docs/dependencies.md` — the new crate in the used-by columns of `serde`, `thiserror`, `toml` and `fresher-testing`'s
  rows, as for `fresher-module-local-ai-models`.

## Produces

`UnusedToolchainsModule::new()`, `Toolchains` with its interests and their kinds, the switch and every text, for
02/02, 02/03, 02/04 and 03/03.

## Checks

- `toolchains.rs`, negatives first: a table made for each `ToolchainsError` variant is refused with it; then the
  shipped table is read: two rows, both categories used, the interests in order (two whole versions interests, the
  `.rustup` item holding `settings.toml`, the two alias items, five marker interests with `.tool-versions` once), the
  Node row's switch.
- The descriptor's tests: the group, the categories in order with their dangers and their three required needs, the
  switch off by default, the five problems.
- `tests/contract.rs` passes.
- `cargo nextest run -p fresher-module-unused-toolchains`; clippy; fmt; the crate guard and the dependency guard (no
  other module, `#![forbid(unsafe_code)]`); `cargo deny check`, `cargo machete`.

## Conclusions

Built as written: the crate `fresher-module-unused-toolchains`, its table `toolchains.toml` (rows with inline
`pointers` and `pins`; `keys` required only for `.tool-versions` pins), the two categories and their texts, and the
settings (the Node switch). Calls taken while building: `Toolchains::parse` takes the setting names without the module
prefix; the interests carry a kind (`Versions`, `Pointer`, `Pin`) the module reads back in `on_match`; unused items
were silenced with `expect(dead_code)` until step 03 used them, and the contract test has only a positive block, as
in the module it was modelled on.
