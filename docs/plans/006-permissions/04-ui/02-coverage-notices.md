# 006/04/02 — Coverage notices

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/ui

## What exists already

- `app/ui/src/product/permission-card/permission-card.tsx` — `PermissionCard({ access: 'denied' | 'unknown',
  onLearnMore? })`: a `Notice` with the chrome texts `ui.permission.{denied,unknown}.{title,text}` and an optional
  "Learn more"; its comment says plan 006 adds the action that opens System Settings. Stories and tests beside it.
- `app/ui/src/screens/overview/overview-screen.tsx` — after the scan hero: the generic partial notice
  (`ui.screen.overview.partial.*`) when the phase is `partial`, `RescanNotice`, then
  `{access === 'denied' ? <PermissionCard access="denied" /> : null}` whatever the result (from `useAccess()`).
  There is no result-summary block on the overview any more (`product/result-summary` is not mounted by a screen).
- The core's dictionary already holds, per system capability, `core.access.<id>.name`, `.unlocks`, `.how`,
  `.missing`, and for the four opt-ins `.label` and `.description` (`crates/core/locales/en.json`). A new core key
  would have to be emitted by the core (`no_key_in_the_file_is_unused` in `crates/core/src/texts.rs`), so this step
  adds none.
- The settings of the opt-ins: `core.access.{desktop,documents,downloads,other-apps-data}` (switches, group
  `permissions`, effect `next-scan`); `updateSetting` in `app/ui/src/ipc/index.ts`.
- Known issue [`scanner-unreadable-folder-in-a-claimed-match`](../../../known-issues/scanner-unreadable-folder-in-a-claimed-match.md):
  a folder unreadable inside a claimed match is not in the coverage, so a notice can undercount. Not fixed here.

## Files

- `app/ui/src/screens/permission/coverage-notices.tsx` (new) — one notice per capability `notSeen(coverage)` lists
  (04/01), in its order, placed in the overview where the `PermissionCard` is now and nowhere else; nothing before a
  scan, nothing for a complete coverage, nothing for a `not-applicable` capability. Each says what was not seen and
  offers one action:
  - unreadable folders → "312 folders could not be read." (chrome `ui.coverage.unreadable.one|other` with `{count}`)
    and the capability's `core.access.<id>.missing`;
  - a place not entered → `core.access.<id>.missing` ("Your Desktop was not looked at.");
  - then `core.access.<id>.unlocks`, and the action:
    - `all-files` (granted only in System Settings) → `PermissionCard` with `[Open System Settings]`
      (`openAccessSettings`, 03/04) and a collapsible "How" (the numbered steps of `core.access.all-files.how`).
      After the button was used in this session, a secondary `[Reopen Fresher]` (`relaunch`, 03/04) with "If you
      turned it on, Fresher sees it after it reopens." (the design's "Probe results": a grant applies after a
      relaunch);
    - an opt-in (`desktop`, `documents`, `downloads`, `other-apps-data`) → `[Let Fresher look there]`, which sets
      its `core.access.<id>` switch on (`updateSetting`), with `core.access.<id>.description` under it; once the
      switch is on, the notice says "Scan again to look there." in place of the button;
    - a volume → its `core.access.<id>.how` (the user opted in by adding the drive; nothing to switch);
  - after a relevant grant (`hasStaleResult`) → "Scan again" in place of the action; the app does not rescan itself.
- `app/ui/src/product/permission-card/permission-card.tsx` — takes the action and the "How" steps as props
  (`onOpenSettings`, `how`, optional `onReopen`), keeps its tone per state; its stories cover every state.
- `app/ui/src/screens/overview/overview-screen.tsx` — the `access === 'denied'` card is replaced by
  `<CoverageNotices />`; the generic partial notice stays (a partial scan may have other causes); `useAccess` is no
  longer used there.
- `app/ui/src/locales/en.json` — the chrome keys `ui.coverage.*` (count lines, buttons, "Scan again to look there.",
  the reopen line); `ui.permission.denied.*` / `ui.permission.unknown.*` and `ui.permission.learn-more` are removed if
  nothing names them any more (the chrome-keys test fails on an unused key).
