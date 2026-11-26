# Deferred

Agreed work postponed to a known stage. Unlike [`../ideas/`](../ideas/README.md), deferred work is decided: it waits
for its turn. Before starting a new stage (a plan, a release), check whether a row here is due and remind the user.

## Format

One file per item, `<topic>.md`: what is postponed, who agreed and when, the stage it is due at, and what doing it
needs. When it is done, its row and file are removed in the same change.

| Item                                                             | Agreed     | Due at                   | What                                                                            |
| ---------------------------------------------------------------- | ---------- | ------------------------ | ------------------------------------------------------------------------------- |
| [The fast scan on the developer's home](./fast-scan-home-run.md) | 2026-11-16 | 0.2.0                    | plan 004's budget run by hand: the home folder's scan, rescans, clones          |
| [Settings export and import](./settings-export-import.md)        | 2026-11-16 | when a user asks         | Export… and Import… with a preview of the changes; today the file itself       |
| [Clearing the app's own data](./own-data-clearing.md)            | 2026-11-16 | 0.2.0                    | clear the scan cache without breaking the deletion rule (stale temporary files: done) |
| [A setting for the default removal](./default-removal-setting.md) | 2026-11-16 | when the user asks       | pre-select a permanent delete in the confirmation (rule 2 says no today)        |
| [The world outside the disk in a sandbox](./sandbox-world.md) | 2026-11-16 | the first module that reads open files or the clock | plan 007's former plan 06: open files, the rest of access and the steps between scans per case file (apps, running apps and the clock: plan 016; stated file facts: plan 017; Spotlight's answers: plan 019; the guarded folders' access and the installs: plan 020; tools and other apps' data: plan 032) |
| [The debug panel and the developer command line](./dev-debug-panel.md) | 2026-11-17 | when the deferred world is built | the panel's knobs need the world's fakes; the command line beyond the generator |
| [A log file of a session](./session-log-file.md)                 | 2026-11-17 | 0.2.0                    | an opt-in log file per session and the scan's phase timings; memory only today  |
| [The bug-report file](./bug-report-file.md)                      | 2026-11-17 | 0.2.0                    | a report the user saves and sends, and the notice after an unexpected exit      |
| [Why an item matched](./match-explanations.md)                   | 2026-11-17 | the first rule-based module that needs more than its reason | "Why this?" on a finding, and a rule's own left-out reasons |
| [The diagnostics corner](./diagnostics-screen.md)                | 2026-11-17 | 0.2.0                    | versions, permissions, scan statistics, recent problems                           |
| [A module names its app's data folders](./module-names-app-data.md) | 2026-11-17 | the next change to the map's app lines | the version in use's folders in the IDE's line of the map, not in the user's folders |
| [A folder's age from its direct children](./folder-age-from-children.md) | 2026-11-17 | plan 016's run by hand (04/02) | a removed IDE's age from the newest of its folders and their direct children |
| [A special entry shown as a file in the browse view](./special-entry-in-browse-view.md) | 2026-11-17 | the next change to the browse view's entry kinds | a socket, a pipe or a device is `EntryKind::Other` in the core but a file in the browse view's `NodeKind` |
| [Containers of an app's extensions in `app-leftovers`](./app-leftovers-containers.md) | 2026-11-18 | when `InstalledApp::helpers` covers every nested bundle | reading containers (proof, age, findings); since plan 006 the module has no `container` row |
| [The steps of a permission sent as a list](./access-steps-as-a-list.md) | 2026-11-19 | the next change to the core's access texts | the core sends `how` as a list; the notice stops parsing "1. … 2. …" with `stepsOf` |
| [Other toolchains' unused versions](./other-toolchain-versions.md) | 2026-11-21 | after plan 024's run on the developer's Mac | pyenv, SDKMAN, fnm, volta, JDKs, the Android SDK, asdf and mise installs: each needs a fact plan 024 did not build |
| [More places of installed apps' caches](./installed-app-caches-more-places.md) | 2026-11-21 | after plan 030's run on the developer's Mac | more rows of plan 034's table (web-engine apps, browsers), the browsers' support-folder caches, game launchers, the Metal shader cache, containers, Apple's apps: each needs a row checked on a real install, or a safety decision |
| [More places of logs](./logs-more-places.md) | 2026-11-22 | after plan 033's run on the developer's Mac | an installed app's logs under its bundle id, files deeper than six names, the system-wide logs |
| [The user's review of 2026-11-22](./user-review-2026-11-22.md) | 2026-11-22 | the user's next session | walks, unverified facts and agent's calls of the plans built on 2026-11-22 |
| [The user's review of 2026-11-24](./user-review-2026-11-24.md) | 2026-11-24 | the user's next session | the walk of plan 042, its rulings, texts, the logo and its known gaps |
| [More on the Largest screen](./largest-files-more.md) | 2026-11-22 | after plan 035's walk on the developer's Mac | "last opened", "the app that made it", a treemap, a sync client's folder marker, places outside the home, a folder's "Changed" from its newest file |
| [Xcode's simulator devices and runtimes](./xcode-simulator-devices.md) | 2026-11-22 | a listing that can never start an installer or a dialog, or the user's decision | unavailable devices and runtimes: `simctl` may start Xcode's first-launch installer; needs the sandbox's tool world (plan 032) |
| [Docker's other runtimes](./docker-other-runtimes.md) | 2026-11-22 | after plan 032's run on the developer's Mac | Colima, Lima, Podman machines and Rancher Desktop, one line per image or volume, "Show in Finder" for a disk file, a link from the storage map to the insight |
| [Demo scenarios for the newer screens](./demo-scenarios-for-new-screens.md) | 2026-11-22 | when a screen of plans 031, 032, 035, 036, 037 or later is wanted in the README or the release notes; the two GIFs again after the user's walk of plan 042 | one scenario file per story, the world extended with the module's scenery and its `shows`; nothing else to build; record `overview` and `cleanup` again on 042's screens |
| [More of the uninstall](./uninstall-system-files.md) | 2026-11-22 | when the user asks for it after the first release | an app's system-wide files (shown, never removed), files kept under the app's name, package managers beyond Homebrew casks, watch mode's growing apps |
| [An app's small Library files in the footprint](./uninstall-small-library-parts.md) | 2026-11-22 | after plan 036's walk on the developer's Mac | the scan folds a Library file under 1 MiB no module asks for, so a settings plist or launch agent may be missing from the footprint; the Apps view should register interests for its places |
| [App Management learned only from a refusal that means it](./app-management-denied-from-not-allowed.md) | 2026-11-22 | after plan 036's walk, or the next change to the cleaner | `Denied` is taught by any Trash refusal of the bundle; it needs a core `Failure` variant for `NotAllowed` and the cleaner's mapping |
| [The README at the first release](./readme-at-release.md) | 2026-11-22 | when a package for a package manager exists | the package manager's line in the README's Install block |
| [The disk image's window](./dmg-window-design.md) | 2026-11-26 | 0.2.0 | a drawn background with an arrow to Applications, a fixed window size and icon places, as other apps' images have |
| [Left-out items folded at the bottom of a category](./left-out-folded-at-the-bottom.md) | 2026-11-26 | 0.2.0 | the left-out lines below the offered items, folded under an accordion, so a long list is not read as what goes |
