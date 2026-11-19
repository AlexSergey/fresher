# Downloads and installers

**Since:** 2026-11-17 · **Plan:** [020](../plans/020-downloads/readme.md) (draft)

## Problem

The Downloads folder collects what the user thinks of as temporary: downloads that never finished, and installers
(`.dmg`, `.pkg`) of apps that are long installed. Nothing removes them, and they can take gigabytes. They were kept out
of the temporary-files module on purpose (the user's decision of 2026-11-17, plan
[017](../plans/017-temp-files/readme.md)): they live among the user's own files, and some can still be resumed.

## Direction

A module of its own, "downloads and installers":

- **Unfinished downloads**: `.crdownload`, `.part` (with its empty placeholder carrying the final name, removed
  together or not at all), `.opdownload`, `.partial` — Low (the file can be downloaded again), offered only when the
  browser that writes that suffix is not running and the file is older than a number of days.
- **Safari's resumable downloads**: a `.download` folder bundle (its `Info.plist` and the data) can be resumed with a
  double-click — Medium, since removing it loses the resumable part.
- **Old installers of installed apps**: a `.dmg` or `.pkg` whose app (by bundle id inside the image or the package, if
  it can be read without mounting or running anything) is installed at the same or a newer version — Medium at first;
  an installer of an app that is not installed is the user's file and never offered.
- **Large downloads untouched for a long time**: any big file in Downloads not opened for a number of days — "your
  call", never pre-selected, since it is the user's own file and only the user knows if it is still needed.
- **Old macOS installers**: `Install macOS <name>.app` in `/Applications` or Downloads, kept after an upgrade, about
  12–15 GB each — Medium, offered when the installed system is the same or newer.
- **Downloads is protected by the system's privacy prompt**: the module reads it only after the user granted access
  (plan [006](../plans/006-permissions/readme.md)); without it, nothing is claimed and the access status says why.

Open questions: whether to look outside `~/Downloads` (the Desktop, a browser's chosen folder); how to read an
installer's bundle id without mounting the image; the age for each kind.
