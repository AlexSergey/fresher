# 003/07/05 — Settings and permission

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/settings/settings-screen.tsx` — answers "What can I change?":
  - `getSettings()` once, then `setting-changed` events keep it current (the reducer starts handling that event here:
    `app/ui/src/state/` gains the settings view and `setting-changed`, under the store's existing rules);
  - one `SettingsRow` per descriptor, the core's settings first, then each module's under its name; a change calls
    `updateSetting` and a refusal shows the core's text in that row;
  - a `Notice` when the file is read-only (`newer-version`, `unreadable`) or broken, with the core's text;
  - a footer with the app's version (`getAppVersion`), moved from the bare screen.
- `app/ui/src/screens/permission/use-access.ts` — `getAccessStatus()` once and on `access-changed` (the reducer
  starts handling it here too); the overview's `PermissionCard` reads it (07/02).
- `locales/en.json` — `ui.screen.settings.*`.

## Produces

The settings screen plan 005 builds on (groups, reset, "Rescan to apply" are its own), and the access status the
overview shows.

## Checks

- Negative: `a refused update shows the core's text in its row and keeps the old value`, `a read-only file disables
  every row and says why`, `an access-changed event of granted hides the permission card`.
- Positive: `every setting of the demo module has a row`, `a switch change is sent and the event updates the row`,
  `the version shows in the footer`.
- `npx vitest run src/screens/settings src/screens/permission src/state` passes.

## Conclusions

Done on 2026-11-16. The settings screen (`screens/settings/`), `useAccess` (`screens/permission/use-access.ts`) and
the store's settings and access, with tests (negative first); WebKit screenshots of the loaded, newer-version and
broken files were looked at. No new dependency, token or icon; 5 `ui.screen.settings.*` keys.

- **The store** keeps `settings` (`SettingsView`) and `access` (`AccessStatus`), `null` until answered.
  `requestSettings(store)` and `requestAccess(store)` (`state/connect.ts`) ask the core only while the store has none
  and nothing is on its way (a failure may be asked again; it goes to the store as `command-failed`, as every command's
  does). A `setting-changed` event replaces the value and drops that setting's problem; an `access-changed` event
  replaces the status. An answer that comes after the store already has settings or an access status is ignored: the
  events since are newer. A `setting-changed` before the settings arrived is dropped (the answer holds it).
- **`useAccess()`** gives `Access | null` (Full Disk Access); the overview (07/02) shows the `PermissionCard` for
  `denied` and `unknown`.
- **Groups**: the core's settings are every setting no module lists in `ModuleInfo.settings`, under "General"; then
  each module's, in its own order, under its name (`<module>.enabled.label`), each group a panel of `SettingsRow`s.
  The groups wait for both `get_settings` and `get_modules`.
- **A refused change**: `SettingsRow` gained one optional prop, `refusal` (the core's text), shown as the row's error in
  place of the stored value's problem; the value stays, since only the core's event changes it. An unreachable core
  goes to the store (the frame's error state). Sending again clears the row's refusal.
- **The file's notice** (warning `Notice`, not live): `newer-version` and `unreadable` make every row read-only; the
  notice's title says which (a UI text) and its text is the core's `core.error.settings-read-only`. `broken` keeps the
  rows editable (the core's first save replaces the file) and says the defaults are in use: the core has no text for
  it, so both its title and text are UI texts.
- **The version** is a `footer` line under the groups (`ui.app.version`), absent when the core does not answer.
- A stored value's `unknown-key` problem names no descriptor, so no row shows it (plan 005 may list them).

