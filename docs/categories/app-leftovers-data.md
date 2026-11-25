# Data of removed apps

**Danger level:** High · **Rule:** none (the module's code, `modules/app-leftovers`) · **Since:** 0.1.0

**Example:** `~/Library/Application Support/<id>` of an app you removed

## What it is

What apps no longer on this Mac kept for you, such as documents, databases or downloads. Made by the apps themselves,
while they were installed; each item names its app by its id, such as `com.example.App`.

## Where it lives

One place in `~/Library`, one finding per folder, named by the app's id: `Application Support/<id>`. An app's
container (`Containers/<id>`) is not offered here (see the end of this page).

## What happens if you remove it

Anything the app kept only here is lost once you empty the Trash, so look inside first. It does not come back. High, not
lower: nothing tells a cache from a library of notes, so when in doubt the level is higher.

## How the app decides

The module `app-leftovers` (code with data tables, `modules/app-leftovers/places.toml` and `exclude.toml`). A leftover
is claimed only when its name, without the row's suffix, is a bundle id (`com.example.App`) and it is of the row's kind
(a folder or a file; a link or anything else is skipped). Then, per id, the first answer that applies:

| #  | Situation                                                                                               | Result                                  |
| -- | ------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| 1  | the id is in the exclusion table (`com.apple.` and the ids shared frameworks write)                     | not claimed                             |
| 2  | no leftover of the id proves an app (a saved state; a container does not count, see below)            | not claimed                             |
| 3  | the installed apps cannot be listed                                                                     | left out: Fresher cannot tell           |
| 4  | an installed app owns the id, or its id or a helper's is the id followed by a dot and more, or the reverse | not claimed                          |
| 5  | the list of installed apps is partial                                                                   | left out: cannot be sure the app is gone |
| 6  | the running apps cannot be listed                                                                       | left out: Fresher cannot tell           |
| 7  | a running app has the id or a related one                                                               | left out: the app is open               |
| 8  | a restored scan, a place not read in full, or a leftover folder read in part                            | left out: scan again                    |
| 9  | a leftover changed within the days set in Settings (default 30), or its time is unknown                 | left out: used recently                 |
| 10 | Spotlight cannot see every app (it is off, or a drive is not indexed)                                   | left out: Spotlight unread              |
| 11 | Spotlight finds an app with the id                                                                      | kept for good: installed elsewhere           |
| 12 | a socket, a pipe or a device inside                                                                     | kept for good: a program may be using it     |
| 13 | not yours, locked by the system, restricted or locked by you, or something inside is                    | kept for good: needs an administrator        |
| 14 | otherwise                                                                                               | **offered**                             |

Never offered: a name that is not an id (`Notes`, `Cookies.binarycookies`), anything of Apple's (`com.apple.`), group
containers, launch agents, an app's container (it is not read, see below), and the files of an app that is installed,
open or found elsewhere. Matching is by id only, never by name. The items go to the Trash; the app runs no command, and
an item a process holds open when you confirm is skipped. If the app is on a drive that is not connected now, nothing
can find it: its files wait the days unchanged, each finding says to keep them in that case, and you confirm each.

Containers: Fresher does not read `Containers/<id>`, the folder of a sandboxed app, even when it may open
`Containers`: a container neither proves its app nor is offered. An installed app's extension keeps its own container
under its own id, which Fresher cannot always tie to the app, so a container could prove, and get offered, the files
of an app that is still installed. An app proven only by its container is not found. The category needs no
permission.
