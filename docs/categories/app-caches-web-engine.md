# Caches of web-based apps

**Danger level:** Low · **Rule:** none (the module's code, `modules/app-caches`, table `apps.toml`) · **Since:** 0.1.0

## What it is

Files that apps built with web technology, such as chat apps and code editors, keep in their own support folder to draw
their windows and run faster. Made by the apps themselves; each item names its app and the cache folder.

## Where it lives

The places of the rows of `modules/app-caches/apps.toml` in this category, one finding per cache folder:

| App           | Place                                        | Cache folders                                                             |
| ------------- | -------------------------------------------- | ------------------------------------------------------------------------- |
| Redis Insight | `~/Library/Application Support/RedisInsight` | `Cache`, `Code Cache`, `GPUCache`, `DawnGraphiteCache`, `DawnWebGPUCache` |

The finding names the app and the cache folder (`Redis Insight: GPUCache`), and is counted in the app's line of the
storage map.

## What happens if you remove it

What is inside the folder goes to the Trash and the folder stays. Your messages, files, settings and sign-ins are kept
elsewhere in the app's folder and stay: they sit beside the caches, and the module never touches a name its row does
not list. Yes, the app makes it again as it needs it. Its next start may be slower. That is why the level is Low, not
Safe.

## How the app decides

The module `app-caches` (code with a data table, `modules/app-caches/apps.toml`, and `exclude.toml`). Only a folder
whose name is in the row's `caches`, only while the row's app is installed, and only while it and its helpers are
closed. The owner is the installed app whose bundle id is the row's, never told by a folder's name. For the folder, the
first answer that applies:

| #  | Situation                                                                            | Result                                       |
| -- | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                              | not claimed                                  |
| 2  | the row's bundle id is in the exclusion table                                        | not claimed                                  |
| 3  | the installed apps cannot be listed                                                  | not claimed                                  |
| 4  | no installed app has the row's bundle id                                             | not claimed                                  |
| 5  | the folder could not be read in full                                                 | not claimed                                  |
| 6  | nothing in it                                                                        | not claimed                                  |
| 7  | a socket, a pipe or a device inside                                                  | left out: a running program may be using it  |
| 8  | not yours, locked by the system, restricted or locked by you, or something inside is | left out: removing it needs an administrator |
| 9  | the running apps cannot be listed                                                    | left out: Fresher cannot tell                |
| 10 | the app, or a helper of it, is open                                                  | left out: the app is open                    |
| 11 | otherwise                                                                            | **offered**                                  |

Never offered: a link, a file, an empty folder, and a name the row does not list (`Cookies`, `Local Storage`,
`Session Storage`, `Preferences`, `config.json`). A removed app's support folder is
[`app-leftovers`](./app-leftovers-data.md)' own. A row is added only after it was checked on a real install. The
folder is emptied into the Trash, never removed, and the app runs no command. An item a process holds open when you
confirm is skipped.
