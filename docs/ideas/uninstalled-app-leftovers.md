# Uninstalled app leftovers

**Since:** 2026-11-15

## Problem

Dragging an app to the Trash removes only the `.app` bundle. Its settings, caches, logs, containers and support files
stay in `~/Library` forever, often gigabytes, and nothing tells the user they belong to an app that is gone.

## Direction

- Build the list of installed apps: bundles in `/Applications`, `~/Applications` and their subfolders, plus whatever
  Launch Services knows about (apps in other places, Setapp, App Store), with each app's bundle identifier
  (`com.jetbrains.WebStorm`) and name.
- Scan the places apps leave data in, and tie every folder and file to an app by its bundle identifier or name:
  - `~/Library/Application Support/<name or id>`
  - `~/Library/Caches/<id>`, `~/Library/Logs/<name>`
  - `~/Library/Preferences/<id>.plist`, `~/Library/Saved Application State/<id>.savedState`
  - `~/Library/Containers/<id>`, `~/Library/Group Containers/<team>.<id>`
  - `~/Library/HTTPStorages/<id>`, `~/Library/WebKit/<id>`, `~/Library/Cookies/<id>.binarycookies`
  - `~/Library/LaunchAgents/<id>*.plist` (a leftover agent may still try to start a missing app)
- When the app is no longer installed, offer its leftovers as one finding per app: the app's name, when it was last
  used, and each leftover with its size and danger level.
- Danger levels per kind: caches, logs, saved state — Safe; preferences, application support, containers — Medium
  (the user loses settings and possibly data if they reinstall later); launch agents — Medium.

## Risks — when a rule must NOT claim a folder

- The app is installed somewhere unusual (a custom folder, an external drive): Launch Services' registry is checked
  besides the standard folders, so an app outside `/Applications` is still found as installed.
- Command-line tools and Homebrew formulas have no bundle (`~/Library/Application Support/<tool>`): match only by
  bundle identifier, never by a loose name guess.
- Shared folders: a group container or a support folder used by several apps of one vendor stays while any of them is
  installed.
- Apple's own (`com.apple.*`) and system components are never offered.
- A folder whose owner cannot be told for sure is shown as unrecognized, not as a leftover (rule 1 of
  [`../rules.md`](../rules.md)).

Decisions (the user, 2026-11-15):

- **Detection from the current state only.** "If the app is removed, it is removed": no snapshot history is needed to
  call an app uninstalled.
- **A one-click "keep settings, remove caches only" choice**, for a user who may reinstall the app. Such a choice gets
  a very clear, short, plain-English description, for example:
  - *Remove everything* — "Deletes all files this app left behind. If you install it again, it starts fresh."
  - *Keep settings* — "Deletes only caches and logs. Your settings stay, in case you install the app again."

Open questions:

- Should it also detect leftovers of apps the user removed with a vendor's own uninstaller (often incomplete)? The same
  rule covers them.
