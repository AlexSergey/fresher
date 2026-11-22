# Deferred

Agreed work postponed to a known stage. Unlike [`../ideas/`](../ideas/README.md), deferred work is decided: it waits
for its turn. Before starting a new stage (a plan, a release), check whether a row here is due and remind the user.

## Format

One file per item, `<topic>.md`: what is postponed, who agreed and when, the stage it is due at, and what doing it
needs. When it is done, its row and file are removed in the same change.

| Item                                                             | Agreed     | Due at                   | What                                                                            |
| ---------------------------------------------------------------- | ---------- | ------------------------ | ------------------------------------------------------------------------------- |
| [Release versioning](./release-versioning.md) | 2026-11-16 | before the first release | the numbering from 0.1.0 and an automated release (bump, changelog, tag) |
| [Licence notices](./licence-notices.md)                          | 2026-11-15 | before the first release | ship the licences of every Rust and npm dependency in the app (MPL-2.0 first)   |
| [The fast scan on the developer's home](./fast-scan-home-run.md) | 2026-11-16 | before the first release | plan 004's budget run by hand: the home folder's scan, rescans, restore, clones |
| [The first real cleanups in a sandbox](./removal-sandbox-run.md) | 2026-11-16 | before the first release | plan 008's run by hand: the real Trash, put back, Finder's Put Back, skips, quit |
| [Settings export and import](./settings-export-import.md)        | 2026-11-16 | when a user asks         | Export… and Import… with a preview of the changes; today the file itself       |
| [Clearing the app's own data](./own-data-clearing.md)            | 2026-11-16 | before the first release | clear the scan cache and stale temporary files without breaking the deletion rule |
| [A setting for the default removal](./default-removal-setting.md) | 2026-11-16 | when the user asks       | pre-select a permanent delete in the confirmation (rule 2 says no today)        |
| [The world outside the disk in a sandbox](./sandbox-world.md) | 2026-11-16 | the first module that reads open files, tools, access or the clock | plan 007's former plan 06: open files, tools, the rest of access and the steps between scans per case file (apps, running apps and the clock: plan 016; stated file facts: plan 017; Spotlight's answers: plan 019; the guarded folders' access and the installs: plan 020) |
| [The debug panel and the developer command line](./dev-debug-panel.md) | 2026-11-17 | when the deferred world is built | the panel's knobs need the world's fakes; the command line beyond the generator |
| [A log file of a session](./session-log-file.md)                 | 2026-11-17 | before the first release | an opt-in log file per session and the scan's phase timings; memory only today  |
| [The bug-report file](./bug-report-file.md)                      | 2026-11-17 | before the first release | a report the user saves and sends, and the notice after an unexpected exit      |
| [Why an item matched](./match-explanations.md)                   | 2026-11-17 | the first rule-based module that needs more than its reason | "Why this?" on a finding, and a rule's own left-out reasons |
| [The diagnostics corner](./diagnostics-screen.md)                | 2026-11-17 | before the first release | versions, permissions, scan statistics, recent problems; left-outs after a restore |
| [A module names its app's data folders](./module-names-app-data.md) | 2026-11-17 | the next change to the map's app lines | the version in use's folders in the IDE's line of the map, not in the user's folders |
| [A folder's age from its direct children](./folder-age-from-children.md) | 2026-11-17 | plan 016's run by hand (04/02) | a removed IDE's age from the newest of its folders and their direct children |
| [A special entry shown as a file in the browse view](./special-entry-in-browse-view.md) | 2026-11-17 | the next change to the browse view's entry kinds | a socket, a pipe or a device is `EntryKind::Other` in the core but a file in the browse view's `NodeKind` |
| [Containers of an app's extensions in `app-leftovers`](./app-leftovers-containers.md) | 2026-11-18 | when `InstalledApp::helpers` covers every nested bundle | reading containers (proof, age, findings); since plan 006 the module has no `container` row |
| [The steps of a permission sent as a list](./access-steps-as-a-list.md) | 2026-11-19 | the next change to the core's access texts | the core sends `how` as a list; the notice stops parsing "1. … 2. …" with `stepsOf` |
| [The core's place queries without a UI](./place-queries-without-ui.md) | 2026-11-20 | the next plan that shows the disk by folder or by app | the view by place and the by-meaning rows are gone from the UI; keep the queries or remove them |
| [Other toolchains' unused versions](./other-toolchain-versions.md) | 2026-11-21 | after plan 024's run on the developer's Mac | pyenv, SDKMAN, fnm, volta, JDKs, the Android SDK, asdf and mise installs: each needs a fact plan 024 did not build |
| [More places of installed apps' caches](./installed-app-caches-more-places.md) | 2026-11-21 | after plan 030's run on the developer's Mac | more rows of plan 034's table (web-engine apps, browsers), the browsers' support-folder caches, game launchers, the Metal shader cache, containers, Apple's apps: each needs a row checked on a real install, or a safety decision |
| [More places of logs](./logs-more-places.md) | 2026-11-22 | after plan 033's run on the developer's Mac | an installed app's logs under its bundle id, files deeper than six names, the system-wide logs |
| [The user's review of 2026-11-22](./user-review-2026-11-22.md) | 2026-11-22 | the user's next session | walks, unverified facts and agent's calls of the plans built on 2026-11-22 |
| [Xcode's simulator devices and runtimes](./xcode-simulator-devices.md) | 2026-11-22 | a listing that can never start an installer or a dialog, or the user's decision | unavailable devices and runtimes: `simctl` may start Xcode's first-launch installer; needs the sandbox's tool world (plan 032) |
