# 019/01/02 — What the walk saw of each place

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** core, scanner, engine, testing, docs

A module that decides from every leftover of an app must know whether the walk could see each of its places in full;
the global `cx.is_walk_complete()` is false on any Mac without Full Disk Access. The walk now says, per anchored
interest, what it saw of the folder the interest matches in. The design: [`../00-design.md`](../00-design.md), section
1.2 (agent's call 5). After 01/01.

## Files

- `crates/core/src/walk.rs` — `PlaceSeen { Read, Absent, Unread, NotScanned, Unknown }` with `is_complete()` (`Read`
  or `Absent`), each variant documented as in the design; `WalkSummary` gains
  `pub places: BTreeMap<usize, PlaceSeen>` (index in the interest set; only interests with one place). `is_complete`
  of the summary is unchanged. Re-exported from `lib.rs` (add the line only).
- `crates/core/src/module.rs` — `ModuleContext` gains `places: &'a [PlaceSeen]` (empty by default) and
  `#[must_use] pub const fn with_places(self, places: &'a [PlaceSeen]) -> Self`; `pub fn place_seen(&self, interest:
  usize) -> PlaceSeen`: `Unknown` when restored, when the index has no entry, and before `with_places` (during the
  walk). `Debug` shows the count.
- `crates/scanner/src/places_seen.rs` (new) — `places_seen(tree, anchors) -> BTreeMap<usize, PlaceSeen>`, from the
  matcher's anchored interests (the compiled absolute segments `Matcher::compile` already keeps) and the frozen tree:
  - an interest has one place when no segment but its last is `*` (plain or shaped); its place is the absolute path
    without the last segment;
  - walking the place's path down from its scan root's node: a folder on the way, or the place itself, unreadable
    (its not-scanned reason `Unreadable`) or `Flag::Incomplete` (an entry of its own listing not read) → `Unread`; a
    folder on the way or the place `Avoided`, `OtherVolume` or `OthersFolder`, or the path outside every root → 
    `NotScanned`; a folder on the way read and lacking the next name, or holding it as anything but a folder →
    `Absent`; the place reached, read, not `Incomplete` → `Read`; `Collapsed` on the way → `Unread`;
  - never `Partial` (it climbs from any unreadable folder below: `Caches/com.apple.Safari` guarded must not make
    `Caches` unread); an avoided child of the place does not make the place `Unread` (its entry was read).
- `crates/scanner/src/scanner.rs` — `walk` fills `summary.places` after every walk kind (full, rescan, restore), from
  the tree it is about to hand to `commit`. Check, and write in the conclusions, that the cache codec keeps the flags
  the function reads (`Incomplete`, `Avoided`, `OtherVolume`, `OthersFolder`, `Collapsed`, the unreadable reason), so a
  folder a rescan copies and a restore answer as the walk did; if one is not kept, keep it (a tree format bump, an
  older cache refused once) and say so.
- `crates/engine/src/session.rs` — `finish_modules` hands each module the states of its own interests: a
  `Vec<PlaceSeen>` per planned module built from `summary.places` with `InterestSet::owner` (missing entries
  `Unknown`), passed with `cx.with_places(&places)` beside `complete_walk()`; a restored scan's context stays
  `restored()` (so `place_seen` answers `Unknown`).
- `crates/testing/src/naive_walk.rs` — the reference walk fills `places` the same way, from what it read.
- `crates/testing/src/run_module.rs` — `Ports` gains `pub avoid: Vec<PathBuf>` (paths relative to the fixture root,
  empty by default) passed to the walk request, so a fixture makes a place `NotScanned`; `run_module` passes the
  module's states to `finish` with `with_places`.
- `docs/architecture/modules.md` — "The ports a module may use": `cx.place_seen(interest)` beside
  `cx.is_walk_complete()`, when to use which.

## Produces

`PlaceSeen`, `WalkSummary.places`, `ModuleContext::{with_places, place_seen}`, `Ports::avoid`, for 02/02 (row 8 of
the decision) and every later module that decides from every match of a place.

## Checks

- `places_seen.rs`, negatives first, on small trees built as the scanner's own tests build them: the place unreadable;
  a folder on the way unreadable; the place `Incomplete`; the place avoided; a folder on the way on another volume; the
  place outside every root; a pattern with `*` before its last segment (no entry); `Collapsed` on the way; then the
  place read (`Read`) while a matched folder inside it is unreadable (still `Read`); the place's parent read without it
  (`Absent`); the place a file (`Absent`); an avoided child of the place (`Read`).
- Scanner walk tests (`walk/places_tests.rs` or beside them): a full walk, a rescan that copies the place's folder and
  one that reads it again, and a restore each give the same `places` for an unchanged tree; a rescan after the place
  became unreadable gives `Unread`.
- The reference property test: the walk and `NaiveWalk` agree on `places` over generated trees with unreadable,
  avoided and missing folders on the way.
- `module.rs`: `place_seen` is `Unknown` without `with_places`, out of range and when restored; then the state given.
- `crates/engine` (session tests with `ScriptedWalk`): a module's `finish` sees its own interests' states, not another
  module's; a restored scan's `Unknown`.
- `run_module`: a fixture with `Ports::avoid` holding a place gives `NotScanned` to the module.
- No module's answers change (`cargo nextest run --workspace` once).
- The scan bench is not rerun for this step (nothing in the walk's hot path), unless the cache format changed: then
  the cache bench before and after, in the conclusions.
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-engine -p fresher-testing`; clippy; fmt.

## Conclusions

Built (e90c6dc): `PlaceSeen` (`Read`, `Absent`, `Unread`, `NotScanned`, `Unknown`; `is_complete()` is `Read` or
`Absent`) and `WalkSummary.places` (what the walk saw of each anchored interest's place, not the walked places) in the
core; `ModuleContext::place_seen(interest)` and `with_places`; `places_seen(tree, anchors)` in the scanner, filled after
a full walk, a rescan and a restore (the restored context answers `Unknown`); the engine builds each module's list in
`finish_modules`; `NaiveWalk` records what it saw and walks each place down the same way; `Ports::avoid` in `run_module`.
`docs/architecture/modules.md` and `docs/testing.md` describe it.

Deviations and calls:

- **A link at a place reads `Absent`** (controller ruling). The first ruling was `NotScanned`, but links are folded out
  of the tree, so a link cannot be told from no entry without a hot-path change; a link only hides data the walk never
  reads, so the cost is missed offers, never false ones, as long as no rule treats absence as proof (02/02: proof is a
  present saved state or container). Known issue
  [`scanner-link-at-a-place-reads-absent.md`](../../../known-issues/scanner-link-at-a-place-reads-absent.md).
- An `Opaque` or `Collapsed` folder before the place answers `Unread` (preflight F11); a place that is itself `Opaque`
  is `Read` unless `Incomplete`.
- A folder on the way marked `Incomplete` makes the place `Unread` even when the next name was read: conservative.
- The cache codec check: the codec writes the whole `NodeFlags::bits()`, so `Incomplete`, `Avoided`, `OtherVolume`,
  `OthersFolder`, `Opaque`, the unreadable reason and `Collapsed` all survive. No format change, no version bump, no
  cache bench; the rescan and restore parity tests confirm it.
- The scanner compares an avoided path canonically, `NaiveWalk` as given: the property tests avoid only canonical
  paths. Known issue
  [`scanner-reference-walk-avoided-paths.md`](../../../known-issues/scanner-reference-walk-avoided-paths.md).
- No bench: the hot path did not change (the matcher keeps the anchors it already built; `places_seen` runs once per
  walk over the anchored interests only).

Tests (negatives first): core walk and module tests; 18 unit tests of `places_seen` (avoided, other volume, another
user's, unread, `Incomplete`, `Opaque`, `Collapsed`, a file or a link where the folder should be); walk tests for the
full walk, the rescan after a `chmod 000` and the restore; the reference property tests (the scanner and `NaiveWalk`
agree on `places`; a mutation that dropped the `Opaque` rule, or the unreadable rule, failed them); an engine test of the
per-module list. `cargo nextest run --workspace`: 2589 passed, 5 skipped; no golden changed.

Parked minors: a folder on the way that a rescan folded answers `Unread` where a full walk answers `Read` (safe);
`session.rs` `places_of` searches the ids linearly (pass `&ids`).
