# 035/01/01 — The core's types, kinds, texts and setting

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core

Everything the engine and the UI will exchange for the Largest screen, in `fresher-core`, with no IO: the kinds table,
the query and answer types, the core's own category `core.own-file`, one setting, one refusal and their texts. Nothing
uses them yet; the workspace stays green. The design: [`../00-design.md`](../00-design.md), sections 2, 3, 4, 5 and 7.

Read first: `CLAUDE.md`, `docs/rules.md`, `crates/core/src/disk_state.rs` (`NodeInfo`, `NodeId`, `NodeKind`),
`crates/core/src/category.rs` (`CategoryDescriptor`), `crates/core/src/ids.rs` (`ModuleId::system()`, `CategoryId`),
`crates/core/src/error.rs` (how a `CommandError` gets its code and text), `crates/core/src/settings/core_schema.rs`
(the motion choice is the model of a core choice setting), `crates/core/src/texts.rs` (the test of every key the core
emits).

## Files

- `crates/core/src/largest/mod.rs` (new) — the module's docs and re-exports; `pub mod kind; pub mod page;` or plain
  files, as the crate does elsewhere. Exported from `crates/core/src/lib.rs` (`pub use largest::{…}`).
- `crates/core/src/largest/kind.rs` (new):
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
    #[serde(rename_all = "kebab-case")] pub enum ItemKind { DiskImage, Archive, Installer, Video, Audio, Image,
    VirtualMachine, MediaLibrary, Project, MemoryDump, Document, DeviceBackup, File, Folder }` with `ALL` and
    `text_key()` → `core.largest.kind.<kebab>`.
  - two `const` tables, `FILE_KINDS: &[(&str, ItemKind)]` and `PACKAGE_KINDS: &[(&str, ItemKind)]`, exactly the
    extensions of the design's section 2 (lowercase, without the dot);
  - `pub fn kind_of(name: &str, folder: bool) -> ItemKind`: the last extension, case ignored, looked up in the table of
    the node's kind; a file not in it is `File`, a folder not in it is `Folder`. `Foo.tar.gz` is an `Archive` (by
    `gz`);
  - `pub fn is_package(name: &str) -> bool`: a folder name whose extension is in `PACKAGE_KINDS`;
  - `pub fn is_app_bundle(name: &str) -> bool`: the extension `app`, case ignored;
  - `pub fn archive_stem(name: &str) -> Option<&str>`: for a file whose kind is `Archive`, the name without its archive
    extensions (`Foo.zip` → `Foo`, `Foo.tar.gz` and `Foo.tgz` → `Foo`); `None` for anything else or an empty stem.
- `crates/core/src/largest/page.rs` (new), every type `Serialize` + `TS` (camelCase, as `NodeInfo`); the inputs also
  `Deserialize` with `deny_unknown_fields`:
  - `pub enum Unchanged { Any, Month, HalfYear, Year, TwoYears }` (kebab-case) with `ALL`, `text_key()` →
    `core.largest.unchanged.<kebab>` and `pub fn before(self, taken: Timestamp) -> Option<Timestamp>` (30, 182, 365 and
    730 days of 86 400 s back from `taken`; `None` for `Any`; saturating);
  - `pub struct LargestQuery { pub kind: Option<ItemKind>, pub place: Option<NodeId>, pub unchanged: Unchanged, pub
    offset: u32, pub limit: u32 }` and `pub const MAX_PAGE: u32 = 100` (a larger limit is capped by the engine);
  - `pub enum Withheld { Explained, Library, Trash, Hidden, App, Link, NotSeen, StandardFolder, Protected,
    HoldsExplained, ReadInPart, Ignored }` (kebab-case) with `ALL` and `text_key()` → `core.largest.withheld.<kebab>`;
  - `pub enum Offer { Offered, Withheld(Withheld) }` serialized `#[serde(tag = "kind", content = "reason",
    rename_all = "kebab-case")]`;
  - `pub struct LargestRow { pub node: NodeInfo, pub place: String, pub kind: ItemKind, pub offer: Offer, pub
    unpacked: Option<String>, pub picked: Option<FindingId> }`;
  - `pub struct KindCount { pub kind: ItemKind, pub count: u32 }`, `pub struct PlaceCount { pub folder: NodeInfo, pub
    count: u32 }`;
  - `pub struct LargestPage { pub rows: Vec<LargestRow>, pub offset: u32, pub total: u32, pub sizes: Sizes, pub kinds:
    Vec<KindCount>, pub places: Vec<PlaceCount>, pub truncated: bool, pub min_size: ByteSize }`;
  - `pub struct OwnRow { pub node: NodeInfo, pub explained: Sizes, pub kind: ItemKind, pub offer: Offer, pub opens:
    bool, pub picked: Option<FindingId> }`;
  - `pub struct OwnPlace { pub path: Vec<NodeInfo>, pub rows: Vec<OwnRow>, pub offset: u32, pub total: u32 }` (`path`:
    the home first, the opened folder last).
  - `pub const MAX_ITEMS: u32 = 10_000;` (the most items kept per state, design section 1).
- `crates/core/src/largest/own_file.rs` (new) — the core's own category: `pub fn own_file() -> CategoryDescriptor`
  with id `core.own-file` of `ModuleId::system()` and `DangerLevel::High`, and `pub const OWN_FILE_ID: &str =
  "core.own-file"`. If `CategoryId::new` refuses the core's namespace, add the narrowest constructor that admits exactly
  this id (documented, tested), never a general bypass.
- `crates/core/src/error.rs` — `CommandError::NotOwnItem` (code `not-own-item`), in every list the file keeps (the
  code, the refusal kind, the sample of each error in its tests).
- `crates/core/src/settings/core_schema.rs` — `pub const LARGEST_MIN_SIZE: &str = "core.largest.min-size";` a
  `SettingKind::Choice { options: vec!["10-mb", "50-mb", "100-mb", "500-mb", "1-gb"] }`, default `"100-mb"`, group
  `Scan`, effect `Now`, placed after `core.scan.roots`; and `pub fn min_size_of(choice: &str) -> ByteSize` (decimal:
  10 000 000, 50 000 000, 100 000 000, 500 000 000, 1 000 000 000; an unknown choice is the default's). Update the
  schema test that lists the core's keys (`crates/core/src/settings/schema.rs`, near `core.scan.roots`).
- `crates/core/locales/en.json` — every new key, in the file's sorted order, the design's section 7 texts:
  `core.own-file.{name,what,made-by,if-removed,comes-back,title,reason.file,reason.folder}`,
  `core.largest.kind.*` (14), `core.largest.withheld.*` (12), `core.largest.unchanged.*` (5),
  `core.largest.unpacked` (arg `folder`), `core.largest.min-size.{label,description}`,
  `core.largest.min-size.options.{10-mb,50-mb,100-mb,500-mb,1-gb}` ("10 MB", "50 MB", "100 MB", "500 MB", "1 GB"),
  `core.error.not-own-item`. Plain English, rule 7; no `{detail}`.
- `crates/core/src/texts.rs` — `core_keys()` builds the new keys from the enums (`ItemKind::ALL`, `Withheld::ALL`,
  `Unchanged::ALL`) and the own-file keys, so a missing or an extra text fails.

## Produces

The types and functions 01/02, 01/03 and 02/01 use; the texts the UI renders.

## Checks

- `kind.rs`, `mod negative` first: `a_name_without_an_extension_is_a_plain_file`, `a_folder_not_in_the_table_is_a_plain_folder`,
  `a_file_extension_does_not_make_a_folder_a_package` (`Foo.zip` as a folder is `Folder`), `an_app_bundle_has_no_kind_of_its_own`
  (`Editor.app` as a folder is `Folder`, and `is_app_bundle`), `a_non_archive_has_no_stem`, `an_archive_named_only_by_its_extension_has_no_stem`
  (`.zip`); `mod positive`: `extensions_are_read_case_insensitively` (`MOVIE.MOV`), `a_double_extension_archive_is_an_archive`,
  `a_virtual_machine_package_is_a_package`, `every_table_entry_is_lowercase_and_unique`, `the_stem_drops_every_archive_extension`.
- `page.rs`: `mod negative` `an_unknown_field_of_a_query_is_refused`, `any_has_no_cutoff`, `a_cutoff_saturates_at_the_epoch`;
  `mod positive` `unchanged_counts_back_from_the_scan_s_time`, `an_offer_serializes_with_its_reason`
  (`{"kind":"withheld","reason":"standard-folder"}`), `a_row_serializes_in_camel_case`, `every_withheld_reason_has_its_text`.
- `own_file.rs`: `the_own_file_category_is_the_core_s_and_high`.
- `core_schema.rs`: `an_unknown_min_size_choice_is_the_default`, then `the_min_size_choices_are_decimal`.
- `texts.rs`: the existing test of every key passes with the new keys; `error.rs`: every error has its text.
- `cargo nextest run -p fresher-core`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`.
  If the bindings test asks for it (the types are not reachable from a command yet, so it may not), regenerate
  `app/ui/src/ipc/bindings.generated.ts` as its failure message says.
- Commit: `feat(core): the largest files' kinds, query types, own-file category and setting`.

## Conclusions

- Built as designed: `crates/core/src/largest/` (kinds, page and row types, the `Withheld` reasons, the own-file
  helpers), the setting `core.largest.min-size`, the refusal `CommandError::NotOwnItem` (`not-own-item`), 48 texts
  (`core.largest.*`, `core.own-file.*`) and the category id `core.own-file` (`CategoryId::own_file()`, crate-private).
- `TextKey::new` had to accept a segment that starts with a digit after the namespace, for the options `10-mb` and
  `1-gb`. A reviewer found nothing keyed off the widening; a later step (04/01) narrowed it to a number, a hyphen and
  a kebab word, so `1npm` is still refused.
- Core 474 tests passed at this step; the bindings were regenerated, additions only.
