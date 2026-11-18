# Yarn's download cache

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

## What it is

Copies of the packages Yarn has downloaded, kept so that the next install needs no download. Made by Yarn, a package
manager for JavaScript projects.

## Where it lives

`~/Library/Caches/Yarn`, one folder. Only the tool's default place is looked at: the app reads no environment variable
and no config file. A cache that `YARN_CACHE_FOLDER` or Yarn's config moves elsewhere is not found.

The tool's sign: the folder holds `v6`, the layout of Yarn's current cache; an older layout is not recognized. Without
it the folder is not claimed.

## What happens if you remove it

It goes to the Trash; the space comes back when you empty it. Your projects and the packages installed in them keep
working. Yes. Yarn downloads each package again on the next install that needs it, which needs the network and takes
longer.

## How the app decides

The module `dev-caches` (code with a data table, `modules/dev-caches/tools.toml`). For the folder, the first answer that
applies:

| # | Situation                                                                            | Result                                       |
| - | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1 | the found item is not a folder (a file, a link, a socket, a pipe)                    | not claimed                                  |
| 2 | the tool's sign is missing (above)                                                   | not claimed                                  |
| 3 | the folder could not be read in full                                                 | not claimed                                  |
| 4 | a socket, a pipe or a device inside                                                  | left out: a running program may be using it  |
| 5 | not yours, locked by the system, restricted or locked by you, or something inside is | left out: removing it needs an administrator |
| 6 | otherwise                                                                            | **offered**                                  |

A folder that is not claimed is not shown and nothing is said about it. Times are not read: a cache is offered whatever
its age. Yarn's per-project caches and the `.yarn` folders of projects are never offered. The folder goes to the Trash;
the app never runs the tool's own command, and the tool makes its cache again when it needs it. A cache the tool holds
open when you confirm is skipped.
