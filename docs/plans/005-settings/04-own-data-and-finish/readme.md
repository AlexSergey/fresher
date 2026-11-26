# 005/04 — Own data and finish

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** app, app/ui, docs, repo

The "Your Fresher data" and "About" groups, then the docs, the version, the end-to-end flows and the conclusions. The
design: [`../00-design.md`](../00-design.md), sections 5–6.

Depends on 01–03.

## Steps

| #   | Step                                             | Status      | Shared files                                              |
| --- | ------------------------------------------------ | ----------- | --------------------------------------------------------- |
| 01  | [Own data and About](./01-own-data-and-about.md) | ✔️ done    | `app/tauri/src/`, the settings screen, the bindings       |
| 02  | [Finish](./02-finish.md)                         | ✔️ done    | `docs/`, the version files, `app/ui/e2e/`                 |

In order.

## Conclusions

Built as written, with the version from the user's decision of 2026-11-16 (0.5.5, not 0.6.0).

- `get_own_data` answers `OwnData { total, budget }` from `DataFolder::footprint`; the app manages its `DataFolder`
  beside the engine. "Show in Finder" is left out until plan 006/03 gives the app a real `SystemUi`.
- **Reset all settings…** asks in a dialog whose first focus is "Keep my settings". About shows the version and the
  app's licence; the dependencies' licences stay [deferred](../../../deferred/licence-notices.md).
- Docs: the architecture's settings and ignore list, a deletion-safety rule ("Settings never weaken deletion
  safety"), the tests, the ideas marked as taken, a known issue
  ([a change during a scan](../../../known-issues/fixed/core-settings-changed-during-a-scan.md)), four deferred items.
- The version files are at 0.5.5; plan 009 targets 0.5.5 too, so the second branch merged keeps it.
