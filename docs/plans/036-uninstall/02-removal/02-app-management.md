# 036/02/02 — App Management

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, macos, system, engine

macOS guards other developers' app bundles with the App Management permission, and no call tells its state without
trying. Fresher never probes it: the capability is `NotAsked` until the first uninstall, then learned from what the
move did, for the session. An app's page says what may happen before the first uninstall and what to do after a
refusal, with **Open System Settings** at the right pane. The design: [`../00-design.md`](../00-design.md), section 8.

## Before you start

Read `docs/architecture/permissions.md` (all), `crates/core/src/access.rs` (`Capability`, `CapabilityText`, ids and
their tests), the core texts' test `every_capability_has_its_texts` (`crates/core/src/texts.rs`),
`crates/macos/src/privacy/panes.rs`, `crates/system/src/access.rs` (`MacAccess`: which capabilities it answers and how
an unknown one is answered), `crates/system/src/system_ui.rs`, `crates/engine/src/access.rs` (`AccessView`: how a
tool's state is merged in), `crates/engine/src/engine.rs` (`open_access_settings`), `crates/sandbox/src/world.rs` (the
`[access]` keys: a new capability must not break a world that does not name it), and 02/01's `operation.rs` changes.

## Files

- `crates/core/src/access.rs` — `Capability::AppManagement` ("Moving other developers' apps"), its id
  `app-management`, in every list of capabilities a test or the status walks (follow `OtherAppsData`); it is no place
  and no switch.
- `crates/core/locales/en.json` — `core.access.app-management.{name,unlocks,how,missing,denied}`:
  - `name` "App Management"; `unlocks` "Lets Fresher move apps you uninstall to the Trash.";
  - `how` "1. Open System Settings. 2. Go to Privacy & Security, then App Management. 3. Turn on Fresher.";
  - `missing` "Fresher may not move apps yet. macOS asks the first time you uninstall one.";
  - `denied` "macOS did not let Fresher move an app. Turn on Fresher under App Management in System Settings, then
    uninstall it again.";
  - `core.apps.management.not-asked` "macOS may ask you to let Fresher manage apps. Fresher needs it only to move the
    apps you uninstall to the Trash."
- `crates/macos/src/privacy/panes.rs` — `Pane::AppManagement` with the address
  `x-apple.systempreferences:com.apple.preference.security?Privacy_AppBundles` (from documentation, unverified: a fact
  for the user's walk, 05/01); its test pins the address.
- `crates/system/src/access.rs` / `system_ui.rs` — `MacAccess` answers `NotAsked` for `AppManagement` and never tries
  it; opening its pane goes to `Pane::AppManagement`.
- `crates/engine/src/apps/management.rs` (new) — `pub(crate) struct Management(Mutex<Access>)`, `NotAsked` at start;
  `learn(outcome)` after an uninstall item ends: `Failed(TrashRefused)` → `Denied` (the bundle was the user's and its
  folder writable, so only the system's guard refuses), `Moved` → `Granted`; any other outcome changes nothing.
  Called from 02/01's `Run::end` for a `Purpose::Uninstall` plan; `AccessChanged` is published when the state changes.
- `crates/engine/src/access.rs` — `AccessView` puts the learned state in the status for `AppManagement`, as it does
  for a tool's.
- `crates/engine/src/apps/page.rs` — `AppDetail.notice`: `core.apps.management.not-asked` while `NotAsked`, the
  capability's `denied` text while `Denied`, none when `Granted` or for an app that is not offered.

## Produces

The capability and the page's notice for 03/02 (the notice and its button) and 04/02 (the learned state).

## Checks

- `crates/core/src/access.rs` and `texts.rs`: `every_capability_has_its_texts` covers it; `the_app_management_id_
  round_trips`.
- `crates/macos/src/privacy/panes.rs`: `app_management_opens_its_own_pane`.
- `crates/system/src/access.rs`: `app_management_is_never_tried` (the probe's tries leave it `NotAsked`).
- `management.rs`, `mod negative`: `it_is_not_asked_until_an_uninstall`, `a_skipped_or_cancelled_uninstall_teaches_
  nothing`, `an_ordinary_cleanup_teaches_nothing`; `mod positive`: `a_refused_move_of_the_bundle_is_denied`, `a_moved_
  bundle_is_granted`, `the_change_is_published_once`.
- `page.rs`: `the_page_says_macos_may_ask_before_the_first_uninstall`, `the_page_says_what_to_do_once_denied`,
  `no_notice_once_granted`.
- `crates/sandbox`: an existing world without the capability still reads (`cargo nextest run -p fresher-sandbox`).
- `cargo nextest run -p fresher-core -p fresher-macos -p fresher-system -p fresher-engine`; clippy; fmt; regenerate
  `bindings.generated.ts` (`cargo nextest run -p fresher-app --test bindings`).

## Conclusions

- Built: `Capability::AppManagement` (never probed, `NotAsked` until the first uninstall), its texts under
  `core.access.app-management.*` (the existing scheme, not `core.capability.*`), `Pane::AppManagement`
  (`Privacy_AppBundles`, unverified), `apps/management.rs` learning `Granted` from a moved bundle and `Denied` from a
  Trash refusal, one `AccessChanged` before the cleanup's events. Engine 664 and 2076 across the crates.
- Over-broad by construction: every Trash refusal of the bundle teaches `Denied`. After the final review a `Partly`
  failure whose bundle moved teaches `Granted`, a bundle in `~/Applications` teaches nothing (App Management is not
  involved there), and an uninstall is never deleted permanently
  ([`../../deferred/app-management-denied-from-not-allowed.md`](../../deferred/app-management-denied-from-not-allowed.md)).
- Settings now shows an App Management row; its NotAsked state shows the "missing" sentence with no button.
