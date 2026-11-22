# 031/01/01 — The crate, the table and the texts

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, docs

The module's frame: its descriptor, its table of places and its interests, every text and the catalogue's four files.
Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 01/02. The design:
[`../00-design.md`](../00-design.md), sections 2, 5 and 6. Read first, as the shape to follow (never depend on it: a
module never depends on another): `modules/app-caches/` (`Cargo.toml`, `places.toml`, `src/places.rs`,
`src/places/tests.rs`, `src/categories.rs`, `src/lib.rs`, `locales/en.json`, `tests/contract.rs`) and
`docs/categories/app-caches-library-caches.md`.

## Files

- `modules/xcode/Cargo.toml` (new) — `fresher-module-xcode`, description "The Xcode module of Fresher: device support
  files, previews, simulator caches and archives Xcode keeps in the user's Library."; the workspace's `version`,
  `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`, `serde`,
  `thiserror`, `toml`; dev-dependency `fresher-testing` (others in the steps that first use them). The workspace's
  members are the glob `modules/*`: no root edit.
- `modules/xcode/places.toml` (new) — the seven rows of the design's section 2, in that order, each with `id`,
  `category`, `folder = "library"`, `path`, `shape` (`device-support`, `fixed` or `archive`), `platform` (only on the
  four device support rows: `iOS`, `watchOS`, `tvOS`, `visionOS`), `sign` (a list of names) and `removal` (`trash` or
  `empty-folder`). A head comment says what each field means, that the paths and signs of device support, previews and
  archives are "to verify" on a Mac that has them (plan 031, 03/02), and that a wrong path or sign claims nothing.
- `modules/xcode/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 4]`:
  `xcode.device-support` Low, `xcode.previews` Safe, `xcode.simulator-caches` Safe, `xcode.archives` High.
- `modules/xcode/src/places.rs` (new) and `src/places/tests.rs` — `Places::shipped()` (`include_str!("../places.toml")`)
  and `Places::parse(text, categories) -> Result<Places, PlacesError>` with every refusal of the design's section 2,
  one variant each: `Format`, `Empty`, `BadId`, `DuplicateId`, `UnknownCategory`, `CategoryWithoutRow`, `WalkedPlace`,
  `BadPath`, `ShapeAndPath`, `Platform`, `NoSign`, `DuplicatePlace`. A `Row` (`id`, `category`, `folder: KnownFolder`,
  `path`, `shape: Shape`, `platform: Option<String>`, `sign: Vec<String>`, `removal: RowRemoval`), with
  `enum Shape { DeviceSupport, Fixed, Archive }` and `enum RowRemoval { Trash, EmptyFolder }` (kebab case in TOML).
  `Places::interests()` builds, in the table's order, one `Interest::new(Pattern::Under(folder, path), Claim::Whole,
  vec![])` `.holding(sign)` per row, and for an `Archive` row also `.only_suffixes(vec![".xcarchive".into()])`.
  `Places::rows()` gives the rows by interest index.
- `modules/xcode/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what it never
  does (no simulator device or runtime, no DerivedData — `dev-caches`' —, nothing while Xcode or the Simulator runs, no
  folder whose name or sign it does not recognise; it runs no command); `XcodeError` (`Id`, `TextKey`, `Descriptor`,
  `Interest`, `Places`) and `PlacesError` re-exported; `XcodeModule::new()`: `ModuleId` `xcode`,
  `ModuleGroup::Development`, the four categories, no setting, the dictionary, `.with_problems(…)` with the four
  left-out keys in the order `special`, `not-yours`, `no-system`, `app-open`; `interests()` from `Places`; `on_match`
  and `finish` return `Ok(())`.
- `modules/xcode/locales/en.json` (new) — every text of the design's section 6, word for word as a start: the five
  texts of each of the four categories (`name`, `what`, `made-by`, `if-removed`, `comes-back`), the switch's `label`
  and `description` (`xcode.enabled.*`), the four titles, the five reasons, the four left-out reasons. Plain English,
  no jargon (rule 7).
- `modules/xcode/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(…))` (extended in
  01/03 with the fixture runs).
- `docs/categories/xcode-device-support.md`, `xcode-previews.md`, `xcode-simulator-caches.md`, `xcode-archives.md`
  (new) — in the catalogue's format (`**Danger level:** <level>` · `**Rule:** none (the module's code,
  `modules/xcode`)` · `**Since:** 0.1.0`): what it is; where it lives (the rows' paths under `~/Library/`, one finding
  per folder, the name's shape); what happens if removed (Trash or emptied; how it comes back); how the module decides
  (the design's section 4 table; kept while Xcode or the Simulator runs; never: a wrong shape, a missing sign, a link,
  Xcode's settings). The archives' file says plainly why it is High and that nothing makes an archive again. The
  simulator caches' file names the fact verified on the developer's Mac (one folder per macOS build under `dyld`).
- `docs/categories/README.md` — the four rows; in the danger table, "Xcode archives" among the High examples and "old
  device support files" among the Low ones (keep "old iOS simulators" among Medium: still the deferred devices).
- `docs/dependencies.md` — the new crate in the used-by columns of `serde`, `thiserror`, `toml` and `fresher-testing`,
  as for `fresher-module-app-caches`.

## Produces

`XcodeModule::new()`, `Places` with its rows and interests, and every text, for 01/02, 01/03 and 02/03.

## Checks

- `places/tests.rs`, `mod negative` first: a table made for each `PlacesError` variant is refused with it (among them a
  `fixed` row with a `*`, a `device-support` row ending in a plain name, a `platform` on the `previews` row, a device
  support row without one, `sign = []`, `path = "a/*b"`, `folder = "user-temp"`); then `mod positive`: the shipped
  table has seven rows in the design's order, uses the four categories, and builds seven whole interests under
  `Library`, the archives' one shaped to `.xcarchive`, each holding its sign.
- The descriptor's tests (in `lib.rs`): the group `Development`, the four categories with their dangers, the four
  problems in order, no setting.
- `tests/contract.rs` passes.
- `cargo nextest run -p fresher-module-xcode`; `cargo clippy -p fresher-module-xcode --all-targets -- -D warnings`;
  `cargo fmt`; the crate guard and the dependency guard (no other module, `#![forbid(unsafe_code)]`);
  `npm run check:machete`; `cargo deny check`.

## Conclusions

Built as the step says: the crate `fresher-module-xcode`, `places.toml` with its seven rows, the four categories with their
dangers, the locale file and the four category docs. 24 tests. `Places::rows` carried a `dead_code` expectation until
01/02 used it. No `fresher-testing` used-by column exists in `dependencies.md`, so only the serde, thiserror and toml
rows changed.