- `app/ui/src/screens/permission/coverage-notices.test.tsx` (new), `permission-card.test.tsx`,
  `overview-screen.test.tsx` — the tests below; `app/ui/e2e/core-mock.ts` — its `denied` state gains a coverage so
  the screenshot shows the notice.

## Produces

The honest "what was not seen" of every result (goal 1 of the design), and the card plan 014 reuses on its first-run
step.

## Checks

- Negative (first): `no_notice_for_a_complete_scan`, `no_notice_for_a_capability_that_is_not_applicable`,
  `no_notice_before_any_scan`, `full_disk_access_denied_without_a_limited_result_shows_no_card` (no nagging),
  `the_open_settings_button_is_not_the_default_action` (Enter does not trigger it), `a_notice_never_names_a_path`,
  `reopen_is_not_offered_before_settings_were_opened`.
- Positive: `unreadable_folders_offer_full_disk_access_with_how`, `skipped_folders_offer_the_opt_in`,
  `the_opt_in_turns_on_its_setting_and_asks_to_scan_again`, `a_relevant_grant_turns_the_action_into_scan_again`,
  `reopen_calls_relaunch`, every notice has its accessible name and is announced once (a polite live region).
- `npx vitest run app/ui/src/screens/permission app/ui/src/product/permission-card app/ui/src/screens/overview`; the
  chrome-keys test; the gallery shows every notice in every state; the Playwright screenshots are added and looked at.

## Conclusions

- Built as specified: `screens/permission/coverage-notices.tsx` (one notice per `notSeen` entry, in the core's order,
  in a polite live region, each in a `section` named by the capability's `core.access.<id>.name`),
  `coverage-notice.tsx` (the notice and its action), `capability-id.ts` (capability to the core's id, and which are
  opt-ins). The overview mounts `<CoverageNotices />` where the always-on card was; `useAccess` is gone from it.
- `PermissionCard` takes its text as children plus `how`, `onOpenSettings` and an optional `onReopen`; its tone stays
  per state. `ui.permission.*.text` and `ui.permission.learn-more` were removed; the titles stay. New chrome keys:
  `ui.coverage.*`. "Scan again" reuses `ui.screen.overview.scan-again`.
- Decision: "relevant grant" is decided per notice (`accessOf(status, capability) === 'granted'` turns that notice's
  action into Scan again), not by the page-wide `hasStaleResult`.
- Decision: the numbered steps of `core.access.all-files.how` are split from the core's one-line text into a list
  (`1. a 2. b` becomes the items `a`, `b`); the core text is unchanged. Volume notices show their `how` as text.
- Decision: a capability without texts of its own (a tool) gets no notice; one whose state is `not-applicable` gets none.
- Decision: the settings are requested only when a notice offers an opt-in, so screens without one ask for nothing new.
- Decision: the gallery shows `PermissionCard` in every state (denied, reopen, unknown). The other notices are the
  generic `Notice`, which has its own stories; they read the store, so no gallery story renders them.
- The Playwright `denied` script now scans with a limited coverage; the axe and screenshot tests scan first, and the two
  `overview-access-denied` baselines were regenerated and looked at (Chromium).
- Folded in from 04/01's review: `same` in `state/access.ts` compares by `kind` and `value`; a negative test in
  `reduce.test.ts` keeps an existing coverage against a later overview's.
- Tests: the checks in `coverage-notices.test.tsx` (17 tests, negative first), `permission-card.test.tsx`
  and `overview-screen.test.tsx`; `src/screens src/product src/state src/i18n` green (616 tests), eslint and `tsc` clean.
- Final review fix: an opt-in that is switched on but `denied`, or a volume that is `denied`, was a dead end (the
  notice asked to scan again, or showed the steps that had just failed). Such a notice now offers "Open System
  Settings" (`openAccessSettings(capability)`; the engine already opens Files and Folders or Privacy & Security for
  it) with the core's new text `core.access.<id>.denied` (`CapabilityText::Denied`, one per capability, checked by the
  core's text tests). Tests: `a_denied_opt_in_no_longer_offers_the_switch_nor_a_scan_to_look`,
  `a_denied_opt_in_opens_system_settings_for_its_capability`,
  `a_denied_volume_opens_system_settings_for_its_capability`.
