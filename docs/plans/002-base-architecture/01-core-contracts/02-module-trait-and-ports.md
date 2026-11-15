# 002/01/02 — The module trait and ports

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core

## Files

- `crates/core/src/module.rs` — `Module` (`descriptor`, `interests`, `on_match`, `finish`), `ModuleDescriptor`
  (validated: unique category ids inside the module's namespace), `Locales { en: &'static str }`, `ModuleError`,
  `FindingSink`, `ModuleContext` (`#[non_exhaustive]`, a constructor for the engine and the test harness).
- `crates/core/src/interest.rs` — `Interest`, `Pattern`, `Claim`, `InterestSet` (every module's interests with the
  owner of each one), `Match` (with the siblings found next to the item), `EntryKind`. Interests hold owned strings:
  a rule-based module builds them from its parsed `rules.toml`.
- `crates/core/src/walk.rs` — the `DiskWalk` port, `WalkProgress`, `WalkSummary` (entries, unreadable folders by kind),
  `WalkError`.
- `crates/core/src/cancel.rs` — `CancelToken` (`cancel`, `is_cancelled`, cheap clone).
- `crates/core/src/ports.rs` — `AccessProbe`, `AccessStatus { full_disk_access: Access }`,
  `Access { Granted, Denied, Unknown }`. The existing `Clock`, `Trash`, `CommandRunner` stay as they are, with
  `Send + Sync` bounds added.

## Produces

The contract a module implements and the engine drives; the walk port plan 004 implements and `NaiveWalk` (06/03)
fakes; the access port plan 006 implements.

## Checks

- Negative: `rejects_a_descriptor_with_duplicate_category_ids`, `rejects_a_category_of_another_module`,
  `rejects_an_interest_with_an_empty_name`, `rejects_an_under_home_path_that_climbs_out` (`..`, absolute).
- Positive: `cancel_token_is_seen_by_every_clone`, `interest_set_remembers_the_owner_of_each_interest`,
  `a_module_without_interests_is_valid`.
- A test module in `#[cfg(test)]` implements `Module` end to end with a `Vec`-backed sink (proves the trait is object
  safe: `Box<dyn Module>`).

## Notes from the implementation

- **The `Send + Sync` bounds reach `fresher-testing`**: `FakeClock`, `FakeTrash` and `FakeCommandRunner` hold their
  state in a `Mutex` instead of a `Cell` / `RefCell`; their API is unchanged.
- **The port's error is renamed `CommandRunError`** (was `CommandError`): step 01/04 gives `CommandError` to the error
  the UI sees, and two types of that name in one crate cannot both be exported.
- `ModuleDescriptor::new` takes the id, the categories and the locales here; step 01/03 adds the settings. In the same
  way `ModuleContext::new` gets the module's settings in 01/03.
- `InterestSet::owner(index)` gives the owner and the interest's index in the owner's own `interests()`; the engine
  passes a `Match` to a module with that local index.
- `Match`, `WalkProgress` and `WalkSummary` are plain records with public fields (built by the walk adapters).
  `WalkSummary.unreadable` counts by `Unreadable { NoPermission, NeedsFullDiskAccess, Failed }`; `WalkError` is
  `Cancelled` or `RootUnreadable { index, kind }` (no path).
- `ModuleError` is `Cancelled` or `Failed { reason: Text }`: the reason is what `ModuleFinished` shows.
- `FindingSink` is implemented for `Vec<NewFinding>`, for the module harness and tests.
- A `Pattern::Extension` is given without the dot (`dmg`); names may not contain `/` or NUL or be `.` / `..`; an
  `UnderHome` path is made of plain names only (no `..`, `.`, or leading `/`).
