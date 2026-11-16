# 006/04/02 — Coverage notices

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app/ui, core (texts)

## Files

- `app/ui/src/screens/access/coverage-notice.tsx` — one `Notice` per capability that limited the last scan, under the
  result summary and nowhere else: the count and the place ("312 folders could not be read." / "Desktop, Documents and
  Downloads were not looked at."), what the capability unlocks, and one action:
  - a permission the user grants in System Settings → `PermissionCard` with `[Open System Settings]`
    (`openAccessSettings`) and a collapsible "How" (the numbered steps of `core.access.<id>.how`);
  - a guarded folder → `[Let Fresher look there]`, which turns its `core.access.*` switch on (`updateSetting`) with the
    description "macOS will ask you once for each folder. Fresher reads names and sizes only."; the notice then says
    "Scan again to look there.";
  - after a relevant grant (`hasStaleResult`) → "Scan again" in place of the action; the app does not rescan itself.
  - if plan 01 found that a grant needs a relaunch → "Reopen Fresher to finish" with `[Reopen]`.
- `app/ui/src/screens/access/coverage-notice.test.tsx`.
- `crates/core/locales/en.json` — the notice texts, if 02/01 did not already add them (keys `core.coverage.*`).

## Produces

The honest "what was not seen" of every result (goal 1 of the design), and the card plan 014 reuses on its first-run
step.

## Checks

- Negative (first): `no_notice_for_a_complete_scan`, `no_notice_for_a_capability_that_is_not_applicable`,
  `no_notice_before_any_scan`, `the_open_settings_button_is_not_the_default_action` (Enter does not trigger it),
  `a_notice_never_names_a_path`.
- Positive: `unreadable_folders_offer_full_disk_access_with_how`, `skipped_folders_offer_the_opt_in`,
  `the_opt_in_turns_on_its_setting_and_asks_to_scan_again`, `a_relevant_grant_turns_the_action_into_scan_again`,
  every notice has its accessible name and is announced once (a polite live region).
- The gallery shows every notice in every state; Playwright screenshots are added and looked at.
