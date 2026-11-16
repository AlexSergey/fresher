# 006/05 — Privileges

**Status:** 🔨 in progress · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, engine, app, app/ui, docs

The app never runs as administrator. This plan makes that visible and safe: every finding knows whether the app can
remove it, an item that needs an administrator (or is locked, or protected by macOS) is shown with its size and a
Finder instruction and can never be selected, and the developer docs say how permissions work and how to test them.
The design: [`../00-design.md`](../00-design.md), sections 9, 10 and 11.

Step 01 depends on 02/01 and on plan 004/03 (the walk's entry facts); step 02 on 01, 03/04 and plan 003; step 03 on
everything else.

## Steps

| #   | Step                                                       | Status   | Depends on         | Shared files                                                                    |
| --- | ---------------------------------------------------------- | -------- | ------------------ | ------------------------------------------------------------------------------- |
| 01  | [Removability of findings](./01-removability.md)           | ✔️ done | 02/01, 004/03      | `crates/core/src/{lib,finding}.rs`, `crates/engine/src/session.rs`, `crates/core/locales/en.json` |
| 02  | [Needs an administrator, in the UI](./02-admin-items-ui.md) | ✅ approved | 01, 03/04, 003    | `crates/engine/src/engine.rs`, `crates/core/src/error.rs`, `app/tauri/src/{commands,lib}.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json`, `app/ui/src/ipc/`, the module page |
| 03  | [Docs and version](./03-docs-and-version.md)               | ✅ approved | all                | `docs/architecture/README.md`, `docs/restrictions/`, `docs/testing.md`, the version files |

## Conclusions

Not written yet for the plan (steps 02 and 03 are open). The item left open by step 01 when plan 004 was taken in —
the scanner's walk gave every match `EntryFacts::default()` — is closed: the walk reads the entry facts (step 01's
Conclusions, "The walk reads the entry facts"). Every match of a full scan, a rescan and a restore carries the true
facts of its entry, from both readers alike; the scanner's comparison with `NaiveWalk` includes them; the golden
reports show the scanned findings as `removable` (only the `demo` module's made-up paths stay
`needs-administrator`). Measured on the SMALL bench tree: no change beyond the noise of a loaded machine in an
interleaved A/B run; the quiet run against the saved baseline was +4 to +9 % at 12 threads, under the 10 % bar. The
cache's tree format went from 1 to 2: an old cache is discarded (one full scan).
