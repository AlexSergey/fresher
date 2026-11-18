# 019/02/02 — The decision

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules

The design's section 3, row by row, as pure functions; the leftovers of a scan gathered and decided in `finish`, by
id. After 02/01, 01/02 (`place_seen`) and 01/03 (`search_complete` behind `find_apps`). Extends `src/lib.rs` of
02/01.

## Files

- `modules/app-leftovers/src/name.rs` (new) — `app_id(name: &OsStr, suffix: Option<&str>) -> Option<BundleId>`: the
  name, which must end with `suffix` (ASCII case ignored) when there is one, without it; then `BundleId::new` with at
  least one dot. `related(a: &BundleId, b: &BundleId) -> bool`: the same id, or one is the other followed by a dot and
  more (on the folded ids; `com.example.Appx` is not related to `com.example.App`).
- `modules/app-leftovers/src/gathered.rs` (new) — the leftovers of one scan, keyed by its cancel token as
  `modules/jetbrains/src/uninstalled.rs`' `Gathered` is: a `Leftover` (row index, id, path, kind, sizes, `modified`,
  `changed`, `inside`, `facts`); `take(cancel) -> BTreeMap<BundleId, Vec<Leftover>>`, each id's leftovers sorted by
  path, handed out once; a scan cancelled before `finish` leaves nothing to the next.
- `modules/app-leftovers/src/decide.rs` (new) — on plain values only:
  - `pub(crate) struct Facts<'a>` — `installed: Result<&'a [InstalledApp], NoSystem>`, `complete: bool` (the list read
    in full), `running: Result<&'a RunningSet, NoSystem>`, `restored: bool`, `places_complete: bool` (every non-optional
    place of the rows the module has `is_complete()`), `now: Timestamp`, `days: u32`;
  - `decide_id(id, leftovers, rows, exclude, facts, search: &mut dyn FnMut(&BundleId) -> Search) -> IdVerdict`, where
    `Search` is `Unread`, `Found` or `NotFound` and `IdVerdict` is `NotClaimed`, `LeftOut(Reason)` or `Offer`; rows 1
    to 11 of the design in order:
    1. `exclude.covers(id)`: `NotClaimed`;
    2. no leftover is a folder of a proof row: `NotClaimed`;
    3. `installed` is `Err`: `LeftOut(NoSystem)`;
    4. an app of `installed` `owns(id)`, or its id or a helper's is `related` to `id`: `NotClaimed`;
    5. `!complete`: `LeftOut(AppsUnread)`;
    6. `running` is `Err`: `LeftOut(NoSystem)`;
    7. a running app's bundle id is `related` to `id`: `LeftOut(Running)`;
    8. `restored`, `!places_complete`, or a folder leftover with `inside` `None` or not `complete`:
       `LeftOut(ScanAgain)`;
    9. a leftover's time within `days` of `now` (a folder's `inside.newest_change`, a file's newest of `modified` and
       `changed`; `None` or after `now` counts as recent; `days` days exactly is old, as `jetbrains`' `age`):
       `LeftOut(UsedRecently)`;
    10. `search(id)` is `Unread`: `LeftOut(SpotlightUnread)`;
    11. `Found`: `LeftOut(InstalledElsewhere)`;
    12. `Offer`.
    `search` is called only at row 10, at most once.
  - `decide_item(leftover) -> ItemVerdict` (rows 13 to 15): `inside.special` → `LeftOut(Special)`; not
    `facts.owned_by_user`, `facts.system_immutable`, `facts.restricted`, `facts.user_immutable`, or `inside.foreign` →
    `LeftOut(NotYours)`; otherwise `Offered`.
- `modules/app-leftovers/src/lib.rs` — `on_match`: the rows of `matched.interest`; the row whose kind is the match's
  (`EntryKind::Folder` or `File`; a link or an `Other` entry matches no row) and whose suffix gives an id
  (`name::app_id`); added to the scan's `Gathered`; a match of no row ignored. `finish`:
  - the facts read once: `installed_apps()`, `installed_apps_complete()`, `running_apps()`, `is_restored()`,
    `place_seen` of every interest whose rows are not optional, the clock, the setting;
  - for each id in order: `cx.cancel()` checked first (`ModuleError::Cancelled`); `decide_id` with `search` calling
    `cx.system().find_apps(&[id], cx.cancel())` (`Err` → `Unread`, non-empty → `Found`);
  - `LeftOut(reason)`: one `out.left_out(LeftOutItem { category: row's, reason, sizes })` per leftover of the id;
    `Offer`: per leftover, `decide_item`, then a left-out item or one `NewFinding` (the design's section 3: identity
    `<row id>/<name>`, `Subject::Path`, `Removal::MoveToTrash` of the same path, the match's sizes, the title
    `app-leftovers.title.<row id>` with `{id}`, the reason with `{days}`, `for_app(id)`).
  - The two `#[expect]`s of 02/01, if any, removed.
