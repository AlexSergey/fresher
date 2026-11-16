# 006/05/02 — Needs an administrator, in the UI

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** engine, app, app/ui

## Files

- `crates/engine/src/engine.rs` — `reveal_finding(FindingId)`: looks the finding up in the current results (an id from
  another scan is refused, as for every finding command), and calls `SystemUi::reveal` with its path; a finding whose
  subject is not a path (a Docker image) is refused with `CommandError::NothingToReveal`.
- `crates/core/src/error.rs` — `NothingToReveal` and its text.
- `app/tauri/src/commands.rs`, `lib.rs`, `build.rs`, `capabilities/default.json` — `reveal_finding(id)`, allowlisted;
  takes an id, never a path.
- `app/ui/src/ipc/index.ts`, `mock.ts`, tests — `revealFinding(id)`.
- The finding row of the generic module page (plan 003's `FindingList` / `ListRow`) — a finding that is not
  `Removable` shows its size, a badge with the short label (icon and word), the instruction on demand, and
  `[Show in Finder]` (for `NeedsAdministrator` and `Locked`); its checkbox is absent, so it cannot be selected; the
  category's "select all" skips it and says how many were left out.

## Produces

The visible half of "the app never runs as administrator": the user sees the space and how to free it themselves.

## Checks

- Negative (first): `an_item_that_needs_an_administrator_has_no_checkbox`, `select_all_skips_items_that_are_not_removable`,
  `a_protected_item_offers_no_instruction_and_no_finder_button`, `reveal_of_an_unknown_id_is_refused`,
  `reveal_of_a_docker_image_is_refused`, no text anywhere contains `sudo` (a test over the core dictionary).
- Positive: `show_in_finder_reveals_the_findings_path` (`FakeSystemUi` records the path), `the_instruction_is_shown_on_demand`,
  `select_all_says_how_many_were_left_out`.
- Gallery states and screenshots: a row in each `Removability`.
