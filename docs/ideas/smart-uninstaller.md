# Smart Uninstaller

**Since:** 2026-11-15

## Problem

Dragging an app to the Trash leaves its settings, caches, containers, helpers and launch agents behind.

## Direction

- Pick an installed app and see everything that belongs to it, matched by bundle identifier (and team identifier for
  shared group containers): the bundle, `~/Library` folders, preferences, containers, launch agents, login items,
  helper tools.
- The same matching as [`uninstalled-app-leftovers.md`](./uninstalled-app-leftovers.md), run before removal instead
  of after.
- Two choices with short, clear descriptions (rule 7 of [`../rules.md`](../rules.md)): "Remove the app and everything
  it left" / "Remove the app, keep its settings".
- The app must not be running; App Store apps and apps managed by Homebrew are removed the way their manager expects
  (`brew uninstall --cask`), never by deleting under the manager's feet.
- Apple's own apps and system components are never offered.

Open questions:

- Drag-and-drop an app onto Fresher to start (a common pattern in uninstallers)?
