# 009 — System knowledge: apps, projects, processes

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, macos, system, scanner, rules,
engine, cleaner, testing, app

Approved on 2026-11-16 by delegation: the user asked the agent to take the recommended option on every open question
and record it ([`00-design.md`](./00-design.md), "Decisions"). The agent's calls wait for the user's check.

**Built and checked on 2026-11-16; the user looked at the result and asked for the squash-merge the same day**
([`../README.md`](../README.md), "Design"). The agent's calls stay open for the user's check.

## Goal

The facts several modules need about the Mac — where the system keeps app data, which apps are installed (by bundle
id and team id), which apps are running, where the projects are — become shared services behind ports of the core,
gathered at most once per scan and only when asked, each with a fake. Modules name places by known folders, never by
macOS paths; the cleaner skips an item that belongs to a running app; an external tool can be stopped.

Branch: `feat/009-system-knowledge`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [002](../002-base-architecture/readme.md), [004](../004-fast-scan/readme.md) and
[008](../008-safe-removal/readme.md) (done). Plan [005](../005-settings/readme.md) is built in parallel; this plan
stays out of its settings files.

## Plans

| #   | Plan                                            | Status      | Depends on | What                                                                                   |
| --- | ----------------------------------------------- | ----------- | ---------- | -------------------------------------------------------------------------------------- |
| 01  | [Known folders](./01-known-folders/readme.md)   | ✔️ done     | —          | `KnownFolders`, `Pattern::Under` instead of `UnderHome`, the crate `fresher-system`    |
| 02  | [Contracts](./02-contracts/readme.md)           | ✔️ done     | 01         | apps, running apps, projects, `SystemFacts` in the module context, stoppable commands  |
| 03  | [Installed apps](./03-installed-apps/readme.md) | ✔️ done     | 02         | the `Info.plist` reader, the team id reader, `MacInstalledApps`                        |
| 04  | [Running apps](./04-running-apps/readme.md)     | ✔️ done     | 03         | `MacRunningApps`, app ownership in the cleaner's check 10                              |
| 05  | [Projects](./05-projects/readme.md)             | ✔️ done     | 02         | projects collected from the shared walk                                                |
| 06  | [Finish](./06-finish/readme.md)                 | ✔️ done     | all        | docs, version 0.5.5, the checks, conclusions                                           |

In order; 03/01 and 03/02 (pure readers) may be built in parallel with 05.

## Ideas this touches

[macOS specifics](../../ideas/architecture/macos.md) ("finding what belongs to an app"),
[rules as data](../../ideas/architecture/rules-as-data.md) (project artifacts),
[stale project cleanup](../../ideas/stale-project-cleanup.md), [application footprint](../036-uninstall/readme.md),
[uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md), the
[smart uninstaller](../036-uninstall/readme.md). None is built here: they are the consumers.

## Conclusions

Built on `feat/009-system-knowledge` in one session on 2026-11-16, step by step, every commit through the hooks.

- **What is there now**: the core's ports `KnownFolders`, `InstalledApps` and `RunningApps` with `SystemPorts` and
  the lazy, per-scan `SystemFacts` (`cx.system()`); `Pattern::Under(KnownFolder, path)` instead of `UnderHome`; the
  projects of the walk; `CommandRunner` with a cancel token. The new crate `fresher-system` holds `MacKnownFolders`,
  `MacInstalledApps` (our own `Info.plist` and code-signature readers), `MacRunningApps`, `SystemClock` and
  `ProcessCommandRunner`. The cleaner's check 10 also skips an item a running app owns by bundle id.
- **Reused, not duplicated**: `Clock`, `CommandRunner` and `Volumes` stayed the ports plans 002/004/008 made;
  `MacRunningApps` reads the process table through `fresher-macos`'s existing `libproc` calls (no new `unsafe`); the
  projects ride on the one shared walk; the scanner's anchor resolver took the known folders as plan 004 foresaw.
- **Changed from the design while building** (each in its step's conclusions): the cleaner's tool removal runs to its
  end rather than taking the cleanup's token (02/03); the projects' artifacts come from their own `FolderName`
  interests, since a marker's sibling names are read beside the matched folder (05/01); the readers were committed
  with their first user (03).
- **Measured**: listing this Mac's `/Applications` (31 entries) and `/System/Applications` with team ids and helpers
  takes 0.25 s in the instrumented test build; the process table 0.06 s. Both run only when asked, at most once per
  scan.
- **Effects to know**: the fingerprint of the interests changed (known folders, the engine's project interests), so
  the first scan after this change reads every folder once; every scan now also reports the projects' markers and
  artifact folders to the engine (`Claim::Item`, no module sees them).
- **Checks on the last code commit (85f748b)**: `npm run pre-push` passes (lint, crate guard, knip, machete, deny, the scripts' 25
  tests; 1536 Rust tests, 4 skipped, 97.4 % of lines; 828 UI tests, 98.6 % of lines). `npm run test:e2e`: the core's
  30 pass; the UI's 213 pass on the second run — the first run had one gallery screenshot (`AppShell/narrow`) differ
  by a few anti-aliased pixels at the sidebar's edge, a flake of a screen this plan does not touch.
- **Left**: the volume facts ([deferred](../../deferred/volume-facts.md), plan 012); a project's precise last activity
  ([deferred](../021-stale-projects/00-design.md)); the known issue of other users' processes is narrowed, not
  closed ([known issue](../../known-issues/cleaner-other-users-processes.md)). No module uses the facts yet: the first
  real module is the first consumer. No shortcut was taken (nothing in `docs/hacks/`).
- **For the user to check**: the design's agent's calls 1–11, and the version decision (0.5.5, the user's).
