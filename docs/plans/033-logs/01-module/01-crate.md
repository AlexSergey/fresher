# 033/01/01 — The crate, the texts and the categories

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, docs

The module's frame: its descriptor (group, two categories, one setting, five left-out keys), its six interests, every
text and the two catalogue files. Nothing is decided yet: `on_match` and `finish` return `Ok(())` until 01/02. The
design: [`../00-design.md`](../00-design.md), sections 1, 2, 6, 7 and 8. Model to follow throughout: `modules/temp`
(read `Cargo.toml`, `src/lib.rs`, `locales/en.json`, `tests/contract.rs`); a module never depends on another module.

## Files

- `modules/logs/Cargo.toml` (new) — `name = "fresher-module-logs"`, `description = "The logs module of Fresher: the
  old logs and crash reports in the user's Library."`; `version`, `edition`, `rust-version`, `publish`, `license` from
  the workspace; `[lints] workspace = true`; dependencies `fresher-core = { path = "../../crates/core" }`,
  `thiserror.workspace = true`; dev-dependency `fresher-testing = { path = "../../crates/testing" }` (the others in the
  steps that first use them). The workspace's members are the glob `modules/*`: no root edit.
- `modules/logs/src/lib.rs` (new) —
  - `#![forbid(unsafe_code)]`; a module doc: what it offers (files under `~/Library/Logs` unchanged for more than the
    setting's days, crash reports apart), what it never does (no folder, no link, nothing under `JetBrains` or a
    bundle-id-shaped top folder, no state or lock file, nothing outside `Logs`; files go to the Trash, folders stay).
  - Constants: `const APP_LOGS: &str = "logs.app-logs";`, `const CRASH_REPORTS: &str = "logs.crash-reports";`,
    `const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 90, 7);`, `const MAX_DEPTH: usize = 6;`.
  - `pub enum LogsError` (`thiserror`, `Clone, Debug, PartialEq, Eq`): `Id(#[from] IdError)`,
    `TextKey(#[from] TextKeyError)`, `Descriptor(#[from] DescriptorError)`, `Interest(#[from] InterestError)`,
    `Setting(#[from] SettingError)`, each `#[error(transparent)]` and documented as `TempError`'s are.
  - `pub struct LogsModule { descriptor, app_logs: CategoryDescriptor, crash_reports: CategoryDescriptor, interests:
    Vec<Interest>, … }` with `pub fn new() -> Result<Self, LogsError>`: `ModuleId::new("logs")`, both categories
    `DangerLevel::Safe` in the order app logs, crash reports; `ModuleGroup::System`; settings
    `vec![OLDER_THAN.descriptor(id)?]`; `Locales { en: include_str!("../locales/en.json") }`;
    `.with_problems(…)` with the five keys in this order: `logs.left-out.scan-again`, `logs.left-out.special`,
    `logs.left-out.not-yours`, `logs.left-out.locked`, `logs.left-out.recent`; the interests: for depth 1 to
    `MAX_DEPTH`, `Interest::new(Pattern::Under(KnownFolder::Logs, vec!["*"; depth].join("/")), Claim::Item, vec![])`
    (`*`, `*/*`, … six stars); the text keys `logs.title`, `logs.reason.top`, `logs.reason.folder`,
    `logs.reason.report` built once.
  - `impl Module for LogsModule`: `descriptor()`, `interests()`; `on_match` and `finish` return `Ok(())` for now.
- `modules/logs/locales/en.json` (new) — every text of the design's section 8, exactly as written there (keys:
  `logs.app-logs.{name,what,made-by,if-removed,comes-back}`, `logs.crash-reports.{name,what,made-by,if-removed,
  comes-back}`, `logs.enabled.{label,description}`, `logs.older-than-days.{label,description}`, `logs.title`,
  `logs.reason.{top,folder,report}`, `logs.left-out.{scan-again,special,not-yours,locked,recent}`). Plain English, no
  jargon (rule 7).
- `modules/logs/tests/contract.rs` (new) — `mod positive` `the_logs_module_passes_the_contract`:
  `assert_module_contract(&LogsModule::new().unwrap(), &Catalogue::Required(<repo>/docs/categories))` (extended in
  01/03 with the fixture runs).
- `docs/categories/logs-app-logs.md` and `docs/categories/logs-crash-reports.md` (new) — in the catalogue's format
  (`**Danger level:** Safe · **Rule:** none (the module's code, `modules/logs`) · **Since:** 0.1.0`): what it is;
  where it lives (`~/Library/Logs/**` up to six names deep, except `DiagnosticReports`, `JetBrains` and bundle-id
  folders, for app logs; `~/Library/Logs/DiagnosticReports/**`, `Retired/` included, for crash reports); what happens
  if removed (the file goes to the Trash, the folders stay; it does not come back, new ones are written); how the app
  decides (the design's section 4 table; one finding per file; never claimed: a folder, a link, `JetBrains/`
  — the `jetbrains` module's —, a bundle-id-shaped top folder — `app-leftovers`' —, `.plist`, `.db`, `.sqlite`,
  `.sqlite-wal`, `.sqlite-shm`, `.lock`, `.pid`, anything outside `~/Library/Logs`, the system-wide logs; the setting
  `logs.older-than-days`, 7 to 90 days, 7 by default). `docs/categories/README.md`: two rows in the catalogue table,
  in the table's order (after `jetbrains`' rows or where the module's rows fit); the Safe examples already say "logs".
- `docs/dependencies.md` — `fresher-module-logs` in the used-by column of `thiserror`'s and `fresher-testing`'s rows
  (and of `fresher-core` if the file lists it), as for `fresher-module-temp`.

## Produces

`LogsModule::new()`, `LogsError`, the six interests, the five left-out keys, every text: for 01/02, 01/03 and 02/03.

## Checks

- Unit tests in `src/lib.rs` (`#[cfg(test)] mod tests` or `src/tests.rs`, with `mod negative` first, even if it holds
  only `the_module_declares_no_other_category`, then `mod positive`): the group is `System`; the two categories with
  `Safe`, in order; the one setting `logs.older-than-days` (min 7, max 90, default 7); the five problems in order; six
  interests, each `Claim::Item`, under `KnownFolder::Logs`, paths `*` to six stars, none shaped to bundle ids or
  suffixes.
- `tests/contract.rs` passes.
- `CARGO_TARGET_DIR=/Users/serhiialeksandrov/Desktop/Work/Projects/my/github/fresher/target cargo nextest run -p
  fresher-module-logs`; `cargo clippy -p fresher-module-logs --all-targets -- -D warnings`; `cargo fmt --check`; the
  crate guard and the dependency guard (`npm run check:crates` or the commit hook: no other module, `#![forbid(unsafe_code)]`);
  `cargo machete` (no unused dependency).
- Commit: `feat(modules): plan 033 — the logs module's crate, texts and categories`.

## Conclusions

Built as designed (abaf1203, lock b05a9a89): the crate `fresher-module-logs`, `locales/en.json`, the two category files and
their catalogue rows, the descriptor with six `Claim::Item` interests, the setting and the problems in order. Two
deviations: `.gitignore` had a bare `logs` line that ignored `modules/logs`, so `!/modules/logs` was added; the problem
keys are built in a loop. The unused fields carried `#[expect(dead_code)]` until 01/02.
