# 020/03/01 — The crate, the tables and the texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** modules, docs

The module's frame: its descriptor, its three tables and its two interests, its setting, every text and the
catalogue's five files. Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 03/02. This step creates
`src/lib.rs`, `locales/en.json` and `tests/contract.rs`; 03/02 and 03/03 extend them. The design:
[`../00-design.md`](../00-design.md), sections 3.1 to 3.3 and 3.5.

## Files

- `modules/downloads/Cargo.toml` (new) — `fresher-module-downloads`, description "The downloads module of Fresher:
  unfinished downloads, installers of installed apps, old macOS installers and big files left in Downloads."; the
  workspace's `version`, `edition`, `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies
  `fresher-core`, `serde`, `thiserror`, `toml`; dev-dependencies only what this step's tests use (`fresher-testing`
  for the contract test); `proptest` waits for 03/02 (`cargo machete`). Root `Cargo.toml`: the member.
- `modules/downloads/unfinished.toml` (new) — the five rows of the design's section 3.3, in its order (`.crdownload`,
  `.part`, `.opdownload`, `.partial`, `.download`), with a head comment: what each field means, that `apps` is checked
  running (a wrong id only weakens that guard: the days still hold), that the ids and `.partial`'s writer are to be
  verified (05/02), and that `.part`'s placeholder is the empty file of the final name.
- `modules/downloads/installers.toml` (new) — `words`, as in the design, with a head comment: a name's rest may hold
  only versions, plain numbers and these words, so `notes-backup.dmg` is never taken for an installer.
- `modules/downloads/macos.toml` (new) — the macOS installers from `Install macOS High Sierra.app` (10.13), `Mojave`
  (10.14), `Catalina` (10.15), the one of macOS 11 (11.0), `Monterey` (12.0), `Ventura` (13.0), `Sonoma` (14.0),
  `Sequoia` (15.0), `Tahoe` (26.0), with a head comment: names added from Apple's list in 05/02 (to verify the names
  after Tahoe and the exact bundle names); a missing name only keeps its installer. macOS 11's name is a word the
  spell checker refuses: this step adds it to `_typos.toml`'s allowed words.
- `modules/downloads/src/categories.rs` (new) — `pub(crate) const CATEGORIES: [(&str, DangerLevel); 5]`:
  `downloads.unfinished` Low, `.safari-resumable` Medium, `.installers` Low, `.macos-installers` Low,
  `.large-untouched` High.
- `modules/downloads/src/unfinished.rs` (new) — `Suffixes::shipped()` (`include_str!("../unfinished.toml")`),
  `Suffixes::parse(text) -> Result<Suffixes, SuffixesError>` with one variant per refusal of the design's section 3.3
  (`Format`, `Empty`, `BadSuffix`, `DuplicateSuffix`, `NoApp`, `BadApp`, `PlaceholderOnBundle`, `BundleRows`); a `Row`
  (`suffix` lowercased, `kind: Form { File, Bundle }`, `apps: Vec<BundleId>`, `placeholder: bool`) read with
  `deny_unknown_fields`; `Suffixes::bundle_suffix()`.
- `modules/downloads/src/installers.rs` (new) — `Words::shipped()`, `Words::parse(text) -> Result<Words,
  WordsError>` (`Format`, `Empty`, `BadWord`, `Duplicate`), `Words::holds(&str) -> bool`.
- `modules/downloads/src/macos.rs` (new) — `MacosInstallers::shipped()`, `parse(text) -> Result<_, MacosError>`
  (`Format`, `Empty`, `BadName`, `Duplicate`, `BadVersion`), `MacosInstallers::version_of(name) ->
  Option<SystemVersion>` (case ignored).
