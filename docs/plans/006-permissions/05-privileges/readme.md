# 006/05 — Privileges

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, engine, app, app/ui, docs

The app never runs as administrator. This plan makes that visible and safe: every finding knows whether the app can
remove it, an item that needs an administrator (or is locked, or protected by macOS) is shown with its size and a
Finder instruction and can never be selected, and the developer docs say how permissions work and how to test them.
The design: [`../00-design.md`](../00-design.md), sections 9, 10 and 11.

Step 01 depends on 02/01 and on plan 004/03 (the walk's entry facts); step 02 on 01, 03/03 (`MacSystemUi::reveal`)
and 03/04 (the command pattern); step 03 on everything else. Refreshed against the code on 2026-11-19.

## Steps

| #   | Step                                                       | Status   | Depends on         | Shared files                                                                    |
| --- | ---------------------------------------------------------- | -------- | ------------------ | ------------------------------------------------------------------------------- |
| 01  | [Removability of findings](./01-removability.md)           | ✔️ done | 02/01, 004/03      | `crates/core/src/{lib,finding}.rs`, `crates/engine/src/session.rs`, `crates/core/locales/en.json` |
| 02  | [Needs an administrator, in the UI](./02-admin-items-ui.md) | ✔️ done | 01, 03/03, 03/04  | `crates/engine/src/engine.rs`, `crates/core/src/error.rs`, `crates/core/locales/en.json`, `app/tauri/src/{commands,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, `app/ui/src/product/{finding-row,finding-list}/`, `app/ui/src/screens/module/category-section.tsx` |
| 03  | [Docs and conclusions](./03-docs-and-version.md)           | ✔️ done | all                | `docs/architecture/{README,modules,permissions}.md`, `docs/restrictions/`, `docs/testing.md`; no version file (the version stays 0.1.0) |

## Conclusions

All three steps are done. The item left open by step 01 when plan 004 was taken in —
the scanner's walk gave every match `EntryFacts::default()` — is closed: the walk reads the entry facts (step 01's
Conclusions, "The walk reads the entry facts"). Every match of a full scan, a rescan and a restore carries the true
facts of its entry, from both readers alike; the scanner's comparison with `NaiveWalk` includes them; the golden
reports show the scanned findings as `removable` (only the `demo` module's made-up paths stay
`needs-administrator`). Measured on the SMALL bench tree: no change beyond the noise of a loaded machine in an
interleaved A/B run; the quiet run against the saved baseline was +4 to +9 % at 12 threads, under the 10 % bar. The
cache's tree format went from 1 to 2: an old cache is discarded (one full scan).

Step 02 made the removability visible: an item the app may not remove has no checkbox, says why in a badge and, where
the user can remove it by hand, offers "Show in Finder" by the finding's id (`reveal_finding`, never a path). Step 03
wrote the contract: [`docs/architecture/permissions.md`](../../../architecture/permissions.md) and the rule "No
administrator rights" in [`docs/restrictions/deletion-safety.md`](../../../restrictions/deletion-safety.md). The app
never elevates; nothing in the plan changed that. The version stays 0.1.0.
