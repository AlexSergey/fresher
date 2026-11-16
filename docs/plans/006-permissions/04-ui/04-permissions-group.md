# 006/04/04 — The Permissions group

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/screens/access/permissions-group.tsx` — the content of the Permissions group of plan 005's settings
  screen: one `SettingsRow`-like row per capability in `AccessStatus` (those `NotApplicable` left out): its name, what it
  unlocks, its state as a word and an icon (never colour alone), and its action — `[Open System Settings]` for an OS
  permission that is not granted, the opt-in switch for a guarded folder (the `core.access.*` setting itself, rendered by
  005's typed control), nothing for a granted one. Tools are listed under their module ("Docker: not running").
  A `[Check again]` button calls `refreshAccess()`.
- `app/ui/src/screens/access/permissions-group.test.tsx`.
- The settings screen of plan 005 — the group mounted in its place; the four `core.access.*` switches are shown here and
  not again in the generic list.
- `app/ui/src/gallery/` — the group with every capability in every state.

## Produces

The one place that lists every permission (design, section 8), and the Playwright screenshots of the access states.

## Checks

- Negative (first): `a_not_applicable_capability_is_not_listed`, `a_granted_permission_has_no_action`,
  `the_opt_in_switches_are_not_listed_twice`.
- Positive: `every_capability_shows_its_state_and_what_it_unlocks`, `open_system_settings_passes_the_capability`,
  `check_again_refreshes_the_status`, keyboard: every action reachable, focus visible.
- `npx vitest run` on the changed files; the Playwright screenshots of the group are added and looked at.
