# Big downloads you have not opened

**Danger level:** High · **Rule:** none (the module's code, `modules/downloads`) · **Since:** 0.1.0

**Example:** a file of 1 GB or more in `~/Downloads` untouched for 30 days

## What it is

Files of 1 GB or more in Downloads that you have not opened or changed for a long time. Only you know whether you
still need them. Made by you, when you downloaded them.

## Where it lives

`~/Downloads`, first level only, one finding per regular file of at least 1 000 000 000 bytes that no other category
of the module claims and that is not a `.dmg` or `.pkg`.

## What happens if you remove it

They go to the Trash. Anything you have only here is lost once you empty the Trash, so look at each one first. It does
not come back.

## How the app decides

The module `downloads`. A restored scan, left out (scan again); the newest of the file's change times unknown, in the
future or within the days set in Settings (default 30, at least 7), left out (changed lately); otherwise **offered**, at
the High level because it is your own file. "Not opened" is the status change: opening a file from Finder, renaming,
tagging or moving it into the folder moves it; a file opened only by an app that bypasses Finder counts as not changed,
which the reason says. A folder of yours, a link, anything below the first level, a smaller file and a `.dmg` or
`.pkg` are never claimed here. Nothing in Downloads is read unless you turn on its switch in Settings or grant Full
Disk Access. The files go to the Trash.
