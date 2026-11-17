# The world outside the disk in a sandbox

**Agreed:** 2026-11-16, by the user (plan 007, decision 5) · **Due at:** when the first module that reads installed
apps, running apps, open files, an external tool's answers, permissions or the clock is planned

## What is postponed

Plan [007](../plans/007-safe-development/readme.md)'s former plan 06: a case file of `fixtures/sandboxes/` also
declares what is not files, and sandbox mode and the sandbox autotests serve it through the fakes plans 008 and 009
built. The design is in plan 007's [`00-design.md`](../plans/007-safe-development/00-design.md), section 10. Until
then, sandbox mode keeps volumes, open files, running apps, commands, the clock and access real and read-only, and the
autotests use empty fakes.

## What doing it needs

1. **Installed apps on disk**: `[[app]]` in the tree vocabulary (`path` ending in `.app`, `bundle-id`, `name`,
   `version`, `build`, `helpers`) writes a bundle with its `Contents/Info.plist` as an XML property list, so plan 009's
   real `MacInstalledApps` reads the sandbox's `Applications/` as it reads a Mac's; the team id, which needs a signed
   executable, comes from the fake. The sandbox's protection list protects its own `Applications/` and
   `System/Applications/`. Checks: a bundle id that is not reverse-DNS and an app outside an apps folder are refused;
   `MacInstalledApps` over a generated sandbox lists exactly the declared apps.
2. **The declared world** (`crates/sandbox/src/world.rs`): `[world]` with `now`, `running` (names and bundle ids),
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
