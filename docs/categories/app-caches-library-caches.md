# App caches

**Danger level:** Low · **Rule:** none (the module's code, `modules/app-caches`) · **Since:** 0.1.0

**Example:** `~/Library/Caches/com.example.Editor` of an app you still use

## What it is

Files apps save so they don't fetch or build them twice. Made by the apps themselves; each item names its app.

## Where it lives

`~/Library/Caches/<id>`, where the id is a bundle id (`com.example.Editor`). One finding per folder, and the folder
belongs to an app that is installed now: the app's own folder, a helper's (an updater or a menu bar part), or an id
under the app's or its helper's (`com.example.Editor.ShipIt`). The finding names the app and is counted in the app's
line of the storage map.

## What happens if you remove it

Nothing is lost. Some apps open a little slower once. What is inside the folder goes to the Trash and the folder stays:
the app may expect it to exist. Your documents and the app's settings are kept elsewhere and stay. Yes, by itself, the
next time the app needs it. An app that kept music, videos or maps for offline use downloads them again; that is why
the level is Low, not Safe.

## How the app decides

The module `app-caches` (code with data tables, `modules/app-caches/places.toml` and `exclude.toml`). The folder's name
must be a bundle id, and an installed app must name it: its id or a helper's is the name, or the name is that id and
a dot and more; when several do, the longest id wins, and never a vendor prefix or a sibling id. For the folder, the
first answer that applies:

| #  | Situation                                                                                      | Result                                       |
| -- | ---------------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                                        | not claimed                                  |
| 2  | the name is not an id, or the id is in the exclusion table (`com.apple.`, Fresher's own)       | not claimed                                  |
| 3  | the installed apps cannot be listed                                                            | not claimed (`app-leftovers` says it)        |
| 4  | no installed app names it                                                                      | not claimed                                  |
| 5  | the folder could not be read in full                                                           | not claimed                                  |
| 6  | nothing in it                                                                                  | not claimed                                  |
| 7  | a socket, a pipe or a device inside                                                            | kept for good: a running program may be using it  |
| 8  | not yours, locked by the system, restricted or locked by you, or something inside is           | kept for good: removing it needs an administrator |
| 9  | the running apps cannot be listed                                                              | left out: Fresher cannot tell                |
| 10 | the app, or a helper of it, is open                                                            | left out: the app is open                    |
| 11 | otherwise                                                                                      | **offered**                                  |

Never offered: a name that is not an id (`Notes`, `JetBrains`), anything of Apple's (`com.apple.`), Fresher's own, a
removed app's cache (that is [`app-leftovers`](./app-leftovers-caches.md)' own, so no folder is offered twice), a vendor
prefix or a sibling id of an app, a link, and the containers of sandboxed apps, which are not read. Times are not read:
a cache is offered whatever its age, and every scan judges the apps open now again. The folder is emptied into the
Trash, never removed, and the app runs no command.
An item a process holds open when you confirm is skipped.
