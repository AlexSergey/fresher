# 036/01/01 — The core's types, texts and categories

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core

The vocabulary every later step uses, in `fresher-core`: the bundle's facts and the trait method that answers them,
the IPC types of the Apps list and an app's page, the two uninstall categories, the new refusals and every text. Pure
types and texts; no IO, no behaviour beyond constructors and text keys. The design:
[`../00-design.md`](../00-design.md), sections 3, 5, 7, 11 and 12.

## Before you start

Read `CLAUDE.md`, `docs/rules.md`, `docs/architecture/README.md` (sections "Problems and reasons" and "Data"), then
`crates/core/src/system/apps.rs`, `crates/core/src/disk_state.rs` (`NodeId`, the pattern for `AppId`),
`crates/core/src/category.rs`, `crates/core/src/ids.rs` (`ModuleId::system()`, `CategoryId::new`),
`crates/core/src/error.rs` (`CommandError` and its refusal texts), `crates/core/src/texts.rs` (its tests list every key
the core can emit: `core_keys()`, `texts_with_args()`), `crates/core/locales/en.json`, and
`app/tauri/tests/bindings.rs` (the list of exported types). The core already has a `PartKind` (the storage map's): the
new kind is named `FootprintKind`.

## Files

- `crates/core/src/apps/mod.rs` (new) — `pub use` of the items below; `crates/core/src/lib.rs`: `mod apps;` and its
  exports (add at the end of the existing lists).
- `crates/core/src/apps/bundle.rs` (new):
  - `pub enum Manager { Homebrew }` with `name()` (a proper name passed as it is; no text key, no serde: never sent
    over IPC on its own — the key and the derives were dropped in the fix round of 01/03).
  - `pub struct BundleFacts { pub sizes: Sizes, pub entry: EntryFacts, pub last_used: Option<Timestamp>, pub
    installed_by: Option<InstalledBy> }` with `pub struct InstalledBy { pub manager: Manager, pub token: String }`
    (`token` the cask's name, used only in the command text; checked to be 1 to 128 of `[a-z0-9@._+-]`, else the
    constructor `InstalledBy::new` refuses it: `AppsError::Token`). `BundleFacts::removability()` =
    `Removability::of(self.entry)`.
- `crates/core/src/system/apps.rs` — the trait gains, after `search_complete`:

  ```rust
  /// The facts of `app`'s bundle: what it takes, whether the user may move it, when it was last used and who
  /// installed it. Reads metadata only. The default answers an error: a port that does not know bundles lists the
  /// app without a size, never offers it.
  fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
      let _ = (app, cancel);
      Err(SystemError { reason: "this system does not read bundles".into() })
  }
  ```

  No existing implementation changes.
- `crates/core/src/apps/view.rs` (new), the IPC types (`Serialize`, `TS`, camelCase like `NodeInfo`; ids as `NodeId`):
  - `pub struct AppId { state: StateId, index: u32 }` with `new`, `state()`, `index()`; `Deserialize` too (commands
    take it).
  - `pub enum FootprintKind { Bundle, Caches, Logs, SavedState, LaunchAgents, WebData, Settings, Support, Container,
    GroupContainer }` with `ALL`, `name_key()` → `core.apps.part.<kebab>`, `danger()` (Medium, Low, Safe, Low,
    Medium, Medium, Medium, High, High, High), `goes_with(choice)` (the design's section 5 table: `Bundle`, `Caches`,
    `Logs`, `SavedState`, `LaunchAgents` go with both; the rest with `Everything` only).
  - `pub enum UninstallChoice { KeepData, Everything }` (kebab `keep-data`, `everything`; `Deserialize`), `ALL`,
    `category()` → the category id below, `label_key()` / `description_key()` →
    `core.uninstall.choice.<choice>.{label,description}`, `reason_key()` → `core.uninstall.reason.<choice>`.
  - `pub enum KeptReason { Shared, TeamShared, Link, NeedsPermission, NotScanned, Ignored }`, `text()` →
    `core.apps.kept.<kebab>`.
  - `pub enum NotOffered { Unmeasured, InstalledBy, NeedsAdministrator, Locked, Protected, Open, TooManyFiles }`,
    `code()` (kebab), `text_key()` → `core.apps.not-offered.<code>`; `NotOffered::of(Removability)` for the three
    removability states (`None` for `Removable`).
  - `pub enum AppOffer { Offered, NotOffered { reason: NotOffered, text: Text } }` (`tag = "kind"`).
  - `pub struct PartSizes { pub kind: FootprintKind, pub sizes: Sizes }`.
  - `pub struct AppSummary { pub app: AppId, pub name: String, pub version: Option<String>, pub sizes: Sizes, pub
    parts: Vec<PartSizes>, pub last_used: Option<Timestamp>, pub summary: Text, pub offer: AppOffer }`.
  - `pub struct FootprintItem { pub kind: FootprintKind, pub path: PathBuf, pub sizes: Sizes, pub danger: DangerLevel,
    pub keep_data: bool, pub everything: bool, pub kept: Option<KeptItem> }`, `pub struct KeptItem { pub reason:
    KeptReason, pub text: Text }` (a kept item has both flags false).
  - `pub struct ChoiceView { pub choice: UninstallChoice, pub label: Text, pub description: Text, pub sizes: Sizes,
    pub items: u32 }` (what goes under the choice: the bundle and its parts).
  - `pub struct AppDetail { pub summary: AppSummary, pub items: Vec<FootprintItem>, pub choices: Vec<ChoiceView>, pub
    categories: Vec<CategoryId>, pub command: Option<String>, pub notice: Option<Text> }` (`command`: a manager's
    uninstall command to copy; `notice`: App Management's line, set by 02/02).
  - `pub enum UnusedFor { ThreeMonths, SixMonths, AYear }` with `days()` (90, 182, 365); `pub struct AppsQuery {
    unused: Option<UnusedFor> }` (`Deserialize`, `deny_unknown_fields`), `unused()`.
  - `pub struct AppsView { pub apps: Vec<AppSummary>, pub notice: Option<Text> }` (`notice`: `core.apps.no-apps` when
    the installed apps could not be listed, `core.apps.other-names` otherwise).
- `crates/core/src/apps/categories.rs` (new) — `pub fn uninstall_categories() -> [CategoryDescriptor; 2]`:
  `core.uninstall-keep-data` at Medium and `core.uninstall-everything` at High
  (`CategoryId::new(ModuleId::system(), …)`). Not registered in any registry: they never appear on Scan.
- `crates/core/src/error.rs` — `CommandError::UnknownApp { id: AppId }` ("app {id} is not in this scan") and
  `CommandError::UninstallRefused { reason: NotOffered }`, each with its refusal text (`core.refusal.unknown-app`,
  `core.refusal.uninstall-refused`, which renders the reason's own text as the refusal's sentence: follow how
  `CleaningBlocked` carries its reason); add both to the test helper `one_of_each`.
- `crates/core/locales/en.json` — every key of the design's section 11, in plain English (rule 7; no "Other", no
  "sudo"), with these placeholders only: `core.uninstall.title` `{app}`; `core.uninstall.reason.*` `{app}`, `{size}`;
  `core.apps.summary.unused` `{app}`, `{date}`, `{size}`; `.used` and `.unknown` `{app}`, `{size}`;
  `core.apps.not-offered.installed-by` `{manager}`. The five texts of each uninstall category (`name`, `what`,
  `made-by`, `if-removed`, `comes-back`). The `core.apps.not-offered.*` sentences:
  - `unmeasured` "Fresher could not read this app, so it does not offer to remove it. Scan again."
  - `installed-by` "{manager} installed this app. Uninstall it with {manager}, so it stays in step."
  - `needs-administrator` "Removing this app needs an administrator. Show it in Finder and drag it to the Trash; macOS
    asks for your password."
  - `locked` "This app is locked in Finder. Unlock it there first."
  - `protected` "macOS protects this app. It cannot be removed."
  - `open` "This app, or a part of it, is open. Quit it, then uninstall it."
  - `too-many-files` "This app has more files than one uninstall can take. Remove the app first; Fresher offers its
    files once it is gone."
- `crates/core/src/texts.rs` (tests) — `core_keys()` and `texts_with_args()` list the new keys, so
  `no_key_in_the_file_is_unused` and `every_placeholder_matches_the_args_the_core_passes` cover them.
- `app/tauri/tests/bindings.rs` — export the new IPC types; regenerate `app/ui/src/ipc/bindings.generated.ts`.

## Produces

The types and texts for 01/02 to 03/02. `InstalledApps::bundle` for 01/03. The categories for 02/01.

## Checks

- Unit tests in each new file, negatives first: `mod negative` — `a_token_with_a_slash_or_a_space_is_refused`,
  `an_empty_or_long_token_is_refused`, `a_removable_bundle_is_not_a_reason`, `an_apps_query_with_an_unknown_field_is_
  refused`, `the_default_bundle_answer_is_an_error` (a test port that implements only the three old methods); `mod
  positive` — `every_kind_has_its_danger_and_its_choices` (the section 5 table, pinned), `each_choice_names_its_
  category` (Medium and High), `each_removability_maps_to_its_reason`, `an_app_id_round_trips_through_json`,
  `the_ipc_types_serialize_in_camel_case`.
- `crates/core/src/texts.rs`' tests pass with the new keys (no unused key, every placeholder passed).
- `cargo nextest run -p fresher-core`; `cargo nextest run -p fresher-app --test bindings`; clippy with `-D warnings`;
  fmt. `git diff -- crates/cleaner modules` is empty.

## Conclusions

- Built as designed: `crates/core/src/apps/` (`kinds.rs`: `PartKind`, `KeptReason`, `NotOffered`, `UninstallChoice`,
  `Manager`; `view.rs`: the summaries, the page, the footprint items, the choices, `AppsView`; `bundle.rs`; `categories.rs`:
  the two uninstall categories), the refusals `UnknownApp` and `UninstallRefused`, the texts (`core.apps.*`,
  `core.uninstall*`) and the port's default `InstalledApps::bundle`. Core 512 tests; the bindings regenerated, additions
  only.
- The refusal texts are `core.error.unknown-app` and `core.error.uninstall-refused` (the core's tests name every
  refusal `core.error.<code>`); `UninstallRefused` carries its reason as a field and reads one fixed sentence, so the
  page's offer text gives the exact reason and the UI reloads the page after a refusal.
- `InstalledBy` keeps its fields private and builds the command (`brew uninstall --cask <token>`) in one checked place;
  `Manager` is in no binding and has no text key of its own (a review removed the unused ones).
- The keep-data texts say "login items": launch agents go with both choices.
