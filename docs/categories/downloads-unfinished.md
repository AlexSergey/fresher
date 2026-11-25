# Unfinished downloads

**Danger level:** Low · **Rule:** none (the module's code, `modules/downloads`) · **Since:** 0.1.0

**Example:** a `.crdownload` or `.part` file in `~/Downloads`

## What it is

Downloads that stopped before they finished, when a browser was closed or the download failed. Made by the browser,
while it downloads a file. A file of Firefox's `.part` comes with an empty file of the final name beside it, which is
offered with it.

## Where it lives

`~/Downloads`, first level only, one finding per file: a name ending `.crdownload` (Chrome, Edge, Brave, Vivaldi,
Chromium), `.part` (Firefox), `.opdownload` (Opera) or `.partial` (Edge), case ignored. The suffixes and the
browsers that write them are in `modules/downloads/unfinished.toml`.

## What happens if you remove it

They go to the Trash; to get the file, download it again. It comes back only if you download the file again.

## How the app decides

The module `downloads` (code with data tables). Per file, the first answer that applies: a restored scan, left out
(scan again); the running apps cannot be listed, left out; a browser of the row is running, left out (the download may
still finish); the newest of the file's change times is unknown, in the future or within the days set in Settings
(default 30, at least 7), left out (changed lately); otherwise **offered**. The empty file of the final name beside an
offered `.part` is offered with it; one that is not empty is yours and is never claimed, and when the folder was not
read in full no placeholder is offered. A link, a folder or anything below the first level is never claimed. Nothing in
Downloads is read unless you turn on its switch in Settings or grant Full Disk Access; without either the category is
unavailable. The items go to the Trash, and one a process holds open when you confirm is skipped.
