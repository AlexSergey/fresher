# 006/03/04 — Commands, focus and `Info.plist`

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app, app/ui

## Files

- `app/tauri/src/commands.rs` — `get_access_status` returns the new `AccessStatus`; new `refresh_access` (returns the
  status) and `open_access_settings(capability: Capability)` (takes a capability, never an address or a path; returns
  `Result<(), Refusal>`). All `async`, like the others.
- `app/tauri/src/lib.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json` — the two commands in
  `generate_handler!`, the app manifest and the allowlist (`allow-refresh-access`, `allow-open-access-settings`); the
  existing test that reads the three files keeps them equal.
- `app/tauri/src/shell.rs` — `on_window_event`: `WindowEvent::Focused(true)` on the main window calls
  `Engine::refresh_access()` (it sends `AccessChanged` itself when something changed). If 01/01 found that a grant
  applies only after a relaunch, `relaunch` (the engine shut down first, then Tauri's `AppHandle::restart`) and a
  `relaunch` command, allowlisted the same way.
- `app/tauri/Info.plist` (new) — the five reason strings of the design, section 7, word for word, and nothing else.
- `app/tauri/tests/info_plist.rs` (new) — reads `Info.plist` as text: exactly those five `NS…UsageDescription` keys,
  each text non-empty and free of "Fresher will delete"-like promises; no `NSAppleEventsUsageDescription`, no
  `SMPrivilegedExecutables`, no other usage key.
- `app/ui/src/ipc/index.ts`, `mock.ts`, `index.test.ts` — `refreshAccess()`, `openAccessSettings(capability)` (and
  `relaunch()` if added); the mock answers every access state; `bindings.generated.ts` regenerated, never edited.

## Produces

The IPC surface plan 04 and plans 003/014 call: `getAccessStatus`, `refreshAccess`, `openAccessSettings`, the
`access-changed` and `scan-coverage` events.

## Checks

- Negative: `open_access_settings_for_a_tool_is_refused`, `the_plist_declares_no_other_usage_key`,
  `the_plist_declares_no_privileged_helper`, `a_focus_without_a_change_sends_no_event` (mocked app, `FakeAccess`), the
  UI's `openAccessSettings` passes a capability object, not a string (type test).
- Positive: `a_focus_after_a_grant_sends_access_changed`, `the_commands_are_allowlisted_everywhere`,
  `the_plist_holds_the_five_reason_strings`, the UI IPC tests for the new functions.
- By hand, in the plan's conclusions: `npm run build`; the bundle's `Contents/Info.plist` holds the five strings; the
  app opens the Full Disk Access pane; granting it, then focusing the window updates the status (or offers to reopen).
