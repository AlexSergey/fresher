# An app's small Library files in the uninstall's footprint

**Agreed:** 2026-11-22, by the user's delegation of plan [036](../plans/036-uninstall/readme.md) (the agent's ruling
while building; to check) · **Due at:** after plan 036's walk on the developer's Mac

## What is postponed

The scan folds a file under 1 MiB that no registered module has an interest in into its folder's size, so it is never
a node. The footprint reads nodes, so a settings plist or a launch agent of an app, usually a few KiB, may be missing
from an app's page, and an uninstall (even "everything") leaves it behind. The direction is safe: less is removed than
the page promises for that part. What is left is the kind of leftover the `app-leftovers` module offers once the app is
gone, if its rules claim it.

Plan 036's case files use 1 MiB parts for that reason.

## What doing it needs

The Apps view registers interests for its places (`~/Library/Preferences`, `LaunchAgents`, `HTTPStorages`) under its own
id, as the engine does for the projects, so the walk keeps the small files there as nodes. That changes the scan, so it
needs plan 004's budget run; the footprint's sizes and the golden reports change with it.
