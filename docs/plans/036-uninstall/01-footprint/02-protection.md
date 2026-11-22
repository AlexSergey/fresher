# 036/01/02 — The apps folder in the protection

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, macos

A deletion-safety change: today `/Applications` is a `never` place, so nothing in it can ever be removed. After this
step it is a never place that lets through **exactly a whole app bundle** at its top or one folder below, and nothing
else; and the core gains the means for the engine to refuse the apps folder again for every plan that is not an
uninstall (used by 02/01). The design: [`../00-design.md`](../00-design.md), section 6.

## Before you start

Read `docs/restrictions/deletion-safety.md`, `docs/architecture/removal.md` ("The guard", check 3),
`crates/core/src/cleanup/protection.rs` (all of it, with its tests), `crates/macos/src/protection.rs` (`SYSTEM`,
`Places::of`, `protection()`, its tests), and `crates/core/src/settings/folders.rs` (a scan root of `/Applications` is
allowed today; this step does not change that). The rule is pure path components: never compare strings.

## Files

- `crates/core/src/cleanup/protection.rs`:
  - `Never` gains `opens_bundles: bool` (false for every place made by `new`).
  - `pub fn with_app_folders(&self, folders: &[PathBuf]) -> Result<Self, ProtectionError>`: each folder must be plain
    and equal to one of the list's own `never` places, else `ProtectionError::LooseAppFolder { path }` (new variant);
    those places get `opens_bundles = true`.
  - `pub fn with_refused(&self, refused: &[PathBuf]) -> Result<Self, ProtectionError>`: adds each plain path as a
    `never` place with no carve-out and `opens_bundles = false` (`NotPlain` otherwise). Answers `Protected::Never` for
    the path, what is inside and (as every never place) `Ancestor` for what holds it.
  - `check`: a never place whose `opens_bundles` is set lets `path` through only when `is_app_bundle_in(path, place)`:
    the components after `place` are exactly `[<name>.app]` or `[<sub>, <name>.app]`, `<sub>` not ending in `.app`,
    every name non-empty, the `.app` suffix compared on the last component's extension (`Path::extension() == "app"`,
    case as written: macOS bundles end in lower-case `.app`; an `.APP` is refused). Every other never place, the
    ancestor rule, the whole folders and the ignored paths still apply in the same order, so Fresher's own bundle
    (its own never place) stays refused.
  - `joined`: each `Never` keeps its own `opens_bundles` (a carve-out of one list never opens another list's place; a
    bundle opening likewise).
- `crates/macos/src/protection.rs`: `/Applications` leaves `SYSTEM` (its comment goes) for `const APP_FOLDERS: [&str;
  1] = ["/Applications"]` ("The apps of every user: a whole app bundle may go, for an uninstall the user confirmed;
  nothing inside a bundle, and not the folder"); `Places::of` pushes it to `never` as before; `protection()` returns
  `Protection::new(…)?.with_app_folders(&app_folders)`. `/System/Applications` stays under `/System`.

## Produces

`Protection::with_app_folders` (used by the macOS list) and `Protection::with_refused` (used by 02/01's context).

## Checks

- `crates/core/src/cleanup/protection.rs`, `mod negative` first:
  `a_path_inside_a_bundle_of_an_app_folder_is_refused` (`/Applications/A.app/Contents`, `…/Contents/MacOS/a`),
  `the_app_folder_itself_and_a_folder_in_it_are_refused` (`/Applications`, `/Applications/Utilities`),
  `a_bundle_two_folders_down_is_refused` (`/Applications/a/b/C.app`), `a_bundle_inside_a_bundle_is_refused`
  (`/Applications/A.app/B.app`), `a_name_that_only_looks_like_a_bundle_is_refused` (`/Applications/A.app.zip`,
  `/Applications/.app`, `/Applications/A.APP`), `a_never_place_inside_an_app_folder_stays_refused` (Fresher's own
  bundle as its own never place), `an_ignored_bundle_is_refused`, `a_loose_app_folder_is_refused`,
  `a_refused_place_is_never_and_its_ancestors_are_refused`, `another_lists_app_folder_opens_nothing_once_joined`; `mod
  positive`: `a_bundle_at_the_top_of_an_app_folder_passes`, `a_bundle_one_folder_down_passes`
  (`/Applications/Utilities/A.app`), `joined_lists_keep_the_opening_of_their_own_place`. A property test
  (proptest): for random component lists under `/Applications`, `check` is `Ok` only for the two bundle shapes.
- `crates/macos/src/protection.rs`, update the existing `/Applications/Notes.app` case: it now passes; add
  `nothing_inside_an_app_bundle_passes` and `the_system_apps_folder_stays_never`
  (`/System/Applications/Chess.app`), and keep `Fresher.app` refused (`app_bundle`).
- `crates/cleaner/tests/never.rs` and the cleaner's own tests pass unchanged (`cargo nextest run -p fresher-cleaner`):
  the cleaner's code does not change.
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-cleaner`; clippy; fmt.

## Conclusions

- Built as designed in `crates/core/src/cleanup/protection.rs` (tests in `protection_tests.rs`) and
  `crates/macos/src/protection.rs`: `/Applications` moved from the system list to `APP_FOLDERS`, opened by
  `Protection::with_app_folders` for exactly `<folder>/<name>.app` and `<folder>/<sub>/<name>.app`. Core, macos and
  cleaner 983 tests; two property tests (only the two shapes pass; no other answer changes).
- `Never` carries a `NeverKind` (`Closed`, `OpensBundles`, `Refused`) instead of a flag, so a refused place cannot be
  opened by a later call; `with_refused` is the engine's second wall; `with_app_folders` refuses a joined list.
- `<sub>` is refused when its extension is `app` in any case. A bundle under an opened place is refused when its names
  match a never, whole or ignored place inside the folder ignoring case (`fresher.app`); no Unicode case folding or
  normalisation (a known issue; the uninstall plan's canonical check covers it).
- A link named `X.app` passes the lexical check by design: the listing and the plan refuse links
  ([`../../known-issues/core-protection-passes-a-bundle-shaped-link.md`](../../known-issues/core-protection-passes-a-bundle-shaped-link.md)).
