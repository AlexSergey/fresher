# The world outside the disk in a sandbox

**Agreed:** 2026-11-16, by the user (plan 007, decision 5) · **Due at:** when the first module that reads open files,
an external tool's answers, permissions or the clock is planned

**Partly taken by plan [016](../plans/016-jetbrains/readme.md)** (2026-11-17), the first module that reads installed and
running apps: its plan 01 builds installed apps as bundles (`[[case.app]]`, item 1 below), the declared running apps
(`running = true`, `[[case.running]]`, `world.toml` at the sandbox's root; the running part of item 2), sandbox mode
taking its running apps from the sandbox, never the real Mac (part of item 3), and the left-out expectation
(`[[case.left-out]]`, from plan 011).

**Partly taken by plan [017](../plans/017-temp-files/readme.md)** (2026-11-18), the first module that judges by file
facts the disk cannot be made to show: a case states a file's or a folder's status change, access time, owner
(`system`, `other-user`), flags (`user-lock`, `system-lock`, `no-unlink`, `restricted`) and kind (`socket`, `pipe`),
and a link's age and status change; `world.toml` lists them as `[[entry]]` lines, and sandbox mode, the autotest, the
fixtures and the core e2e read the disk through `FactsLens`, which reports them for exactly the stated paths (the
cleaner reads the disk). The sandbox also lays out the two temporary places. Open files, tools, access and the steps
between scans stay deferred.

**Partly taken by plan [019](../plans/019-uninstalled-app-leftovers/readme.md)** (2026-11-18), the first module that
proves an app absent through Spotlight: Spotlight's answers. A case declares an app Spotlight finds outside the apps
folders (`[[case.elsewhere]]`, a name and a bundle id, answered at `Volumes/Elsewhere/<name>.app`, never written or
read) and `spotlight = "off"` (Spotlight cannot be asked); `world.toml` lists them (`[[elsewhere]]`, `spotlight`),
sandbox mode and the autotest search the sandbox's listed apps and these (`AppSearch::Declared`), never the Mac's, and
the banner says Spotlight is simulated when the world declares any of it. Open files, tools, access and the steps
between scans stay deferred.

**Partly taken by plan [020](../plans/020-downloads/readme.md)** (2026-11-19), the first module that needs a guarded
folder: the access of the guarded folders. A case states `access = { downloads = "granted" }` (`desktop`, `documents`,
`downloads`; `granted`, `denied`, `not-asked`; the most restrictive statement wins across cases); `world.toml` lists
it (`[access]`), sandbox mode and the autotest answer access from it (`World::access`: the stated folders guarded
under the sandbox's home, `not-asked` answering `granted` once the switch is on), and the standing avoid list no
longer hides the sandbox home's places of the stated folders. Full Disk Access stays `Unknown` and other apps' data
`NotAsked`; the real home's guarded places stay avoided. The banner lists what is simulated ("Simulated: open apps,
Spotlight, access, installs.").

Plan 020 also takes the installs: a case states `macos = "15.4"`, `[[case.receipt]]` (`package`, `installed`) and
`[[case.app]] changed = "10d"` (an app's change time, which the generator cannot write because every bundle is
written now); `world.toml` lists `macos` and `[[receipt]]`, sandbox mode and the autotest answer the receipts and
the version from it (`World::install_records`, never the Mac's), and `StatedInstalledApps` puts the stated change on
the installed app. Open files, tools and the steps between scans stay deferred.

**Partly taken by plan [032](../plans/032-docker/readme.md)** (2026-11-22), the first module that reads an external
tool's answer: the tools. A case states `[[case.tool]]` (a program relative to the sandbox, exact arguments, status,
inline output of at most 64 KiB); `world.toml` lists them (`[[tool]]`); the autotest answers those calls from it, fails
a case file on any call it does not state and records every other call, and **sandbox mode runs no program of the Mac
but `/usr/bin/open`**. Other apps' data is in the world's access (`access = { other-apps-data = "granted" }`). Open
files and the steps between scans stay deferred.

## What is postponed

Plan [007](../plans/007-safe-development/readme.md)'s former plan 06: a case file of `fixtures/sandboxes/` also
declares what is not files, and sandbox mode and the sandbox autotests serve it through the fakes plans 008 and 009
built. The design is in plan 007's [`00-design.md`](../plans/007-safe-development/00-design.md), section 10. Until
then, sandbox mode keeps volumes, open files, commands and access real and read-only, and the autotests use empty
fakes; the running apps and the clock's start already come from the sandbox's `world.toml` in both (plan 016), the
stated file facts too (plan 017), Spotlight's answers (plan 019), the access of the guarded folders and the installs (plan 020).
Plan [031](../plans/031-xcode/readme.md) dropped its tool category (the simulator devices are deferred, see [Xcode's
simulator devices](./xcode-simulator-devices.md)), so plan 032 was the first module that needed tools (taken, above).

## What doing it needs

1. ~~**Installed apps on disk**~~ (plan 016): `[[app]]` in the tree vocabulary (`path` ending in `.app`, `bundle-id`, `name`,
   `version`, `build`, `helpers`) writes a bundle with its `Contents/Info.plist` as an XML property list, so plan 009's
   real `MacInstalledApps` reads the sandbox's `Applications/` as it reads a Mac's; the team id, which needs a signed
   executable, comes from the fake. The sandbox's protection list protects its own `Applications/` and
   `System/Applications/`. Checks: a bundle id that is not reverse-DNS and an app outside an apps folder are refused;
   `MacInstalledApps` over a generated sandbox lists exactly the declared apps.
2. **The declared world** (`crates/sandbox/src/world.rs`, made by plan 016 for the running apps): `[world]` with `now`,
   `open-files` (sandbox paths), `access` (each capability's state), `[[world.tool]]` (a program, its exact arguments,
   its exit code, its output from a file next to the case file) and `[[world.step]]` (later changes played between two
   scans: "advance 7 days", "quit an app", "grow a file"). `World::ports()` gives `FakeClock`, `FakeRunningApps`,
   `FakeOpenFiles`, `FakeCommandRunner` and `FakeAccess` with these answers. Checks: an unknown capability, a tool
   answer for a program with a path, a step before the first scan are refused.
3. **The world in sandbox mode and in the harness**: when the sandbox's case files declare a world, sandbox mode takes
   `World::ports()` instead of the real ports and the banner adds "simulated: apps, tools, clock"; the harness plays
   the steps between scans and checks the expectations after each. Checks: a world with a running app skips that
   app's folder as in use; "advance 90 days" turns a fresh cache stale on the next scan.
4. Then the debug panel, deferred by plan 007's step 05/01, can be weighed again: its knobs are these fakes.
