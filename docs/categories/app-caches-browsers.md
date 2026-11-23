# Browser caches

**Danger level:** Low · **Rule:** none (the module's code, `modules/app-caches`, table `apps.toml`) · **Since:** 0.1.0

**Example:** Chrome's and Firefox's caches in `~/Library/Caches`

## What it is

Copies of web pages, scripts and pictures your browsers keep so sites open faster, kept for each browser profile. Made
by the browsers themselves; each item names its browser, its profile and the cache folder.

## Where it lives

The places of the rows of `modules/app-caches/apps.toml` in this category, one finding per cache folder, per profile:

| Browser       | Place                                         | Cache folders            |
| ------------- | --------------------------------------------- | ------------------------ |
| Google Chrome | `~/Library/Caches/Google/Chrome/<profile>`    | `Cache`, `Code Cache`    |
| Chromium      | `~/Library/Caches/Chromium/<profile>`         | `Cache`, `Code Cache`    |
| Firefox       | `~/Library/Caches/Firefox/Profiles/<profile>` | `cache2`, `startupCache` |

The finding names the browser, the profile and the cache folder (`Google Chrome, Profile 1: Code Cache`), and is
counted in the browser's line of the storage map.

## What happens if you remove it

What is inside the folder goes to the Trash and the folder stays. Your history, bookmarks, passwords, cookies and open
tabs are kept elsewhere and stay: they sit beside the caches, and the module never touches a name its row does not
list. Yes, the browser fills it again as you browse. Sites may open a little slower the first time, and pictures and
videos download again. That is why the level is Low, not Safe.

## How the app decides

The module `app-caches` (code with a data table, `modules/app-caches/apps.toml`, and `exclude.toml`). Only a folder
whose name is in the row's `caches`, only while the row's browser is installed, and only while it and its helpers are
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
| 10 | the browser, or a helper of it, is open                                              | left out: the app is open                    |
| 11 | otherwise                                                                            | **offered**                                  |

Never offered: a link, a file, an empty folder, a name the row does not list (history, cookies, passwords, `Storage`,
`image_cache`), the caches the browsers keep in their support folders (small, and the system guards them), and
Safari's, which are not read. A row is added only after it was checked on a real install. The folder is emptied into
the Trash, never removed, and the app runs no command. An item a process holds open when you confirm is skipped.
