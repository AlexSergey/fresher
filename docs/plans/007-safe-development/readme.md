# 007 — Safe development: sandbox, emulation and debug tools

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** testing, cleaner, engine, app, repo

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "We also need debug and emulation tools for tests and safe development — so that we do not
wreck something by accident during development."

The product deletes files, and it is developed on the developer's own Mac, partly by agents. A bug or a careless run must
not be able to touch real files.

## Goals

1. By default a development build and every test work on a made-up home folder, never on the real one.
2. Real deletion is physically impossible outside a sandbox in development builds and in tests — by construction, not by
   care.
3. Everything outside the process can be emulated: the disk, Docker, the Trash, permissions, installed apps, running
   processes, the clock, changes on disk over time.
4. What the app is doing can be seen: events, why something matched, what a cleanup would do.

## Non-goals

- Tools for end users (the diagnostics the user sees are plan [011](../011-diagnostics/readme.md)).
- Anything of this in a release build.

## Design

### Layers of protection

| Layer                 | What it does                                                                                           |
| --------------------- | ------------------------------------------------------------------------------------------------------ |
| Sandbox marker        | In debug builds and in tests the cleaner really removes only under a folder that holds a sandbox marker file; anywhere else it refuses. The check is on the physical path. |
| Dry run by default    | A development run against the real disk is read-only: the cleaner reports what it would do. Real scanning needs an explicit flag; real cleaning in a debug build does not exist. |
| A separate identity   | Development builds use their own app id and data folder, so they never touch the settings and cache of an installed Mac Fresher. |
| Test environment      | The test runner sets `HOME` to a temporary folder; a test that forgets to inject a root finds an empty home, not the real one. |
| Lints                 | Besides the deletion lint: looking up the home folder and starting processes are allowed in one place each; everything else gets its roots and its command runner injected. |
| A visible state       | A development window always shows a banner: "Sandbox" or "Real disk · read-only".                      |

### Emulation

- **Scenarios**: one TOML file describes a whole world — the home tree (sparse files, so 10 GB costs nothing),
  installed apps, Docker objects, running processes, permissions, the date, and a script of later changes ("a week
  passes, this cache grows by 2 GB"). They live in `fixtures/scenarios/`.
- One scenario serves the unit and end-to-end tests, the development app, the UI gallery and screenshots, and the demo
  mode — the same world everywhere.
- **Fakes behind the ports** in `fresher-testing`: the Trash, the Docker CLI (scripted answers), permissions, apps and
  processes, the clock, the source of disk changes. Plan 001/04 starts them; this plan completes the set.
- **A generator of large trees** (millions of entries) for the performance benches.

### Debug tools

- **A debug panel** in development builds: pick a scenario, flip a permission, move the clock and apply the scripted
  changes, watch the event stream, see why an item matched or was skipped, preview a cleanup as a dry run.
- **A developer command-line tool** (a dev-only crate, not bundled): scan a folder and print the report, explain a path,
  build a scenario, run a bench, print a cleanup plan as a dry run. It lets an agent or a script check behaviour without
  the UI.

### Not in release

The panel, the fakes and the command-line tool sit behind a `dev-tools` feature; a guard checks that a release build has
none of them.

## Plans (after approval)

| #   | Plan                  | What                                                                           |
| --- | --------------------- | ------------------------------------------------------------------------------ |
| 01  | Guards                | the sandbox marker, dry run by default, the separate identity, test `HOME`, lints |
| 02  | Scenarios             | the scenario format, the loader, the change script, the large-tree generator   |
| 03  | Fakes                 | the complete set of fakes behind the ports                                     |
| 04  | Developer CLI         | scan, explain, scenario, bench, dry-run plan                                   |
| 05  | Debug panel           | the panel and the banner; the release guard                                    |

The guards (01) come right after plan 002 and before any code that deletes; the rest grows with the plans it serves.

## Ideas this touches

The demo mode, automatic screenshots and the dry run of [best practices](../../ideas/best-practices.md), the
[cleanup simulator](../../ideas/cleanup-simulator.md) (the same dry-run path).

## Decisions

None yet.

## Agent's calls (to check)

1. **A debug build cannot really clean the real disk at all**; real cleaning is tested on sandboxes and in release
   candidates.
2. **The sandbox marker** as the last line of defence, checked by the cleaner itself.
3. **One scenario format** for tests, development, the gallery and the demo.
4. **A separate app id for development builds.**
5. **Dev tools behind a feature flag**, absent from release.
6. **This plan's guards are built before the cleaner** (plan 008).
7. **Target version 0.1.0.**

## Conclusions

Not written yet.