- `modules/app-leftovers/Cargo.toml` — `proptest` as a dev-dependency (first used here).

## Produces

The module's answers, for 02/03's fixtures and every later test.

## Checks

- `name.rs`, negatives first: `Notes`, `Cookies.binarycookies` (no dot left), `com.example.App.plistx` and
  `com.example.App` with the suffix `.plist`, `.plist` alone, a non-ASCII name refused; then `com.example.App`
  (no suffix), `com.example.App.plist`, `COM.EXAMPLE.APP.PLIST` (case ignored, the id as written). `related`:
  `com.example.Appx`/`com.example.App` and `com.example`/`org.example` not related; then the same id in another case,
  `com.example.App.Helper` with `com.example.App` both ways.
- `gathered.rs`: as `jetbrains`' `Gathered` (a cancelled scan leaves nothing; nothing kept once taken; ids sorted,
  leftovers by path).
- `decide.rs`, negatives first, each row's keep before its offer: excluded; no proof (only caches and preferences; a
  saved state that is a file); the installed list failing; installed by own id, by a helper, by `com.example.App.Updater`
  under installed `com.example.App`, by `com.example` under installed `com.example.App`; a partial list; running
  failing; running a related id; restored; a place not complete; a folder read in part; changed 5 days ago; a time
  unknown; a time in the future; Spotlight unread; found. Then the offers: `days` days exactly; an optional place not
  scanned keeps nothing (it is not in `places_complete`). Precedence: excluded over no proof, no proof over the apps
  failing, installed over a partial list, a partial list over running, running over the scan, the scan over the age,
  the age over Spotlight (`search` never called when an earlier row holds). `decide_item`: a socket inside, each fact,
  a foreign entry inside, then offered.
- Properties (`mod properties`): adding an installed or running app related to the id never turns a keep into an
  offer; adding a leftover changed within the days never does either; whatever the leftovers' order, the verdict is
  the same.
- `lib.rs` (unit, with plain `Match` values, as `modules/dev-caches/src/tests.rs`): the same matches in two orders
  give the same findings and left-outs; a left-out id reports every leftover in its row's category with its sizes;
  Spotlight asked once per id that reaches row 10 and never for an earlier keep; a scan cancelled before `finish`
  leaves nothing to the next; a cancel during `finish` ends `Cancelled`.
- Test names state only what the test asserts.
- `cargo nextest run -p fresher-module-app-leftovers`; clippy; fmt.

## Conclusions

Built (d748516): `name.rs` (`app_id`, `related`, `search_ids`), `gathered.rs`, `decide.rs` (`decide_id` over rows 1 to 12
in order, `decide_item` over rows 13 to 15; Spotlight is asked at row 10 at most once), the module's `on_match` and
`finish` (the leftovers of an id are gathered, the facts are read once, the cancel token is checked before each id), the
exclusion check and the settings reader. The proof (row 2) is a leftover that is a folder of a row marked as proof:
an absent place or a missing entry adds none, so it never proves (the ruling of 01/02). Every leftover in a left-out id
is reported with its reason; `decide_item` leaves out a socket or a foreign entry alone. Docs: the design says nine
places; `dependencies.md` rows.

Deviations and calls (all keep more, never offer more):

- **Spotlight is asked for every dot-prefix of the id** that still has a dot, `com.example` included, in one call
  (preflight F1: a helper whose app lies elsewhere is installed elsewhere). The sentence and the example of the ruling
  disagreed; the superset was taken.
- `Exclude::covers` is wider than a prefix or an id: an excluded id also covers related ids (helpers and parents), and
  `com.apple` itself.
- A file with either `modified` or `changed` unknown counts as recent; exactly `days` counts as old.
- `PlacesError::OwnKnownFolder` for a library path that names a folder with its own known folder.
- Row 9 (the age) counts an `Absent` place as seen: a recent cache behind a link at `~/Library/Caches` does not hold the
  id back by age. It only weakens an extra margin of an app already proven not installed; recorded with the link known
  issue.
- A finding uses `for_app(id)`, as the brief says, not `for_app_named`.

Tests (negatives first): 110. Every row's keep; the absent-proof and version-like cases (`Python3.11`,
`IntelliJIdea2019.3`, never offered); eight precedence tests (Spotlight never asked after an earlier keep); four
properties (a related id installed or running, a recent leftover, any order); each form offered; Spotlight asked once
per id; cancel before and during `finish`. Mutations (a row removed, `related` dropped, the prefixes dropped, an unknown
time counted as old, the cancel check removed) each turned tests red. Clippy and fmt clean.

Parked minors: `decide_item` ignores `Inside::lock_named` (the cleaner's in-use check covers it).
