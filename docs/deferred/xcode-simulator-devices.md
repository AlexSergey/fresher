# Xcode's simulator devices and runtimes

**Agreed:** 2026-11-22, by the user's delegation (plan [031](../plans/031-xcode/readme.md), agent's call 1) · **Due
at:** when a way to list simulator devices that can never start an installer or a system dialog is found, or when the
user decides

## What is postponed

The unavailable simulator devices (the draft's category `xcode.unavailable-devices`, Medium: a device whose runtime is
no longer installed, removed with `xcrun simctl delete <id>`, never forced) and the simulator runtimes. Plan 031
builds Xcode's device support files, previews, simulator caches and archives without them.

## Why (facts of 2026-11-22 on the developer's Mac, read-only)

- `simctl` in Xcode 27.2 is a shell script: when the CoreSimulator framework is older than the one it expects, it runs
  `xcodebuild -runFirstLaunch`, an installer of system components, before the framework's own `simctl`. A listing
  (`xcrun simctl list -j devices`) started that installer; it was stopped at once and nothing was installed. A scan
  must never start an installer (the app never runs as administrator; rule 1).
- An older framework's answer may call a device unavailable that the current Xcode would run, so a working device
  could be offered for deletion.
- `xcrun` on a Mac without the developer tools opens the system's install dialog; a module's tool check runs at the
  start of every scan.
- Every device there (`~/Library/Developer/CoreSimulator/Devices/<device id>/device.plist`) named a runtime that was not
  installed: 2.1 GB the app could have offered.

## What doing it needs

1. A listing that can never start an installer or a dialog, proven on a Mac before and after an Xcode update: for
   example, the devices read from each `device.plist` and the installed runtimes from files only, or the framework's
   own `simctl` called only when its version is the one Xcode expects.
2. The sandbox's tool world (`[[world.tool]]`, item 2 of [`sandbox-world.md`](./sandbox-world.md)), which plan
   [032](../plans/032-docker/readme.md) builds.
3. A removal by the tool (`Removal::ToolCommand`, `Subject::External`): the cleaner has it (`remove/tool.rs`) and
   never runs a tool in a sandbox (`Reach::SandboxFolder`), so a case file expects `skipped` with `not-in-sandbox`.
4. The confirmation's tool sentence ("Removed by its own tool, such as Docker. It cannot be put back.") written
   without Docker, since plan 032 removes nothing of Docker's.
