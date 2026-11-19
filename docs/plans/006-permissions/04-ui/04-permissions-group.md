# 006/04/04 — The Permissions group

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/ui

## What exists already

- `app/ui/src/screens/settings/settings-screen.tsx` — the groups in order: General, Scan, the module groups, Ignored,
  then `groups.after` (the core group `permissions`), Your Fresher data, About. Each core group renders its
  descriptors as generic `SettingsRow`s (`product/settings-row`) under its heading (`ui.screen.settings.group.permissions`).
- `app/ui/src/screens/settings/settings-groups.ts` — `settingsGroups` puts `core('permissions')` in `after`.
- The four switches `core.access.{desktop,documents,downloads,other-apps-data}` are the only settings of the group
  `permissions` (`crates/core/src/settings/core_schema.rs`), shown today as plain rows.
- 04/01's `useAccessStatus()`; 03/04's `refreshAccess()` and `openAccessSettings(capability)`.

## Files

- `app/ui/src/screens/permission/permissions-group.tsx` (new) — the content of the Permissions group: one row per
  capability of `AccessStatus` (those `not-applicable` left out), in the core's order: its name
  (`core.access.<id>.name`), what it unlocks (`.unlocks`), its state as a word and an icon
  (`core.access.state.<state>`, never colour alone), and its action — `[Open System Settings]` for `all-files` when
  not granted; the opt-in switch for the three folders and other apps' data (the `core.access.<id>` setting itself,
  rendered by `SettingsRow`, the typed control of plan 005, with its `.label` and `.description`); for a volume its
  `.how`; nothing for a granted one. Tools are listed under the name of the module whose categories need them (`CategoryInfo.descriptor.needs`), with their note ("Docker is installed
  but not running. Start it, then scan again."). A `[Check again]` button calls `refreshAccess()`.
- `app/ui/src/screens/settings/settings-groups.ts` — the `permissions` core group leaves `after` (the four switches are
  shown by the new group and not again as generic rows: agent's call 22).
- `app/ui/src/screens/settings/settings-screen.tsx` — `<PermissionsGroup />` mounted where `groups.after` was, under
  the same heading; the screen's comment keeps the order.
- `app/ui/src/screens/permission/permissions-group.test.tsx` (new), `app/ui/src/screens/settings/settings-screen.test.tsx`
  — the tests below (there is no separate test file of `settings-groups.ts`; the screen's tests cover it).
- `app/ui/src/screens/permission/permissions-group.stories.tsx` (new; the gallery collects every `*.stories.tsx`) — the group with every capability in every state.
- `app/ui/src/locales/en.json` — the chrome labels of `[Check again]` and `[Open System Settings]` if 04/02 did not add
  them.

## Produces

The one place that lists every permission (design, section 8), and the Playwright screenshots of the access states.

## Checks

- Negative (first): `a_not_applicable_capability_is_not_listed`, `a_granted_permission_has_no_action`,
  `the_opt_in_switches_are_not_listed_twice`, `a_read_only_settings_file_disables_the_switches` (as the other rows).
- Positive: `every_capability_shows_its_state_and_what_it_unlocks`, `open_system_settings_passes_the_capability`,
  `check_again_refreshes_the_status`, `a_tool_is_listed_under_its_module_with_its_note`, keyboard: every action
  reachable, focus visible.
- `npx vitest run` on the changed files; the Playwright screenshots of the group are added and looked at.

## Conclusions

- Built `permission-rows.tsx` (presentational: one row per capability in the core's order, the state as an icon and a
  word, the action; `not-applicable` left out; a tool is listed under each module whose categories need it, with the
  core's note; a tool no module needs is not listed) and `permissions-group.tsx` (the section under the existing
  heading, with `[Check again]` at the heading's end calling `refreshAccess()`, `openAccessSettings` and `relaunch`
  through the ipc module). The opt-in switches are the typed `SettingsRow`s of `core.access.*`, passed in as a render
  function, so they follow the read-only settings file and show their refusals like every other row.
- `permissions` left `settingsGroups` (the type `CoreRowsGroup` excludes it, `after` is gone): the four switches
  appear once. `Reopen Fresher` appears next to `[Open System Settings]` of Full Disk Access once the pane was opened;
  `relaunch()` is called and never awaited.
- Decision: the group renders nothing until the core has answered `get_access_status`, so no heading over an empty
  panel. The chrome key `ui.permissions.check-again` was added; the other labels existed from 04/02.
- Tests: `permissions-group.test.tsx` (5 negative, 6 positive, through the settings screen). Gallery: stories
  `PermissionRows/every-state` and `PermissionRows/all-granted`; their screenshots were looked at and committed.
- Final review fixes: a `denied` opt-in or volume row offers "Open System Settings" with `core.access.<id>.denied`
  (the opt-in keeps its switch); the heading, "Check again" and the opt-in switches (the core's settings of the
  group `permissions`) show before the core has answered the status, the state rows once it has; a tool row with no
  note has no empty description. Tests: `a_tool_without_a_note_has_no_empty_description`,
  `a_failed_check_again_reports_the_failure`, `a_failed_open_system_settings_reports_the_failure`,
  `before_the_core_answers_no_state_is_shown`, `the_heading_and_the_switches_show_before_the_core_answers`,
  `a_denied_opt_in_offers_system_settings_with_its_text`, `a_denied_volume_offers_system_settings_instead_of_its_steps`;
  the settings screen's list of headings now includes Permissions.