- `modules/downloads/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it offers and what it
  never does (read Downloads without access, below the first level, mount, expand or run anything, claim a `.dmg` or
  `.pkg` no installed app or receipt proves); `DownloadsError` (`Id`, `TextKey`, `Descriptor`, `Interest`, `Setting`,
  `Suffixes`, `Words`, `Macos`); `const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 3650, 30)`;
  `DownloadsModule::new()`: `ModuleId` `downloads`, `ModuleGroup::Files`, the five categories each
  `.with_needs(vec![Need::required(Capability::Folder(GuardedFolder::Downloads))])`, the setting, the dictionary,
  `.with_problems(…)` with the nine left-out keys in the design's order (`no-system`, `apps-unread`, `browser-open`,
  `recent`, `not-older`, `newer-macos`, `scan-again`, `special`, `not-yours`); `interests()`: interest 0
  `Under(Downloads, "*")` `Claim::Item`, interest 1 `Under(Downloads, "*")` `Claim::Whole`
  `.only_suffixes(vec![".download".into(), ".app".into()])` `.holding(vec!["Info.plist".into(), "Contents".into()])`;
  `on_match` and `finish` return `Ok(())`.
- `modules/downloads/locales/en.json` (new) — every text of the design's section 3.5: the five categories' five texts
  each (`downloads.<name>.{name,what,made-by,if-removed,comes-back}`), the switch's and the setting's label and
  description, `downloads.title`, the reasons (`downloads.reason.{unfinished,placeholder,safari-resumable,installer,
  installer-no-version,installer-receipt,macos-installer,large-untouched}`), the nine left-out reasons. Plain English,
  no jargon (rule 7).
- `modules/downloads/tests/contract.rs` (new) — `assert_module_contract(&module,
  &Catalogue::Required(docs/categories))`.
- `docs/categories/downloads-unfinished.md`, `-safari-resumable.md`, `-installers.md`, `-macos-installers.md`,
  `-large-untouched.md` (new) — in the catalogue's format (`**Danger level:** Low`, `Medium`, `Low`, `Low`, `High` ·
  `**Rule:** none (the module's code, `modules/downloads`)` · `**Since:** 0.1.0`): what it is; where it lives
  (`~/Downloads`, first level, the suffixes or names); what happens if removed and whether it comes back (the texts);
  how the app decides (the design's section 3.4 for that category, what is never claimed, the access it needs, the
  Trash). `docs/categories/README.md`: their five rows.
- `docs/dependencies.md` — the new crate's rows, as for `fresher-module-app-leftovers`.
- `_typos.toml` — macOS 11's name among the allowed words (with a comment: a macOS version's name).

## Produces

`DownloadsModule::new()`, `Suffixes`, `Words`, `MacosInstallers`, the interests and every text, for 03/02, 03/03 and
04/03.

## Checks

- `unfinished.rs`, negatives first: a table made for each `SuffixesError` variant is refused with it; then
  `the_shipped_table_has_five_rows_and_one_bundle` (`.part` alone with a placeholder, `.download` the bundle).
- `installers.rs`: each `WordsError`; then `the_shipped_words_hold_installer_and_arm64` and not `backup`.
- `macos.rs`: each `MacosError` (`Install Sonoma.app`, `Install macOS Sonoma` without `.app`, a duplicate in another
  case, `version = "14"` and `version = "fourteen"`, neither `<major>.<minor>`); then
  `install_macos_sonoma_is_version_14_in_any_case`.
- `lib.rs` tests: `every_category_needs_downloads`, `the_interests_are_one_item_star_and_one_whole_suffixed_star`.
- `tests/contract.rs` passes (the five files, their dangers, every text key the descriptor implies).
- `cargo nextest run -p fresher-module-downloads`; clippy; fmt; the crate guard and the dependency guard (no other
  module, `#![forbid(unsafe_code)]`); `npm run check:deny`; `npm run check:machete`.

## Conclusions

Done as written. Notes: `macos.toml` is checked against `<major>.<minor>` before `SystemVersion::parse` (so `14` and
`14.0.1` are refused); interest 1's suffixes are built from the table's `bundle_suffix()` plus `.app`, not a second
`.download` literal; the dependency on `tracing` for 03/02's logging is added in 03/02, where it is first used
(`cargo machete`). The tables' accessors and the decision's inputs carry `expect(dead_code)` until 03/02 uses them.
