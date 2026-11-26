# 043 — Build flags, and High and Largest back on

**Status:** 📝 draft · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, app/tauri, app/ui, scripts, e2e,
docs

A one-file draft with the recommended decisions taken. Asked by the user on 2026-11-26 after the first tester's
feedback ([`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)): "a more advanced
system of feature flags, set before the build (look at how it is today and improve it, as its own plan); turn Large and
the High level back on". Goes before [044](../044-largest-folders/readme.md), which needs Largest shown.

## How it is today

- **High** is one hard-coded value: `compose.rs` builds every engine with `HighLevel::Withheld` (plan 042), the same in
  release and development builds; tests and the sandbox name it explicitly. The engine refuses every High finding and
  every way to remove one.
- **Largest** has no switch of its own: the sidebar shows it only while `get_app_info` reports `high: offered`
  (`frame-sidebar.tsx`). Its files and folders carry High ticks, so the two were tied on purpose.
- **Modules** are chosen by `ModuleSet` (`Every`, `Shipped` without the Debug group, `Only(id)` for a case file's
  autotest); a debug build adds the Debug group by `cfg(debug_assertions)`.
- Nothing lists what a build ships with; turning one thing on means editing code, and the release notes cannot say
  which switches were on.

## Goals

1. One place, read before the build, that says what a build ships with: `flags.toml` at the repository root, one
   table per flag with its value for a release and for a development build, and a line saying what it does.
2. Each flag is a typed value in the core's composition (`Flags` struct with enums, never strings at the use site),
   given to the engine and reported to the UI through `get_app_info`; the UI hides or shows a screen only from it.
3. A release build may override a flag only from the command line of the build (`FRESHER_FLAGS=largest=off`), never
   from a file the user can write or a setting: a flag is the build's, not the user's.
4. The first flags: `high` (offered / withheld) and `largest` (shown / hidden), apart: Largest no longer follows High.
   0.2.0 ships both on: High offered, Largest shown.
5. A guard (`npm run check:flags`, in the pre-push checks) checks that every flag in `flags.toml` has a doc line, both
   values, and a use in the code; the release workflow prints the flags into the release notes' draft.
6. The e2e core mock and the sandbox take the flags like the app, so a test names the flags it runs with.

## Decisions taken (recommended)

- **Build time, not runtime.** A build script of `app/tauri` reads `flags.toml` (and `FRESHER_FLAGS`) and generates
  the `Flags` constants; nothing reads the file at runtime. Reason: what can delete at High must not be switchable by a
  file in the home folder.
- **Development builds follow release by default** (as plan 042 chose for High), so the developer sees what ships; the
  debug panel ([deferred](../../deferred/dev-debug-panel.md)) may later switch a flag in a debug build only.
- **Modules are not flags yet**: `ModuleSet` stays; a flag per module is added when a half-built module must stay out
  of a release (YAGNI).
- **Turning High on again** re-runs the High parts of plan 042's walk: the "Only if you are sure" group, the High tick
  in the confirmation, Largest's ticks; `docs/test-cases/` gets a case for each.

## Plans

1. `flags.toml`, the build script and the `Flags` type; `compose.rs` reads `high` from it; `get_app_info` reports the
   flags; tests for both values.
2. The UI: the sidebar shows Largest from `flags.largest`; the e2e mock and the gallery take flags; the guard
   `check:flags`; `docs/architecture/` and `docs/quality.md` describe the flags.
3. Turn High and Largest on for 0.2.0: baselines, test cases, the release notes' line; the user's walk.

## Agent's calls to check

- The file's name and place (`flags.toml` at the root) and the override's form (`FRESHER_FLAGS`).
- Development builds follow release, not "everything on".
