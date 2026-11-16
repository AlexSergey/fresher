# 006/02/02 — Fakes

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** testing

## Files

- `crates/testing/src/fakes.rs` — `FakeAccess`: built with `FakeAccess::with(&[(Capability, Access)])`, every capability
  not given is `NotApplicable`; `guarded_places` answers places on a `FixtureTree` (`.guard("Desktop",
  Capability::Folder(Desktop))`); `try_place` answers from a script per place and records every call; `set(capability,
  access)` changes a state between calls (for the re-check tests). `FakeSystemUi`: records every
  `open_access_settings` and `reveal` call, and can be told to fail. Both keep their state in a `Mutex`, like the other
  fakes, because the ports are `Send + Sync`.
- `crates/testing/src/lib.rs` — the exports; the module harness's `Ports` gains `access` and `system_ui` with the fakes
  as defaults, so existing module tests do not change.

## Produces

The fakes every later test of this plan, plan 007's scenarios and plan 008's guards use. No test after this step needs
macOS to reach any access state.

## Checks

- Negative: `an_unlisted_capability_is_not_applicable`, `try_place_on_an_unscripted_place_is_unknown`,
  `a_failing_system_ui_returns_its_error`.
- Positive: `set_changes_the_state_seen_by_the_next_status`, `try_place_calls_are_recorded_in_order`,
  `guarded_places_are_on_the_fixture_tree`, `system_ui_calls_are_recorded`.
- The existing tests of `fresher-testing` and of the `demo` module pass unchanged.

## Conclusions

- `FakeAccess` and `FakeSystemUi` are in `fresher-testing`, with `SystemUiCall` (`OpenAccessSettings(Capability)`,
  `Reveal(PathBuf)`) for what the UI fake records. The module harness's `Ports` gained `access` and `system_ui`,
  defaulting to `FakeAccess::default()` (no capability: all `NotApplicable`) and a recording `FakeSystemUi`; no module
  test changed.
- **Deviation: the two fakes got their own file**, `crates/testing/src/fake_access.rs`, instead of `fakes.rs`, which
  would have passed 500 lines (as `FixedDiskState` did in plan 004/02). They share `fakes.rs`'s `lock`.
- Not spelled out by the step, taken the simplest way:
  - The fixture tree is given as the fake's home folder: `FakeAccess::with(&[..]).home(tree.root()).guard("Desktop",
    Capability::Folder(Desktop))`; `guarded_places(roots)` answers the places inside any of the roots.
  - The `try_place` script is `.answering("Desktop", Access::Granted)`, by the place's path under the home folder; an
    unscripted place answers `Unknown`. `tried()` gives the places tried, in order.
  - `set` adds a capability that was not listed. `FakeSystemUi::failing(error)` records each call and answers
    `error` to all of them.
