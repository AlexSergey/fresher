# Data of removed apps

**Danger level:** High · **Rule:** none (the module's code, `modules/app-leftovers`) · **Since:** 0.1.0

## What it is

The support folders and containers of apps that are no longer on this Mac. An app may have kept documents, databases
or downloads here. Made by the apps themselves, while they were installed; each item names its app by its id, such as
`com.example.App`.

## Where it lives

Two places in `~/Library`, one finding per folder, named by the app's id:

- `Application Support/<id>`;
- `Containers/<id>`, the folder of a sandboxed app. Fresher does not open `Containers` until the plan that settles its
  permission; until then only support folders are offered here.

## What happens if you remove it

They go to the Trash. Anything the app kept only here is lost once you empty the Trash, so look inside first if you are
not sure. It does not come back. High, not lower: nothing tells a cache from a library of notes, so when in doubt the
level is higher.

## How the app decides

The module `app-leftovers` (code with data tables, `modules/app-leftovers/places.toml` and `exclude.toml`). A leftover
is claimed only when its name, without the row's suffix, is a bundle id (`com.example.App`) and it is of the row's kind
(a folder or a file; a link or anything else is skipped). Then, per id, the first answer that applies:

| #  | Situation                                                                                               | Result                                  |
| -- | ------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| 1  | the id is in the exclusion table (`com.apple.` and the ids shared frameworks write)                     | not claimed                             |
| 2  | no leftover of the id proves an app (a saved state, or a container, which Fresher does not open yet)   | not claimed                             |
| 3  | the installed apps cannot be listed                                                                     | left out: Fresher cannot tell           |
| 4  | an installed app owns the id, or its id or a helper's is the id followed by a dot and more, or the reverse | not claimed                          |
| 5  | the list of installed apps is partial                                                                   | left out: cannot be sure the app is gone |
| 6  | the running apps cannot be listed                                                                       | left out: Fresher cannot tell           |
| 7  | a running app has the id or a related one                                                               | left out: the app is open               |
| 8  | a restored scan, a place not read in full, or a leftover folder read in part                            | left out: scan again                    |
| 9  | a leftover changed within the days set in Settings (default 30), or its time is unknown                 | left out: used recently                 |
| 10 | Spotlight cannot see every app (it is off, or a drive is not indexed)                                   | left out: Spotlight unread              |
| 11 | Spotlight finds an app with the id                                                                      | left out: installed elsewhere           |
| 12 | a socket, a pipe or a device inside                                                                     | left out: a program may be using it     |
| 13 | not yours, locked by the system, restricted or locked by you, or something inside is                    | left out: needs an administrator        |
| 14 | otherwise                                                                                               | **offered**                             |

Never offered: a name that is not an id (`Notes`, `Cookies.binarycookies`), anything of Apple's (`com.apple.`), group
containers, launch agents, and the files of an app that is installed, open or found elsewhere. Matching is by id only,
never by name. The items go to the Trash; the app runs no command, and an item a process holds open when you confirm is
skipped. If the app is on a drive that is not connected now, nothing can find it: its files wait the days unchanged,
each finding says to keep them in that case, and you confirm each.
